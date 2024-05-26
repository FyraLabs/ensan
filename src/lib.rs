#[doc = include_str!("../README.md")]
#[warn(clippy::cargo)]
#[warn(clippy::complexity)]
#[warn(clippy::correctness)]
#[warn(clippy::nursery)]
#[warn(clippy::pedantic)]
#[warn(clippy::perf)]
#[warn(clippy::style)]
#[warn(clippy::suspicious)]
// followings are from clippy::restriction
#[warn(clippy::missing_errors_doc)]
#[warn(clippy::missing_panics_doc)]
#[warn(clippy::missing_safety_doc)]
#[warn(clippy::unwrap_used)]
#[warn(clippy::expect_used)]
#[warn(clippy::format_push_string)]
#[warn(clippy::get_unwrap)]
#[allow(clippy::missing_inline_in_public_items)]
#[allow(clippy::implicit_return)]
#[allow(clippy::blanket_clippy_restriction_lints)]
#[allow(clippy::pattern_type_mismatch)]
pub mod engine;
pub mod errors;
pub mod functions;
pub mod macros;

pub use engine::Engine;
pub use errors::Error;
