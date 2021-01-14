# Overview

`Clockwork` is a framework for developing asynchronous applications. The framework wraps around the `tokio` runtime and 
provides an easy to use interface for developing applications.

# Dependencies

* [tokio](https://tokio.rs/) - asynchronous runtime used under the hood for `Clockwork` applications
* [toml](https://github.com/alexcrichton/toml-rs) - toml encoder/decoder
* [serde](https://serde.rs/) - serializing/deserializing framework
* [tracing](https://github.com/tokio-rs/tracing) - logging framework (`logging` feature only)
* [tracing-subscriber](https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber/) - logger implementation (`logging` feature only)
* [tracing-appender](https://github.com/tokio-rs/tracing/tree/master/tracing-appender) - non blocking file writer (`logging` feature only)

# Features

* `logging` - enables logging based on configuration

# Usage

### Simple 'Echoer' App

Below is an example of an app, that when given the configured string and a repeating frequency, repeatedly prints the string 
at every interval. The `ClockworkApp<T>` generic type have a constraint where each type `T` have to implement both the `Runnable` and
`Configurable` traits. The `Configurable` trait defines how to construct `T` from the associated configuration type `Config`. 
`Config` must be a type that implements `Deserialize`. The `Runnable` trait defines how `T` application should be run. The trait
defines three functions: `setup`, `run` and `shutdown`. Only `setup` is required, the others have default behavious that can be overriden.

```rust
use clockwork::{ClockworkApp, ClockworkHandle, Configurable, Runnable};
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::Duration;

#[derive(Deserialize)]
struct EchoerConfig {
    str: String,
    repeat_period_millis: u64,
}

struct EchoerApp {
    str: Arc<String>,
    repeat_period: Duration,
}

impl Configurable for EchoerApp {
    type Config = EchoerConfig;

    fn from(app_conf: Self::Config) -> Self {
        Self {
            str: Arc::new(app_conf.str),
            repeat_period: Duration::from_millis(app_conf.repeat_period_millis),
        }
    }
}

impl Runnable for EchoerApp {
    fn setup(&self, handle: ClockworkHandle) {
        let str = self.str.clone();
        handle.schedule_repeating_task(move || println!("{}", str), self.repeat_period);
    }
}

fn main() {
    let conf_str = r#"
        [app]
        str = 'Hello World!'
        repeat_period_millis = 500
    "#;

    let app: ClockworkApp<EchoerApp> = ClockworkApp::from_config_str(conf_str.to_string());
    app.start();
}
```

### Simple 'Echoer' App with Logging

Same example, but instead of printing to `stdout`, it logs the string into a file. The `logging` feature has to be enabled. 
Note how the configuration now contains a `logger` section. This whole section can actually be omitted. If it is omitted, 
it will default to log into `stdout` instead. You can also explicitly do this by setting `write_target` to `'STDOUT'`. 
See `clockwork_logger` for more configuration option.

**CAUTION!** when the `logging` feature is enabled, the user should not install their own logger.

```rust
use clockwork::{ClockworkApp, ClockworkHandle, Configurable, Runnable};
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::info;

#[derive(Deserialize)]
struct EchoerConfig {
    str: String,
    repeat_period_millis: u64,
}

struct EchoerApp {
    str: Arc<String>,
    repeat_period: Duration,
}

impl Configurable for EchoerApp {
    type Config = EchoerConfig;

    fn from(app_conf: Self::Config) -> Self {
        Self {
            str: Arc::new(app_conf.str),
            repeat_period: Duration::from_millis(app_conf.repeat_period_millis),
        }
    }
}

impl Runnable for EchoerApp {
    fn setup(&self, handle: ClockworkHandle) {
        let str = self.str.clone();
        handle.schedule_repeating_task(move || info!("{}", str), self.repeat_period);
    }
}

fn main() {
    let conf_str = r#"
        [app]
        str = 'Hello World!'
        repeat_period_millis = 500

        [logger]
        write_target = 'FILE'
        file_name = 'test.log'
    "#;

    let app: ClockworkApp<EchoerApp> = ClockworkApp::from_config_str(conf_str.to_string());
    app.start();
}
```
