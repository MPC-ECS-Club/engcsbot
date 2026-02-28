use std::ops::Deref;

use serenity::all::Context;

use crate::{UPDATE_RATE, data::{saveutil, scheduled_meeting::ScheduleManager}, reset_suspended_if_necessary};


async fn reset_announced_state() {
    tokio::time::sleep(UPDATE_RATE.div_f64(2.0)).await; // offset from  regular update rate

    loop {
        tokio::time::sleep(UPDATE_RATE).await;

        for meeting in ScheduleManager::get_schedule().await.deref() {
            let should_refresh = reset_suspended_if_necessary(meeting).await;
            if should_refresh {
                println!("refreshing suspend.json");
                saveutil::save_suspended().await;
            }
        }
    }
}

async fn run(_ctx: Context) {
    reset_announced_state().await;
}

pub fn start(ctx: Context) {
    tokio::spawn(run(ctx));
}