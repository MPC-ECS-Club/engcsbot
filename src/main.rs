mod commands;
mod data;

use std::collections::HashMap;
use crate::data::saveutil;
use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting, Suspended};
use chrono::{DateTime, Datelike, Local, Timelike};
use serenity::all::{
    ActivityData, Color, Command, CommandId, CreateEmbedFooter, CreateMessage, GuildId,
    Interaction, OnlineStatus, ReactionType, Ready, ShardManager,
};
use serenity::builder::CreateEmbed;
use serenity::{
    all::{ChannelId, Message},
    async_trait,
    prelude::*,
};
use std::convert::Into;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;

const ANNOUNCEMENT_EPSILON_MINS: u32 = 0;

const MEETING_JSON_PATH: &str = "./meetings.json";
const SUSPENDED_JSON_PATH: &str = "./suspended.json";

const LOG_CHANNEL_ID: ChannelId = ChannelId::new(1470495355329183744);

#[cfg(not(debug_assertions))]
const STATUSES: &[&str] = &["engineering...", "programming...", "procrastinating..."];
#[cfg(not(debug_assertions))]
const STATUS_TIME: Duration = Duration::from_mins(2);

#[cfg(debug_assertions)]
const ANNOUNCEMENT_CHANNEL_ID: u64 = 839277529511755786;

#[cfg(not(debug_assertions))]
const ANNOUNCEMENT_CHANNEL_ID: u64 = 1153591616301432834;

#[cfg(debug_assertions)]
const UPDATE_RATE: Duration = Duration::from_secs(1);

#[cfg(not(debug_assertions))]
const UPDATE_RATE: Duration = Duration::from_mins(1);

pub async fn discord_log(http: impl CacheHttp, val: impl Into<String>) {
    let val = val.into();
    println!("{}", &val);

    if let Err(why) = LOG_CHANNEL_ID
        .send_message(&http, CreateMessage::new().content(val))
        .await
    {
        println!("failed to log to discord channel: {why:?}");
    }
}

#[macro_export]
macro_rules! discord_log {
    ( $http:expr, $($arg:expr),* ) => {
        discord_log($http, format!($($arg),*)).await;
    };
}

// why not
fn get_clock_emoji_for_hour(hour: u32) -> &'static str {
    match hour % 12 {
        0 => "🕛️",
        1 => "🕐️",
        2 => "🕑️️",
        3 => "🕒️",
        4 => "🕓️",
        5 => "🕔️",
        6 => "🕕️",
        7 => "🕖️",
        8 => "🕗️",
        9 => "🕘️",
        10 => "🕙️",
        11 => "🕚️",

        _ => "⏰️",
    }
}

struct Handler;

// TODO: cleanup
async fn start_time_checking_loop(ctx: Context) {
    let chan = ChannelId::new(ANNOUNCEMENT_CHANNEL_ID);
    loop {
        tokio::time::sleep(UPDATE_RATE).await;

        let dt = Local::now();
        let weekday = dt.weekday();

        let mut to_remove: Vec<usize> = vec![];

        let mut meetings = ScheduleManager::get_schedule().await;
        for (i, meeting) in meetings.deref().iter().enumerate() {
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

                let meet_time = set_today_to_hr_min_sec(start_hr, start_min, 0);
                let seconds_since_epoch = meet_time.timestamp();

                let (end_hr, end_min) = meeting.end;

                let meeting_end_epoch_time =
                    set_today_to_hr_min_sec(end_hr, end_min, 0).timestamp();

                let msg = CreateMessage::new()
                    .content("@everyone")
                    .embed(CreateEmbed::new()
                        .title("🎉 Meeting Alert 🚨")
                        .description(format!("There will be a meeting today <t:{seconds_since_epoch}:R>"))
                        .color(Color::DARK_GREEN)
                        .field("Location 🪐", &meeting.location, true)
                        .field(format!("Until {}", get_clock_emoji_for_hour(end_hr)), format!("<t:{meeting_end_epoch_time}:t>"), true)
                        .footer(CreateEmbedFooter::new("Please react to this message if you plan on attending!\nNote this message was automated, and if a previous agreed upon arrangement for the meeting was made (such as date, time, location, or entirely canceled, please disregard this message.)"))
                );

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

                if meeting.onetime {
                    to_remove.push(i);
                } else {
                    // maybe avoid this clone, doesn't really matter it's not *that* expensive, and it doesn't occur that often.
                    ScheduleManager::set_already_announced(meeting.clone(), meeting_end_epoch_time)
                        .await;
                }
            }
        }

        to_remove
            .iter()
            .rev()
            .for_each(|i| _ = meetings.swap_remove(*i));
    }
}

fn is_suspension_done(meeting: &ScheduledMeeting, reset_timestamp: i64) -> bool{
    let now = Local::now().timestamp();

    reset_timestamp != -1 && now > reset_timestamp
}

async fn reset_suspended_if_necessary(meeting: &ScheduledMeeting) {
    let time = ScheduleManager::get_announced_reset_timestamp(meeting).await;

    if is_suspension_done(meeting, time) { // maybe don't lock again, and just store the map?
        ScheduleManager::reset_announced_state(meeting).await;
    }
}

async fn reset_announced_state() {
    tokio::time::sleep(UPDATE_RATE.div_f64(2.0)).await; // offset from  regular update rate

    loop {
        tokio::time::sleep(UPDATE_RATE).await;

        for meeting in ScheduleManager::get_schedule().await.deref() {
            reset_suspended_if_necessary(meeting).await;
        }
    }
}

async fn bot_shell(ctx: Context) {
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    loop {
        let mut buf = String::new();
        if let Err(why) = reader.read_line(&mut buf).await {
            eprintln!("failed to read from stdin: {}", why);
        }
        let buf = buf.trim();
        println!(">> {buf}");

        match buf {
            "delete-commands" => {
                // for debugging
                println!("deleting commands");
                let cmds = Command::get_global_commands(&ctx.http).await;
                let Ok(cmds) = cmds else {
                    continue;
                };

                for cmd in cmds {
                    _ = Command::delete_global_command(&ctx.http, cmd.id).await;
                }
                println!("done.");
            }
            "delete-cmd-id" => {
                println!("enter desired id:");
                let mut buf = String::new();
                reader.read_line(&mut buf).await.unwrap();
                buf = buf.trim().to_string();
                let id: u64 = buf.parse().unwrap();
                println!("deleting...");
                _ = Command::delete_global_command(&ctx.http, CommandId::new(id)).await;
                println!("done.");
            }
            _ => (),
        };
    }
}

// certainly not the prettiest function
async fn load_save_data(ctx: &Context) {
    let meeting_json = Path::new(MEETING_JSON_PATH);
    if !meeting_json.exists() {
        _ = File::create(meeting_json)
            .await
            .expect("failed to create file");
    }

    let json = tokio::fs::read_to_string(&meeting_json)
        .await
        .unwrap_or("[]".to_string());

    if json == "[]" {
        discord_log!(&ctx.http, "**warn**: meetings.json was empty.");
    }

    ScheduleManager::deserialize_from_json(json.as_str()).await;
    println!(
        "loaded {} meetings.",
        ScheduleManager::meeting_count().await
    );

    let suspended_json = Path::new(SUSPENDED_JSON_PATH);
    if !suspended_json.exists() {
        _ = File::create(suspended_json)
            .await
            .expect("failed to create suspended.json");
    }


    let json = tokio::fs::read_to_string(&suspended_json)
        .await
        .unwrap_or("".to_string());

    // so yeah, this is ugly, maybe refactor sometime so this isn't all a mess.
    if let Ok(data) = serde_json::from_str::<Vec<(ScheduledMeeting, Suspended)>>(&json) {
        let schedule = ScheduleManager::get_schedule().await;
        let mut temp_sus = ScheduleManager::get_suspension_map().await;

        let mut count = 0usize;
        for (meet, sus) in data {
            if !schedule.contains(&meet) || is_suspension_done(&meet, sus.reschedule) {
                continue;
            }

            temp_sus.insert(meet, sus);
            count += 1;
        }
        println!("loaded {} suspended meetings", count);
    } else {
        discord_log!(&ctx.http, "**warn**: suspended.json was empty or unable to be read.");
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {}

    async fn message(&self, _ctx: Context, _msg: Message) {}

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        println!("connected to {}", data_about_bot.user.name);

        load_save_data(&ctx).await;

        Command::create_global_command(&ctx.http, commands::announce::register())
            .await
            .expect("announce command");
        Command::create_global_command(&ctx.http, commands::schedule::register())
            .await
            .expect("schedule command");
        Command::create_global_command(&ctx.http, commands::info::register())
            .await
            .expect("info command");
        Command::create_global_command(&ctx.http, commands::shutdown::register())
            .await
            .expect("shutdown command");
        Command::create_global_command(&ctx.http, commands::upcoming::register())
            .await
            .expect("upcoming command");
        Command::create_global_command(&ctx.http, commands::removemeeting::register())
            .await
            .expect("removemeeting command");
        Command::create_global_command(&ctx.http, commands::cancelday::register())
            .await
            .expect("cancelday command");

        println!("commands registered successfully!");

        {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                start_time_checking_loop(ctx).await;
            });
        }

        tokio::spawn(async {
            reset_announced_state().await;
        });

        #[cfg(debug_assertions)]
        {
            ctx.set_presence(
                Some(ActivityData::playing("debug mode")),
                OnlineStatus::Online,
            );
        }
        #[cfg(not(debug_assertions))]
        {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                let mut i = 0usize;
                loop {
                    let desired = STATUSES[i];
                    ctx.set_presence(Some(ActivityData::custom(desired)), OnlineStatus::Online);

                    i = (i + 1) % STATUSES.len();
                    tokio::time::sleep(STATUS_TIME).await;
                }
            });
        }

        {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                bot_shell(ctx).await;
            });
        }

        discord_log!(&ctx.http, "Ready!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            match cmd.data.name.as_str() {
                "info" => {
                    with_timeout(async move {
                        commands::info::run(&ctx, cmd).await;
                    })
                    .await
                }
                "shutdown" => {
                    with_timeout(async move {
                        commands::shutdown::run(&ctx, cmd).await;
                    })
                    .await
                }
                "announce" => {
                    with_timeout(async move {
                        commands::announce::run(&ctx, cmd).await;
                    })
                    .await
                }
                "schedule" => {
                    with_timeout(async move {
                        commands::schedule::run(&ctx, cmd).await;
                    })
                    .await
                }
                "upcoming" => {
                    with_timeout(async move {
                        commands::upcoming::run(&ctx, cmd).await;
                    })
                    .await
                }
                "removemeeting" => {
                    with_timeout(async move {
                        commands::removemeeting::run(&ctx, cmd).await;
                    })
                    .await
                }
                "cancelday" => {
                    with_timeout(async move {
                        commands::cancelday::run(&ctx, cmd).await;
                    })
                    .await
                }

                _ => println!("called unimplemented cmd"),
            };
        }
    }
}

async fn with_timeout<F>(f: F)
where
    F: Future<Output = ()>,
{
    const TIMEOUT: Duration = Duration::from_secs(5);
    with_timeout_of(TIMEOUT, "function timed out.", f).await;
}

async fn with_timeout_of<F>(time: Duration, msg: impl Into<String>, f: F)
where
    F: Future<Output = ()>,
{
    tokio::select! {
        _ = tokio::time::sleep(time) => {
            println!("({}) {}", std::any::type_name::<F>(), msg.into());
        },
        _ = f => (),
    }
}

pub fn set_today_to_hr_min_sec(hr: u32, min: u32, sec: u32) -> DateTime<Local> {
    Local::now()
        .with_hour(hr)
        .unwrap()
        .with_minute(min)
        .unwrap()
        .with_second(sec)
        .unwrap()
}

pub fn to_12_hr_clock_str(clock: (u32, u32)) -> String {
    let ampm = if clock.0 < 12 { "am" } else { "pm" };
    let hr = clock.0 % 12;
    let hr = if hr == 0 { 12 } else { hr };

    format!("{:02}:{:02}{ampm}", hr, clock.1)
}

pub struct ClientShardManager;

impl TypeMapKey for ClientShardManager {
    type Value = Arc<ShardManager>;
}

#[tokio::main]
async fn main() {
    let token =
        std::env::var("DISCORD_TOKEN").expect("Missing discord bot token environment variable");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_WEBHOOKS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client.");

    client
        .data
        .write()
        .await
        .insert::<ClientShardManager>(client.shard_manager.clone());

    tokio::select! {
        res = client.start() => { // select chooses the first *matching* branch
            if let Err(why) = res {
                println!("client error: {why:?}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("received Ctrl-C, shutting down ...");
        }
    }

    with_timeout_of(
        Duration::from_secs(8),
        "failed to save, timed out.",
        async move {
            println!(
                "Saving data (meetings={})",
                ScheduleManager::get_schedule().await.len()
            );
            saveutil::save_all_meetings().await;
            println!("saved meetings");
            saveutil::save_suspended().await;
            println!("saved suspended");
        },
    )
    .await;

    exit(0); // required to prevent the bot shell function from blocking
}
