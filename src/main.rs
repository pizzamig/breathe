use async_std::prelude::*;
use std::collections::HashMap;
use strum::Display;

#[derive(PartialEq, Hash, Display)]
enum BreathPhase {
    BreathIn,
    HoldIn,
    BreathOut,
    HoldOut,
}
impl Eq for BreathPhase {}

struct State {
    phase: BreathPhase,
    counter: u32,
}

impl State {
    fn new() -> Self {
        State {
            phase: BreathPhase::BreathIn,
            counter: 0,
        }
    }
    fn next(&mut self) {
        self.phase = match self.phase {
            BreathPhase::BreathIn => BreathPhase::HoldIn,
            BreathPhase::HoldIn => BreathPhase::BreathOut,
            BreathPhase::BreathOut => BreathPhase::HoldOut,
            BreathPhase::HoldOut => BreathPhase::BreathIn,
        };
        self.counter = 0;
    }
}

async fn breath() {
    let mut interval = async_std::stream::interval(std::time::Duration::from_secs(1));
    let mut state = State::new();
    let mut limits = HashMap::new();
    limits.insert(BreathPhase::BreathIn, 4u32);
    limits.insert(BreathPhase::HoldIn, 7u32);
    limits.insert(BreathPhase::BreathOut, 8u32);
    limits.insert(BreathPhase::HoldOut, 0u32);

    let multibar = indicatif::MultiProgress::new();
    let pb = multibar.add(indicatif::ProgressBar::new(56));
    pb.set_style(indicatif::ProgressStyle::default_bar().template("{spinner} {bar:40} {msg}"));
    pb.set_message(&state.phase.to_string());
    let total = multibar.add(indicatif::ProgressBar::new(300));
    total.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{percent:>3}% {wide_bar} {elapsed_precise}"),
    );
    async_std::task::spawn(async move {
        multibar.join_and_clear().unwrap_or_default();
    });
    while interval.next().await.is_some() {
        state.counter += 1;
        pb.inc(56 / (*limits.get(&state.phase).unwrap() as u64));
        total.inc(1);
        while *limits.get(&state.phase).unwrap() <= state.counter {
            state.next();
            pb.set_message(&state.phase.to_string());
            pb.reset();
        }
        if total.position() == total.length() {
            break;
        }
    }
}

#[async_std::main]
async fn main() {
    breath().await;
}
