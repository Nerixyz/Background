use std::sync::{Arc, RwLock, Weak};

use dwd_fetch::RadarReading;
use jiff::{SignedDuration, tz::TimeZone};

/// Max interval without rain where different streaks absorb each other.
const ABSORB_UNTIL_MIN: i64 = 15;
/// Minutes to notify before rain is expected.
const NOTIFY_BEFORE_MIN: i64 = 7;

#[derive(Clone)]
pub struct NotifyHandle {
    state: Arc<RwLock<NotifyState>>,
}

#[derive(Clone)]
pub struct WeakNotifyHandle {
    state: Weak<RwLock<NotifyState>>,
}

impl NotifyHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(NotifyState::new())),
        }
    }

    pub fn tick(&self, radar: &[RadarReading]) {
        let Ok(mut v) = self.state.write() else {
            return;
        };
        v.tick(radar);
    }

    pub fn weak(&self) -> WeakNotifyHandle {
        WeakNotifyHandle {
            state: Arc::downgrade(&self.state),
        }
    }
}

impl WeakNotifyHandle {
    pub fn set_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        let Some(s) = self.state.upgrade() else {
            anyhow::bail!("Ticker went away");
        };
        s.write()
            .map_err(|_| anyhow::anyhow!("Ticker panicked"))?
            .set_enabled(enabled);
        Ok(())
    }
}

struct NotifyState {
    enabled: bool,
    last_notification: Option<NotifyRecord>,
}

impl NotifyState {
    fn new() -> Self {
        Self {
            enabled: false,
            last_notification: None,
        }
    }

    fn tick(&mut self, radar: &[RadarReading]) {
        if !self.enabled {
            return;
        }
        let now = jiff::Timestamp::now();
        if let Some(notif) = &mut self.last_notification {
            if notif.try_extend(radar) || notif.is_active(now) {
                return;
            }
            self.last_notification = None;
        }
        let Some((start, end)) = find_rain_streak(radar) else {
            return;
        };

        let dur = start.duration_since(now);
        if dur > SignedDuration::from_mins(NOTIFY_BEFORE_MIN) {
            return;
        }
        self.last_notification = Some(NotifyRecord {
            for_rain_at: start - SignedDuration::from_mins(NOTIFY_BEFORE_MIN),
            active_until: end,
        });
        let res = notify_rust::Notification::new()
            .summary("Rain Notification")
            .body(&format!(
                "Rain is expected in {} min ({}).",
                dur.max(SignedDuration::ZERO).as_mins(),
                start.to_zoned(TimeZone::system()).strftime("%H:%M")
            ))
            .appname("Background")
            .finalize()
            .show();
        if let Err(e) = res {
            tracing::warn!(%e, "Failed to show notification");
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.last_notification = None;
    }
}

struct NotifyRecord {
    for_rain_at: jiff::Timestamp,
    active_until: jiff::Timestamp,
}

impl NotifyRecord {
    fn try_extend(&mut self, radar: &[RadarReading]) -> bool {
        let after = split_binary(radar, |x| x.timestamp.cmp(&self.active_until)).1;
        if after.is_empty() {
            return false;
        }
        let Some((start, end)) = find_rain_streak(after) else {
            return false;
        };
        if start.duration_since(self.active_until) >= SignedDuration::from_mins(ABSORB_UNTIL_MIN) {
            return false;
        }

        self.active_until = end;
        true
    }

    fn is_active(&self, now: jiff::Timestamp) -> bool {
        now >= self.for_rain_at && now <= self.active_until
    }
}

fn split_binary<'a, T>(
    arr: &'a [T],
    f: impl FnMut(&'a T) -> std::cmp::Ordering,
) -> (&'a [T], &'a [T]) {
    let i = arr.binary_search_by(f).unwrap_or_else(|x| x);
    if i >= arr.len() {
        (arr, &[])
    } else {
        (&arr[..i], &arr[i..])
    }
}

fn find_rain_streak(radar: &[RadarReading]) -> Option<(jiff::Timestamp, jiff::Timestamp)> {
    let mut range = None;
    for reading in radar {
        match &mut range {
            None => {
                if reading.value > 0.0f32 {
                    range = Some((reading.timestamp, reading.timestamp));
                }
            }
            Some((_start, end)) => {
                *end = reading.timestamp;
                if reading.value == 0.0f32
                    && reading.timestamp.duration_since(*end)
                        >= SignedDuration::from_mins(ABSORB_UNTIL_MIN)
                {
                    break;
                }
            }
        }
    }
    range
}
