use chrono::Weekday;
use crate::commands::util;
use crate::data::saveutil;
use crate::data::scheduled_meeting::ScheduleManager;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let options = &cmd.data.options;

    // kinda of an error-prone approach, perhaps make some utility to help with this.
    let Some(day) = &options.first().unwrap().value.as_str() else {
        return;
    };
    let Some(start) = &options.get(1).unwrap().value.as_str() else {
        return;
    };
    let Some(end) = &options.get(2).unwrap().value.as_str() else {
        return;
    };
    let onetime = options
        .iter()
        .find(|d| d.name == "onetime")
        .is_some_and(|v| v.value.as_bool().unwrap_or(false));

    let Ok(day) = day.parse::<Weekday>() else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid week day!");
        return;
    };

    let Some(start) = util::parse_time(start) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid start time!");
        return;
    };

    let Some(end) = util::parse_time(end) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid end time!");
        return;
    };

    let total = ScheduleManager::remove_matching(day, start, end, onetime).await;

    let msg = CreateInteractionResponseMessage::new()
        .content(format!("Removing {} meetings.", total))
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;

    saveutil::save_all_meetings().await;
    saveutil::save_suspended().await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("removemeeting")
        .description("Remove a particular meeting based on it's day, start and end times.")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "day", "Monday, Tue, ...")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "start", "12:00pm, 1:30pm")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "end", "2:00pm, 3:30pm")
                .required(true),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "onetime",
            "Whether or not this meeting is a 'onetime' (non-repeating) meeting.",
        ))
}
