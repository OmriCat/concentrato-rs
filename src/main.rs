use crate::state::{State, TimedState};
use color_eyre::eyre;
use console::{Key, Term};
use either::Either;
use std::fmt::Debug;
use std::io::Write;
use std::process;
use std::time::Duration;
use tokio::time;
use tokio::time::{Instant, Interval};

mod state;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let mut tick_interval = time::interval(time::Duration::from_secs(1));
    let work_duration = Duration::from_secs(2);
    let break_duration = Duration::from_secs(2);

    let term = Term::stdout();

    loop {
        term.write_line("Starting work")?;

        let working_state = State::new().start_working(work_duration, Instant::now().into_std());
        let post_work_state = run_timer(working_state, &mut tick_interval, |_s, d| {
            term.clear_line()?;
            write!(
                &term,
                "State: Working\tTime remaining {}",
                format_duration(d)
            )?;
            Ok(())
        })
        .await?;

        term.clear_line()?;
        term.write_line("Work completed. Continue with break (Y/n)?")?;
        if read_continue(&term)? {
            term.write_line("Starting break")?;
            let break_state =
                post_work_state.start_break(break_duration, Instant::now().into_std());
            let _ = run_timer(break_state, &mut tick_interval, |_s, d| {
                term.clear_line()?;
                write!(&term, "State: Break\tTime remaining {}", format_duration(d))?;
                Ok(())
            })
            .await?;
        }

        term.clear_line()?;
        term.write_line("All complete! Ready for another (Y/n)?")?;
        if !read_continue(&term)? {
            process::exit(0)
        }
    }
}

fn read_continue(term: &Term) -> eyre::Result<bool> {
    loop {
        match term.read_key()? {
            Key::Enter | Key::Char('y') | Key::Char('Y') | Key::Char(' ') => return Ok(true),
            Key::Escape | Key::Char('n') | Key::Char('N') => return Ok(false),
            _ => continue,
        }
    }
}

fn format_duration(duration: &Duration) -> String {
    let rounded_seconds: u64 = ((duration.as_millis() + 1000 - 1) / 1000)
        .try_into()
        .unwrap_or(u64::MAX);
    let minutes = rounded_seconds / 60;
    let seconds = rounded_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

async fn run_timer<I, T, S, F>(
    initial_state: S,
    interval: &mut Interval,
    f: F,
) -> eyre::Result<State<T>>
where
    S: TimedState<I, T>,
    I: Debug + Eq + PartialEq + Clone,
    T: Debug + Eq + PartialEq + Clone,
    State<I>: TimedState<I, T>,
    F: Fn(&State<I>, &Duration) -> eyre::Result<()>,
{
    let start_time = initial_state.start_time();
    let mut new_state_either = initial_state.tick(&start_time.elapsed());
    interval.tick().await;
    while let Either::Left(new_state) = new_state_either {
        let remaining_time = new_state.period_length() - start_time.elapsed();
        f(&new_state, &remaining_time)?;
        interval.tick().await;
        new_state_either = new_state.tick(&start_time.elapsed())
    }
    // Unwrap is safe here because this line can only be reached if the if let above failed
    // i.e. the either is Either::Right
    Ok(new_state_either.right().unwrap())
}
