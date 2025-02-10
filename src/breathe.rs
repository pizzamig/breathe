use crate::config::{CounterType, Pattern};
use std::collections::HashMap;
use strum::{Display, IntoStaticStr};

/// Breathing can be in 4 possible phases.
/// This struct represent those 4 possible values
#[derive(Debug, Copy, Clone, PartialEq, Hash, Display, IntoStaticStr)]
pub(crate) enum BreathPhase {
    BreathIn,
    HoldIn,
    BreathOut,
    HoldOut,
}
impl Eq for BreathPhase {}

impl BreathPhase {
    // Breath phases are ordered. This function returns the next breathing phase
    fn next(self) -> Self {
        match self {
            BreathPhase::BreathIn => BreathPhase::HoldIn,
            BreathPhase::HoldIn => BreathPhase::BreathOut,
            BreathPhase::BreathOut => BreathPhase::HoldOut,
            BreathPhase::HoldOut => BreathPhase::BreathIn,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BreathCycle {
    cycle: HashMap<BreathPhase, u64>,
    pub(crate) cycle_length: u64,
    pub(crate) lcm: u64,
}

fn from_pattern(pattern: &Pattern) -> BreathCycle {
    let mut cycle = HashMap::new();
    cycle.insert(BreathPhase::BreathIn, pattern.breath_in);
    cycle.insert(BreathPhase::BreathOut, pattern.breath_out);
    cycle.insert(BreathPhase::HoldIn, pattern.hold_in.unwrap_or(0));
    cycle.insert(BreathPhase::HoldOut, pattern.hold_out.unwrap_or(0));
    let lcm = cycle
        .values()
        .filter(|&&x| x != 0)
        .fold(1, |lcm, &x| num_integer::lcm(lcm, x));
    BreathCycle {
        cycle,
        cycle_length: pattern.breath_in
            + pattern.breath_out
            + pattern.hold_in.unwrap_or(0)
            + pattern.hold_out.unwrap_or(0),
        lcm,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BreathingSession {
    cycle: BreathCycle,
    session_length: u64,
    total_counter: u64,
    pub(crate) current_state: BreathPhase,
    state_counter: u64,
    state_changed: bool,
}

impl BreathingSession {
    pub(crate) fn new(pattern: &Pattern, counter_type: CounterType, duration: u64) -> Self {
        let cycle: BreathCycle = from_pattern(pattern);
        let session_length = match counter_type {
            CounterType::Iteration => cycle.cycle_length * duration,
            CounterType::Time => duration,
        };
        BreathingSession {
            cycle,
            session_length,
            total_counter: 0,
            state_counter: 0,
            current_state: BreathPhase::BreathIn,
            state_changed: true,
        }
    }

    pub(crate) fn get_current_phase_length(&self) -> u64 {
        *self.cycle.cycle.get(&self.current_state).unwrap()
    }

    pub(crate) fn phase_as_str(&self) -> &'static str {
        self.current_state.into()
    }

    pub(crate) fn get_lengths_lcm(&self) -> u64 {
        self.cycle.lcm
    }

    fn next_state(&mut self) {
        let mut temp_state = self.current_state.next();
        while self.cycle.cycle.get(&temp_state).unwrap() == &0 {
            temp_state = temp_state.next();
        }
        self.current_state = temp_state;
        self.state_counter = 0;
    }
    pub(crate) fn inc(&mut self) {
        if self.total_counter >= self.session_length {
            return;
        }
        self.total_counter += 1;
        self.state_counter += 1;
        if self.state_counter >= *self.cycle.cycle.get(&self.current_state).unwrap() {
            self.next_state();
            self.state_changed = true;
        } else {
            self.state_changed = false;
        }
    }
    pub(crate) fn is_completed(&self) -> bool {
        self.total_counter >= self.session_length
    }
    pub(crate) fn is_state_changed(&self) -> bool {
        self.state_changed
    }
    pub(crate) fn print_params(&self) {
        println!("session lenght: {} seconds", self.session_length);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn breath_phase_next() {
        let phase = BreathPhase::BreathIn;
        assert_eq!(phase.next(), BreathPhase::HoldIn);
        assert_eq!(phase.next().next(), BreathPhase::BreathOut);
        assert_eq!(phase.next().next().next(), BreathPhase::HoldOut);
        assert_eq!(phase.next().next().next().next(), BreathPhase::BreathIn);
    }
    #[test]
    fn breath_cycle_from_pattern() {
        let uut = Pattern {
            breath_in: 4,
            hold_in: Some(7),
            breath_out: 8,
            hold_out: None,
            pattern_duration: None,
            description: "Test pattern".to_string(),
        };
        let got: BreathCycle = from_pattern(&uut);
        assert_eq!(got.cycle.get(&BreathPhase::BreathIn).unwrap(), &4);
        assert_eq!(got.cycle.get(&BreathPhase::HoldIn).unwrap(), &7);
        assert_eq!(got.cycle.get(&BreathPhase::BreathOut).unwrap(), &8);
        assert_eq!(got.cycle.get(&BreathPhase::HoldOut).unwrap(), &0);
        assert_eq!(got.cycle_length, 19);
    }

    #[test]
    fn breath_get_lengths_lcm() {
        let uut = Pattern {
            breath_in: 4,
            hold_in: Some(7),
            breath_out: 8,
            hold_out: None,
            pattern_duration: None,
            description: "Test pattern".to_string(),
        };
        let got: BreathCycle = from_pattern(&uut);
        assert_eq!(got.lcm, 56);
    }
    #[test]
    fn breath_session_ctor_time_session() {
        let p = Pattern {
            breath_in: 4,
            hold_in: Some(7),
            breath_out: 8,
            hold_out: None,
            pattern_duration: None,
            description: "Test pattern".to_string(),
        };
        let got = BreathingSession::new(&p, CounterType::Time, 60);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::BreathIn).unwrap(), &4);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::HoldIn).unwrap(), &7);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::BreathOut).unwrap(), &8);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::HoldOut).unwrap(), &0);
    }

    #[test]
    fn breath_session_ctor_iter_session() {
        let p = Pattern {
            breath_in: 4,
            hold_in: Some(7),
            breath_out: 8,
            hold_out: None,
            pattern_duration: None,
            description: "Test pattern".to_string(),
        };
        let got = BreathingSession::new(&p, CounterType::Iteration, 8);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::BreathIn).unwrap(), &4);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::HoldIn).unwrap(), &7);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::BreathOut).unwrap(), &8);
        assert_eq!(got.cycle.cycle.get(&BreathPhase::HoldOut).unwrap(), &0);
    }

    #[test]
    fn breath_session_ctor_next() {
        let p = Pattern {
            breath_in: 4,
            hold_in: Some(7),
            breath_out: 8,
            hold_out: None,
            pattern_duration: None,
            description: "Test pattern".to_string(),
        };
        let mut got = BreathingSession::new(&p, CounterType::Iteration, 2);
        assert_eq!(got.get_current_phase_length(), 4);
        for _ in 0..4 {
            got.inc();
        }
        assert_eq!(got.total_counter, 4);
        assert_eq!(got.state_counter, 0);
        assert_eq!(got.current_state, BreathPhase::HoldIn);
        assert_eq!(got.get_current_phase_length(), 7);
        for _ in 0..7 {
            got.inc();
        }
        assert_eq!(got.total_counter, 11);
        assert_eq!(got.state_counter, 0);
        assert_eq!(got.current_state, BreathPhase::BreathOut);
        assert_eq!(got.get_current_phase_length(), 8);
        for _ in 0..8 {
            got.inc();
        }
        assert_eq!(got.total_counter, 19);
        assert_eq!(got.state_counter, 0);
        assert_eq!(got.current_state, BreathPhase::BreathIn);
        for _ in 0..19 {
            got.inc();
        }
        assert_eq!(got.total_counter, 38);
        assert_eq!(got.state_counter, 0);
        assert_eq!(got.current_state, BreathPhase::BreathIn);
        assert!(got.is_completed());
        got.inc();
        assert_eq!(got.total_counter, 38);
        assert_eq!(got.state_counter, 0);
        assert_eq!(got.current_state, BreathPhase::BreathIn);
        assert!(got.is_completed());
    }
}
