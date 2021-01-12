#![allow(dead_code)]
use super::Clockwork;
use super::ClockworkConfig;
use super::ClockworkHandle;
use super::Runnable;
use crate::Configurable;
use std::any::Any;
use std::thread::JoinHandle;

// TODO: Should refactor this out, might be useful
pub struct ClockworkRunnable<T: Runnable> {
    cw: Clockwork,
    t: T,
}

impl<T: Runnable> ClockworkRunnable<T> {
    pub fn new(cw: Clockwork, t: T) -> Self {
        Self { cw, t }
    }

    pub fn handle(&self) -> ClockworkHandle {
        self.cw.handle()
    }

    pub fn start(&self) {
        self.t.setup(self.handle());
        self.cw.run(&self.t);
        self.t.shutdown();
    }
}

impl<T: Runnable + Configurable> ClockworkRunnable<T> {
    pub fn from_config(cw_conf: ClockworkConfig, t: T::Config) -> Self {
        Self::new(cw_conf.into(), T::from(t))
    }

    pub fn from_runnable_config(t: T::Config) -> Self {
        Self::new(ClockworkConfig::default().into(), T::from(t))
    }
}

/// Wraps around a `JoinHandle` and a `ClockworkHandle`.
/// Treat this like you would a `JoinHandle`.  
/// This is returned by `spawn`.
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

    /// Waits for the thread to finish, this will block.
    /// Exact same behaviour as `JoinHandle::join`.
    /// Use this if you can guarantee the thread will stop.
    /// ```
    /// use clockwork::{Clockwork, spawn_from_runnable, ClockworkHandle};
    /// use tokio::time::Duration;
    /// let cw = Clockwork::default();
    /// let thread = spawn_from_runnable(cw, |orig_handle: ClockworkHandle| {
    ///     let handle = orig_handle.clone();
    ///     orig_handle.schedule_oneof_task(move || handle.stop(), Duration::from_secs(1));
    /// });
    /// thread.join();
    /// ```
    pub fn join(self) -> Result<(), Box<dyn Any + Send + 'static>> {
        self.join_handle.join()
    }

    /// Stops the `Clockwork` runtime and waits for the thread to finish, blocking.
    /// This does not guarantee that the thread will eventually join, user have to ensure that the
    /// Runnable will eventually terminate after being stopped.
    /// ```
    /// use clockwork::{Clockwork, spawn_from_runnable, ClockworkHandle};
    /// let cw = Clockwork::default();
    /// let thread = spawn_from_runnable(cw, |orig_handle: ClockworkHandle| {});
    /// thread.stop_and_join();
    /// ```
    pub fn stop_and_join(self) -> Result<(), Box<dyn Any + Send + 'static>> {
        self.stop();
        self.join()
    }

    /// Determines if the handle is ready to be joined.
    /// ```
    /// use clockwork::{Clockwork, spawn_from_runnable, ClockworkHandle};
    /// let cw = Clockwork::default();
    /// let thread = spawn_from_runnable(cw, |orig_handle: ClockworkHandle| {});
    /// assert_eq!(thread.joinable(), false);
    /// thread.stop();
    /// assert_eq!(thread.joinable(), true);
    /// thread.join();
    /// ```
    pub fn joinable(&self) -> bool {
        self.cw_handle.stopped()
    }

    /// Tells the `Clockwork` run time to stop.
    /// Whether it will actually stop depends on the `Runnable`
    pub fn stop(&self) {
        self.cw_handle.stop();
    }
}

/// Implements Runnable for any closure that takes in a ClockworkHandle
/// Allows user to directly use an anonymous function as a parameter to `spawn`
impl<T: Fn(ClockworkHandle) + Send + Sync + 'static> Runnable for T {
    fn setup(&self, handle: ClockworkHandle) {
        self(handle);
    }
}

/// Spawns a `ClockworkThread` using the given `Clockwork` runtime and `Runnable` instance.
///
/// ```
/// use clockwork::{Clockwork, spawn_from_runnable, ClockworkHandle};
/// let cw = Clockwork::default();
/// let thread = spawn_from_runnable(cw, |_handle: ClockworkHandle| {
///     // set up the runtime (scheduled tasks/etc)
/// });
/// thread.stop_and_join();
/// ```
pub fn spawn_from_runnable<T>(cw: Clockwork, t: T) -> ClockworkJoinHandle
where
    T: Runnable + Send + Sync + 'static,
{
    let cw_runnable = ClockworkRunnable::new(cw, t);
    spawn(cw_runnable)
}

pub fn spawn_from_config<T>(cw_conf: ClockworkConfig, t: T::Config) -> ClockworkJoinHandle
where
    T: Runnable + Configurable + Send + Sync + 'static,
{
    let cw_runnable: ClockworkRunnable<T> = ClockworkRunnable::from_config(cw_conf, t);
    spawn(cw_runnable)
}

pub fn spawn<T>(cw_runnable: ClockworkRunnable<T>) -> ClockworkJoinHandle
where
    T: Runnable + Send + Sync + 'static,
{
    let runnable_handle = cw_runnable.handle();
    ClockworkJoinHandle {
        join_handle: std::thread::spawn(move || cw_runnable.start()),
        cw_handle: runnable_handle,
    }
}
