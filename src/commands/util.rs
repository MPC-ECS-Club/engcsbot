use std::str::FromStr;
use chrono::Weekday;
use serenity::all::{CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage, Member};
use serenity::http::CacheHttp;

pub async fn is_user_admin(member: &Option<Box<Member>>) -> bool {
    if let Some(member) = member {
        if let Some(permissions) = member.permissions {
            return permissions.administrator();
        }
    }

    false
}

// This function returns 24 hour time. First hour, then minute.
pub fn parse_time(val: &str) -> Option<(u32, u32)> {
    let is_pm = val.ends_with("pm");
    if val.ends_with("am") || is_pm {
        let res: Vec<&str> = val.split(":").collect();
        if res.len() != 2 { return None; }

        let hour: u32 = res[0].parse().ok()?;
        let minute: u32 = res[1].trim_end_matches(&['a', 'p', 'm']).parse().ok()?;


        if (hour < 1 || hour > 12 || minute > 60) { return None; }
        let hour = if is_pm { if hour != 12 { hour + 12 } else { hour } } else { hour % 12 };

        Some((hour, minute))
    } else {

        None
    }
}

pub async fn create_private_response(cmd: &CommandInteraction, http: impl CacheHttp, message: &str) -> Result<(), serenity::Error> {
    if let Err(why) = cmd.create_response(&http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(message).ephemeral(true))).await {
        println!("Error sending message '{message}' because: {why:?}");
        Err(why)
    } else {
        Ok(())
    }
}
// duplicated code
pub async fn create_public_response(cmd: &CommandInteraction, http: impl CacheHttp, message: &str) -> Result<(), serenity::Error> {
    if let Err(why) = cmd.create_response(&http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(message))).await {
        println!("Error sending message '{message}' because: {why:?}");
        Err(why)
    } else {
        Ok(())
    }
}
