pub(crate) struct Defer<F: FnOnce()> {
    cleanup: Option<F>,
}

impl <F: FnOnce()> Defer<F> {
    pub(crate) fn new(f: F) -> Self {
        Defer { cleanup: Some(f) }
    }
}

impl <F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}