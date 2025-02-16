mod breathe;
mod config;

fn print_session_opt(opt: &breathe::BreathSessionOpt) {
    let pl = opt.pattern.pattern_length.unwrap();
    let duration_unit = if matches!(pl, config::PatternLength::Time(_)) {
        "seconds"
    } else {
        ""
    };
    println!(
        "Description:   {}
Breathe in:     {}
Hold:           {}
Breathe out:    {}
Hold:           {}
Session length: {} {}",
        opt.pattern.description,
        opt.pattern.breath_in,
        opt.pattern.hold_in.unwrap_or(0),
        opt.pattern.breath_out,
        opt.pattern.hold_out.unwrap_or(0),
        pl,
        duration_unit
    )
}

use std::sync::{Arc, Mutex};
use std::thread;

fn breathe(opt: breathe::BreathSessionOpt) {
    let session = breathe::BreathingSession::with_opt(&opt);

    print_session_opt(&opt);
    if matches!(
        opt.pattern.pattern_length.unwrap(),
        config::PatternLength::Iterations(_)
    ) {
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
    breathe(bso);
    Ok(())
}
