use std::ops::Deref;
use crate::{MEETING_JSON_PATH, SUSPENDED_JSON_PATH};
use crate::data::scheduled_meeting::{ScheduleManager, ScheduledMeeting, Suspended};

pub async fn save_all_meetings() {
    let json = ScheduleManager::serialize_to_json()
        .await
        .expect("failed to serialize data");
    tokio::fs::write(MEETING_JSON_PATH, json)
        .await
        .expect("failed to create file");
}

pub async fn save_suspended() {
    let suspended = ScheduleManager::get_suspension_map().await;
    
    let mut pairs: Vec<(ScheduledMeeting, Suspended)> = Vec::with_capacity(suspended.len());
    
    for (meet, sus) in suspended.deref() {
        pairs.push((meet.clone(), sus.clone()));
    }
    
    let json = serde_json::to_string(&pairs).expect("failed to serialize suspended");
    tokio::fs::write(SUSPENDED_JSON_PATH, json)
        .await
        .expect("failed to save to suspended.json");
}
