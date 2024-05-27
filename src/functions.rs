//! # HCL functions
//!
//! This module contains a re-implementations of the HCL2 built-in functions in Rust.
//!
//! ensan aims to re-implement all of the built-in functions for HCL2 in Rust,
//! allowing for a consistent experience between gohcl and ensan/hcl-rs.
//!
//! The code is currently a work in progress and is not yet complete, see
//! https://developer.hashicorp.com/terraform/language/functions for the full list of functions both implemented and not implemented.

use hcl::{eval::FuncArgs, Value};

type FnRes = Result<Value, String>;

/// Serializes YAML from a string to HCL
///
/// Accepts: String
///
/// Returns: Value (Any)
///
/// Example:
/// ```hcl
/// yamldecode("key: value") => { key = "value" }
/// ```
pub fn yamldecode(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    serde_yml::from_str(&args.to_string()).map_err(|e| format!("Failed to deserialize YAML: {}", e))
}

/// Deserializes HCL from a string to YAML
///
/// Accepts: String
///
/// Returns: String
///
/// Example:
/// ```hcl
/// yamlencode({ key = "value" }) => "key: value"
/// ```
pub fn yamlencode(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;

    Ok(Value::String(
        serde_yml::to_string(&args.to_string())
            .map_err(|e| format!("Failed to serialize YAML: {}", e))?,
    ))
}

/// Get value from environment variable
///
/// Accepts: String
///
/// Returns: String
///
/// Example:
/// ```hcl
/// env("HOME") => "/home/user"
/// ```
pub fn env(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    let key = args.to_string();
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
/// ```hcl
/// lower("HELLO") => "hello"
/// ```
pub fn lower(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    Ok(Value::String(args.to_string().to_lowercase()))
}

/// Make all characters in a string uppercase
///
/// Accepts: String
///
/// Returns: String
///
/// Example:
/// ```
/// let eval = ensan::parse("hi = upper(\"hello\")").unwrap();
///
/// let expected = ensan::parse("hi = \"HELLO\"").unwrap();
/// assert_eq!(eval, expected);
/// ```
pub fn upper(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    Ok(Value::String(args.to_string().to_uppercase()))
}

/// Split a string into a list of string with a separator
///
/// Accepts: String, String
///
/// Returns: [String]
///
/// Example:
/// ```hcl
/// split(",", "a,b,c") => ["a", "b", "c"]
/// ```
pub fn split(args: FuncArgs) -> FnRes {
    let mut args = args.iter();
    // If arg larger than 2, return error
    if args.len() != 2 {
        return Err("Invalid number of arguments".to_string());
    }
    let sep = args.next().ok_or("No separator provided")?;

    let sep = sep.to_string();

    // Second argument is the string to split
    let args = args.next().ok_or("No arguments provided")?;

    let args = args.to_string();

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
/// ```hcl
/// join(",", ["a", "b", "c"]) => "a,b,c"
/// ```
pub fn join(args: FuncArgs) -> FnRes {
    let mut args = args.iter();
    // If arg larger than 2, return error
    if args.len() != 2 {
        return Err("Invalid number of arguments".to_string());
    }
    let sep = args.next().ok_or("No separator provided")?;

    let sep = sep.to_string();

    // Second argument is the string to split
    let args = args.next().ok_or("No arguments provided")?;

    let args = args.as_array().unwrap_or_else(|| unreachable!());

    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    Ok(Value::String(args.join(&sep)))
}

/// Get the length of a string
///
/// Accepts: String
///
/// Returns: Number
///
/// Example:
/// ```hcl
/// strlen("hello") => 5
/// ```
pub fn strlen(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    let len = args.to_string().len();
    Ok(len.into())
}
