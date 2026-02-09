use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting};
use crate::{get_clock_emoji_for_hour, to_12_hr_clock_str};
use chrono::Weekday;
use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

fn meeting_to_string(m: &ScheduledMeeting) -> String {
    let extra = if m.onetime { " (just this week)" } else { "" };

    format!(
        "{} {}: to {} {}{}",
        get_clock_emoji_for_hour(m.start.0),
        to_12_hr_clock_str(m.start),
        get_clock_emoji_for_hour(m.end.0),
        to_12_hr_clock_str(m.end),
        extra
    )
}

fn get_meetings_for_day(day: Weekday, meetings: &[ScheduledMeeting]) -> String {
    meetings
        .iter()
        .filter(|m| m.day == day)
        .map(meeting_to_string)
        .collect::<Vec<String>>()
        .join("\n")
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let schedule = ScheduleManager::get_schedule().await;

    // duplicated code!!
    let mondays = get_meetings_for_day(Mon, &schedule);
    let tuesdays = get_meetings_for_day(Tue, &schedule);
    let wednesdays = get_meetings_for_day(Wed, &schedule);
    let thursdays = get_meetings_for_day(Thu, &schedule);
    let fridays = get_meetings_for_day(Fri, &schedule);
    let saturdays = get_meetings_for_day(Sat, &schedule);
    let sundays = get_meetings_for_day(Sun, &schedule);

    drop(schedule);

    let embed = CreateEmbed::new()
        .title("Upcoming Meetings")
        .field("Monday", mondays, false)
        .field("Tuesdays", tuesdays, false)
        .field("Wednesdays", wednesdays, false)
        .field("Thursdays", thursdays, false)
        .field("Fridays", fridays, false)
        .field("Saturdays", saturdays, false)
        .field("Sundays", sundays, false);

    let msg = CreateInteractionResponseMessage::new().embed(embed);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("upcoming").description("See all upcoming meetings.")
}
