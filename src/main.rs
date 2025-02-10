mod breathe;
mod config;

struct BreathSessionParams<'a> {
    pattern: &'a config::Pattern,
    session_type: config::CounterType,
    duration: u64,
}

impl std::fmt::Display for BreathSessionParams<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = match self.session_type {
            config::CounterType::Time => "Duration:  ",
            config::CounterType::Iteration => "Iterations:",
        };
        let duration_unit = match self.session_type {
            config::CounterType::Time => "seconds",
            _ => "",
        };
        write!(
            f,
            "Description: {}
Breathe in:   {}
Hold:         {}
Breathe out:  {}
Hold:         {}
Session type: {}
{}   {} {}",
            self.pattern.description,
            self.pattern.breath_in,
            self.pattern.hold_in.unwrap_or(0),
            self.pattern.breath_out,
            self.pattern.hold_out.unwrap_or(0),
            self.session_type,
            duration,
            self.duration,
            duration_unit
        )
    }
}

use std::sync::{Arc, Mutex};
use std::thread;

fn breathe(params: BreathSessionParams) {
    let session =
        breathe::BreathingSession::new(params.pattern, params.session_type, params.duration);

    println!("{}", params);
    if let config::CounterType::Iteration = params.session_type {
        session.print_params();
    }
    let user_choice = dialoguer::Confirm::new()
        .with_prompt("Would you like to start the breathing session?")
        .default(true)
        .interact()
        .unwrap_or(false);
    if !user_choice {
        return;
    }
    let pb = indicatif::ProgressBar::new(session.get_lengths_lcm());
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .progress_chars("=>-")
            .tick_chars(r#"-\|/ "#)
            .template("{spinner:>4} {wide_bar} {msg}")
            .unwrap(),
    );
    pb.set_message(session.phase_as_str());
    let session = Arc::new(Mutex::new(session));
    let timer = timer::Timer::new();
    let guard = {
        let session = session.clone();
        timer.schedule_repeating(chrono::Duration::seconds(1), move || {
            let mut session = session.lock().unwrap();
            if !session.is_completed() {
                session.inc();
                if session.is_state_changed() {
                    pb.inc(session.get_lengths_lcm() / session.get_current_phase_length());
                    pb.set_message(session.phase_as_str());
                    pb.dec(pb.position());
                } else {
                    pb.inc(session.get_lengths_lcm() / session.get_current_phase_length());
                }
            }
        })
    };
    loop {
        thread::sleep(std::time::Duration::new(0, 501));
        {
            let session = session.clone();
            let session = session.lock().unwrap();
            if session.is_completed() {
                break;
            }
        }
    }
    drop(guard);
}

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
    pattern_duration: Option<config::PatternDuration>,
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
    let pattern = config.get_pattern(&opt.pattern)?;
    let pattern_duration = match opt.pattern_duration {
        Some(pd) => pd,
        None => pattern.pattern_duration.unwrap_or(config.pattern_duration),
    };
    let session = BreathSessionParams {
        pattern,
        session_type: pattern_duration.counter_type,
        duration: pattern_duration.duration,
    };
    breathe(session);
    Ok(())
}
