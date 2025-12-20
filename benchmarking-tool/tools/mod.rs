pub mod qbdi;
pub mod wall_clock;

#[cfg(target_family = "unix")]
pub mod perf_events;

#[cfg(target_arch = "x86")]
pub mod sde;

use super::BenchmarkInput;
