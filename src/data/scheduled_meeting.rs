use chrono::Weekday;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, MutexGuard};

// probably doesn't need to be Arc
static SCHEDULED: LazyLock<Arc<Mutex<Vec<ScheduledMeeting>>>> = LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

// probably not the best way to do this, but who cares.
// mapping from Meeting => unix timestamp of when it should be rescheduled.
static ALREADY_ANNOUNCED: LazyLock<Mutex<HashMap<ScheduledMeeting, i64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SchedulingError {
    MeetingAlreadyExists,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
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

    pub async fn set_already_announced(meeting: ScheduledMeeting, restore_at: i64) {
        ALREADY_ANNOUNCED.lock().await.insert(meeting, restore_at);
    }

    pub async fn is_already_announced(meeting: &ScheduledMeeting) -> bool {
        ALREADY_ANNOUNCED.lock().await.contains_key(meeting)
    }

    pub async fn get_announced_reset_timestamp(meeting: &ScheduledMeeting) -> i64 {
        *ALREADY_ANNOUNCED.lock().await.get(meeting).unwrap_or(&-1)
    }

    pub async fn reset_announced_state(meeting: &ScheduledMeeting) {
        ALREADY_ANNOUNCED.lock().await.remove(meeting);
    }

    pub async fn remove_meeting(meeting: &ScheduledMeeting) {
        let mut sch = Self::get_schedule().await;
        if let Some((i, _)) = sch.iter()
            .enumerate()
            .find(|(_, m)| *m == meeting) {

            sch.remove(i);
        }

        ALREADY_ANNOUNCED.lock().await.remove(meeting);
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