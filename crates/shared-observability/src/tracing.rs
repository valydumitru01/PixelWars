use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(service_name: &str, otel_endpoint: &str) -> anyhow::Result<()> {
    // 1. Create the OTLP Exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()?;

    // 2. Define the Resource (Service Name, etc.)
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", service_name.to_string()),
        ])
        .build();

    // 3. Setup the Tracer Provider with the Batch Span Processor
    let tracer_provider = sdktrace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    // 4. Create the tracing layer
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer(service_name.to_string()));

    // 5. Initialize Registry
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .with(otel_layer)
        .init();

    // Set the global provider so other parts of the app can use it
    opentelemetry::global::set_tracer_provider(tracer_provider);

    tracing::info!(service = service_name, "Tracing initialized with OTLP export");
    Ok(())
}