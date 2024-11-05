use std::fmt::Debug;
use std::time::{Duration, Instant};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PreWork;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Working {
    start_time: Instant,
    working_period: Duration,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PostWork;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Break {
    start_time: Instant,
    break_length: Duration,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Complete;

#[derive(Debug)]
pub enum TickResult<C, T> {
    Continue(State<C>),
    Complete(State<T>),
}

impl<C, T> TickResult<C, T> {
    pub(crate) fn complete_value(self) -> Option<State<T>> {
        match self {
            TickResult::Continue(_) => None,
            TickResult::Complete(value) => Some(value),
        }
    }
}

pub trait TimedState<SelfState, NextState>
where
    State<SelfState>: TimedState<SelfState, NextState>,
{
    fn period_length(&self) -> Duration;
    fn start_time(&self) -> Instant;
    fn tick(self, elapsed_time: &Duration) -> TickResult<SelfState, NextState>;
}

pub trait StoppableState<StopState> {
    fn stop(self) -> State<StopState>;
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct State<T> {
    state: T,
}

impl State<PreWork> {
    pub fn new() -> Self {
        State { state: PreWork }
    }

    pub fn start_working(self, working_period: Duration, start_time: Instant) -> State<Working> {
        State {
            state: Working {
                working_period,
                start_time,
            },
        }
    }
}

impl StoppableState<PreWork> for State<Working> {
    fn stop(self) -> State<PreWork> {
        State::new()
    }
}

impl TimedState<Working, PostWork> for State<Working> {
    fn period_length(&self) -> Duration {
        self.state.working_period
    }

    fn start_time(&self) -> Instant {
        self.state.start_time
    }

    fn tick(self, elapsed_time: &Duration) -> TickResult<Working, PostWork> {
        if elapsed_time < &self.period_length() {
            TickResult::Continue(self)
        } else {
            TickResult::Complete(State { state: PostWork })
        }
    }
}

impl State<PostWork> {
    pub fn start_break(self, break_length: Duration, start_time: Instant) -> State<Break> {
        State {
            state: Break {
                break_length,
                start_time,
            },
        }
    }
}

impl StoppableState<Complete> for State<Break> {
    fn stop(self) -> State<Complete> {
        State { state: Complete }
    }
}
impl TimedState<Break, Complete> for State<Break> {
    fn period_length(&self) -> Duration {
        self.state.break_length
    }

    fn start_time(&self) -> Instant {
        self.state.start_time
    }

    fn tick(self, elapsed_time: &Duration) -> TickResult<Break, Complete> {
        if elapsed_time < &self.period_length() {
            TickResult::Continue(self)
        } else {
            TickResult::Complete(State { state: Complete })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn working_state_remains_working_before_timeout() {
        let start_time = Instant::now();
        let working_state = State {
            state: Working {
                working_period: Duration::from_secs(30),
                start_time,
            },
        };
        let new_state = working_state.tick(&Duration::from_secs(5));
        assert!(matches!(new_state, TickResult::Continue(_)))
    }

    #[test]
    fn working_state_transitions_to_post_work() {
        let start_time = Instant::now();
        let working_state = State {
            state: Working {
                working_period: Duration::from_secs(30),
                start_time,
            },
        };
        let new_state = working_state.tick(&Duration::from_millis(30_005));
        assert!(matches!(
            new_state,
            TickResult::Complete(State { state: PostWork })
        ))
    }

    #[test]
    fn break_state_remains_break_before_timeout() {
        let start_time = Instant::now();
        let break_state = State {
            state: Break {
                break_length: Duration::from_secs(30),
                start_time,
            },
        };
        let new_state = break_state.tick(&Duration::from_secs(5));
        assert!(matches!(new_state, TickResult::Continue(_)))
    }

    #[test]
    fn break_state_transitions_to_complete() {
        let start_time = Instant::now();
        let break_state = State {
            state: Break {
                break_length: Duration::from_secs(30),
                start_time,
            },
        };
        let new_state = break_state.tick(&Duration::from_millis(30_005));
        assert!(matches!(
            new_state,
            TickResult::Complete(State { state: Complete })
        ))
    }
}
