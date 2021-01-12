mod clockwork;
mod clockwork_app;
mod clockwork_config;
mod clockwork_thread;

#[cfg(feature = "logging")]
mod clockwork_logger;

pub type Clockwork = clockwork::Clockwork;
pub type ClockworkHandle = clockwork::ClockworkHandle;
pub type ClockworkConfig = clockwork_config::ClockworkConfig;
pub type ClockworkApp<T> = clockwork_app::ClockworkApp<T>;

pub use clockwork_thread::spawn_from_runnable;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use tokio::time::{sleep, Duration};

/// A data structure that is compatible with the `Clockwork` framework.
/// This trait is required for `ClockworkApp` and `ClockworkThread`.
/// The data structure needs to define how the app should run.
/// It needs to provide the following three functions: `setup`, `shutdown` and `run`.
/// Only `setup` is mandatory, the others have default implementations.
pub trait Runnable {
    /// Defines how to set up the application. A user can spawn tasks here.
    fn setup(&self, handle: ClockworkHandle);

    /// Defines how to tear down the application. The default behaviour is to do nothing.
    /// Useful for logging, etc.
    fn shutdown(&self) {}

    /// Defines how the application should be run. i.e. stopping conditions, etc.
    /// The default behaviour is to loop and yield infinitely until the handle is stopped.
    /// FIXME: there maybe a better solution instead of using this while loop
    /// FIXME: also, would be good if we can hide away the state of the handle
    ///        user should not need to know if it is still running or have been stopped
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

/// A data structure that can is compatible with the `Clockwork` framework.
/// This trait is required for `ClockworkApp`.
/// The data structure defines how to configure the application.
/// It needs to define the associated type `Config`, a data structure that contains all the required
/// information to configure the application.
/// It also needs to define a function `from` that takes in a `Config` and should return an instance
/// of itself.
pub trait Configurable {
    type Config: DeserializeOwned;

    /// Construct an instance of itself using the information in `Config`
    fn from(config: Self::Config) -> Self;
}

/// An app needs to implement both `Runnable` and `Configurable`
pub trait App: Runnable + Configurable {}

impl<T: Runnable + Configurable> App for T {}
