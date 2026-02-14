use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting};
use crate::{get_clock_emoji_for_hour, to_12_hr_clock_str};
use chrono::Weekday;
use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
use serenity::all::{
    Color, CommandInteraction, Context, CreateCommand, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

async fn meeting_to_string(m: &ScheduledMeeting) -> String {
    let extra = if m.onetime { " (just this week)" } else { "" };
    let canceled_extra = if ScheduleManager::is_meeting_cancelled(m).await {
        "❌️ CANCELLED "
    } else {
        "✅️ "
    };

    format!(
        "{canceled_extra}{} {} **to** {} {}{} 🪐 {}",
        get_clock_emoji_for_hour(m.start.0),
        to_12_hr_clock_str(m.start),
        get_clock_emoji_for_hour(m.end.0),
        to_12_hr_clock_str(m.end),
        extra,
        m.location,
    )
}

async fn get_meetings_for_day(day: Weekday, meetings: &[ScheduledMeeting]) -> String {
    let mut res: Vec<String> = Vec::with_capacity(meetings.len());

    for meet in meetings.iter().filter(|m| m.day == day) {
        res.push(meeting_to_string(meet).await);
    }

    res.join("\n")
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let schedule = ScheduleManager::get_schedule().await;

    // duplicated code!!
    let mondays = get_meetings_for_day(Mon, &schedule).await;
    let tuesdays = get_meetings_for_day(Tue, &schedule).await;
    let wednesdays = get_meetings_for_day(Wed, &schedule).await;
    let thursdays = get_meetings_for_day(Thu, &schedule).await;
    let fridays = get_meetings_for_day(Fri, &schedule).await;
    let saturdays = get_meetings_for_day(Sat, &schedule).await;
    let sundays = get_meetings_for_day(Sun, &schedule).await;

    drop(schedule);

    let embed = CreateEmbed::new()
        .title("🗓️ Upcoming Meetings")
        .color(Color::ORANGE)
        .field("Monday", mondays, false)
        .field("Tuesday", tuesdays, false)
        .field("Wednesday", wednesdays, false)
        .field("Thursday", thursdays, false)
        .field("Friday", fridays, false)
        .field("Saturday", saturdays, false)
        .field("Sunday", sundays, false);

    let msg = CreateInteractionResponseMessage::new().embed(embed);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("upcoming").description("See all upcoming meetings.")
}
