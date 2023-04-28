use tracing::{subscriber::set_global_default, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{format::FmtSpan, MakeWriter},
    EnvFilter, FmtSubscriber,
};

pub fn get_tracing_subscriber<Sink>(env_filter: &str, sink: Sink) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    //parse env variable, use info by default
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    //pretty log data format
    FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(sink)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish()
}

pub fn init_tracing_subscriber(subscriber: impl Subscriber + Sync + Send) -> Result<(), String> {
    LogTracer::init().map_err(|_e| String::from("Logger has already been initialized"))?;
    set_global_default(subscriber)
        .map_err(|_e| String::from("Logger has already been initialized"))?;
    Ok(())
}
