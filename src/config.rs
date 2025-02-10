use anyhow::{anyhow, Context};
use serde::Deserialize;
use std::collections::HashMap;
use strum::{Display, EnumString};

const _GLOBAL_CONFIG_DIR_1: &str = "/etc";
const _GLOBAL_CONFIG_DIR_2: &str = "/usr/local/etc";
const CONFIG_DEFAULT_NAME: &str = "breathe.toml";

pub(crate) fn get_default_config_file() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join(CONFIG_DEFAULT_NAME)
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    patterns: HashMap<String, Pattern>,
    pub(crate) counter_type: CounterType,
    pub(crate) duration: u64,
}

pub(crate) fn from_file(config_file: &std::path::Path) -> anyhow::Result<Config> {
    if config_file.exists() && config_file.is_file() {
        let temp_str = std::fs::read_to_string(config_file)
            .with_context(|| format!("Failed to read config from {}", config_file.display()))?;
        let conf: Config = toml::from_str(&temp_str)
            .with_context(|| format!("Invalid configuration in {}", config_file.display()))?;
        Ok(conf)
    } else {
        Err(anyhow!(
            "File {} doesn't exist or is not readable",
            config_file.display()
        ))
    }
}

impl Config {
    pub(crate) fn get_pattern(&self, pattern_name: &str) -> anyhow::Result<&Pattern> {
        if self.patterns.contains_key(pattern_name) {
            Ok(self.patterns.get(pattern_name).unwrap())
        } else {
            Err(anyhow!("Pattern {pattern_name} not found"))
        }
    }

    pub(crate) fn print_pattern_list(&self) {
        self.patterns.iter().for_each(|(name, pattern)| {
            println!(
                "{} [{}] [{}]: {}",
                name,
                pattern.get_short_string(),
                pattern.get_short_session_string(),
                pattern.description
            )
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Pattern {
    pub(crate) breath_in: u64,
    pub(crate) hold_in: Option<u64>,
    pub(crate) breath_out: u64,
    pub(crate) hold_out: Option<u64>,
    pub(crate) counter_type: Option<CounterType>,
    pub(crate) duration: Option<u64>,
    pub(crate) description: String,
}

impl Pattern {
    fn get_short_string(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.breath_in,
            self.hold_in.unwrap_or(0),
            self.breath_out,
            self.hold_out.unwrap_or(0)
        )
    }
    fn get_short_session_string(&self) -> String {
        if self.counter_type.is_none() || self.duration.is_none() {
            "".to_string()
        } else {
            match self.counter_type.unwrap() {
                CounterType::Time => format!("{} seconds", self.duration.unwrap()),
                CounterType::Iteration => format!("{} iterations", self.duration.unwrap()),
            }
        }
    }
}

#[derive(Debug, Clone, EnumString, Display, Deserialize, PartialEq, Copy)]
pub(crate) enum CounterType {
    #[strum(serialize = "time", serialize = "Time")]
    Time,
    #[strum(
        serialize = "iteration",
        serialize = "iterations",
        serialize = "Iteration",
        serialize = "Iterations"
    )]
    Iteration,
}

#[derive(Debug, Clone)]
pub(crate) struct PatternDuration {
    pub(crate) counter_type: CounterType,
    pub(crate) duration: u64,
}

use std::str::FromStr;

impl FromStr for PatternDuration {
    type Err = anyhow::Error;

    fn from_str(src: &str) -> anyhow::Result<PatternDuration> {
        let v: Vec<&str> = src.trim().split('=').collect();
        if v.len() != 2 {
            return Err(anyhow!("Invalid duration: no '=' found in {}", src));
        }
        let counter_type = CounterType::from_str(v.first().unwrap())
            .with_context(|| format!("Invalid counter type {}", v.first().unwrap()))?;
        let duration = u64::from_str(v.get(1).unwrap())
            .with_context(|| format!("Invalid duration {}", v.get(1).unwrap()))?;
        Ok(PatternDuration {
            counter_type,
            duration,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    fn get_standard_config() -> Config {
        let result = from_file(Path::new("resources/tests/config.toml"));
        assert!(result.is_ok());
        result.unwrap()
    }

    #[test]
    fn config_from_file_success() {
        let config = get_standard_config();
        assert!(config.patterns.contains_key("relax"))
    }

    #[test]
    fn config_from_file_failures() {
        let result = from_file(Path::new(""));
        assert!(result.is_err());
        let result = from_file(Path::new("resources/tests/notoml.toml"));
        assert!(result.is_err());
        let result = from_file(Path::new("resources/tests/noconfig.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn get_patterns() {
        let config = get_standard_config();
        let pattern = config.get_pattern("relax");
        assert!(pattern.is_ok());
        let pattern = pattern.unwrap();
        assert_eq!(pattern.breath_in, 4);
        assert_eq!(pattern.breath_out, 8);
        assert_eq!(pattern.hold_in, Some(7));
        assert_eq!(pattern.hold_out, None);
        assert_eq!(pattern.counter_type, Some(CounterType::Iteration));
        assert_eq!(pattern.duration, Some(8));
    }

    #[test]
    fn counter_type_deserialization() {
        let uut = CounterType::Iteration;
        assert_eq!(uut.to_string(), "Iterations".to_string());
        let uut: CounterType = "iteration".to_string().parse().unwrap();
        assert_eq!(uut, CounterType::Iteration);
    }
    #[test]
    fn deserialization() {
        let input = include_str!("../resources/tests/config.toml");
        let got = toml::from_str::<Config>(input);
        assert!(got.is_ok());
    }

    #[test]
    fn pattern_duration_parsing() {
        let input = "iterations=5";
        let uut = PatternDuration::from_str(input);
        assert!(uut.is_ok());
        let uut = uut.unwrap();
        assert_eq!(uut.counter_type, CounterType::Iteration);
        assert_eq!(uut.duration, 5);

        let input = "Time=300";
        let uut = PatternDuration::from_str(input);
        assert!(uut.is_ok());
        let uut = uut.unwrap();
        assert_eq!(uut.counter_type, CounterType::Time);
        assert_eq!(uut.duration, 300);

        let input = " Time=300 ";
        let uut = PatternDuration::from_str(input);
        assert!(uut.is_ok());

        let input = "seconds=300";
        let uut = PatternDuration::from_str(input);
        assert!(uut.is_err());
    }
}
