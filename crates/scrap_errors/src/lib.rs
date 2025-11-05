#![feature(negative_impls)]

/// Useful type to use with `Result<>` indicate that an error has already
/// been reported to the user, so no need to continue checking.
///
/// The `()` field is necessary: it is non-`pub`, which means values of this
/// type cannot be constructed outside of this crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ErrorGuaranteed(());

impl ErrorGuaranteed {
    /// Don't use this outside of `DiagCtxtInner::emit_diagnostic`!
    #[deprecated = "should only be used in `DiagCtxtInner::emit_diagnostic`"]
    pub fn unchecked_error_guaranteed() -> Self {
        ErrorGuaranteed(())
    }

    pub fn raise_fatal(self) -> ! {
        FatalError.raise()
    }
}

/// Used as a return value to signify a fatal error occurred.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct FatalError;

// Don't implement Send on FatalError. This makes it impossible to `panic_any!(FatalError)`.
// We don't want to invoke the panic handler and print a backtrace for fatal errors.
impl !Send for FatalError {}

struct FatalErrorMarker;

impl FatalError {
    pub fn raise(self) -> ! {
        std::panic::resume_unwind(Box::new(FatalErrorMarker))
    }
}

impl std::fmt::Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fatal error")
    }
}

impl std::error::Error for FatalError {}

