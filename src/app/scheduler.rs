use crate::aria2::client::BatchRefreshRequest;
use crate::aria2::domain::Gid;

const ACTIVE_TICK_INTERVAL: u64 = 1;
const IDLE_TICK_INTERVAL: u64 = 5;
const WAITING_TICK_INTERVAL: u64 = 5;
const STOPPED_TICK_INTERVAL: u64 = 30;
const SETTINGS_TICK_INTERVAL: u64 = 10;
const MAX_BACKOFF_TICKS: u16 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshTrigger {
    Scheduled,
    Manual,
    Dirty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshLifecycle {
    Idle,
    InFlight {
        generation: u64,
    },
    #[expect(
        dead_code,
        reason = "reserved for terminal failures that should not back off"
    )]
    Failed,
    Backoff {
        attempt: u8,
        remaining_ticks: u16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RefreshDirtyFlags {
    pub active: bool,
    pub waiting: bool,
    pub stopped: bool,
    pub selected: bool,
}

impl RefreshDirtyFlags {
    fn any(self) -> bool {
        self.active || self.waiting || self.stopped || self.selected
    }

    fn clear_included(&mut self, request: &BatchRefreshRequest) {
        if request.include_active() {
            self.active = false;
        }
        if request.include_waiting() {
            self.waiting = false;
        }
        if request.include_stopped() {
            self.stopped = false;
        }
        if request.selected_gid().is_some() {
            self.selected = false;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RefreshMetrics {
    pub stale_responses_discarded: u64,
    pub failed_refreshes: u64,
    pub successful_refreshes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshScheduler {
    lifecycle: RefreshLifecycle,
    tick: u64,
    next_generation: u64,
    last_active_tick: Option<u64>,
    last_waiting_tick: Option<u64>,
    last_stopped_tick: Option<u64>,
    in_flight_request: Option<BatchRefreshRequest>,
    dirty: RefreshDirtyFlags,
    failure_attempt: u8,
    metrics: RefreshMetrics,
}

impl Default for RefreshScheduler {
    fn default() -> Self {
        Self {
            lifecycle: RefreshLifecycle::Idle,
            tick: 0,
            next_generation: 0,
            last_active_tick: None,
            last_waiting_tick: None,
            last_stopped_tick: None,
            in_flight_request: None,
            dirty: RefreshDirtyFlags::default(),
            failure_attempt: 0,
            metrics: RefreshMetrics::default(),
        }
    }
}

impl RefreshScheduler {
    #[cfg(test)]
    pub fn lifecycle(&self) -> RefreshLifecycle {
        self.lifecycle
    }

    #[cfg(test)]
    pub fn metrics(&self) -> RefreshMetrics {
        self.metrics
    }

    pub fn mark_dirty(&mut self, dirty: RefreshDirtyFlags) {
        self.dirty.active |= dirty.active;
        self.dirty.waiting |= dirty.waiting;
        self.dirty.stopped |= dirty.stopped;
        self.dirty.selected |= dirty.selected;
    }

    pub fn begin_refresh(
        &mut self,
        trigger: RefreshTrigger,
        has_active_downloads: bool,
        settings_open: bool,
        selected_gid: Option<&Gid>,
    ) -> Option<(u64, BatchRefreshRequest)> {
        if matches!(trigger, RefreshTrigger::Scheduled) {
            self.tick += 1;
        }

        if matches!(self.lifecycle, RefreshLifecycle::InFlight { .. }) {
            return None;
        }

        if matches!(trigger, RefreshTrigger::Scheduled) && !self.advance_backoff() {
            return None;
        }

        if !matches!(trigger, RefreshTrigger::Scheduled) {
            self.failure_attempt = 0;
            if matches!(self.lifecycle, RefreshLifecycle::Backoff { .. }) {
                self.lifecycle = RefreshLifecycle::Idle;
            }
        }

        let request =
            self.plan_request(trigger, has_active_downloads, settings_open, selected_gid)?;

        self.next_generation += 1;
        let generation = self.next_generation;
        self.in_flight_request = Some(request.clone());
        self.lifecycle = RefreshLifecycle::InFlight { generation };

        Some((generation, request))
    }

    pub fn complete_success(&mut self, generation: u64) -> Option<BatchRefreshRequest> {
        if !matches!(self.lifecycle, RefreshLifecycle::InFlight { generation: current } if current == generation)
        {
            self.metrics.stale_responses_discarded += 1;
            return None;
        }

        let request = self.in_flight_request.take()?;
        self.dirty.clear_included(&request);
        self.failure_attempt = 0;
        self.metrics.successful_refreshes += 1;
        self.lifecycle = RefreshLifecycle::Idle;

        if request.include_active() {
            self.last_active_tick = Some(self.tick);
        }
        if request.include_waiting() {
            self.last_waiting_tick = Some(self.tick);
        }
        if request.include_stopped() {
            self.last_stopped_tick = Some(self.tick);
        }

        Some(request)
    }

    pub fn complete_failure(&mut self, generation: u64) -> bool {
        if !matches!(self.lifecycle, RefreshLifecycle::InFlight { generation: current } if current == generation)
        {
            self.metrics.stale_responses_discarded += 1;
            return false;
        }

        self.in_flight_request = None;
        self.failure_attempt = self.failure_attempt.saturating_add(1);
        self.metrics.failed_refreshes += 1;
        self.lifecycle = RefreshLifecycle::Backoff {
            attempt: self.failure_attempt,
            remaining_ticks: backoff_ticks(self.failure_attempt),
        };

        true
    }

    pub fn cancel_in_flight(&mut self) {
        if matches!(self.lifecycle, RefreshLifecycle::InFlight { .. }) {
            self.in_flight_request = None;
            self.lifecycle = RefreshLifecycle::Idle;
        }
    }

    fn advance_backoff(&mut self) -> bool {
        let RefreshLifecycle::Backoff {
            attempt,
            remaining_ticks,
        } = self.lifecycle
        else {
            return true;
        };

        if remaining_ticks > 0 {
            self.lifecycle = RefreshLifecycle::Backoff {
                attempt,
                remaining_ticks: remaining_ticks - 1,
            };
            return false;
        }

        self.lifecycle = RefreshLifecycle::Idle;
        true
    }

    fn plan_request(
        &self,
        trigger: RefreshTrigger,
        has_active_downloads: bool,
        settings_open: bool,
        selected_gid: Option<&Gid>,
    ) -> Option<BatchRefreshRequest> {
        let mut request = BatchRefreshRequest::stats_only();

        match trigger {
            RefreshTrigger::Manual => {
                request.include_all_summaries();
            }
            RefreshTrigger::Dirty => {
                request.set_include_active(self.dirty.active || !self.dirty.any());
                request.set_include_waiting(self.dirty.waiting || !self.dirty.any());
                request.set_include_stopped(self.dirty.stopped || !self.dirty.any());
            }
            RefreshTrigger::Scheduled => {
                if settings_open {
                    if !self.is_due(self.last_active_tick, SETTINGS_TICK_INTERVAL) {
                        return None;
                    }
                    request.set_include_active(false);
                    request.set_include_waiting(false);
                    request.set_include_stopped(false);
                } else {
                    let active_interval = if has_active_downloads {
                        ACTIVE_TICK_INTERVAL
                    } else {
                        IDLE_TICK_INTERVAL
                    };
                    request.set_include_active(
                        self.dirty.active || self.is_due(self.last_active_tick, active_interval),
                    );
                    request.set_include_waiting(
                        self.dirty.waiting
                            || self.is_due(self.last_waiting_tick, WAITING_TICK_INTERVAL),
                    );
                    request.set_include_stopped(
                        self.dirty.stopped
                            || self.is_due(self.last_stopped_tick, STOPPED_TICK_INTERVAL),
                    );
                }
            }
        }

        if self.dirty.selected
            && let Some(gid) = selected_gid
        {
            request.set_selected_gid(Some(gid.clone()));
        }

        if request.refreshes_anything() {
            Some(request)
        } else {
            None
        }
    }

    fn is_due(&self, last_tick: Option<u64>, interval: u64) -> bool {
        last_tick.is_none_or(|last_tick| self.tick.saturating_sub(last_tick) >= interval)
    }
}

fn backoff_ticks(attempt: u8) -> u16 {
    let shift = u32::from(attempt.saturating_sub(1)).min(5);
    (1_u16 << shift).min(MAX_BACKOFF_TICKS)
}

#[cfg(test)]
mod tests {
    use super::{RefreshDirtyFlags, RefreshLifecycle, RefreshScheduler, RefreshTrigger};

    #[test]
    fn scheduled_refresh_does_not_reenter_while_in_flight() {
        let mut scheduler = RefreshScheduler::default();
        let first = scheduler.begin_refresh(RefreshTrigger::Scheduled, true, false, None);

        assert!(first.is_some());
        assert_eq!(
            scheduler.begin_refresh(RefreshTrigger::Scheduled, true, false, None),
            None
        );
    }

    #[test]
    fn stale_generation_is_discarded_without_clearing_in_flight_request() {
        let mut scheduler = RefreshScheduler::default();
        let _first = scheduler.begin_refresh(RefreshTrigger::Manual, true, false, None);

        assert_eq!(scheduler.complete_success(99), None);
        assert_eq!(scheduler.metrics().stale_responses_discarded, 1);
    }

    #[test]
    fn failed_refresh_enters_backoff_and_scheduled_ticks_wait() {
        let mut scheduler = RefreshScheduler::default();
        let (generation, _) = scheduler
            .begin_refresh(RefreshTrigger::Manual, true, false, None)
            .expect("refresh starts");

        assert!(scheduler.complete_failure(generation));
        assert!(matches!(
            scheduler.lifecycle(),
            RefreshLifecycle::Backoff { attempt: 1, .. }
        ));
        assert_eq!(
            scheduler.begin_refresh(RefreshTrigger::Scheduled, true, false, None),
            None
        );
    }

    #[test]
    fn manual_refresh_bypasses_backoff() {
        let mut scheduler = RefreshScheduler::default();
        let (generation, _) = scheduler
            .begin_refresh(RefreshTrigger::Manual, true, false, None)
            .expect("refresh starts");
        assert!(scheduler.complete_failure(generation));

        assert!(
            scheduler
                .begin_refresh(RefreshTrigger::Manual, true, false, None)
                .is_some()
        );
    }

    #[test]
    fn dirty_flags_are_coalesced_into_next_refresh() {
        let mut scheduler = RefreshScheduler::default();
        scheduler.mark_dirty(RefreshDirtyFlags {
            stopped: true,
            ..RefreshDirtyFlags::default()
        });

        let (_, request) = scheduler
            .begin_refresh(RefreshTrigger::Scheduled, true, false, None)
            .expect("dirty stopped section refreshes");

        assert!(request.include_stopped());
    }
}
