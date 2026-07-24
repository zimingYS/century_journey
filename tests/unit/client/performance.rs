use super::*;

#[test]
fn performance_summary_uses_a_stable_nearest_rank_p95() {
    let values: Vec<_> = (1..=100).map(f64::from).collect();
    let summary = summarize(&values);

    assert_eq!(summary.mean, 50.5);
    assert_eq!(summary.p95, 95.0);
    assert_eq!(summary.max, 100.0);
}
