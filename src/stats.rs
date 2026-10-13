//! Statistical aggregation for ping round-trip times and packet loss.

use std::time::{Duration, Instant};

/// Aggregated metrics and round-trip time calculations matching standard ping.
#[derive(Debug, Clone)]
pub struct PingStatistics {
    /// Target host or IP string description.
    target: String,
    /// Number of probes transmitted.
    transmitted: u64,
    /// Number of successful responses received.
    received: u64,
    /// Minimum observed RTT in milliseconds.
    rtt_min: f64,
    /// Maximum observed RTT in milliseconds.
    rtt_max: f64,
    /// Sum of all observed RTTs in milliseconds.
    rtt_sum: f64,
    /// Sum of squares of all observed RTTs in milliseconds.
    rtt_sum_sq: f64,
    /// Instant when probing started.
    start_time: Instant,
}

/// Snapshot summary of calculated statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct StatsSummary {
    /// Target description.
    pub target: String,
    /// Total probes sent.
    pub transmitted: u64,
    /// Total responses received.
    pub received: u64,
    /// Packet loss percentage in range 0.0..=100.0.
    pub packet_loss_percent: f64,
    /// Total wall-clock time elapsed.
    pub total_duration: Duration,
    /// RTT minimum in milliseconds (None if 0 packets received).
    pub rtt_min: Option<f64>,
    /// RTT average in milliseconds (None if 0 packets received).
    pub rtt_avg: Option<f64>,
    /// RTT maximum in milliseconds (None if 0 packets received).
    pub rtt_max: Option<f64>,
    /// RTT mean deviation (standard deviation) in milliseconds (None if 0 packets received).
    pub rtt_mdev: Option<f64>,
}

impl PingStatistics {
    /// Creates a new statistics tracker for the given target.
    #[must_use]
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            transmitted: 0,
            received: 0,
            rtt_min: f64::INFINITY,
            rtt_max: 0.0,
            rtt_sum: 0.0,
            rtt_sum_sq: 0.0,
            start_time: Instant::now(),
        }
    }

    /// Records a transmitted probe packet.
    pub const fn record_sent(&mut self) {
        self.transmitted = self.transmitted.saturating_add(1);
    }

    /// Records a successful reply with observed duration.
    pub fn record_reply(&mut self, duration: Duration) {
        self.received = self.received.saturating_add(1);
        let millis = duration.as_secs_f64() * 1000.0;

        if millis < self.rtt_min {
            self.rtt_min = millis;
        }
        if millis > self.rtt_max {
            self.rtt_max = millis;
        }

        self.rtt_sum += millis;
        self.rtt_sum_sq = millis.mul_add(millis, self.rtt_sum_sq);
    }

    /// Generates a summary snapshot of all recorded metrics.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn summary(&self) -> StatsSummary {
        let total_duration = self.start_time.elapsed();
        let loss_percent = if self.transmitted == 0 {
            0.0
        } else {
            let lost = self.transmitted.saturating_sub(self.received);
            (lost as f64 / self.transmitted as f64) * 100.0
        };

        if self.received == 0 {
            StatsSummary {
                target: self.target.clone(),
                transmitted: self.transmitted,
                received: self.received,
                packet_loss_percent: loss_percent,
                total_duration,
                rtt_min: None,
                rtt_avg: None,
                rtt_max: None,
                rtt_mdev: None,
            }
        } else {
            let n = self.received as f64;
            let avg = self.rtt_sum / n;
            let variance = avg.mul_add(-avg, self.rtt_sum_sq / n);
            let mdev = if variance > 0.0 { variance.sqrt() } else { 0.0 };

            StatsSummary {
                target: self.target.clone(),
                transmitted: self.transmitted,
                received: self.received,
                packet_loss_percent: loss_percent,
                total_duration,
                rtt_min: Some(self.rtt_min),
                rtt_avg: Some(avg),
                rtt_max: Some(self.rtt_max),
                rtt_mdev: Some(mdev),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_packets() {
        let stats = PingStatistics::new("localhost");
        let summary = stats.summary();
        assert_eq!(summary.transmitted, 0);
        assert_eq!(summary.received, 0);
        assert!((summary.packet_loss_percent - 0.0).abs() < f64::EPSILON);
        assert!(summary.rtt_avg.is_none());
    }

    #[test]
    fn test_all_lost() {
        let mut stats = PingStatistics::new("1.1.1.1");
        stats.record_sent();
        stats.record_sent();
        let summary = stats.summary();
        assert_eq!(summary.transmitted, 2);
        assert_eq!(summary.received, 0);
        assert!((summary.packet_loss_percent - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rtt_calculations() {
        let mut stats = PingStatistics::new("8.8.8.8");
        stats.record_sent();
        stats.record_reply(Duration::from_millis(10));
        stats.record_sent();
        stats.record_reply(Duration::from_millis(20));
        stats.record_sent();
        stats.record_reply(Duration::from_millis(30));

        let summary = stats.summary();
        assert_eq!(summary.transmitted, 3);
        assert_eq!(summary.received, 3);
        assert!((summary.packet_loss_percent - 0.0).abs() < f64::EPSILON);

        let min = summary.rtt_min.expect("min should be present");
        let avg = summary.rtt_avg.expect("avg should be present");
        let max = summary.rtt_max.expect("max should be present");
        let mdev = summary.rtt_mdev.expect("mdev should be present");

        assert!((min - 10.0).abs() < 1e-3);
        assert!((avg - 20.0).abs() < 1e-3);
        assert!((max - 30.0).abs() < 1e-3);
        // variance = (100 + 400 + 900)/3 - 400 = 1400/3 - 400 = 466.666 - 400 = 66.666
        // sqrt(66.666) approx 8.1649
        assert!((mdev - 8.165).abs() < 1e-2);
    }
}
