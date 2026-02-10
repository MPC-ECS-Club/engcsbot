use crate::commands::util;
use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting};
use chrono::Weekday;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let options = &cmd.data.options;

    let Some(day) = options.first().unwrap().value.as_str() else {
        return;
    };

    let Ok(day) = day.parse::<Weekday>() else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid weekday").await;
        return;
    };

    let schedule = ScheduleManager::get_schedule().await;
    let desired: Vec<ScheduledMeeting> =
        schedule.iter().filter(|m| m.day == day).cloned().collect();

    let canceled_count = desired.len();

    for meeting in desired {
        let until = ScheduleManager::cancel_meeting(meeting).await;
        println!("meeting cancelled until {}", until);
    }

    let msg = CreateInteractionResponseMessage::new()
        .content(format!("Canceled **{}** meetings.", canceled_count));

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("cancelday")
        .description("Cancel an entire day of meetings (only for that one day, also does not apply to onetime meetings.)")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "day", "What day to cancel: (this) Monday, Tuesday, Wednesday, ...").required(true))
}
