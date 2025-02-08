mod breathe;
mod config;

struct BreathSessionParams {
    pattern: config::Pattern,
    session_type: config::CounterType,
    duration: u64,
}

impl std::fmt::Display for BreathSessionParams {
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
        breathe::BreathingSession::new(&params.pattern, params.session_type, params.duration);

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
    pb.set_message(session.get_phase_str());
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
                    pb.set_message(session.get_phase_str());
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

use structopt::StructOpt;
use structopt_flags::LogLevel;

#[derive(Debug, StructOpt)]
#[structopt(name = "breathe", about = "A cli tool with breathing exercises")]
struct Opt {
    #[structopt(flatten)]
    config_file: structopt_flags::ConfigFileNoDef,
    #[structopt(flatten)]
    verbose: structopt_flags::Verbose,
    /// select the breathe pattern you want to practice
    #[structopt(short, long, default_value = "relax")]
    pattern: String,
    /// list all available breathe patterns
    #[structopt(short, long)]
    list: bool,
    /// specify a different duartion in the form of durationType=nn
    #[structopt(short = "d",long, parse(try_from_str = config::parse_pattern_duration))]
    pattern_duration: Option<config::PatternDuration>,
}

use structopt_flags::GetWithDefault;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    env_logger::builder().filter_level(opt.verbose.get_level_filter());
    let config_file = opt
        .config_file
        .get_with_default(config::get_default_config_file());
    if let Some(config) = config::get_config(&config_file) {
        if opt.list {
            config.print_pattern_list();
            return Ok(());
        } else if let Some(pattern) = config.get_pattern(&opt.pattern) {
            let pattern_duration = match opt.pattern_duration {
                Some(pd) => pd,
                None => config::PatternDuration {
                    counter_type: pattern.counter_type.unwrap_or(config.counter_type),
                    duration: pattern.duration.unwrap_or(config.duration),
                },
            };
            let session = BreathSessionParams {
                pattern: pattern.clone(),
                session_type: pattern_duration.counter_type,
                duration: pattern_duration.duration,
            };
            breathe(session);
        } else {
            // TODO: implement proper error
            eprintln!("no patter found, damn");
        }
    } else {
        // TODO: implement proper error
        eprintln!("no config file found, damn");
    }
    Ok(())
}
