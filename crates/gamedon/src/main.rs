use gamedon::run;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};

/// Run a graphics app.
fn main() -> color_eyre::Result<()> {
    color_eyre::install().expect("only called once.");
    let filter = EnvFilter::builder()
        .from_env()?
        .add_directive("rustyline=info".parse()?)
        .add_directive("calloop=info".parse()?)
        .add_directive("wgpu=info".parse()?);
    tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(filter)
        .init();
    run()?;
    Ok(())
}
