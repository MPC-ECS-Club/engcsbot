use serenity::all::{
    Color, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, InteractionContext, Mentionable, ResolvedOption,
    ResolvedValue,
};

use crate::commands::util;
use crate::discord_log;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if !util::is_user_admin(&cmd.member).await {
        println!("no permission to make announcement for {}", cmd.user.name);
        return;
    }

    let options = cmd.data.options();

    if let Some(ResolvedOption {
        value: ResolvedValue::String(title),
        ..
    }) = options.first()
        && let Some(ResolvedOption {
            value: ResolvedValue::String(description),
            ..
        }) = options.get(1)
    {
        let mut msg = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&cmd.user.name)
                    .icon_url(cmd.user.avatar_url().unwrap_or("".into())),
            )
            .color(Color::BLUE)
            .title(*title)
            .description(*description);

        if let Some(ResolvedOption {
            value: ResolvedValue::String(foot),
            ..
        }) = options.iter().find(|a| a.name == "footer")
        {
            msg = msg.footer(CreateEmbedFooter::new(*foot));
        }

        let mut msg_content = "".to_string();
        if let Some(ResolvedOption {
            value: ResolvedValue::Role(mention),
            ..
        }) = options.iter().find(|a| a.name == "mention")
        {
            msg_content = format!("{}", mention.mention());
        }

        if let Err(why) = cmd
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(msg_content)
                        .add_embed(msg),
                ),
            )
            .await
        {
            discord_log!(
                &ctx.http,
                "failed to create interaction response while making announcement: {why:?}"
            );
        }
    } else {
        discord_log!(
            &ctx.http,
            "announcement command was missing title or description "
        );
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("announce")
        .description("Make a nicely formatted announcement.")
        .add_context(InteractionContext::Guild)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "title",
                "title for the announcement",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "desc",
                "the description of the announcement",
            )
            .required(true),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "footer",
            "footer for the announcement",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Role,
            "mention",
            "who to mention for the announcement",
        ))
}
