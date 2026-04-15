use etcetera::AppStrategy;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::file::FOLDERS;

pub fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().compact().with_writer(
                std::io::stdout.with_max_level(Level::DEBUG).and(
                    tracing_appender::rolling::hourly(FOLDERS.in_data_dir("logs"), "log"),
                ),
            ),
        )
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::try_new("info").unwrap()),
        )
        .with(ErrorLayer::default())
        .init();
}
