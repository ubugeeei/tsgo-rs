use std::time::Duration;

use tsgo_rs::fast::SmallVec;

#[derive(Clone, Debug)]
pub struct Stats {
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    p95: Duration,
}

impl Stats {
    pub fn from_samples(mut samples: SmallVec<[Duration; 32]>) -> Self {
        samples.sort_unstable();
        let min = samples[0];
        let max = samples[samples.len() - 1];
        let median = samples[samples.len() / 2];
        let p95 = samples[percentile_index(samples.len(), 95)];
        let total_nanos = samples
            .iter()
            .fold(0_u128, |acc, sample| acc + sample.as_nanos());
        let mean = Duration::from_nanos((total_nanos / samples.len() as u128) as u64);
        Self {
            min,
            max,
            mean,
            median,
            p95,
        }
    }

    pub fn mean_ms(&self) -> f64 {
        millis(self.mean)
    }

    pub fn median_ms(&self) -> f64 {
        millis(self.median)
    }

    pub fn p95_ms(&self) -> f64 {
        millis(self.p95)
    }

    pub fn min_ms(&self) -> f64 {
        millis(self.min)
    }

    pub fn max_ms(&self) -> f64 {
        millis(self.max)
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn percentile_index(len: usize, percentile: usize) -> usize {
    len.saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tsgo_rs::fast::SmallVec;

    use super::Stats;

    #[test]
    fn stats_use_sorted_median_and_p95() {
        let mut samples = SmallVec::<[Duration; 32]>::new();
        samples.push(Duration::from_millis(9));
        samples.push(Duration::from_millis(1));
        samples.push(Duration::from_millis(5));
        samples.push(Duration::from_millis(7));
        samples.push(Duration::from_millis(3));
        let stats = Stats::from_samples(samples);
        assert_eq!(stats.min_ms(), 1.0);
        assert_eq!(stats.median_ms(), 5.0);
        assert_eq!(stats.p95_ms(), 9.0);
        assert_eq!(stats.max_ms(), 9.0);
    }
}
