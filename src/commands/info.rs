use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;

const INFO: &str = r#"
Made by <@329319801589596160>
Github: (not public yet)
"#;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let msg = CreateInteractionResponseMessage::new()
        .content(INFO)
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("info").description("Retrieve information related to this discord bot")
}
