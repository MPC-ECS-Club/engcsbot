use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;

const INFO: &str = r#"
Made by <@329319801589596160>
Github: <https://github.com/MPC-ECS-Club/engcsbot>

If there is an issue with the bot, or you would like to submit feedback, send a message to <@329319801589596160> or open an issue on the github page.
"#;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let msg = CreateInteractionResponseMessage::new()
        .content(INFO)
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("info")
        .description("Retrieve information related to this discord bot")
}
