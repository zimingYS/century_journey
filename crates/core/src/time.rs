//! 世界时间与模拟时钟

/// 模拟频率（Tick / 秒）
pub const SIM_TPS: u32 = 20;

/// 单 Tick 秒数
pub const SIM_DT: f32 = 1.0 / SIM_TPS as f32;

/// 5 Hz 任务间隔
pub const TICKS_PER_5HZ: u64 = 4;

/// 1 Hz 任务间隔
pub const TICKS_PER_1HZ: u64 = 20;

/// 世界时间
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WorldTime {
    /// 自世界诞生以来的模拟 tick
    pub tick: u64,
}

/// 模拟时钟（固定步长）
#[derive(Debug, Clone, Copy)]
pub struct SimClock {
    /// 当前模拟 Tick
    pub tick: u64,
    /// 单 Tick 秒数
    pub dt: f32,
}

impl SimClock {
    /// 创建模拟时钟
    pub const fn new(dt: f32) -> Self {
        SimClock { tick: 0, dt }
    }

    /// 推进一个 Tick
    #[inline]
    pub fn advance(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// 当前 Tick 是否应执行指定间隔的任务
    #[inline]
    pub fn should_run(&self, every: u64) -> bool {
        debug_assert!(every > 0, "simulation frequency interval must be > 0");
        self.tick.is_multiple_of(every)
    }

    /// 自模拟开始经过的秒数
    #[inline]
    pub fn elapsed_seconds(&self) -> f32 {
        self.tick as f32 * self.dt
    }
}

impl Default for SimClock {
    /// 使用项目规定的固定步长
    fn default() -> Self {
        Self::new(SIM_DT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_starts_at_zero() {
        let clock = SimClock::default();
        assert_eq!(clock.tick, 0);
        assert_eq!(clock.elapsed_seconds(), 0.0);
    }

    #[test]
    fn advance_increments_by_one() {
        let mut clock = SimClock::default();
        for expected in 1..=10 {
            clock.advance();
            assert_eq!(clock.tick, expected);
        }
    }

    #[test]
    fn elapsed_seconds_matches_tick_and_dt() {
        let clock = SimClock {
            tick: SIM_TPS as u64,
            ..Default::default()
        };
        assert!((clock.elapsed_seconds() - 1.0).abs() < 1e-4);

        let clock = SimClock {
            tick: (SIM_TPS as u64) * 60,
            ..Default::default()
        };
        assert!((clock.elapsed_seconds() - 60.0).abs() < 1e-2);
    }

    #[test]
    fn sim_dt_is_reciprocal_of_tps() {
        assert!((SIM_DT * SIM_TPS as f32 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn should_run_matches_tick_multiples() {
        let mut clock = SimClock::default();

        assert!(clock.should_run(TICKS_PER_1HZ));
        assert!(clock.should_run(TICKS_PER_5HZ));

        for tick in 1..=40u64 {
            clock.tick = tick;
            assert_eq!(
                clock.should_run(TICKS_PER_5HZ),
                tick.is_multiple_of(4),
                "5Hz gate is wrong at tick {tick}"
            );
            assert_eq!(
                clock.should_run(TICKS_PER_1HZ),
                tick.is_multiple_of(20),
                "1Hz gate is wrong at tick {tick}"
            );
        }
    }

    #[test]
    fn should_run_every_one_is_always_true() {
        let mut clock = SimClock::default();
        for tick in 0..100u64 {
            clock.tick = tick;
            assert!(
                clock.should_run(1),
                "every-1 must always run at tick {tick}"
            );
        }
    }

    #[test]
    fn advance_wraps_without_panic() {
        let mut clock = SimClock {
            tick: u64::MAX,
            ..Default::default()
        };
        clock.advance();
        assert_eq!(clock.tick, 0, "advance must wrap, not panic");
    }
}
