use serenity::all::{CommandInteraction, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, UserId};
use std::process::exit;

// just in case the bot needs to be shutdown remotely

const REV_ID: UserId = UserId::new(329319801589596160);

async fn shutdown(ctx: &Context, cmd: CommandInteraction) {
    let msg = CreateInteractionResponseMessage::new()
        .content("Shutting down!")
        .ephemeral(true);

    let builder = CreateInteractionResponse::Message(msg);

    _ = cmd.create_response(&ctx.http, builder).await;

    if let Err(why) = REV_ID.dm(&ctx.http, CreateMessage::new().content("The bot was forcefully shutdown.")).await {
        eprintln!("failed to send dm to rev: {why:?}");
    }

    println!("SHUTTING DOWN!");

    ctx.shard.shutdown_clean();
    println!("Goodnight!");

    exit(1);
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if cmd.user.id == REV_ID {
        shutdown(ctx, cmd).await;
    } else if let Some(guild_id) = cmd.guild_id {
        if let Ok(member) = guild_id.member(&ctx.http, &cmd.user).await {
            if let Some(permissions) = member.permissions {
                if permissions.administrator() {
                    shutdown(ctx, cmd).await;
                }
            }
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("shutdown")
        .description("Shutdown the bot if something goes wrong.")
}