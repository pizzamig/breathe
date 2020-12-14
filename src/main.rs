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
    limits.insert(BreathPhase::HoldIn, 8u32);
    limits.insert(BreathPhase::BreathOut, 7u32);
    limits.insert(BreathPhase::HoldOut, 0u32);

    println!("NOW: {}", state.phase);
    while interval.next().await.is_some() {
        state.counter += 1;
        while *limits.get(&state.phase).unwrap() <= state.counter {
            state.next();
            println!("NOW: {}", state.phase);
        }
    }
}

#[async_std::main]
async fn main() {
    breath().await;
}
