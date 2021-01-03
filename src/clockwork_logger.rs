use serde::Deserialize;
use std::path::PathBuf;
use tracing::Dispatch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

const fn default_as_true() -> bool {
    true
}

const fn default_as_false() -> bool {
    false
}

#[derive(Deserialize)]
pub enum LoggerFormat {
    COMPACT,
    PRETTY,
    JSON,
    FULL,
}

impl Default for LoggerFormat {
    fn default() -> Self {
        LoggerFormat::FULL
    }
}

#[derive(Deserialize, Copy, Clone)]
pub enum LoggerLevel {
    OFF,
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl Into<LevelFilter> for LoggerLevel {
    fn into(self) -> LevelFilter {
        match self {
            LoggerLevel::OFF => LevelFilter::OFF,
            LoggerLevel::TRACE => LevelFilter::TRACE,
            LoggerLevel::DEBUG => LevelFilter::DEBUG,
            LoggerLevel::INFO => LevelFilter::INFO,
            LoggerLevel::WARN => LevelFilter::WARN,
            LoggerLevel::ERROR => LevelFilter::ERROR,
        }
    }
}

impl Default for LoggerLevel {
    fn default() -> Self {
        LoggerLevel::TRACE
    }
}

#[derive(Deserialize)]
#[serde(tag = "write_target")]
pub enum WriteTarget {
    STDOUT,
    FILE { file_name: String },
}

impl Default for WriteTarget {
    fn default() -> Self {
        WriteTarget::STDOUT
    }
}

impl WriteTarget {
    pub(crate) fn writer(self) -> BoxMakeWriter {
        let writer: Box<dyn std::io::Write + Send + Sync> = match self {
            WriteTarget::STDOUT => Box::new(std::io::stdout()),
            WriteTarget::FILE { file_name } => {
                let path = PathBuf::from(file_name);
                Box::new(tracing_appender::rolling::never(
                    path.parent().unwrap(),
                    path.file_name().unwrap(),
                ))
            }
        };

        let (non_blocking, guard) = tracing_appender::non_blocking(writer);

        static mut GUARD: Option<WorkerGuard> = None;
        unsafe { GUARD = Some(guard) }

        BoxMakeWriter::new(non_blocking)
    }
}

#[derive(Deserialize)]
pub struct LoggerConfig {
    #[serde(default = "default_as_true")]
    pub(crate) show_time: bool,
    #[serde(default = "default_as_false")]
    pub(crate) show_thread_names: bool,
    #[serde(default = "default_as_true")]
    pub(crate) show_thread_ids: bool,
    #[serde(default)]
    pub(crate) log_format: LoggerFormat,
    #[serde(default)]
    pub(crate) log_level: LoggerLevel,
    #[serde(default, flatten)]
    pub(crate) write_target: WriteTarget,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            show_time: default_as_true(),
            show_thread_names: default_as_false(),
            show_thread_ids: default_as_true(),
            log_format: Default::default(),
            log_level: Default::default(),
            write_target: Default::default(),
        }
    }
}

pub struct ClockworkLogger {
    dispatch: Dispatch,
}

impl From<LoggerConfig> for ClockworkLogger {
    fn from(conf: LoggerConfig) -> Self {
        let writer = conf.write_target.writer();
        let builder = tracing_subscriber::fmt()
            .with_thread_names(conf.show_thread_names)
            .with_thread_ids(conf.show_thread_ids)
            .with_max_level(conf.log_level)
            .with_writer(writer);

        // FIXME: Surely this can be made more compact!
        let dispatch = match conf.log_format {
            LoggerFormat::COMPACT => {
                let builder = builder.compact();
                match conf.show_time {
                    true => builder.into(),
                    false => builder.without_time().into(),
                }
            }
            LoggerFormat::PRETTY => {
                let builder = builder.pretty();
                match conf.show_time {
                    true => builder.into(),
                    false => builder.without_time().into(),
                }
            }
            LoggerFormat::JSON => {
                let builder = builder.json();
                match conf.show_time {
                    true => builder.into(),
                    false => builder.without_time().into(),
                }
            }
            LoggerFormat::FULL => match conf.show_time {
                true => builder.into(),
                false => builder.without_time().into(),
            },
        };

        Self { dispatch }
    }
}

impl ClockworkLogger {
    pub fn enable_logging(&self) {
        tracing::dispatcher::set_global_default(self.dispatch.clone())
            .expect("Unable to set logger");
    }
}

#[cfg(test)]
mod tests {}
