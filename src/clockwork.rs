use super::ClockworkConfig;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use tokio::time::{interval_at, sleep, Duration, Instant};

type SharedRuntime = Arc<Runtime>;
type SharedAtomicBool = Arc<AtomicBool>;

#[derive(Clone)]
pub struct ClockworkHandle {
    rt: SharedRuntime,
    stopped: SharedAtomicBool,
}

impl ClockworkHandle {
    /// Creates a new ClockworkHandle from a SharedRuntime
    pub fn new(rt: SharedRuntime) -> Self {
        Self {
            rt,
            stopped: SharedAtomicBool::new(AtomicBool::new(false)),
        }
    }

    /// Schedules a task that repeats every interval starting from the specified time until
    /// runtime is stopped
    pub fn schedule_repeating_task_at<F: 'static + Fn() + std::marker::Sync + std::marker::Send>(
        &self,
        f: F,
        start: Instant,
        period: Duration,
    ) {
        let stopped = Arc::clone(&self.stopped);
        self.rt.spawn(async move {
            let interval = interval_at(start, period);
            tokio::pin!(interval);

            while !stopped.load(Ordering::Relaxed) {
                interval.as_mut().tick().await;
                f();
            }
        });
    }

    /// Schedules a task that runs once after duration elapsed.
    /// If runtime is stopped before duration elapsed, the task may not be run.
    pub fn schedule_oneof_task<F: 'static + Fn() + std::marker::Sync + std::marker::Send>(
        &self,
        f: F,
        duration: Duration,
    ) {
        self.rt.spawn(async move {
            sleep(duration).await;
            f();
        });
    }

    /// Raise the 'stopped' flag, triggers shutdown sequence on the next spin 'tick'
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }

    /// Runs a future on the runtime, blocking until completion
    fn run<F: Future>(&self, f: F) {
        self.rt.block_on(f);
    }
}

pub struct Clockwork {
    handle: ClockworkHandle,
}

impl Clockwork {
    /// Creates a new Clockwork instance from a ClockworkHandle
    pub fn new(handle: ClockworkHandle) -> Self {
        Self { handle }
    }

    /// Schedules a task that repeats every interval starting from the specified time until
    /// runtime is stopped
    pub fn schedule_repeating_task_at<F: 'static + Fn() + std::marker::Sync + std::marker::Send>(
        &self,
        f: F,
        start: Instant,
        period: Duration,
    ) {
        self.handle().schedule_repeating_task_at(f, start, period)
    }

    /// Schedules a task that runs once after duration elapsed.
    /// If runtime is stopped before duration elapsed, the task may not be run.
    pub fn schedule_oneof_task<F: 'static + Fn() + std::marker::Sync + std::marker::Send>(
        &self,
        f: F,
        duration: Duration,
    ) {
        self.handle().schedule_oneof_task(f, duration)
    }

    /// Runs a future on the runtime, blocking until completion
    pub fn run<F: Future>(&self, f: F) {
        self.handle.run(f);
    }

    /// Returns a clone of the ClockworkHandle (allows scheduling from different thread, etc)
    pub fn handle(&self) -> ClockworkHandle {
        self.handle.clone()
    }

    /// Raise the 'stopped' flag, triggers shutdown sequence on the next spin 'tick'
    pub fn stop(&mut self) {
        self.handle.stop();
    }
}

impl From<ClockworkConfig> for Clockwork {
    fn from(conf: ClockworkConfig) -> Self {
        let mut builder = Builder::new_current_thread();
        if conf.runtime.enable_io {
            builder.enable_io();
        }

        if conf.runtime.enable_time {
            builder.enable_time();
        }

        builder.max_threads(conf.runtime.max_threads);

        Self::new(ClockworkHandle::new(Arc::new(
            builder.build().expect("Failed to Build Runtime"),
        )))
    }
}

impl Default for Clockwork {
    fn default() -> Self {
        Self::from(ClockworkConfig::default())
    }
}
