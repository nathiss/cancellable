pub enum CancellationResult<T> {
    Item(T),
    Continue,
    Break,
}

impl<T> CancellationResult<T> {
    pub fn item(t: impl Into<T>) -> Self {
        Self::Item(t.into())
    }
}
