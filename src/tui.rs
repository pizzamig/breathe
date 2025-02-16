use crate::breathe;
use crate::config;

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

pub(crate) fn run(opt: breathe::BreathSessionOpt) {
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
