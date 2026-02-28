use crate::commands::util;
use crate::data::scheduled_meeting::ScheduleManager;
use crate::periodic::announcements::make_announcement;
use crate::{ANNOUNCEMENT_CHANNEL_ID};
use serenity::all::{
    ChannelId, CommandInteraction, Context
    ,
};
use serenity::builder::CreateCommand;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if !util::is_user_admin(&cmd.member).await {
        return;
    }

    let meet = ScheduleManager::get_closest_future_meeting().await.unwrap();
    make_announcement(ChannelId::new(ANNOUNCEMENT_CHANNEL_ID), ctx, &meet).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("forceannounce").description("Force an announcement (TESTING ONLY)")
}
