use crate::dht::metrics::DhtMetrics;

pub fn record_store_attempt(metrics: &DhtMetrics, success: bool) {
    metrics.inc_store_ops();
    if success {
        metrics.inc_store_success();
    }
}

pub fn record_find_attempt(metrics: &DhtMetrics, success: bool) {
    metrics.inc_find_value_ops();
    if success {
        metrics.inc_find_value_success();
    }
}
