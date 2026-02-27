use crate::commands::util;
use chrono::Weekday;
use serenity::all::{
    CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, InteractionContext,
};
use std::str::FromStr;

use crate::data::saveutil;
use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting};
use crate::discord_log;

pub async fn run(ctx: &Context, cmd: CommandInteraction) {
    if !util::is_user_admin(&cmd.member).await {
        return;
    }

    let options = &cmd.data.options;

    let CommandDataOptionValue::String(day) = &options.first().expect("day").value else {
        return;
    };
    let CommandDataOptionValue::String(location) = &options.get(1).expect("location").value else {
        return;
    };
    let CommandDataOptionValue::String(start) = &options.get(2).expect("start").value else {
        return;
    };
    let CommandDataOptionValue::String(end) = &options.get(3).expect("end").value else {
        return;
    };
    let onetime = options
        .iter()
        .find(|o| o.name == "onetime")
        .is_some_and(|opt| opt.value.as_bool().unwrap_or(false));

    let Some((start_hour, start_minute)) = util::parse_time(start) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Please enter a valid start time!")
            .await;
        return;
    };

    let Some((end_hour, end_minute)) = util::parse_time(end) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Please enter a valid end time!").await;
        return;
    };

    let Ok(day) = Weekday::from_str(day.as_str()) else {
        _ = util::create_private_response(&cmd, &ctx.http, "Please enter a valid weekday!").await;
        return;
    };

    if (start_hour == end_hour && start_minute >= end_minute) || (start_hour > end_hour) {
        _ = util::create_private_response(
            &cmd,
            &ctx.http,
            "End time must be later than the start time.",
        )
        .await;
        return;
    }

    let sch = ScheduledMeeting {
        day,
        location: location.clone(),
        start: (start_hour, start_minute),
        end: (end_hour, end_minute),
        onetime,
        day_before_announced: false,
    };

    // if ScheduleManager::has_meeting(&sch).await {
    //     _ = util::create_private_response(&cmd, &ctx.http, "A meeting with the same specifications already exists!").await;
    //     return;
    // }

    match ScheduleManager::add_meeting(sch).await {
        Ok(_) => {
            _ = util::create_public_response(&cmd, &ctx.http, &format!("Scheduled a meeting for {day} at {start_hour:02}:{start_minute:02} until {end_hour:02}:{end_minute:02}. Location: {location}. Only this week? {onetime}")).await;
            if !onetime {
                saveutil::save_all_meetings().await;
            }
        }
        Err(why) => {
            _ = util::create_private_response(
                &cmd,
                &ctx.http,
                "Something went wrong while making this meeting. You cannot schedule a meeting that already exists.",
            )
            .await;
            discord_log!(&ctx.http, "failed to create meeting: {why:?}");
        }
    }

    // current way to do modals is kinda depercated, waiting until next version of serenity to use them
    // let f = async move || {
    //     let day_of_the_week = CreateActionRow::SelectMenu(
    //         CreateSelectMenu::new("0", CreateSelectMenuKind::String {
    //             options: vec![
    //                 CreateSelectMenuOption::new("a", "b"),
    //                 CreateSelectMenuOption::new("c", "d"),
    //             ],
    //         })
    //     );
    //
    //     let modal_id = cmd.id.get().to_string();
    //     let modal = CreateModal::new(&modal_id, "Schedule a Meeting")
    //         .components(vec![day_of_the_week]);
    //
    //     let builder = CreateInteractionResponse::Modal(modal);
    //
    //     println!("before");
    //     builder.execute(&ctx.http, (cmd.id, &cmd.token)).await?;
    //
    //     println!("here");
    //
    //     let collector = ModalInteractionCollector::new(&ctx.shard)
    //         .custom_ids(vec![modal_id])
    //         .timeout(Duration::from_secs(120));
    //     println!("after");
    //
    //     let modal_interaction = collector.next().await;
    //     let Some(modal_interaction) = modal_interaction else { return Ok(()) };
    //
    //     println!("data: {:?}", modal_interaction.data);
    //
    //     modal_interaction
    //         .create_response(
    //             ctx,
    //             CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("hi").ephemeral(true))
    //         ).await?;
    //
    //     Ok::<(), serenity::Error>(())
    // };
    //
    // if let Err(why) = f().await {
    //     println!("error ocurred while running schedule command: {why:?}");
    // }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("schedule")
        .description("Schedule a new meeting")
        .add_context(InteractionContext::Guild)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "day",
                "Day of the week (monday, tuesday, ...)",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "location",
                "Where the meeting takes place (BMC 204, STEM Center, ...)",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "start",
                "Time of day (12:00pm, 1:30pm, 2:00pm)",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "end",
                "Time of day (12:00pm, 1:30pm, 2:00pm)",
            )
            .required(true),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "onetime",
            "If this is true, this meeting will only be scheduled for this week.",
        ))
}
