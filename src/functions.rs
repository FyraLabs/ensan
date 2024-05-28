//! # HCL functions
//!
//! This module contains a re-implementations of the HCL2 built-in functions in Rust.
//!
//! ensan aims to re-implement all of the built-in functions for HCL2 in Rust,
//! allowing for a consistent experience between gohcl and ensan/hcl-rs.
//!
//! The code is currently a work in progress and is not yet complete, see
//! https://developer.hashicorp.com/terraform/language/functions for the full list of functions both implemented and not implemented.

// TODO: Figure out why Value::String values include the quotes, and fix it or report it as a bug upstream!
use hcl::{eval::FuncArgs, Value};

type FnRes = Result<Value, String>;

macro_rules! must_let {
    ($left:pat = $right:expr) => {
        let $left = $right else { unreachable!() };
    };
}

#[ensan_proc_macro::ensan_internal_fn_mod(yaml)]
pub mod yaml {
    use super::*;

    /// Serializes YAML from a string to HCL
    ///
    /// Accepts: String
    ///
    /// Returns: Value (Any)
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = yamldecode("key: value")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = { key = "value" }"#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn yamldecode(args: FuncArgs) -> FnRes {
        must_let!([arg] = &args[..]);
        must_let!(Some(arg) = arg.as_str());

        serde_yml::from_str(&arg).map_err(|e| format!("Failed to deserialize YAML: {e}"))
    }

    #[test]
    fn test_yamldecode() {
        crate::parse(r#"hi = yamldecode()"#).expect_err("yamldecode() runs without args");
        crate::parse(r#"hi = yamldecode(1)"#).expect_err("yamldecode() runs with wrong-type args");
    }

    /// Deserializes HCL from a object to YAML
    ///
    /// Accepts: Any
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = yamlencode({ key = "value" })"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "key: value""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    // todo: fix Object type
    #[ensan_fn(Any)]
    pub fn yamlencode(args: FuncArgs) -> FnRes {
        must_let!([arg] = &args[..]);

        let ymlstring = serde_yml::to_string(arg)
            .map_err(|e| format!("Failed to serialize YAML: {e}"))?
            .trim()
            .to_string();

        Ok(Value::String(ymlstring))
    }
}

#[ensan_proc_macro::ensan_internal_fn_mod(string_manipulation)]
pub mod string_manipulation {
    use itertools::Itertools;

    use super::*;
    /// Make all characters in a string lowercase
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"
    /// hi = lower("HELLO")
    /// "#).unwrap();
    /// let expected = ensan::parse(r#"
    /// hi = "hello"
    /// "#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn lower(args: FuncArgs) -> FnRes {
        must_let!([arg] = &args[..]);
        must_let!(Some(arg) = arg.as_str());

        Ok(Value::String(arg.to_lowercase()))
    }

    /// Make all characters in a string uppercase
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = upper("hello")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "HELLO""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn upper(args: FuncArgs) -> FnRes {
        must_let!([arg] = &args[..]);
        must_let!(Some(arg) = arg.as_str());

        Ok(Value::String(arg.to_uppercase()))
    }

    /// Split a string into a list of string with a separator
    ///
    /// Accepts: String, String
    ///
    /// Returns: [String]
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = split(",", "a,b,c")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = ["a", "b", "c"]"#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String, String)]
    pub fn split(args: FuncArgs) -> FnRes {
        must_let!([sep, args] = &args[..]);
        must_let!(Some((sep, args)) = sep.as_str().zip(args.as_str()));

        Ok(Value::Array(
            args.split(&sep)
                .map(ToString::to_string)
                .map(Value::String)
                .collect(),
        ))
    }

    /// Join a list of strings into a single string with a separator
    ///
    /// Accepts: String, [String]
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = join(",", ["a", "b", "c"])"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "a,b,c""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String, Array(String))]
    pub fn join(args: FuncArgs) -> FnRes {
        must_let!([sep, args] = &args[..]);
        must_let!(Some((sep, args)) = sep.as_str().zip(args.as_array()));

        Ok(Value::String(
            args.iter().filter_map(Value::as_str).join(&sep),
        ))
    }

    /// Get the length of a string
    ///
    /// Accepts: String
    ///
    /// Returns: Number
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = strlen("hello")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = 5"#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn strlen(args: FuncArgs) -> FnRes {
        must_let!([Value::String(s)] = &args[..]);
        Ok(s.len().into())
    }
}

#[ensan_proc_macro::ensan_internal_fn_mod(ensan_builtin_fns)]
pub mod ensan_internal_fns {
    use super::*;

    /// Get value from environment variable
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    // Doctests are ignored here because environment variables are system-specific
    /// Example:
    /// ```ignore
    /// let eval = ensan::parse(r#"hi = env("HOME")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "/home/user""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn env(args: FuncArgs) -> FnRes {
        must_let!([Value::String(key)] = &args[..]);
        Ok(std::env::var(key)
            .map_err(|e| format!("Failed to get environment variable: {e}"))?
            .into())
    }
}
