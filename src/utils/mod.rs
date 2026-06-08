#![expect(clippy::non_std_lazy_statics)]

use async_executor::StaticExecutor;
use lazy_regex::{Regex, lazy_regex};

pub mod coord;
pub mod file;
pub mod load_save;
pub mod pointer;
pub mod warnings;

pub static EXECUTOR: StaticExecutor = StaticExecutor::new();
pub static PATH_SANITISER: lazy_regex::Lazy<Regex> = lazy_regex!("[<>:/\\|?*\"]");
