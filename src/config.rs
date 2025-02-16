use anyhow::{anyhow, Context};
use serde::Deserialize;
use std::collections::HashMap;
use strum::Display;

const _GLOBAL_CONFIG_DIR_1: &str = "/etc";
const _GLOBAL_CONFIG_DIR_2: &str = "/usr/local/etc";
const CONFIG_DEFAULT_NAME: &str = "breathe.toml";

pub(crate) fn get_default_config_file() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join(CONFIG_DEFAULT_NAME)
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    patterns: HashMap<String, Pattern>,
    #[serde(flatten)]
    pub(crate) pattern_length: PatternLength,
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
    pub(crate) fn compute_pattern(
        &self,
        pattern_name: &str,
        opt_pattern_length: Option<PatternLength>,
    ) -> anyhow::Result<Pattern> {
        let mut result = self
            .patterns
            .get(pattern_name)
            .with_context(|| format!("Pattern {pattern_name} not found"))?
            .clone();
        result.pattern_length = Some(
            opt_pattern_length.unwrap_or(result.pattern_length.unwrap_or(self.pattern_length)),
        );
        Ok(result)
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
    pub(crate) description: String,
    #[serde(flatten)]
    pub(crate) pattern_length: Option<PatternLength>,
}

impl Pattern {
    pub(crate) fn length(&self) -> u64 {
        self.breath_in
            + self.breath_out
            + self.hold_in.unwrap_or_default()
            + self.hold_out.unwrap_or_default()
    }

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
        if let Some(pl) = self.pattern_length {
            match pl {
                PatternLength::Time(d) => format!("{d} seconds"),
                PatternLength::Iterations(d) => format!("{d} iterations"),
            }
        } else {
            "".to_string()
        }
    }
}

use std::str::FromStr;

#[derive(Debug, Clone, Display, Deserialize, PartialEq, Copy)]
#[strum(ascii_case_insensitive)]
pub(crate) enum PatternLength {
    #[strum(to_string = "Time={0}")]
    #[serde(alias = "time")]
    Time(u64),
    #[strum(to_string = "Iterations={0}")]
    #[serde(alias = "iteration", alias = "Iteration", alias = "iterations")]
    Iterations(u64),
}

impl FromStr for PatternLength {
    type Err = anyhow::Error;

    fn from_str(src: &str) -> anyhow::Result<Self> {
        let mut src = src.to_string();
        src.retain(|c| c != ' ');
        let v: Vec<&str> = src.trim().split('=').collect();
        if v.len() != 2 {
            return Err(anyhow!(
                "Invalid pattern length specification: no '=' found in {}",
                src
            ));
        }
        let duration = u64::from_str(v.get(1).unwrap())
            .with_context(|| format!("Invalid duration {}", v.get(1).unwrap()))?;
        match *v.first().unwrap() {
            "time" | "Time" => Ok(PatternLength::Time(duration)),
            "iterations" | "Iterations" | "iteration" | "Iteration" => {
                Ok(PatternLength::Iterations(duration))
            }
            _ => Err(anyhow!(
                "Duration type {} not recognized",
                v.first().unwrap()
            )),
        }
    }
}

impl PatternLength {
    fn _is_iterations(&self) -> bool {
        matches!(self, PatternLength::Iterations(_))
    }

    fn _is_time(&self) -> bool {
        matches!(self, PatternLength::Time(_))
    }

    fn _duration(&self) -> u64 {
        match self {
            PatternLength::Time(d) => *d,
            PatternLength::Iterations(d) => *d,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    fn get_standard_config() -> Config {
        let result = from_file(Path::new("resources/tests/config.toml"))
            .inspect_err(|e| eprintln!("{:?}", e));
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
    fn compute_pattern_lengths() {
        let config = get_standard_config();
        let p1 = config
            .compute_pattern("relax", None)
            .inspect_err(|e| eprintln!("{:?}", e));
        assert!(p1.is_ok());
        assert_eq!(
            p1.unwrap().pattern_length,
            Some(PatternLength::Iterations(8))
        );
        let p2 = config
            .compute_pattern("relax", Some(PatternLength::Iterations(20)))
            .inspect_err(|e| eprintln!("{:?}", e));
        assert!(p2.is_ok());
        assert_eq!(
            p2.unwrap().pattern_length,
            Some(PatternLength::Iterations(20))
        );
        let p3 = config
            .compute_pattern("long_and_deep", None)
            .inspect_err(|e| eprintln!("{:?}", e));
        assert!(p3.is_ok());
        assert_eq!(p3.unwrap().pattern_length, Some(PatternLength::Time(300)));
    }

    #[test]
    fn compute_patterns() {
        let config = get_standard_config();
        let pattern = config.compute_pattern("relax", None);
        assert!(pattern.is_ok());
        let pattern = pattern.unwrap();
        assert_eq!(pattern.breath_in, 4);
        assert_eq!(pattern.breath_out, 8);
        assert_eq!(pattern.hold_in, Some(7));
        assert_eq!(pattern.hold_out, None);
        assert!(pattern.pattern_length.is_some());
        assert_eq!(pattern.pattern_length, Some(PatternLength::Iterations(8)))
    }

    #[test]
    fn deserialization() {
        let input = include_str!("../resources/tests/config.toml");
        let got = toml::from_str::<Config>(input).inspect_err(|e| eprintln!("{:?}", e));
        assert!(got.is_ok());
    }

    fn pl_parse_test(s: &str, pl: PatternLength) {
        // toml
        let result: Result<PatternLength, _> = toml::from_str(s);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, pl);
        // from_str
        let result: Result<PatternLength, _> = s.to_string().parse();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, pl);
    }
    #[test]
    fn pattern_length_deserialization() {
        pl_parse_test("Iterations=5", PatternLength::Iterations(5));
        pl_parse_test("iterations = 5", PatternLength::Iterations(5));
        pl_parse_test("iteration = 123", PatternLength::Iterations(123));
        pl_parse_test("time = 20", PatternLength::Time(20))
    }
}
