use crate::data::scheduled_meeting::ScheduleManager;
use crate::MEETING_JSON_PATH;

pub async fn save_all() {
    let json = ScheduleManager::serialize_to_json().await.expect("failed to serialize data");
    tokio::fs::write(MEETING_JSON_PATH, json).await.expect("failed to create file");
}
