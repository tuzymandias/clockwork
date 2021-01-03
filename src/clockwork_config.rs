use serde::Deserialize;

const fn default_as_true() -> bool {
    true
}

const fn default_max_thread() -> usize {
    512
}

#[derive(Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_as_true")]
    pub enable_io: bool,
    #[serde(default = "default_as_true")]
    pub enable_time: bool,
    #[serde(default = "default_max_thread")]
    pub max_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            enable_io: default_as_true(),
            enable_time: default_as_true(),
            max_threads: default_max_thread(),
        }
    }
}

#[derive(Deserialize)]
pub struct ClockworkConfig {
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

impl Default for ClockworkConfig {
    fn default() -> Self {
        ClockworkConfig {
            runtime: RuntimeConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ClockworkConfig;

    #[test]
    fn test_default_runtime_conf() {
        let conf: ClockworkConfig = toml::from_str("").unwrap();

        assert_eq!(conf.runtime.enable_time, true);
        assert_eq!(conf.runtime.enable_io, true);
        assert_eq!(conf.runtime.max_threads, 512);
    }
}
