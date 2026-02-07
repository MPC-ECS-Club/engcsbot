use serenity::all::{Color, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue};

use crate::commands::util;


pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if !util::is_user_admin(&cmd.member).await { 
        println!("no permission to make announcement for {}", cmd.user.name);
        return; 
    }

    println!("attempting to send announcement from: {}", cmd.user.name);
    let options = cmd.data.options();

    if let Some(ResolvedOption { value: ResolvedValue::String(title), .. }) = options.get(0) {
        if let Some(ResolvedOption { value: ResolvedValue::String(description), .. }) = options.get(1) {
            let mut msg = CreateEmbed::new()
                .author(CreateEmbedAuthor::new(&cmd.user.name).icon_url(&cmd.user.avatar_url().unwrap_or("".into())))
                .color(Color::BLUE)
                .title(*title)
                .description(*description);

            if let Some(ResolvedOption { value: ResolvedValue::String(foot), .. }) = options.get(2) {
                msg = msg.footer(CreateEmbedFooter::new(*foot));
            }

    
            if let Err(why) = cmd.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().add_embed(msg))).await {
                println!("failed to create response: {why:?}");
            }
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("announce")
        .description("Make a nicely formatted announcement.")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "title", "title for the announcement")
                .required(true)
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "desc", "the description of the announcement")
                .required(true)
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "footer", "footer for the announcement")
        )
}
