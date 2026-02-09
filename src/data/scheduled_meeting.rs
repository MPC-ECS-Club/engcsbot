use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, MutexGuard};
use chrono::Weekday;
use serde::{Deserialize, Serialize};

// probably doesn't need to be Arc
static SCHEDULED: LazyLock<Arc<Mutex<Vec<ScheduledMeeting>>>> = LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SchedulingError {
    MeetingAlreadyExists,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct ScheduledMeeting {
    pub day: Weekday,
    pub location: String,
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub onetime: bool,
}
pub struct ScheduleManager;

impl ScheduleManager {
    pub async fn get_schedule() -> MutexGuard<'static, Vec<ScheduledMeeting>> {
        SCHEDULED.lock().await
    }

    pub async fn meeting_count() -> usize {
        Self::get_schedule().await.len()
    }

    pub async fn add_meeting(meeting: ScheduledMeeting) -> Result<(), SchedulingError> {
        let mut schedule = Self::get_schedule().await;
        if schedule.contains(&meeting) {
            return Err(SchedulingError::MeetingAlreadyExists);
        }

        schedule.push(meeting);
        
        Ok(())
    }

    pub async fn serialize_to_json() -> Result<String, serde_json::Error>{
        let schedule = Self::get_schedule().await;
        serde_json::to_string_pretty(schedule.deref())
    }

    pub async fn deserialize_from_json(json: &str) {
        let res: Vec<ScheduledMeeting> = serde_json::from_str(json).expect("failed to deserialize json");
        *Self::get_schedule().await = res;
    }
}