use std::{backtrace::Backtrace, io::ErrorKind, panic::PanicInfo, sync::Arc, time::SystemTime};

use bevy::prelude::EventWriter;
use itertools::Itertools;
use tracing::{error, warn};

use crate::{misc::data_dir, ui::popup::Popup};

pub fn panic(panic: &PanicInfo) {
    error!("Caught panic: {panic:?}");
    let (log1, log2) = if let Ok(read_dir) = data_dir("logs").read_dir() {
        let mut list = read_dir.filter_map(|a| Some(a.ok()?.path())).sorted().rev();
        (list.next(), list.next())
    } else {
        (None, None)
    };
    let log1_contents = log1
        .and_then(|log1| std::fs::read_to_string(log1).ok())
        .unwrap_or_default();
    let log2_contents = log2
        .and_then(|log2| std::fs::read_to_string(log2).ok())
        .unwrap_or_default();
    let backtrace = Backtrace::force_capture();
    let panics_dir = data_dir("panics");
    let panic_file = {
        let mut file = panics_dir.to_owned();
        file.push(format!(
            "panic-{}.txt",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));
        file
    };
    let _ = std::fs::write(
        {
            let mut file = panics_dir.to_owned();
            file.push(&panic_file);
            file
        },
        format!("{panic:#}\n\n{backtrace}\n\n{log2_contents}\n\n{log1_contents}"),
    )
    .map_err(|e| warn!("Unable to write crash log: {e:?}"));
    let _ = std::fs::write(
        {
            let mut file = panics_dir;
            file.push(".to_show");
            file
        },
        panic_file.to_string_lossy().to_string(),
    )
    .map_err(|e| warn!("Unable to write .to_show: {e:?}"));
}

#[tracing::instrument(skip_all)]
pub fn ack_panic_sy(mut popup: EventWriter<Arc<Popup>>) {
    let panics_dir = data_dir("panics");
    let to_show_file = {
        let mut file = panics_dir;
        file.push(".to_show");
        file
    };
    let panic_file = match std::fs::read_to_string(&to_show_file) {
        Ok(content) => content,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return,
            e => panic!("{:?}", e),
        },
    };
    std::fs::remove_file(&to_show_file).unwrap();
    popup.send(Popup::base_alert(
        "ack_panic",
        "Panic",
        format!(
            "Stencil2 encountered an error and panicked the last time it was open. \
            A crash log has been produced at:\n\n{panic_file}\n\nIf you think it's a bug, \
            go through the file to redact any personal details, and then create a issue \
            on our GitHub at https://github.com/MRT-Map/stencil2 and attach the file, \
            or if you know __7d's Discord account, send the file over via Discord.\n\n\
            We apologise if you had lost any data."
        ),
    ));
}
