use std::cmp::{min, max};
use time;

/// Collects FPS statistics.
pub struct Fps {
  state: State,
}

/// FPS statistics.
pub struct Stats {
  /// Minimum FPS.
  pub min: f32,
  /// Average FPS.
  pub avg: f32,
  /// Maximum FPS.
  pub max: f32,
}

#[derive(Debug)]
enum State {
  Stopped,
  Started { start_ns: u64, },
  FirstTick { start_ns: u64, first_ns: u64, },
  MoreTicks { start_ns: u64, prev_ns: u64, stats: StatsCollected, },
}

#[derive(Clone, Debug)]
struct StatsCollected {
  count: u64,
  sum: u64,
  min: u64,
  max: u64,
}

impl StatsCollected {
  fn new(interval: u64) -> StatsCollected {
    StatsCollected {
      count: 1,
      sum: interval,
      min: interval,
      max: interval,
    }
  }

  fn add(&mut self, interval: u64) {
    if interval == 0 {
      panic!("Cannot handle zero intervals");
    }
    self.count += 1;
    self.sum += interval;
    self.min = min(self.min, interval);
    self.max = max(self.max, interval);
  }

  fn done(&self) -> Stats {
    if self.count == 0 {
      panic!("No stats collected");
    }
    Stats {
      min: 1e9 / self.max as f32,
      avg: self.count as f32 * 1e9 / self.sum as f32,
      max: 1e9 / self.min as f32,
    }
  }
}

impl Fps {
  /// Creates a Fps struct in stopped state.
  pub fn stopped() -> Fps {
    Fps {
      state: State::Stopped,
    }
  }

  /// Start timing.
  pub fn start(&mut self) {
    let start_ns = time::precise_time_ns();
    self.state = State::Started {
      start_ns: start_ns,
    }
  }

  /// Stop timing.  Returns collected FPS statistics.
  pub fn stop(&mut self) -> Option<Stats> {
    let stats = self.stats();
    self.state = State::Stopped;
    stats.map(|s| s.done())
  }

  /// Register a frame.  Occasionally, returns collected FPS statistics.
  pub fn tick(&mut self) -> Option<Stats> {
    let curr_ns = time::precise_time_ns();
    let new_state = match self.state {
      State::Started { start_ns } => State::FirstTick {
        start_ns: start_ns,
        first_ns: curr_ns,
      },
      State::FirstTick { start_ns, first_ns } => State::MoreTicks {
        start_ns: start_ns,
        prev_ns: curr_ns,
        stats: StatsCollected::new(curr_ns - first_ns),
      },
      State::MoreTicks { start_ns, prev_ns, ref stats } => State::MoreTicks {
        start_ns: start_ns,
        prev_ns: curr_ns,
        stats: {
          let mut result = stats.clone();
          result.add(curr_ns - prev_ns);
          result
        },
      },
      State::Stopped => panic!("Fps is stopped"),
    };
    self.state = new_state;

    let elapsed_s = (curr_ns - self.start_ns()) as f32 / 1e9;
    if elapsed_s > 1.0 {
      let stats = self.stats();
      self.state = State::FirstTick {
        start_ns: curr_ns,
        first_ns: curr_ns,
      };
      return stats.map(|s| s.done());
    } else {
      return None;
    }
  }

  fn start_ns(&self) -> u64 {
    match self.state {
      State::Stopped => panic!("Fps.start_ns called in stopped state"),
      State::Started { start_ns } => start_ns,
      State::FirstTick { start_ns, .. } => start_ns,
      State::MoreTicks { start_ns, .. } => start_ns,
    }
  }

  fn stats(&self) -> Option<StatsCollected> {
    match self.state {
      State::MoreTicks { ref stats, .. } => Some(stats.clone()),
        _ => None,
    }
  }
}
