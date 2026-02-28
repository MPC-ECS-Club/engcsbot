use crate::data::saveutil;
use chrono::{DateTime, Datelike, Local, Timelike, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Add, Deref, DerefMut};
use std::sync::LazyLock;
use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;

static SCHEDULED: LazyLock<Mutex<Vec<ScheduledMeeting>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// probably not the best way to do this, but who cares.
// mapping from Meeting => unix timestamp of when it should be rescheduled.
static TEMPORARILY_SUSPENDED: LazyLock<Mutex<HashMap<Uuid, Suspended>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SuspendReason {
    AlreadyAnnounced,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Suspended {
    pub reason: SuspendReason,
    pub reschedule: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SchedulingError {
    MeetingAlreadyExists,
}

// Note: This struct is serialized using serde_json! Be careful when renaming fields, since that will break old save files from being loaded.
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ScheduledMeeting {
    pub uuid: Uuid,
    pub day: Weekday,
    pub location: String,
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub onetime: bool,
    pub day_before_announced: bool,
    pub note: Option<String>,
}

impl ScheduledMeeting {
    pub fn get_datetime_of_next(&self) -> DateTime<Local> {
        let now = Local::now();

        let today = now.weekday().num_days_from_monday();
        let meeting = self.day.num_days_from_monday();

        let diff = if meeting >= today {
            meeting - today
        } else {
            7 + meeting - today
        };

        let desired = now
            .add(chrono::Duration::days(diff as i64))
            .with_hour(self.start.0)
            .unwrap()
            .with_minute(self.start.1)
            .unwrap()
            .with_second(0)
            .unwrap();

        // earlier we calculated the 'diff' by checking if meeting >= today
        // if the diff is 0, the next meeting is today, we want to check if it was earlier today
        // or later today.
        if now > desired {
            // if it was earlier today
            desired.add(chrono::Duration::days(7)) // then we are probably talking about next week's meeting
        } else {
            desired // cancel today's meeting.
        }
    }
}

pub struct ScheduleManager;

impl ScheduleManager {
    pub async fn set_note(u: Uuid, s: String) {
        let mut schedule = Self::get_schedule().await;

        let meet = schedule.deref_mut().iter_mut().find(|x| x.uuid == u);
        if let Some(meet) = meet {
            meet.note = Some(s);
        }
    }

    pub async fn get_by_uuid(u: Uuid) -> Option<ScheduledMeeting> {
        // TODO: use a hashmap internally, although there aren't that many meetings in general so whatever
        Self::get_schedule()
            .await
            .iter()
            .find(|s| s.uuid == u)
            .cloned()
    }

    pub async fn get_closest_future_meeting() -> Option<ScheduledMeeting> {
        let meeting = Self::get_schedule().await;

        meeting
            .iter()
            .reduce(|a, b| {
                if a.get_datetime_of_next() < b.get_datetime_of_next() {
                    a
                } else {
                    b
                }
            })
            .cloned()
    }

    pub async fn remove_matching(
        day: Weekday,
        start: (u32, u32),
        end: (u32, u32),
        onetime: bool,
    ) -> usize {
        let mut schedule = ScheduleManager::get_schedule().await;
        let mut temp_sus = ScheduleManager::get_suspension_map().await;

        let meetings_to_remove: Vec<(usize, ScheduledMeeting)> = schedule
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.day == day && m.start == start && m.end == end && m.onetime == onetime
            })
            .map(|(i, m)| (i, m.clone()))
            .collect();

        let total = meetings_to_remove.len();

        meetings_to_remove.iter().rev().for_each(|(idx, meeting)| {
            temp_sus.remove(&meeting.uuid);
            schedule.swap_remove(*idx);
        });

        total
    }

    pub async fn get_schedule() -> MutexGuard<'static, Vec<ScheduledMeeting>> {
        SCHEDULED.lock().await
    }

    pub async fn set_already_announced(meeting: &ScheduledMeeting, restore_at: i64) {
        TEMPORARILY_SUSPENDED.lock().await.insert(
            meeting.uuid,
            Suspended {
                reason: SuspendReason::AlreadyAnnounced,
                reschedule: restore_at,
            },
        );
    }

    pub async fn is_already_announced(meeting: &ScheduledMeeting) -> bool {
        TEMPORARILY_SUSPENDED
            .lock()
            .await
            .contains_key(&meeting.uuid)
    }

    pub async fn get_suspension_restore_timestamp(meeting: &ScheduledMeeting) -> i64 {
        TEMPORARILY_SUSPENDED
            .lock()
            .await
            .get(&meeting.uuid)
            .map(|v| v.reschedule)
            .unwrap_or(-1)
    }

    pub async fn get_suspension_map() -> MutexGuard<'static, HashMap<Uuid, Suspended>> {
        TEMPORARILY_SUSPENDED.lock().await
    }

    pub async fn cancel_meeting(meeting: ScheduledMeeting) -> DateTime<Local> {
        // ensure that the meeting is announced so we shave off a bit off time from the suspension
        let next_time = meeting.get_datetime_of_next();
        let when = next_time
            .add(chrono::Duration::days(1))
            .with_hour(0)
            .unwrap();

        TEMPORARILY_SUSPENDED.lock().await.insert(
            meeting.uuid,
            Suspended {
                reason: SuspendReason::Cancelled,
                reschedule: when.timestamp(),
            },
        );

        saveutil::save_suspended().await;

        when
    }

    pub async fn is_meeting_cancelled(meeting: &ScheduledMeeting) -> bool {
        TEMPORARILY_SUSPENDED
            .lock()
            .await
            .get(&meeting.uuid)
            .is_some_and(|val| val.reason == SuspendReason::Cancelled)
    }

    pub async fn unsuspend(meeting: &ScheduledMeeting) {
        TEMPORARILY_SUSPENDED.lock().await.remove(&meeting.uuid);
    }

    // pub async fn remove_meeting(meeting: &ScheduledMeeting) {
    //     let mut sch = Self::get_schedule().await;
    //     if let Some((i, _)) = sch.iter().enumerate().find(|(_, m)| *m == meeting) {
    //         sch.swap_remove(i);
    //     } else {
    //         println!("attempted to remove meeting that wasn't registered.")
    //     }
    //
    //     TEMPORARILY_SUSPENDED.lock().await.remove(meeting);
    // }

    // pub async fn has_meeting(meeting: &ScheduledMeeting) -> bool {
    //     SCHEDULED.lock().await.contains(meeting)
    // }

    pub async fn meeting_count() -> usize {
        Self::get_schedule().await.len()
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
