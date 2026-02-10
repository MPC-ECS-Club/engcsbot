use crate::commands::util;
use crate::data::scheduled_meeting::ScheduleManager;
use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use crate::data::saveutil;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let options = &cmd.data.options;

    let Some(start) = &options.first().unwrap().value.as_str() else { return; };
    let Some(end) = &options.get(1).unwrap().value.as_str() else { return; };
    let onetime = options.iter().find(|d| d.name == "onetime").is_some_and(|v| v.value.as_bool().unwrap_or(false));

    let Some(start) = util::parse_time(start) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid start time!");
        return;
    };

    let Some(end) = util::parse_time(end) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid end time!");
        return;
    };

    let mut schedule = ScheduleManager::get_schedule().await;

    let meetings_to_remove: Vec<usize> = schedule.iter()
        .enumerate()
        .filter(|(_, m)| m.start == start && m.end == end && m.onetime == onetime)
        .map(|(i, _)| i)
        .collect();

    meetings_to_remove.iter()
        .rev()
        .for_each(|i| _ = schedule.swap_remove(*i));

    let msg = CreateInteractionResponseMessage::new()
        .content(format!("Removing {} meetings.", meetings_to_remove.len()))
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;

    saveutil::save_all().await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("removemeeting")
        .description("Remove a particular meeting based on it's start and end date.")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "start", "12:00pm, 1:30pm").required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::String, "end", "2:00pm, 3:30pm").required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::Boolean, "onetime", "Whether or not this meeting is a 'onetime' (non-repeating) meeting."))
}
