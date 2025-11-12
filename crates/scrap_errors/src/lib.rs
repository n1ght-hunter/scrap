#![feature(negative_impls)]

/// Useful type to use with `Result<>` indicate that an error has already
/// been reported to the user, so no need to continue checking.
///
/// The `()` field is necessary: it is non-`pub`, which means values of this
/// type cannot be constructed outside of this crate.
#[derive(
    Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
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

static VERBOSE_ERRORS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_verbose_errors(verbose: bool) {
    VERBOSE_ERRORS.store(verbose, std::sync::atomic::Ordering::Relaxed);
}

#[inline]
fn maybe_print_backtrace() {
    if VERBOSE_ERRORS.load(std::sync::atomic::Ordering::Relaxed) {
        // print full backtrace
        let bt = std::backtrace::Backtrace::force_capture();
        tracing::error!("backtrace:\n{:#?}", bt);
    }
}

#[macro_export]
macro_rules! simple_panic {
    (
        $($arg:tt)*
    ) => {
        tracing::error!($($arg)*);
        scrap_errors::FatalError.raise();
    };
}

pub trait SimpleError<T, E> {
    fn sunwrap(self) -> T;
    fn sexpect(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Debug> SimpleError<T, E> for Result<T, E> {
    fn sunwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("encountered error: {:?}", e);
                maybe_print_backtrace();
                FatalError.raise();
            }
        }
    }

    fn sexpect(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{}: {:?}", msg, e);
                maybe_print_backtrace();
                FatalError.raise();
            }
        }
    }
}
