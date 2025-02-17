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
    let mb = indicatif::MultiProgress::new();
    let pb = indicatif::ProgressBar::new(session.get_lengths_lcm());
    let pb = mb.add(pb);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .progress_chars("=>-")
            .tick_chars(r#"-\|/ "#)
            .template(
                format!(
                    "{{spinner:>4}} {{wide_bar:.cyan/blue}} {{msg:<{}}}",
                    breathe::MAX_BREATHE_PHASE_STR_LEN + 1
                )
                .as_str(),
            )
            .unwrap(),
    );

    let total_pb = indicatif::ProgressBar::new(session.session_length);
    let total_pb = mb.add(total_pb);
    total_pb.set_style(
        indicatif::ProgressStyle::with_template(
            format!(
                "{{percent:>3}}% {{wide_bar:.cyan/blue}} {{eta:<{}}}",
                breathe::MAX_BREATHE_PHASE_STR_LEN + 1
            )
            .as_str(),
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    let session = Arc::new(Mutex::new(session));
    let timer = timer::Timer::new();
    let guard = {
        let session = session.clone();
        pb.set_message(session.lock().unwrap().phase_as_str());
        total_pb.reset();
        let mb = mb.clone();
        timer.schedule_repeating(chrono::Duration::seconds(1), move || {
            let mut session = session.lock().unwrap();
            if !session.is_completed() {
                session.inc();
                total_pb.inc(1);
                if session.is_state_changed() {
                    pb.inc(session.get_lengths_lcm() / session.get_current_phase_length());
                    pb.set_message(session.phase_as_str());
                    pb.dec(pb.position());
                } else {
                    pb.inc(session.get_lengths_lcm() / session.get_current_phase_length());
                }
            } else {
                mb.clear().unwrap();
            }
        })
    };
    loop {
        thread::sleep(std::time::Duration::new(0, 501));
        {
            let session = session.clone();
            let session = session.lock().unwrap();
            if session.is_completed() {
                mb.clear().unwrap();
                break;
            }
        }
    }
    drop(guard);
}
