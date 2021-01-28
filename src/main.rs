mod breathe;
mod config;

use async_std::prelude::*;

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
            "Breathe in:   {}
Hold:         {}
Breathe out:  {}
Hold:         {}
Session type: {}
{}   {} {}",
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
async fn breathe(params: BreathSessionParams) {
    let mut interval = async_std::stream::interval(std::time::Duration::from_secs(1));
    let mut session =
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
    let multibar = indicatif::MultiProgress::new();
    let pb = multibar.add(indicatif::ProgressBar::new(session.get_lengths_lcm()));
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:>4} {bar:80} {msg}")
            .progress_chars("=>-")
            .tick_chars(r#"-\|/ "#),
    );
    pb.set_message(&session.get_phase_str());
    let total = multibar.add(indicatif::ProgressBar::new(session.get_session_length()));
    total.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{percent:>3}% {bar:80} {elapsed_precise}")
            .progress_chars("=>-"),
    );
    async_std::task::spawn(async move {
        multibar.join_and_clear().unwrap_or_default();
    });
    while interval.next().await.is_some() {
        session.inc();
        if session.is_completed() {
            break;
        }
        total.inc(1);
        if session.is_state_changed() {
            pb.set_message(&session.get_phase_str());
            pb.reset();
        } else {
            pb.inc(session.get_lengths_lcm() / session.get_current_phase_length());
        }
    }
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
}

use structopt_flags::GetWithDefault;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            let session = BreathSessionParams {
                pattern: pattern.clone(),
                session_type: pattern.counter_type.unwrap_or(config.counter_type),
                duration: pattern.duration.unwrap_or(config.duration),
            };
            breathe(session).await;
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
