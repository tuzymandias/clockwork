#![allow(dead_code)]
use super::App;
use super::Clockwork;
use super::ClockworkConfig;
use crate::clockwork::ClockworkHandle;
#[cfg(feature = "logging")]
use crate::clockwork_logger::{ClockworkLogger, LoggerConfig};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct ClockworkAppConfig<T> {
    #[serde(default)]
    pub(crate) clockwork: ClockworkConfig,
    #[cfg(feature = "logging")]
    #[serde(default)]
    pub(crate) logger: LoggerConfig,
    pub(crate) app: T,
}

pub struct ClockworkApp<T: App> {
    cw: Clockwork,
    #[cfg(feature = "logging")]
    logger: Option<ClockworkLogger>,
    app: T,
}

impl<T: App> ClockworkApp<T> {
    #[cfg(feature = "logging")]
    pub fn from_config(conf: ClockworkAppConfig<T::Config>) -> Self {
        Self {
            cw: Clockwork::from(conf.clockwork),
            logger: Some(ClockworkLogger::from(conf.logger)),
            app: T::from(conf.app),
        }
    }

    #[cfg(not(feature = "logging"))]
    pub fn from_config(conf: ClockworkAppConfig<T::Config>) -> Self {
        Self {
            cw: Clockwork::from(conf.clockwork),
            app: T::from(conf.app),
        }
    }

    pub fn from_config_str(conf_string: String) -> Self
    where
        T::Config: DeserializeOwned,
    {
        let conf: ClockworkAppConfig<T::Config> =
            toml::from_str(conf_string.as_str()).expect("Failed to parse config!");

        Self::from_config(conf)
    }

    pub fn from_path<'a>(path: PathBuf) -> Self
    where
        T::Config: DeserializeOwned,
    {
        let file = File::open(path).expect("Path cannot be opened!");
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .expect("File cannot be read!");

        Self::from_config_str(contents)
    }

    pub fn start(&self) {
        #[cfg(feature = "logging")]
        if self.logger.is_some() {
            self.logger.as_ref().unwrap().enable_logging();
        }

        self.app.setup(self.cw.handle());
        self.cw.run(async { self.app.run(self.cw.handle()).await });
        self.app.shutdown();
    }

    pub fn handle(&self) -> ClockworkHandle {
        self.cw.handle()
    }

    #[cfg(not(feature = "logging"))]
    pub(crate) fn new(cw: Clockwork, app: T) -> Self {
        Self { cw, app }
    }

    #[cfg(feature = "logging")]
    pub(crate) fn new(cw: Clockwork, app: T) -> Self {
        Self {
            cw,
            logger: None,
            app,
        }
    }

    pub(crate) fn mut_app(&mut self) -> &mut T {
        &mut self.app
    }

    pub(crate) fn app(&self) -> &T {
        &self.app
    }
}

#[cfg(test)]
mod tests {
    use crate::clockwork_app::ClockworkApp;
    use crate::{App, ClockworkHandle, Configurable, Runnable};
    use serde::Deserialize;

    #[test]
    fn test_clockwork_app_conf() {
        struct BasicApp {
            val: String,
        }

        #[derive(Deserialize)]
        struct BasicAppConf {
            val: String,
        }

        impl Configurable for BasicApp {
            type Config = BasicAppConf;

            fn from(app_conf: Self::Config) -> Self {
                Self { val: app_conf.val }
            }
        }

        impl Runnable for BasicApp {
            fn setup(&self, handle: ClockworkHandle) {
                assert!(!self.val.is_empty())
            }
        }

        let conf_str = r#"
            [app]
            val = 'Hello World'
        "#
        .to_string();

        type BasicClockworkApp = ClockworkApp<BasicApp>;
        let cw_app = BasicClockworkApp::from_config_str(conf_str);
        assert_eq!(cw_app.app().val, "Hello World")
    }
}
