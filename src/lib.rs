//! The [`helpful::Error`] is an upgraded version of [`anyhow::Error`].
//! It provides extra information to users and developers, which simplifies debugging & diagnosing the root cause.
//!
//! # Compare
//!
//! ## Anyhow
//!
//! ```shell
//! $ example_anyhow --config some/config.json
//! Error: No such file or directory (os error 2)
//! ```
//!
//! No extra information is provided - we have to guess what went wrong.
//!
//! ## Helpful
//!
//! ```shell
//! $ example_helpful --config some/config.json
//! Error: No such file or directory (os error 2)
//!
//! Call history (recent first):
//!    0: config::load
//!            with path="some/config.json"
//!              at examples/simple_helpful.rs:42
//!    1: cli::run
//!            with self=Cli { config: "some/config.json" }
//!              at examples/simple_helpful.rs:28
//! ```
//!
//! Extra information is provided - we can see that the error happened in `config::load` because `some/config.json` does not exist.
//!
//! Note: if you set `RUST_BACKTRACE=1`, both `anyhow` and `helpful` will display a full backtrace. However, the backtrace doesn't contain the values of the function arguments, so `helpful` will display both the span trace and the backtrace.
//!
//! # Features
//!
//! * ✅ Can be propagated up the call stack, just like [`anyhow::Error`]
//! * ✅ Can be constructed from existing error types, just like [`anyhow::Error`]
//! * ✅ Captures the current tracing span, just like [`tracing_error::TracedError<E>`]
//!
//! # Benefits
//!
//! * Provides a detailed span trace to the user (which makes it easier to diagnose the root cause of the error).
//! * Provides a detailed span trace to the developer (which simplifies debugging).
//!
//! # Comparison with [`anyhow::Error`]
//!
//! **Advantages:**
//!
//! * Provides additional information from the current span trace.
//!
//! **Disadvantages:**
//!
//! * Uses `Box<dyn Error>` instead of a slim pointer (this will be improved in the future release).
//!
//! # Comparison with [`tracing_error::TracedError<E>`]
//!
//! **Advantages:**
//!
//! * Can be propagated up the call stack with `?` operator (no explicit conversion needed). This is because [`helpful::Error`] doesn't have any generic arguments, so you can compose the functions that return a [`helpful::Result<T>`] with the `?` operator. By contrast, [`tracing_error::TracedError<E>`] is generic over `E`, so you can't compose the functions that return different `Result<T, TracedError<E>>`.
//!
//! **Disadvantages:**
//!
//! * Uses `Box<dyn Error>` instead of a slim pointer (this will be improved in the future release).
//!
//! # Setup
//!
//! * Initialize the tracing subscriber in `main`.
//! * Ensure the tracing subscriber has an [`ErrorLayer`](tracing_error::ErrorLayer).
//! * Ensure the default level is set to `Level::INFO` (or modify your `instrument` attributes to collect the data at a higher level).
//! * Optional: if you are going to use the example below, please run `cargo add tracing-subscriber --features env-filter`
//!
//! Example:
//!
//! ```
//! fn main() {
//!    init_tracing_subscriber();
//!     // your code here
//! }
//!
//! fn init_tracing_subscriber() {
//!    use tracing_subscriber::util::SubscriberInitExt;
//!    use tracing::level_filters::LevelFilter;
//!    use tracing_error::ErrorLayer;
//!    use tracing_subscriber::layer::SubscriberExt;
//!
//!    let env_filter = tracing_subscriber::EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy();
//!    let subscriber = tracing_subscriber::fmt()
//!        .with_env_filter(env_filter)
//!        .finish()
//!        .with(ErrorLayer::default());
//!    subscriber.init();
//! }
//! ```
//!
//! # Important setup note
//!
//! If you don't see any tracing spans in the error message, check your tracing subscriber configuration (see "[Setup](#setup)" for an example of a correct configuration).
//!
//! ## No-std support
//!
//! You can use this crate in `no_std` environment by disabling `default-features`:
//!
//! ```toml
//! [dependencies]
//! helpful = { version = "0.1.0", default-features = false }
//! ```
//!
//! The no_std mode enables the internal `StdError` trait which is a replacement for `std::error::Error`.
//! Since the `?`-based error conversions would normally rely on the `std::error::Error`, no_std mode will require an explicit `.map_err()`.
//!
//! # Tips
//!
//! ## Formatting
//!
//! You can format the fields using `Display` instead of `Debug` using the `%` symbol:
//!
//! ```
//! use std::fmt::{Display, Formatter};
//! use tracing::instrument;
//! #
//! # pub struct Url;
//! #
//! # impl Display for Url {
//! #   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//! #       todo!()
//! #   }
//! # }
//!
//! #[instrument(fields(url = %url))]
//! pub fn load(url: &Url) -> helpful::Result<String> {
//!     todo!()
//! }
//! ```
//!
//! [`helpful::Error`]: Error
//! [`helpful::Result<T>`]: Result
//! [`anyhow::Error`]: https://docs.rs/anyhow/latest/anyhow/struct.Error.html
//! [`tracing_error::TracedError<E>`]: https://docs.rs/tracing-error/latest/tracing_error/struct.TracedError.html
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

use alloc::boxed::Box;
use core::fmt::{Debug, Display, Formatter};
use core::result::Result as StdResult;
#[cfg(feature = "std")]
use std::backtrace::{Backtrace, BacktraceStatus};
#[cfg(feature = "std")]
use std::error::Error as StdError;
#[cfg(feature = "std")]
use std::process::{ExitCode, Termination};

use tracing_error::SpanTrace;

#[cfg(not(feature = "std"))]
pub trait StdError: Debug + Display {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

mod wrapper;

pub use wrapper::*;

/// The main `Error` type that provides additional information via `SpanTrace`.
///
/// This type doesn't implement the `Error` trait because it conflicts with a blanket `From<E>` implementation (which allows converting any error to this type). This is the same reason why `anyhow::Error` doesn't implement `Error`.
pub struct Error {
    pub source: Box<dyn StdError + Send + Sync + 'static>,
    pub span_trace: SpanTrace,
    #[cfg(feature = "std")]
    pub backtrace: Backtrace,
}

impl Error {
    pub fn new<E: StdError + Send + Sync + 'static>(source: E) -> Self {
        Self {
            source: Box::new(source),
            span_trace: SpanTrace::capture(),
            #[cfg(feature = "std")]
            backtrace: Backtrace::capture(),
        }
    }

    #[cold]
    #[must_use]
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self::new(MessageError(message))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            Display::fmt(self.source.as_ref(), f)
        } else {
            f.pad("Error: ")?;
            Debug::fmt(self, f)
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            Debug::fmt(self.source.as_ref(), f)
        } else {
            Display::fmt(self.source.as_ref(), f)?;
            f.pad("\n\n")?;
            f.pad("Call history (recent first):\n")?;
            Display::fmt(&self.span_trace, f)?;
            #[cfg(feature = "std")]
            if let BacktraceStatus::Captured = self.backtrace.status() {
                f.pad("\n\n")?;
                f.pad("Backtrace:\n")?;
                Display::fmt(&self.backtrace, f)?;
            }
            Ok(())
        }
    }
}

impl<E: StdError + Send + Sync + 'static> From<E> for Error {
    fn from(source: E) -> Self {
        Self::new(source)
    }
}

/// A type alias for `Result`, analogous to `anyhow::Result`
pub type Result<T = ()> = StdResult<T, Error>;

/// An extension trait to convert `Result` to `helpful::Result`
pub trait Traced {
    type Output;

    fn traced(self) -> Self::Output;
}

impl<T, E: Into<Error>> Traced for StdResult<T, E> {
    type Output = StdResult<T, Error>;

    fn traced(self) -> Self::Output {
        self.map_err(Into::into)
    }
}

/// A return type for `main` that automatically displays the error (see examples)
pub enum MainResult<T = (), E = Error> {
    Ok(T),
    Err(E),
}

impl<T, E> From<StdResult<T, E>> for MainResult<T, E> {
    fn from(value: StdResult<T, E>) -> Self {
        match value {
            Ok(value) => MainResult::Ok(value),
            Err(error) => MainResult::Err(error),
        }
    }
}

#[cfg(feature = "std")]
impl<T: Termination, E: Display> Termination for MainResult<T, E> {
    fn report(self) -> ExitCode {
        match self {
            MainResult::Ok(value) => value.report(),
            MainResult::Err(error) => {
                // TODO: attempt_print_to_stderr is private, need a workaround
                // std::io::attempt_print_to_stderr(format_args_nl!("Error: {err:?}"));
                eprintln!("{}", error);
                ExitCode::FAILURE
            }
        }
    }
}

// TODO: Implement more sophisticated branches (like in anyhow)
#[macro_export]
macro_rules! helpful {
    ($msg:literal $(,)?) => {
        $crate::__private::must_use({
            let error = $crate::__private::format_err($crate::__private::format_args!($msg));
            error
        })
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::msg($crate::__private::format!($fmt, $($arg)*))
    };
}

// Not public API. Referenced by macro-generated code.
// Copied from `anyhow` with omissions
#[doc(hidden)]
pub mod __private {
    use self::not::Bool;
    use crate::Error;
    use alloc::fmt;
    use core::fmt::Arguments;

    #[doc(hidden)]
    pub use alloc::format;
    #[doc(hidden)]
    pub use core::result::Result::Err;
    #[doc(hidden)]
    pub use core::{concat, format_args, stringify};

    #[doc(hidden)]
    #[inline]
    #[cold]
    pub fn format_err(args: Arguments) -> Error {
        // #[cfg(anyhow_no_fmt_arguments_as_str)]
        // let fmt_arguments_as_str = None::<&str>;
        // #[cfg(not(anyhow_no_fmt_arguments_as_str))]
        // Stable in Rust 1.52
        let fmt_arguments_as_str = args.as_str();

        if let Some(message) = fmt_arguments_as_str {
            // error!("literal"), can downcast to &'static str
            Error::msg(message)
        } else {
            // error!("interpolate {var}"), can downcast to String
            Error::msg(fmt::format(args))
        }
    }

    #[doc(hidden)]
    #[inline]
    #[cold]
    #[must_use]
    pub fn must_use(error: Error) -> Error {
        error
    }

    #[doc(hidden)]
    #[inline]
    pub fn not(cond: impl Bool) -> bool {
        cond.not()
    }

    mod not {
        #[doc(hidden)]
        pub trait Bool {
            fn not(self) -> bool;
        }

        impl Bool for bool {
            #[inline]
            fn not(self) -> bool {
                !self
            }
        }

        impl Bool for &bool {
            #[inline]
            fn not(self) -> bool {
                !*self
            }
        }
    }
}
