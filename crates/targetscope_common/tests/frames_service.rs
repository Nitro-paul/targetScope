// doc/testing/frame_transport.md#service-configuration-integration

use std::time::{Duration, Instant};

use targetscope_common::frames::*;

fn service(name: &str) -> String {
    format!("targetscope/test/{name}/{}", std::process::id())
}

fn header(seq: u64, width: u32, height: u32) -> FrameHeader {
    FrameHeader { seq, timestamp_ns: seq, source_seq: seq, width, height, stride: width * 3, format: 1, px_per_mm: 0.0 }
}

// FT-SVC-01
#[test]
fn ft_svc_01_same_configuration_in_both_languages() {
    assert_eq!((MAX_PUBLISHERS, MAX_SUBSCRIBERS, HISTORY_SIZE, SUBSCRIBER_MAX_BUFFER_SIZE, ENABLE_SAFE_OVERFLOW), (1, 4, 0, 2, true));
}

// FT-SVC-02
#[test]
fn ft_svc_02_one_publisher() {
    let name = service("one_pub");
    let mut first = frame_publisher(&name, 100).unwrap();
    assert!(frame_publisher(&name, 100).is_err());
    let mut sub = frame_subscriber(&name).unwrap();
    first.publish(header(1, 4, 4), |_| {}).unwrap();
    assert!(sub.wait(Duration::from_secs(1)).unwrap().is_some());
}

// FT-SVC-03
#[test]
fn ft_svc_03_four_subscribers() {
    let name = service("four_subs");
    let mut subs: Vec<_> = (0..4).map(|_| frame_subscriber(&name).unwrap()).collect();
    assert!(frame_subscriber(&name).is_err());
    let mut publ = frame_publisher(&name, 100).unwrap();
    publ.publish(header(1, 4, 4), |_| {}).unwrap();
    for s in &mut subs {
        assert!(s.wait(Duration::from_secs(1)).unwrap().is_some());
    }
}

// FT-SVC-04
#[test]
fn ft_svc_04_no_history() {
    let name = service("history");
    let mut publ = frame_publisher(&name, 100).unwrap();
    for i in 0..3 {
        publ.publish(header(i, 4, 4), |_| {}).unwrap();
    }
    let mut sub = frame_subscriber(&name).unwrap();
    assert!(sub.wait(Duration::from_millis(100)).unwrap().is_none());
    publ.publish(header(9, 4, 4), |_| {}).unwrap();
    assert_eq!(sub.wait(Duration::from_secs(1)).unwrap().unwrap().header().seq, 9);
}

// FT-SVC-05
#[test]
fn ft_svc_05_slow_subscriber() {
    let name = service("slow");
    let mut sub = frame_subscriber(&name).unwrap();
    let mut publ = frame_publisher(&name, 64 * 64 * 3).unwrap();
    let mut slowest = Duration::ZERO;
    let mut seqs = Vec::new();
    for i in 0..100u64 {
        let start = Instant::now();
        publ.publish(header(i, 64, 64), |_| {}).unwrap();
        slowest = slowest.max(start.elapsed());
        std::thread::sleep(Duration::from_millis(10));
        if i % 20 == 19 {
            if let Some(f) = sub.wait(Duration::from_millis(1)).unwrap() {
                seqs.push(f.header().seq);
            }
        }
    }
    assert!(slowest < Duration::from_millis(1), "{slowest:?}");
    assert_eq!(seqs, vec![19, 39, 59, 79, 99]);
}

// FT-SVC-06
#[test]
fn ft_svc_06_borrow_limit() {
    let name = service("borrow");
    let mut sub = frame_subscriber(&name).unwrap();
    let mut publ = frame_publisher(&name, 100).unwrap();
    publ.publish(header(1, 4, 4), |_| {}).unwrap();
    let first = sub.wait(Duration::from_secs(1)).unwrap().unwrap();
    publ.publish(header(2, 4, 4), |_| {}).unwrap();
    let second = sub.wait(Duration::from_secs(1)).unwrap().unwrap();
    publ.publish(header(3, 4, 4), |_| {}).unwrap();
    let third = sub.wait(Duration::from_millis(100));
    assert!(third.is_err() || third.as_ref().unwrap().is_none());
    drop(third);
    drop(first);
    publ.publish(header(4, 4, 4), |_| {}).unwrap();
    assert!(sub.wait(Duration::from_secs(1)).unwrap().is_some());
    drop(second);
}

// FT-SVC-07
#[test]
fn ft_svc_07_resolution_change() {
    let name = service("resolution");
    let mut sub = frame_subscriber(&name).unwrap();
    let mut publ = frame_publisher(&name, 1280 * 720 * 3).unwrap();
    for (i, (w, h)) in [(1280, 720), (1920, 1080), (3840, 2160)].into_iter().enumerate() {
        publ.publish(header(i as u64, w, h), |buf| buf.fill(i as u8)).unwrap();
        let f = sub.wait(Duration::from_secs(1)).unwrap().unwrap();
        assert_eq!((f.header().width, f.header().height), (w, h));
        assert_eq!(f.data().len(), (h * w * 3) as usize);
    }
}

// FT-SVC-08
#[test]
fn ft_svc_08_notification() {
    let name = service("notify");
    let mut publ = frame_publisher(&name, 100).unwrap();
    let (ready, is_ready) = std::sync::mpsc::channel();
    let n = name.clone();
    let waiter = std::thread::spawn(move || {
        let mut sub = frame_subscriber(&n).unwrap();
        ready.send(()).unwrap();
        let f = sub.wait(Duration::from_secs(5)).unwrap().map(|f| f.header().seq);
        (f, Instant::now())
    });
    is_ready.recv().unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let sent = Instant::now();
    publ.publish(header(5, 4, 4), |_| {}).unwrap();
    let (seq, received) = waiter.join().unwrap();
    assert_eq!(seq, Some(5));
    let latency = received.duration_since(sent);
    assert!(latency < Duration::from_millis(5), "{latency:?}");
}

// FT-SVC-09
#[test]
fn ft_svc_09_wait_timeout() {
    let mut sub = frame_subscriber(&service("timeout")).unwrap();
    let start = Instant::now();
    assert!(sub.wait(Duration::from_secs(2)).unwrap().is_none());
    let t = start.elapsed();
    assert!(t >= Duration::from_secs(2) && t < Duration::from_millis(2200), "{t:?}");
}

// FT-SVC-10
#[test]
#[ignore = "needs a separate publisher process (doc/testing/frame_transport.md#service-configuration-integration)"]
fn ft_svc_10_publisher_restart() {}

// FT-SVC-11
#[test]
#[ignore = "needs a separate publisher process (doc/testing/frame_transport.md#service-configuration-integration)"]
fn ft_svc_11_stale_resources() {}

// FT-SVC-12
#[test]
fn ft_svc_12_publisher_allocation() {
    let name = service("alloc");
    let mut sub = frame_subscriber(&name).unwrap();
    let mut publ = frame_publisher(&name, 1280 * 720 * 3).unwrap();
    publ.publish(header(1, 1920, 1080), |buf| buf.fill(3)).unwrap();
    let f = sub.wait(Duration::from_secs(1)).unwrap().unwrap();
    assert_eq!(f.data().len(), 1920 * 1080 * 3);
}
