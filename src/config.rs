use serde::Deserialize;
use std::collections::HashMap;
use strum::{Display, EnumString};

const _GLOBAL_CONFIG_DIR_1: &str = "/etc";
const _GLOBAL_CONFIG_DIR_2: &str = "/usr/local/etc";
const CONFIG_DEFAULT_NAME: &str = "breathe.toml";

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    patterns: HashMap<String, Pattern>,
    pub(crate) counter_type: CounterType,
    pub(crate) duration: u64,
}

impl Config {
    pub(crate) fn get_pattern(&self, pattern_name: &str) -> Option<Pattern> {
        self.patterns.get(pattern_name).cloned()
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

pub(crate) fn get_default_config_file() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join(CONFIG_DEFAULT_NAME)
}

pub(crate) fn get_config(config_file: &std::path::Path) -> Option<Config> {
    if config_file.exists() && config_file.is_file() {
        let temp_str = std::fs::read_to_string(config_file).unwrap();
        let g1: Config = toml::from_str(&temp_str).unwrap();
        Some(g1)
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PatternDuration {
    pub(crate) counter_type: CounterType,
    pub(crate) duration: u64,
}

use std::str::FromStr;
pub(crate) fn parse_pattern_duration(src: &str) -> Result<PatternDuration, String> {
    let v: Vec<&str> = src.trim().split('=').collect();
    if v.len() != 2 {
        return Err(format!("Invalid duration: no '=' found in {}", src));
    }
    if let Ok(counter_type) = CounterType::from_str(v.get(0).unwrap()) {
        if let Ok(duration) = u64::from_str(v.get(1).unwrap()) {
            Ok(PatternDuration {
                counter_type,
                duration,
            })
        } else {
            Err(format!("Invalid duration in {}", src))
        }
    } else {
        Err(format!("Invalid counter type in {}", src))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn counter_type_deserialization() {
        let uut = CounterType::Iteration;
        assert_eq!(uut.to_string(), "Iterations".to_string());
        let uut: CounterType = "iteration".to_string().parse().unwrap();
        assert_eq!(uut, CounterType::Iteration);
    }
    #[test]
    fn deserialization() {
        let input = include_bytes!("../resources/tests/config.toml");
        let got = toml::from_slice::<Config>(input);
        assert!(got.is_ok());
    }

    #[test]
    fn pattern_duration_parsing() {
        let input = "iterations=5";
        let uut = parse_pattern_duration(input);
        assert!(uut.is_ok());
        let uut = uut.unwrap();
        assert_eq!(uut.counter_type, CounterType::Iteration);
        assert_eq!(uut.duration, 5);

        let input = "Time=300";
        let uut = parse_pattern_duration(input);
        assert!(uut.is_ok());
        let uut = uut.unwrap();
        assert_eq!(uut.counter_type, CounterType::Time);
        assert_eq!(uut.duration, 300);

        let input = " Time=300 ";
        let uut = parse_pattern_duration(input);
        assert!(uut.is_ok());

        let input = "seconds=300";
        let uut = parse_pattern_duration(input);
        assert!(uut.is_err());
    }
}
