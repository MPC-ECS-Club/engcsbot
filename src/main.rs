mod commands;

use chrono::{DateTime, Datelike, Local, Timelike, Weekday};
use serenity::all::{ActivityData, Color, Command, CreateEmbedFooter, CreateMessage, Interaction, OnlineStatus, ReactionType, Ready};
use serenity::builder::CreateEmbed;
use serenity::{all::{ChannelId, Message}, async_trait, prelude::*};
use std::time::Duration;

const ANNOUNCEMENT_OFFSET_MINS: u32 = 0;

#[cfg(not(debug_assertions))]
const MEETING_HOUR: u32 = 12;

const MEETING_END: u32 = 14;

const STATUSES: &[&str] = &["engineering...", "programming...", "procrastinating..."];
const STATUS_TIME: Duration = Duration::from_mins(2);

#[cfg(debug_assertions)]
const MEETING_HOUR: u32 = 14;

#[cfg(debug_assertions)]
const ANNOUNCEMENT_CHANNEL_ID: u64 = 839277529511755786;

#[cfg(not(debug_assertions))]
const ANNOUNCEMENT_CHANNEL_ID: u64 = 1153591616301432834;

#[cfg(debug_assertions)]
const UPDATE_RATE: Duration = Duration::from_secs(1);

#[cfg(not(debug_assertions))]
const UPDATE_RATE: Duration = Duration::from_mins(1);

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

        _ => "⏰️"
    }
}

struct Handler;

// TODO: cleanup
async fn start_time_checking_loop(ctx: Context) {
    let chan = ChannelId::new(ANNOUNCEMENT_CHANNEL_ID);
    loop {
        tokio::time::sleep(UPDATE_RATE).await;
        let dt = Local::now();

        // todo, perhaps add a config that can be configured from within discord? (using modals perhaps?)
        let weekday = dt.weekday();
        if ![Weekday::Fri, Weekday::Sat].contains(&weekday) { continue; }

        let desired_location = if weekday == Weekday::Fri {
            "BMC 204"
        } else {
            "STEM Center (1st floor library)"
        };
        if dt.hour() == (MEETING_HOUR - 1) { // FIXME!!! potential underflow!!!
            if dt.minute() >= ANNOUNCEMENT_OFFSET_MINS {
                let meet_time = get_meeting_time_for_today();
                let seconds_since_epoch = meet_time.timestamp();

                let meeting_end_epoch_time = get_end_meeting_time_for_today().timestamp();

                let msg = CreateMessage::new()
                    .content("@everyone")
                    .embed(CreateEmbed::new()
                        .title("🎉 Meeting Alert 🚨")
                        .description(format!("There will be a meeting today <t:{seconds_since_epoch}:R>"))
                        .color(Color::DARK_GREEN)
                        .field("Location 🪐", desired_location, true)
                        .field(format!("Until {}", get_clock_emoji_for_hour(MEETING_END)), format!("<t:{meeting_end_epoch_time}:t>"), true)
                        .footer(CreateEmbedFooter::new("Please react to this message if you plan on attending!\nNote this message was automated, and if a previous agreed upon arrangement for the meeting was made (such as date, time, location, or entirely canceled, please disregard this message.)"))
                    );


                if let Ok(msg) = chan.send_message(&ctx.http, msg).await {
                    if let Err(why) = msg.react(&ctx.http, ReactionType::Unicode("\u{2705}".into())).await {
                        println!("failed to send message: {why:?}");
                    }
                }

                println!("sleeping for 12 hours... gn!"); // lmao, probably not the best way to be doing this
                tokio::time::sleep(Duration::from_hours(12)).await;
            }
        }

    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _msg: Message) {
        
    }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        println!("connected to {}", data_about_bot.user.name);

        

        Command::create_global_command(&ctx.http, commands::info::register()).await.expect("info command");
        Command::create_global_command(&ctx.http, commands::shutdown::register()).await.expect("shutdown command");
        Command::create_global_command(&ctx.http, commands::announce::register()).await.expect("announce command");

        {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                start_time_checking_loop(ctx).await;
            });
        }

        #[cfg(debug_assertions)]
        {
            ctx.set_presence(Some(ActivityData::playing("debug mode")), OnlineStatus::Online);
        } #[cfg(not(debug_assertions))]
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


        println!("ready!");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            match cmd.data.name.as_str() {
                "info" => commands::info::run(&ctx, cmd).await,
                "shutdown" => commands::shutdown::run(&ctx, cmd).await,
                "announce" => commands::announce::run(&ctx, cmd).await,

                _ => println!("called unimplemented cmd"),
            };
        }
    }
}

fn get_meeting_time_for_today() -> DateTime<Local> {
    let dt = Local::now();

    dt
        .with_hour(MEETING_HOUR).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap()
}

fn get_end_meeting_time_for_today() -> DateTime<Local> {
    let dt = Local::now();

    dt
        .with_hour(MEETING_END).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap()
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Missing discord bot token environment variable");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::DIRECT_MESSAGES;

    let mut client = 
        Client::builder(&token, intents).event_handler(Handler)
            .await
            .expect("Error creating client.");

    if let Err(why) = client.start().await {
        println!("client error: {why:?}");
    }
}