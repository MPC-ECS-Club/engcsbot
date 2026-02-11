use crate::commands::util;
use crate::discord_log;
use serde::Deserialize;
use serenity::all::{
    Color, CommandInteraction, CommandOptionType, Context, CreateCommandOption, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::builder::CreateCommand;
// NOTE: potentially unsafe command since I'm deserializing untrusted input... look into further

#[derive(Debug, Clone, Deserialize)]
struct JsonEmbedFooter {
    text: String,
    icon_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonEmbedField {
    title: String,
    description: String,
    inline: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonEmbed {
    title: String,
    description: Option<String>,
    fields: Vec<JsonEmbedField>,
    color: Option<Color>,
    footer: Option<JsonEmbedFooter>,
    thumbnail: Option<String>,
    image: Option<String>,
}

impl JsonEmbed {
    fn build_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::new().title(self.title.clone());

        if let Some(color) = self.color {
            embed = embed.color(color);
        }

        if let Some(desc) = &self.description {
            embed = embed.description(desc);
        }

        if let Some(thumbnail) = &self.thumbnail {
            embed = embed.thumbnail(thumbnail);
        }

        if let Some(image) = &self.image {
            embed = embed.image(image);
        }

        if let Some(footer) = &self.footer {
            let mut embed_footer = CreateEmbedFooter::new(footer.text.clone());
            if let Some(icon_url) = &footer.icon_url {
                embed_footer = embed_footer.icon_url(icon_url);
            }

            embed = embed.footer(embed_footer);
        }

        for field in &self.fields {
            embed = embed.field(
                field.title.clone(),
                field.description.clone(),
                field.inline.unwrap_or(false),
            );
        }

        embed
    }
}

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    let options = &cmd.data.options;

    let Some(Some(json)) = options.first().map(|o| o.value.as_str()) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Invalid input").await;
        return;
    };

    if json.len() > 700 {
        _ = util::create_private_response(&cmd, &ctx.http, "Too long!").await;
        return;
    }

    match serde_json::from_str::<JsonEmbed>(json) {
        Ok(embed) => {
            // log the json in case a vulnerability is found
            discord_log!(
                &ctx.http,
                "A successful json embed has been sent\n```json\n{json}```"
            );

            let msg = CreateInteractionResponseMessage::new().embed(embed.build_embed());

            let builder = CreateInteractionResponse::Message(msg);
            _ = cmd.create_response(&ctx.http, builder).await;
        }
        Err(why) => {
            _ = util::create_private_response(
                &cmd,
                &ctx.http,
                &format!("failed to parse json: {why:?}"),
            )
            .await;
        }
    };
}

pub fn register() -> CreateCommand {
    CreateCommand::new("jsonembed")
        .description("Create a custom embed using JSON.")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "json",
                "The provided json to parse.",
            )
            .required(true),
        )
}
