/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

/// Simple wrapper for perf-event to measure the number of instructions
/// executed by a current thread.
pub struct PerThreadInstructionCounter {
    #[cfg(target_os = "linux")]
    counter: perf_event::Counter,
    #[cfg(not(target_os = "linux"))]
    non_linux: std::convert::Infallible,
}

impl PerThreadInstructionCounter {
    /// Create a new instruction counter.
    ///
    /// Return `Err` is `perf_event` failed, `None` on unsupported platforms.
    pub fn init() -> anyhow::Result<Option<PerThreadInstructionCounter>> {
        Self::init_impl()
    }

    #[cfg(target_os = "linux")]
    fn init_impl() -> anyhow::Result<Option<PerThreadInstructionCounter>> {
        let mut counter = perf_event::Builder::new()
            .observe_self()
            .any_cpu()
            .inherit(false)
            .kind(perf_event::events::Hardware::INSTRUCTIONS)
            .build()?;
        counter.enable()?;
        Ok(Some(PerThreadInstructionCounter { counter }))
    }

    #[cfg(not(target_os = "linux"))]
    fn init_impl() -> anyhow::Result<Option<PerThreadInstructionCounter>> {
        Ok(None)
    }

    /// Collect the number of instructions executed by the thread.
    pub fn collect(self) -> anyhow::Result<u64> {
        self.collect_impl()
    }

    #[cfg(target_os = "linux")]
    fn collect_impl(mut self) -> anyhow::Result<u64> {
        self.counter.disable()?;
        let count = self.counter.read_count_and_time()?;
        if count.time_running == 0 {
            Err(anyhow::anyhow!("No counter data collected"))
        } else {
            let count =
                (count.count as u128) * (count.time_enabled as u128) / (count.time_running as u128);
            Ok(count as u64)
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn collect_impl(self) -> anyhow::Result<u64> {
        match self.non_linux {}
    }
}

#[cfg(test)]
mod tests {
    use three_billion_instructions::three_billion_instructions;

    use crate::per_thread_instruction_counter::PerThreadInstructionCounter;

    #[allow(unreachable_code)] // Compiler says it is uninhabited on non-linux platforms.
    #[allow(unused_variables)] // This seems like a compiler bug.
    #[test]
    fn test_perf_thread_instruction_counter() {
        if !cfg!(target_os = "linux") {
            assert!(PerThreadInstructionCounter::init().unwrap().is_none());
        } else {
            let counter = PerThreadInstructionCounter::init().unwrap().unwrap();
            three_billion_instructions().unwrap();
            let count = counter.collect().unwrap();
            assert!((3_000_000_000..=3_100_000_000).contains(&count));
        }
    }
}
