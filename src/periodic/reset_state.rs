use std::ops::Deref;
use chrono::Local;
use serenity::all::Context;

use crate::{UPDATE_RATE, data::{saveutil, scheduled_meeting::ScheduleManager}};
use crate::data::scheduled_meeting::ScheduledMeeting;

pub fn is_suspension_done(reset_timestamp: i64) -> bool {
    let now = Local::now().timestamp();

    reset_timestamp != -1 && now > reset_timestamp
}

// Returns whether or not the suspend.json file should be refreshed.
async fn reset_suspended_if_necessary(meeting: &ScheduledMeeting) -> bool {
    let time = ScheduleManager::get_suspension_restore_timestamp(meeting).await;

    let mut should_refresh = false;
    if is_suspension_done(time) {
        // maybe don't lock again, and just store the map?
        should_refresh = true;
        ScheduleManager::unsuspend(meeting).await;
    }

    should_refresh
}
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