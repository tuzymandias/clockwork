#![allow(dead_code)]
use super::Clockwork;
use super::ClockworkConfig;
use super::ClockworkHandle;
use super::Runnable;
use std::thread::JoinHandle;

pub struct ClockworkRunnable<T: Runnable + Send + Sync> {
    cw: Clockwork,
    t: T,
}

impl<T: Runnable + Send + Sync> ClockworkRunnable<T> {
    pub fn new(cw: Clockwork, t: T) -> Self {
        Self { cw, t }
    }

    pub fn from_config_and_runnable(cw: ClockworkConfig, t: T) -> Self {
        let cw = Clockwork::from(cw);
        Self::new(cw, t)
    }

    pub fn handle(&self) -> ClockworkHandle {
        self.cw.handle()
    }

    pub fn start(&self) {
        self.t.setup(self.handle());
        self.cw.run(async { self.t.run(self.cw.handle()).await });
        self.t.shutdown();
    }
}

pub struct ClockworkJoinHandle {
    join_handle: JoinHandle<()>,
    cw_handle: ClockworkHandle,
}

impl ClockworkJoinHandle {
    pub(crate) fn new(join_handle: JoinHandle<()>, cw_handle: ClockworkHandle) -> Self {
        Self {
            join_handle,
            cw_handle,
        }
    }
}

pub fn spawn<T: Runnable + Send + Sync + 'static>(cw: Clockwork, t: T) -> ClockworkJoinHandle {
    let runnable = ClockworkRunnable::new(cw, t);
    let runnable_handle = runnable.handle();
    ClockworkJoinHandle {
        join_handle: std::thread::spawn(move || runnable.start()),
        cw_handle: runnable_handle,
    }
}

pub fn spawn_anonymous<F: Fn(ClockworkHandle) + Send + Sync + 'static>(
    cw: Clockwork,
    f: F,
) -> ClockworkJoinHandle {
    struct AnonymousRunnable<F: Fn(ClockworkHandle) + Send + Sync + 'static> {
        f: F,
    }

    impl<F: Fn(ClockworkHandle) + Send + Sync + 'static> Runnable for AnonymousRunnable<F> {
        fn setup(&self, handle: ClockworkHandle) {
            (self.f)(handle);
        }
    }

    spawn(cw, AnonymousRunnable { f })
}
