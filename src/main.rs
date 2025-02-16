mod breathe;
mod config;
mod tui;

use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "breathe", about = "A cli tool with breathing exercises")]
struct Opt {
    #[arg(
        name = "config_file",
        long = "config",
        short = 'c',
        default_value_os_t = config::get_default_config_file(),
    )]
    config_file: PathBuf,
    #[arg(
        name = "verbose",
        long = "verboe",
        short = 'v',
        action = clap::ArgAction::Count,
        global = true
    )]
    verbosity_level: u8,
    /// select the breathe pattern you want to practice
    #[arg(short, long, default_value = "relax")]
    pattern: String,
    /// list all available breathe patterns
    #[arg(short, long)]
    list: bool,
    /// specify a different duartion in the form of durationType=nn
    #[arg(short = 'd', long)]
    pattern_length: Option<config::PatternLength>,
}

fn get_level_filter(verbosity_level: u8) -> log::LevelFilter {
    match verbosity_level {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}
fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    env_logger::builder().filter_level(get_level_filter(opt.verbosity_level));
    let config = config::from_file(&opt.config_file)?;
    if opt.list {
        config.print_pattern_list();
        return Ok(());
    }
    let mut pattern = config.get_pattern(&opt.pattern)?.clone();
    pattern.pattern_length = Some(
        opt.pattern_length
            .unwrap_or(pattern.pattern_length.unwrap_or(config.pattern_length)),
    );
    let bso = breathe::BreathSessionOpt { pattern: &pattern };
    tui::run(bso);
    Ok(())
}
