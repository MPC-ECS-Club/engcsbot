use std::ops::DerefMut;

use chrono::{DateTime, Datelike, Local, Timelike};
use serenity::all::{CacheHttp, ChannelId, Color, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, ReactionType};

use crate::{ANNOUNCEMENT_CHANNEL_ID, ANNOUNCEMENT_EPSILON_MINS, AUTOMATION_NOTICE_MESSAGE, UPDATE_RATE, data::{saveutil, scheduled_meeting::{ScheduleManager, ScheduledMeeting}}, discord_log, get_clock_emoji_for_hour, set_today_to_hr_min_sec, to_12_hr_clock_str};

async fn check_if_should_announce_day_before(
    http: impl CacheHttp,
    chan: ChannelId,
    dt: DateTime<Local>,
    meeting: &mut ScheduledMeeting,
) -> bool {
    if meeting.day_before_announced {
        return false;
    }

    let weekday = dt.weekday();

    if weekday != meeting.day.pred() {
        return false;
    }

    if dt.hour() >= 9 {
        meeting.day_before_announced = true;

        let emoji = get_clock_emoji_for_hour(meeting.start.0);

        let message = CreateMessage::new()
            .content("@everyone") // blame jordi
            .add_embed(
                CreateEmbed::new()
                    .title("Meeting Notice 📒")
                    .color(Color::PURPLE)
                    .description(format!(
                        "This is a notice for an upcoming meeting ***tomorrow*** ({}).",
                        meeting.day
                    ))
                    .field("Location 🪐", &meeting.location, true)
                    .field(format!("Time {}", emoji), to_12_hr_clock_str(meeting.start), true)
                    .footer(CreateEmbedFooter::new(AUTOMATION_NOTICE_MESSAGE)),
            );

        _ = chan.send_message(&http, message).await;

        return true;
    }

    false
}


pub async fn make_announcement(chan: ChannelId, ctx: &Context, meeting: &ScheduledMeeting) {
    let (start_hr, start_min) = meeting.start;
    let meet_time = set_today_to_hr_min_sec(start_hr, start_min, 0);
    let seconds_since_epoch = meet_time.timestamp();

    let (end_hr, end_min) = meeting.end;

    let meeting_end_epoch_time = set_today_to_hr_min_sec(end_hr, end_min, 0).timestamp();

    let mut embed = CreateEmbed::new()
        .title("🎉 Meeting Alert 🚨")
        .description(format!(
            "There will be a meeting today <t:{seconds_since_epoch}:R>"
        ))
        .color(Color::DARK_GREEN)
        .field("Location 🪐", &meeting.location, true)
        .field(
            format!("Until {}", get_clock_emoji_for_hour(end_hr)),
            format!("<t:{meeting_end_epoch_time}:t>"),
            true,
        )
        .footer(CreateEmbedFooter::new(format!(
            "Please react to this message if you plan on attending!\n{AUTOMATION_NOTICE_MESSAGE}"
        )));

    if let Some(note) = &meeting.note {
        embed = embed.field("Note", note.as_str(), false);
    }

    let msg = CreateMessage::new().content("@everyone").embed(embed);

    if let Ok(msg) = chan.send_message(&ctx.http, msg).await
        && let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode("\u{2705}".into()))
            .await
    {
        discord_log!(
            &ctx.http,
            "failed to send automatic announcement, or reaction to it. {why:?}"
        );
    }
}


// TODO: cleanup
async fn start_time_checking_loop(ctx: Context) {
    let chan = ChannelId::new(ANNOUNCEMENT_CHANNEL_ID);
    loop {
        tokio::time::sleep(UPDATE_RATE).await;

        let dt = Local::now();
        let weekday = dt.weekday();

        let mut to_remove: Vec<usize> = vec![];

        let mut meetings = ScheduleManager::get_schedule().await;
        let mut reload_save_data = false;

        for (i, meeting) in meetings.deref_mut().iter_mut().enumerate() {
            if check_if_should_announce_day_before(&ctx.http, chan, dt, meeting).await {
                reload_save_data = true;
                continue;
            }

            if meeting.day != weekday {
                continue;
            }
            if ScheduleManager::is_already_announced(meeting).await {
                continue;
            }

            let (start_hr, start_min) = meeting.start;

            // this prevents the underflow, although it would mean that we would be checking an hour before,
            // since we skip any meetings not on the current day
            // I highly doubt we'll have midnight meetings, so I won't fix this,
            // but I'll leave this message here so it is documented.
            let desired_hr_to_check = if start_hr == 0 {
                start_hr
            } else {
                start_hr - 1
            };

            if dt.hour() == desired_hr_to_check
                && dt.minute() >= (ANNOUNCEMENT_EPSILON_MINS + start_min)
            {
                // meeting!
                meeting.day_before_announced = false;
                reload_save_data = true;
                make_announcement(chan, &ctx, meeting).await;
                meeting.note = None;

                if meeting.onetime {
                    to_remove.push(i);
                } else {
                    let (end_hr, end_min) = meeting.end;
                    let meeting_end_epoch_time =
                        set_today_to_hr_min_sec(end_hr, end_min, 0).timestamp();
                    ScheduleManager::set_already_announced(meeting, meeting_end_epoch_time).await;
                }
            }
        }

        to_remove
            .iter()
            .rev()
            .for_each(|i| _ = meetings.swap_remove(*i));

        drop(meetings);

        if reload_save_data {
            saveutil::save_all_meetings().await;
        }
    }
}

async fn ready(ctx: Context) {
    start_time_checking_loop(ctx).await;
}

pub fn start(ctx: Context) {
    tokio::spawn(ready(ctx));
}
