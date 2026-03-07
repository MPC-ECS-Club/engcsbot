use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;

const DEVELOPER_PING: &str = "<@329319801589596160>";

fn get_info() -> String {
    format!(r#"
EngCS Bot {}

Made by {DEVELOPER_PING}
Github: <https://github.com/MPC-ECS-Club/engcsbot>

If there is an issue with the bot, or you would like to submit feedback, send a message to {DEVELOPER_PING} or open an issue on the github page.
"#, env!("CARGO_PKG_VERSION"))
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let msg = CreateInteractionResponseMessage::new()
        .content(get_info())
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("info").description("Retrieve information related to this discord bot")
}
