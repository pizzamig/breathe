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
        match self.patterns.get(pattern_name) {
            None => None,
            Some(p) => Some(p.clone()),
        }
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
}

#[derive(Debug, Clone, EnumString, Display, Deserialize, PartialEq, Copy)]
pub(crate) enum CounterType {
    Time,
    #[strum(
        serialize = "iteration",
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
}
