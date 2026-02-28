use crate::commands::util;
use crate::data::saveutil;
use crate::data::scheduled_meeting::ScheduleManager;
use crate::discord_log;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;
use uuid::Uuid;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let options = &cmd.data.options;

    let Some(note) = &options.first().iter().flat_map(|value| value.value.as_str()).next() else {
        return;
    };
    let uuid = &options
        .iter()
        .find(|c| c.name == "uuid")
        .iter()
        .flat_map(|u| u.value.as_str())
        .next();

    let desired_meeting = if let Some(uuid) = uuid {
        let u = Uuid::parse_str(uuid);
        match u {
            Ok(uuid) => ScheduleManager::get_by_uuid(uuid).await,
            Err(why) => {
                _ = util::create_private_response(&cmd, &ctx.http, "Invalid UUID provided.").await;
                discord_log!(&ctx.http, "failed to parse uuid: {why}");
                return;
            }
        }
    } else {
        ScheduleManager::get_closest_future_meeting().await
    };

    if let Some(meeting) = desired_meeting {
        ScheduleManager::set_note(meeting.uuid, note.to_string()).await;

        let msg = CreateInteractionResponseMessage::new()
            .content("Note added successfully.")
            .ephemeral(true);

        let builder = CreateInteractionResponse::Message(msg);

        _ = cmd.create_response(&ctx.http, builder).await;

        saveutil::save_all_meetings().await;
    } else {
        _ = util::create_private_response(&cmd, &ctx.http, "There was no meeting found.").await;
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("setnote")
        .description("Set a note for a particular meeting. Will be displayed when announced. This note is not permenant")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "note", "The description of the meeting. This note is temporary and will be removed after it is announced.").required(true))
        .add_option(CreateCommandOption::new(CommandOptionType::String, "uuid", "The UUID of the meeting you wish to add this note to. This defaults to the next meeting.").required(false))
}
