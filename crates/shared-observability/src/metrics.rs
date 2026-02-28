use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::info;

/// Initialize Prometheus metrics exporter on the given port.
/// Metrics will be available at /metrics on port `metrics_port`.
pub fn init_metrics(metrics_port: u16) -> anyhow::Result<()> {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(([0, 0, 0, 0], metrics_port))
        .install()?;

    info!(port = metrics_port, "Prometheus metrics exporter started");
    Ok(())
}
