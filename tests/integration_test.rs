//! Integration tests for rping.

use rping::app::resolve_target;
use rping::stats::PingStatistics;
use std::time::Duration;

#[test]
fn test_resolve_ipv4_literal() {
    let target = "127.0.0.1";
    let resolved = resolve_target(target, false, false).expect("failed to parse IPv4");
    assert!(resolved.is_ipv4());
    assert_eq!(resolved.to_string(), "127.0.0.1");
}

#[test]
fn test_resolve_ipv6_literal() {
    let target = "::1";
    let resolved = resolve_target(target, false, false).expect("failed to parse IPv6");
    assert!(resolved.is_ipv6());
    assert_eq!(resolved.to_string(), "::1");
}

#[test]
fn test_resolve_invalid_target() {
    let target = "invalid.host.test.locally.unresolvable.invalid";
    let res = resolve_target(target, false, false);
    assert!(res.is_err());
}

#[test]
fn test_statistics_accumulation() {
    let mut stats = PingStatistics::new("127.0.0.1");
    for i in 1..=5 {
        stats.record_sent();
        stats.record_reply(Duration::from_millis(i * 10));
    }

    let summary = stats.summary();
    assert_eq!(summary.transmitted, 5);
    assert_eq!(summary.received, 5);
    assert_eq!(summary.packet_loss_percent, 0.0);
    assert_eq!(summary.rtt_min, Some(10.0));
    assert_eq!(summary.rtt_max, Some(50.0));
    assert_eq!(summary.rtt_avg, Some(30.0));
}

#[test]
#[cfg(windows)]
fn test_windows_loopback_ping() {
    use rping::engine::create_engine;
    use std::net::IpAddr;

    let target: IpAddr = "127.0.0.1".parse().expect("valid loopback ip");
    let mut engine = create_engine(target).expect("failed to create windows engine");
    let payload = vec![0x55u8; 32];
    let reply = engine
        .send_probe(target, 1, &payload, Duration::from_secs(1), 64)
        .expect("loopback ping should succeed on windows");

    assert_eq!(reply.sequence, 1);
    assert!(reply.ttl > 0);
    assert_eq!(reply.source, target);
}
