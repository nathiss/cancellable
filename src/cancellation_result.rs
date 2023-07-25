/// Result of a single iteration of the service loop.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CancellationResult<T> {
    /// Indicates that the loop should continue and wraps value yielded by the
    /// finished iteration.
    Item(T),

    /// Indicates that the loop should continue.
    Continue,

    /// Indicates that the loop should end.
    Break,
}

impl<T> CancellationResult<T> {
    /// Constructs a new `CancellationResult::Item`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cancellable::CancellationResult;
    ///
    /// fn construct_result() -> CancellationResult<String> {
    ///     CancellationResult::item("foo")
    /// }
    /// ```
    pub fn item(t: impl Into<T>) -> Self {
        Self::Item(t.into())
    }
}

impl<T> From<T> for CancellationResult<T> {
    fn from(value: T) -> Self {
        Self::Item(value)
    }
}
