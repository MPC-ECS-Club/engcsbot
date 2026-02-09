use crate::commands::util;
use crate::{discord_log, ClientShardManager};
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, UserId};

// just in case the bot needs to be shutdown remotely

const REV_ID: UserId = UserId::new(329319801589596160);

async fn shutdown(ctx: &Context, cmd: CommandInteraction) {
    let msg = CreateInteractionResponseMessage::new()
        .content("Shutting down!")
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;

    discord_log!(&ctx.http, "A user ({}, id={}) requested the bot to be shutdown.", cmd.user.name, cmd.user.id);

    println!("SHUTTING DOWN!");

    let data = ctx.data.read().await;
    let shard_manager = data.get::<ClientShardManager>().unwrap();
    shard_manager.shutdown_all().await;

    println!("Goodnight!");
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if cmd.user.id == REV_ID || util::is_user_admin(&cmd.member).await {
        shutdown(ctx, cmd).await;
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("shutdown")
        .description("Shutdown the bot if something goes wrong.")
}