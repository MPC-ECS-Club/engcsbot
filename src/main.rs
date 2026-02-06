use chrono::{Datelike, Local};
use serenity::{all::{ChannelId, Message}, async_trait, prelude::*};


async fn send_message(chan: &ChannelId, http: impl CacheHttp, msg: impl Into<String>) {
    if let Err(err) = chan.say(http, msg).await {
        eprintln!("failed to send message: {err:?}");
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            send_message(&msg.channel_id, &ctx.http, "Pong!").await;
        } else if msg.content == "!day" {
            let now = Local::now();
            let day = now.weekday();
            let epoch = now.timestamp_millis();
            
            send_message(&msg.channel_id, &ctx.http, format!("Today is {day} <t:{epoch}>")).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Missing discord bot token environment variable");
    let intents = GatewayIntents::GUILD_MESSAGES;

    let mut client = 
        Client::builder(&token, intents).event_handler(Handler)
            .await
            .expect("Error creating client.");

    if let Err(why) = client.start().await {
        eprintln!("client error: {why:?}");
    }
}

// #[cfg(test)]
// mod tests {
//     use chrono::{Datelike, Local, NaiveDate, Weekday};

//     #[test]
//     fn datetime() {
//         let fri = NaiveDate::from_weekday_of_month_opt(2026, 2, Weekday::Fri, 1).expect("hmm");

//         let dt = Local::now();
        
//         let today = dt.weekday();
//         let timestamp = dt.timestamp_millis();
//         println!("hi: {fri}, today is a {today}");
        
        
//         assert!(false);
//     }
// }