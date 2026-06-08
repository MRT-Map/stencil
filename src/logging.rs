use std::{borrow::Cow, io::ErrorKind, panic::PanicHookInfo, time::SystemTime};

use backtrace::Backtrace;
use color_backtrace::BacktracePrinter;
use etcetera::AppStrategy;
use serde::{Deserialize, Serialize};
use tracing::{Level, error, warn};
use tracing_error::{ErrorLayer, SpanTrace};
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::{
    App,
    ui::popup::{Popup, Popups},
    utils::file::{FOLDERS, safe_delete, safe_write},
};

pub fn init_logger() {
    std::panic::set_hook(Box::new(panic));

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

pub fn panic(panic: &PanicHookInfo) {
    error!("Caught panic: {panic:#}");
    let logs_dir = FOLDERS.in_data_dir("logs");
    let log_contents = logs_dir
        .read_dir()
        .map_err(|e| warn!("Unable to read logs directory: {e:#}"))
        .ok()
        .and_then(|read_dir| {
            read_dir
                .filter_map(|a| a.map_err(|e| warn!("Unable to see file: {e:#}")).ok())
                .filter(|a| a.file_name().to_string_lossy().starts_with("log"))
                .max_by_key(std::fs::DirEntry::path)
        })
        .and_then(|log| {
            std::fs::read_to_string(log.path())
                .map_err(|e| warn!("Unable to read {}: {e:#}", log.path().display()))
                .ok()
        })
        .unwrap_or_default();

    let backtrace = Backtrace::new();
    let span_trace = SpanTrace::capture();
    error!(
        "Backtrace:\n{}",
        BacktracePrinter::new()
            .format_trace_to_string(&backtrace)
            .unwrap_or_default()
    );
    error!("Span trace:\n{}", color_spantrace::colorize(&span_trace));
    let panic_file = logs_dir.join(format!(
        "panic-{}.log",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));
    let _ = safe_write(
        &panic_file,
        format!("{panic:#}\n\n{backtrace:#?}\n\n{span_trace:#?}\n\n{log_contents}"),
    )
    .map_err(|e| warn!("Unable to write crash log: {e:#}"));
    let _ = std::fs::write(
        logs_dir.join(".panicked"),
        panic_file.to_string_lossy().to_string(),
    )
    .map_err(|e| warn!("Unable to write .panicked: {e:#}"));
}

impl App {
    #[tracing::instrument(skip_all)]
    pub fn ack_panic(&mut self) {
        let panicked_file = FOLDERS.in_data_dir("logs").join(".panicked");
        let panic_file = match std::fs::read_to_string(&panicked_file) {
            Ok(content) => content,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => return,
                _ => panic!("Cannot read .panicked: {e:#}"),
            },
        };
        let _ = safe_delete(&panicked_file);
        self.add_popup(AckPanicPopup { panic_file })
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AckPanicPopup {
    panic_file: String,
}

impl Popup for AckPanicPopup {
    fn id(&self) -> Cow<'static, str> {
        "ack_panic".into()
    }

    fn title(&self) -> egui::WidgetText {
        "Panic".into()
    }

    fn ui(&mut self, app: &mut App, ui: &mut egui::Ui) -> bool {
        let text = format!(
            "Stencil3 panicked the last time it was open. A crash log has been produced at:\
            \n\n{}\n\n If you wish to report this as a bug, go through the file to redact any \
            personal details if necessary, then create an issue on stencil3's GitHub repository \
            and attach this crash log. You may also send the crash log via Discord.",
            self.panic_file,
        );
        Popups::alert_ui(app, ui, text, Option::<fn(&_, &mut _)>::None)
    }
}
