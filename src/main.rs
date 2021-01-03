mod breath;
mod config;

use async_std::prelude::*;

struct BreathSessionParams {
    pattern: config::Pattern,
    session_type: config::CounterType,
    duration: u64,
}

async fn breath(params: BreathSessionParams) {
    let mut interval = async_std::stream::interval(std::time::Duration::from_secs(1));
    let mut session =
        breath::BreathingSession::new(&params.pattern, params.session_type, params.duration);

    session.print_params();
    let multibar = indicatif::MultiProgress::new();
    let pb = multibar.add(indicatif::ProgressBar::new(session.get_lengths_lcm()));
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner} {bar:40} {msg}")
            .progress_chars("=>-"),
    );
    pb.set_message(&session.get_phase_str());
    let total = multibar.add(indicatif::ProgressBar::new(session.get_session_length()));
    total.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{percent:>3}% {wide_bar} {elapsed_precise}")
            .progress_chars("=>-"),
    );
    async_std::task::spawn(async move {
        multibar.join_and_clear().unwrap_or_default();
    });
    while interval.next().await.is_some() {
        pb.inc(session.get_lengths_lcm() / session.get_current_phase_length());
        total.inc(1);
        session.inc();
        if session.is_completed() {
            break;
        }
        if session.is_state_changed() {
            pb.set_message(&session.get_phase_str());
            pb.reset();
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
        if let Some(pattern) = config.get_pattern(&opt.pattern) {
            let session = BreathSessionParams {
                pattern: pattern.clone(),
                session_type: pattern.counter_type.unwrap_or(config.counter_type),
                duration: pattern.duration.unwrap_or(config.duration),
            };
            breath(session).await;
        }
    }
    Ok(())
}
