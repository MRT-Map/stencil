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
    pub fn handle_warnings<R, F: FnOnce(Vec<W>) -> R>(self, f: F) -> (T, Option<R>) {
        let result = (!self.warnings.is_empty()).then(|| f(self.warnings));
        (self.value, result)
    }
    pub fn handle_warnings2<R, F: FnOnce(Vec<W>) -> R, G: FnOnce() -> R>(
        self,
        f: F,
        g: G,
    ) -> (T, R) {
        let result = if self.warnings.is_empty() {
            g()
        } else {
            f(self.warnings)
        };
        (self.value, result)
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
    pub fn handle_warning<R, F: FnOnce(W) -> R>(self, f: F) -> (T, Option<R>) {
        let result = self.warning.map(f);
        (self.value, result)
    }
    pub fn handle_warning2<R, F: FnOnce(W) -> R, G: FnOnce() -> R>(self, f: F, g: G) -> (T, R) {
        let result = self.warning.map_or_else(g, f);
        (self.value, result)
    }
}
