mod cursor;
mod drawing;
mod focus;
mod handlers;
mod render;
mod shell;
mod state;
mod udev;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    crate::udev::init_udev()?;

    Ok(())
}
