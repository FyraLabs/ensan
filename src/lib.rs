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

pub use engine::Engine;
pub use errors::Error;

/// Quickly evaluate an HCL file
///
/// Outputs a fully-evaluated `hcl::Body` object, which is `hcl-rs`'s representation of an HCL file.
///
/// This type also implements `serde::Deserialize`, so you can easily convert it to any format you want.
///
/// # Example
///
/// ```rust
/// use hcl::Value;
///
/// let hcl = r#"
/// var "foo" {
///     bar = "baz"
/// }
/// test = var.foo.bar
///
/// "#;
///
/// let expected = r#"
/// var "foo" {
///     bar = "baz"
/// }
/// test = "baz"
///
/// "#;
///
/// let body = ensan::parse(hcl).unwrap();
///
/// let expected_body = hcl::from_str(expected).unwrap();
///
/// assert_eq!(body, expected_body);
///
/// ```
///
pub fn parse(s: impl AsRef<str>) -> Result<hcl::Body, Error> {
    Engine::new().parse(s)
}
