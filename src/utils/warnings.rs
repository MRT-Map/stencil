use std::fmt::{Debug, Display};

use crate::notif;

pub trait ResultWithWarningsExt {
    type Error;
    type Warning;
    type Output1;
    type Output2;
    fn notify_w<M: Display>(self, message: M) -> Self::Output1;
    fn error_warnings_to_vec<VE: From<Self::Error> + From<Self::Warning>>(
        self,
        vec: &mut Vec<VE>,
    ) -> Self::Output2;
}

#[must_use]
pub struct WithWarnings<T, W = eyre::Report> {
    pub value: T,
    pub warnings: Vec<W>,
}

impl<T, W> From<(T, Vec<W>)> for WithWarnings<T, W> {
    fn from((value, warnings): (T, Vec<W>)) -> Self {
        Self { value, warnings }
    }
}

impl<T, W> WithWarnings<T, W> {
    pub const fn new(value: T, warnings: Vec<W>) -> Self {
        Self { value, warnings }
    }
    pub const fn ok(value: T) -> Self {
        Self {
            value,
            warnings: Vec::new(),
        }
    }
    fn handle<R, F: FnOnce(Vec<W>) -> R>(self, f: F) -> (T, Option<R>) {
        let result = (!self.warnings.is_empty()).then(|| f(self.warnings));
        (self.value, result)
    }
    pub fn warnings_to_vec<VE: From<W>>(self, vec: &mut Vec<VE>) -> T {
        self.handle(|e| vec.extend(e.into_iter().map(Into::into))).0
    }
}
impl<T, W: ToString + Debug> WithWarnings<T, W> {
    pub fn notify<M: Display>(self, message: M) -> T {
        self.handle(|errors| {
            notif!(warning message, errors &errors);
        })
        .0
    }
}
impl<T, W: ToString + Debug, E: Display + Debug> ResultWithWarningsExt
    for Result<WithWarnings<T, W>, E>
{
    type Error = E;
    type Warning = W;
    type Output1 = Result<T, E>;
    type Output2 = Option<T>;
    fn notify_w<M: Display>(self, message: M) -> Self::Output1 {
        match self {
            Ok(ww) => Ok(ww.notify(message)),
            Err(e) => {
                notif!(error message, error &e);
                Err(e)
            }
        }
    }
    fn error_warnings_to_vec<VE: From<Self::Error> + From<Self::Warning>>(
        self,
        vec: &mut Vec<VE>,
    ) -> Self::Output2 {
        match self {
            Ok(ww) => Some(ww.warnings_to_vec(vec)),
            Err(e) => {
                vec.push(e.into());
                None
            }
        }
    }
}

#[must_use]
pub struct WithWarning<T, W = eyre::Report> {
    value: T,
    warning: Option<W>,
}

impl<T, W> From<(T, Option<W>)> for WithWarning<T, W> {
    fn from((value, warning): (T, Option<W>)) -> Self {
        Self { value, warning }
    }
}

impl<T, W> WithWarning<T, W> {
    pub const fn new(value: T, warning: Option<W>) -> Self {
        Self { value, warning }
    }
    pub const fn ok(value: T) -> Self {
        Self {
            value,
            warning: None,
        }
    }
    fn handle<R, F: FnOnce(W) -> R>(self, f: F) -> (T, Option<R>) {
        let result = self.warning.map(f);
        (self.value, result)
    }
    pub fn warning_to_vec<VE: From<W>>(self, vec: &mut Vec<VE>) -> T {
        self.handle(|e| vec.push(e.into())).0
    }
}
impl<T, W: Display + Debug> WithWarning<T, W> {
    pub fn notify<M: Display>(self, message: M) -> T {
        self.handle(|error| {
            notif!(warning message, error &error);
        })
        .0
    }
}
impl<T, W: Display + Debug, E: Display + Debug> ResultWithWarningsExt
    for Result<WithWarning<T, W>, E>
{
    type Error = E;
    type Warning = W;
    type Output1 = Result<T, E>;
    type Output2 = Option<T>;
    fn notify_w<M: Display>(self, message: M) -> Self::Output1 {
        match self {
            Ok(ww) => Ok(ww.notify(message)),
            Err(e) => {
                notif!(error message, error &e);
                Err(e)
            }
        }
    }
    fn error_warnings_to_vec<VE: From<Self::Error> + From<Self::Warning>>(
        self,
        vec: &mut Vec<VE>,
    ) -> Self::Output2 {
        match self {
            Ok(ww) => Some(ww.warning_to_vec(vec)),
            Err(e) => {
                vec.push(e.into());
                None
            }
        }
    }
}

pub trait ResultExt {
    type Error;
    type Output;
    fn notify<M: Display>(self, message: M) -> Self;
    fn error_to_vec<VE: From<Self::Error>>(self, vec: &mut Vec<VE>) -> Self::Output;
}
impl<T, E: Display + Debug> ResultExt for Result<T, E> {
    type Error = E;
    type Output = Option<T>;

    fn notify<M: Display>(self, message: M) -> Self {
        self.inspect_err(|e| {
            notif!(error message, error &e);
        })
    }

    fn error_to_vec<VE: From<Self::Error>>(self, vec: &mut Vec<VE>) -> Self::Output {
        self.map_err(|e| vec.push(e.into())).ok()
    }
}
