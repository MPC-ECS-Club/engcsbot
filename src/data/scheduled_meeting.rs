use chrono::{DateTime, Datelike, Duration, Local, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Add, Deref, Sub};
use std::sync::LazyLock;
use tokio::sync::{Mutex, MutexGuard};

static SCHEDULED: LazyLock<Mutex<Vec<ScheduledMeeting>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// probably not the best way to do this, but who cares.
// mapping from Meeting => unix timestamp of when it should be rescheduled.
static TEMPORARILY_SUSPENDED: LazyLock<Mutex<HashMap<ScheduledMeeting, Suspended>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
enum SuspendReason {
    AlreadyAnnounced,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
struct Suspended {
    reason: SuspendReason,
    reschedule: i64,
}

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

impl ScheduledMeeting {
    pub fn get_datetime_of_next(&self) -> DateTime<Local> {
        let now = Local::now();

        let today = now.weekday().num_days_from_monday();
        let meeting = self.day.num_days_from_monday();

        let diff = if meeting > today {
            meeting - today
        } else {
            7 + meeting - today
        };

        now.add(chrono::Duration::days(diff as i64))
            .with_hour(self.start.0)
            .unwrap()
            .with_minute(self.start.1)
            .unwrap()
            .with_second(0)
            .unwrap()
    }
}

pub struct ScheduleManager;

impl ScheduleManager {
    pub async fn get_schedule() -> MutexGuard<'static, Vec<ScheduledMeeting>> {
        SCHEDULED.lock().await
    }

    pub async fn set_already_announced(meeting: ScheduledMeeting, restore_at: i64) {
        TEMPORARILY_SUSPENDED.lock().await.insert(
            meeting,
            Suspended {
                reason: SuspendReason::AlreadyAnnounced,
                reschedule: restore_at,
            },
        );
    }

    pub async fn is_already_announced(meeting: &ScheduledMeeting) -> bool {
        TEMPORARILY_SUSPENDED.lock().await.contains_key(meeting)
    }

    pub async fn get_announced_reset_timestamp(meeting: &ScheduledMeeting) -> i64 {
        TEMPORARILY_SUSPENDED
            .lock()
            .await
            .get(meeting)
            .map(|v| v.reschedule)
            .unwrap_or(-1)
    }

    pub async fn cancel_meeting(meeting: ScheduledMeeting) -> DateTime<Local> {
        // ensure that the meeting is announced so we shave off a bit off time from the suspension
        let when = meeting.get_datetime_of_next().with_hour(0).unwrap();

        TEMPORARILY_SUSPENDED.lock().await.insert(
            meeting,
            Suspended {
                reason: SuspendReason::Cancelled,
                reschedule: when.timestamp(),
            },
        );

        when
    }

    pub async fn is_meeting_cancelled(meeting: &ScheduledMeeting) -> bool {
        TEMPORARILY_SUSPENDED
            .lock()
            .await
            .get(meeting)
            .is_some_and(|val| val.reason == SuspendReason::Cancelled)
    }

    pub async fn reset_announced_state(meeting: &ScheduledMeeting) {
        TEMPORARILY_SUSPENDED.lock().await.remove(meeting);
    }

    pub async fn remove_meeting(meeting: &ScheduledMeeting) {
        let mut sch = Self::get_schedule().await;
        if let Some((i, _)) = sch.iter().enumerate().find(|(_, m)| *m == meeting) {
            sch.swap_remove(i);
        } else {
            println!("attempted to remove meeting that wasn't registered.")
        }

        TEMPORARILY_SUSPENDED.lock().await.remove(meeting);
    }

    pub async fn meeting_count() -> usize {
        Self::get_schedule().await.len()
    }

    pub async fn has_meeting(meeting: &ScheduledMeeting) -> bool {
        SCHEDULED.lock().await.contains(meeting)
    }

    pub async fn add_meeting(meeting: ScheduledMeeting) -> Result<(), SchedulingError> {
        let mut schedule = SCHEDULED.lock().await;
        if schedule.contains(&meeting) {
            return Err(SchedulingError::MeetingAlreadyExists);
        }

        schedule.push(meeting);

        Ok(())
    }

    pub async fn serialize_to_json() -> Result<String, serde_json::Error> {
        let schedule = Self::get_schedule().await;
        serde_json::to_string_pretty(schedule.deref())
    }

    pub async fn deserialize_from_json(json: &str) {
        let res: Vec<ScheduledMeeting> =
            serde_json::from_str(json).expect("failed to deserialize json");
        *Self::get_schedule().await = res;
    }
}
