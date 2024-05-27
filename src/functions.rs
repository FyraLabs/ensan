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

/// Unquote a string
/// Workaround for a weird bug in hcl-rs (or maybe ensan itself?) where Value::String() includes the quotes too
pub(crate) fn unquote<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
#[ensan_proc_macro::ensan_internal_fn_mod(init_ctx_with_ensan_internal_fns)]
pub mod ensan_internal_fns {
    use hcl::{eval::FuncArgs, Value};

    use super::unquote;

    type FnRes = Result<Value, String>;

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
        let args = args.iter().next().ok_or("No arguments provided")?;

        let args = unquote(args.to_string());

        serde_yml::from_str(&args).map_err(|e| format!("Failed to deserialize YAML: {}", e))
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
        let args = args.iter().next().ok_or("No arguments provided")?;

        let ymlstring = serde_yml::to_string(args)
            .map_err(|e| format!("Failed to serialize YAML: {}", e))?
            .trim()
            .to_string();

        Ok(Value::String(ymlstring))
    }

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
        let args = args.iter().next().ok_or("No arguments provided")?;
        let key = unquote(args.to_string());
        std::env::var(key)
            .map(Value::String)
            .map_err(|e| format!("Failed to get environment variable: {}", e))
    }

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
        let args = args.iter().next().ok_or("No arguments provided")?;
        Ok(Value::String(unquote(args.to_string().to_lowercase())))
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
        let args = args.iter().next().ok_or("No arguments provided")?;
        Ok(Value::String(unquote(args.to_string().to_uppercase())))
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
        let mut args = args.iter();
        // If arg larger than 2, return error
        if args.len() != 2 {
            return Err("Invalid number of arguments".to_string());
        }
        let sep = args.next().ok_or("No separator provided")?;

        let sep = unquote(sep.to_string());

        // Second argument is the string to split
        let args = args.next().ok_or("No arguments provided")?;

        let args = unquote(args.to_string());

        let splitted: Vec<Value> = args
            .split(&sep)
            .map(|s| Value::String(s.to_string()))
            .collect();

        Ok(Value::Array(splitted))
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
        let mut args = args.iter();
        // If arg larger than 2, return error
        if args.len() != 2 {
            return Err("Invalid number of arguments".to_string());
        }
        let sep = args.next().ok_or("No separator provided")?;

        let sep = unquote(sep.to_string());

        // Second argument is the string to split
        let args = args.next().ok_or("No arguments provided")?;

        let args = args.as_array().unwrap_or_else(|| unreachable!());

        let args: Vec<String> = args.iter().map(|s| unquote(s.to_string())).collect();

        Ok(Value::String(args.join(&sep)))
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
        let args = args.iter().next().ok_or("No arguments provided")?;
        let len = unquote(args.to_string()).len();
        Ok(len.into())
    }
}
