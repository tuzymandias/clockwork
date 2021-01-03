mod clockwork;
mod clockwork_app;
mod clockwork_config;
mod clockwork_thread;

pub type Clockwork = clockwork::Clockwork;
pub type ClockworkHandle = clockwork::ClockworkHandle;
pub type ClockworkConfig = clockwork_config::ClockworkConfig;
pub type ClockworkApp<T> = clockwork_app::ClockworkApp<T>;

pub use clockwork_thread::{spawn, spawn_anonymous};
use std::pin::Pin;
use tokio::time::{sleep, Duration};

pub trait Runnable {
    fn setup(&self, handle: ClockworkHandle);
    fn shutdown(&self) {}

    /// Infinitely yielding loop until 'stopped' flag is raised
    fn run<'a>(
        &'a self,
        handle: ClockworkHandle,
    ) -> Pin<Box<dyn core::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            while !handle.stopped() {
                sleep(Duration::from_secs(0)).await
            }
        })
    }
}

pub trait Configurable {
    type Config;

    fn from(config: Self::Config) -> Self;
}

pub trait App: Runnable + Configurable {}

impl<T: Runnable + Configurable> App for T {}
