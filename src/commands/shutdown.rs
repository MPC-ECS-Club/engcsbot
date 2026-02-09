use serenity::all::{CommandInteraction, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, UserId};
use std::process::exit;
use crate::ClientShardManager;
use crate::commands::util;

// just in case the bot needs to be shutdown remotely

const REV_ID: UserId = UserId::new(329319801589596160);

async fn shutdown(ctx: &Context, cmd: CommandInteraction) {
    let msg = CreateInteractionResponseMessage::new()
        .content("Shutting down!")
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;

    if let Err(why) = REV_ID.dm(&ctx.http, CreateMessage::new().content("The bot was forcefully shutdown.")).await {
        println!("failed to send dm to rev: {why:?}");
    }

    println!("SHUTTING DOWN!");

    // ctx.shard.shutdown_clean();
    let data = ctx.data.read().await;
    let shard_manager = data.get::<ClientShardManager>().unwrap();
    shard_manager.shutdown_all().await;

    println!("Goodnight!");
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if cmd.user.id == REV_ID {
        shutdown(ctx, cmd).await;
    } else if util::is_user_admin(&cmd.member).await {
        shutdown(ctx, cmd).await;
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("shutdown")
        .description("Shutdown the bot if something goes wrong.")
}