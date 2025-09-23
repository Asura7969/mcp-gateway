use axum_prometheus::PrometheusMetricLayer;
use prometheus::{Encoder, TextEncoder};
use std::collections::HashMap;

pub fn create_prometheus_layer() -> (PrometheusMetricLayer<'static>, prometheus::Registry) {
    let registry = prometheus::Registry::new();
    let metric_layer = PrometheusMetricLayer::new();
    
    // Register the metrics with the registry
    registry.register(Box::new(metric_layer.clone())).unwrap();
    
    (metric_layer, registry)
}

pub async fn metrics_handler(
    registry: std::sync::Arc<prometheus::Registry>,
) -> Result<String, (axum::http::StatusCode, String)> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(output) => Ok(output),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics".to_string()))
        }
    }
}