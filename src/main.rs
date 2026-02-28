mod commands;
mod data;
mod periodic;

use crate::data::saveutil;
use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting, Suspended};
use chrono::{DateTime, Local, Timelike};
use serenity::all::{
    Command, CommandId, CreateMessage, GuildId,
    Interaction, Ready, ShardManager,
};
use serenity::{
    all::{ChannelId, Message},
    async_trait,
    prelude::*,
};
use std::collections::HashMap;
use std::convert::Into;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use uuid::Uuid;

const ANNOUNCEMENT_EPSILON_MINS: u32 = 0;

const DATA_DIRECTORY: &str = "./bot-storage/";
const MEETING_JSON_PATH: &str = "./bot-storage/meetings.json";
const SUSPENDED_JSON_PATH: &str = "./bot-storage/suspended.json";

const LOG_CHANNEL_ID: ChannelId = ChannelId::new(1470495355329183744);

#[cfg(debug_assertions)]
const ANNOUNCEMENT_CHANNEL_ID: u64 = 839277529511755786;

#[cfg(not(debug_assertions))]
const ANNOUNCEMENT_CHANNEL_ID: u64 = 1153591616301432834;

#[cfg(debug_assertions)]
const UPDATE_RATE: Duration = Duration::from_secs(1);

#[cfg(not(debug_assertions))]
const UPDATE_RATE: Duration = Duration::from_mins(1);

const AUTOMATION_NOTICE_MESSAGE: &str = "Note this message was automated, and if this message contradicts previous arrangements, please ignore this message. See /info for more information.";

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


#[cfg(debug_assertions)]
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
                if let Err(why) = Command::set_global_commands(&ctx.http, vec![]).await {
                    eprintln!("failed to set global commands: {:?}", why);
                }
                // let cmds = Command::get_global_commands(&ctx.http).await;
                // let Ok(cmds) = cmds else {
                //     continue;
                // };
                //
                // for cmd in cmds {
                //     _ = Command::delete_global_command(&ctx.http, cmd.id).await;
                // }
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

fn is_suspension_done(reset_timestamp: i64) -> bool {
    let now = Local::now().timestamp();

    reset_timestamp != -1 && now > reset_timestamp
}

// Returns whether or not the suspend.json file should be refreshed.
async fn reset_suspended_if_necessary(meeting: &ScheduledMeeting) -> bool {
    let time = ScheduleManager::get_suspension_restore_timestamp(meeting).await;

    let mut should_refresh = false;
    if is_suspension_done(time) {
        // maybe don't lock again, and just store the map?
        should_refresh = true;
        ScheduleManager::unsuspend(meeting).await;
    }

    should_refresh
}

// certainly not the prettiest function
async fn load_save_data(ctx: &Context) {
    let data = Path::new(DATA_DIRECTORY);

    if !data.exists() {
        tokio::fs::create_dir(data).await.unwrap();
    }

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
    if let Ok(data) = serde_json::from_str::<HashMap<Uuid, Suspended>>(&json) {
        let schedule = ScheduleManager::get_schedule().await;
        let mut temp_sus = ScheduleManager::get_suspension_map().await;

        let mut count = 0usize;
        for (meeting_uuid, sus) in data {
            let meet = schedule.iter().find(|v| v.uuid == meeting_uuid);
            let Some(meet) = meet else {
                continue;
            };

            if is_suspension_done(sus.reschedule) {
                continue;
            }

            temp_sus.insert(meet.uuid, sus);
            count += 1;
        }
        println!("loaded {} suspended meetings", count);
    } else {
        discord_log!(
            &ctx.http,
            "**warn**: suspended.json was empty or unable to be read."
        );
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
        Command::create_global_command(&ctx.http, commands::jsonembed::register())
            .await
            .expect("jsonembed command");
        Command::create_global_command(&ctx.http, commands::setnote::register())
            .await
            .expect("setnote command");

        #[cfg(debug_assertions)]
        {
            Command::create_global_command(&ctx.http, commands::forceannounce::register())
                .await
                .expect("forceannounce command");
        }

        println!("commands registered successfully!");

        periodic::announcements::start(ctx.clone());
        periodic::reset_state::start(ctx.clone());
        periodic::status_manager::start(ctx.clone());
        
        #[cfg(debug_assertions)]
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
                "jsonembed" => {
                    with_timeout(async move {
                        commands::jsonembed::run(&ctx, cmd).await;
                    })
                    .await
                }
                "setnote" => {
                    with_timeout(async move {
                        commands::setnote::run(&ctx, cmd).await;
                    })
                    .await
                }
                "forceannounce" => {
                    with_timeout(async move {
                        commands::forceannounce::run(&ctx, cmd).await;
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

pub fn set_day_to_hr_min_sec(dt: DateTime<Local>, hr: u32, min: u32, sec: u32) -> Option<DateTime<Local>> {
    dt
        .with_hour(hr)?
        .with_minute(min)?
        .with_second(sec)
}

pub fn set_today_to_hr_min_sec(hr: u32, min: u32, sec: u32) -> DateTime<Local> {
    set_day_to_hr_min_sec(Local::now(), hr, min, sec).expect("invalid date time.")
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
