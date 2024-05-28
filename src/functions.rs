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

#[ensan_proc_macro::ensan_internal_fn_mod(encoding)]
pub mod encoding {
    use super::*;
    use base64::prelude::*;

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
        must_let!([Value::String(arg)] = &args[..]);

        serde_yml::from_str(arg).map_err(|e| format!("Failed to deserialize YAML: {e}"))
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

    /// Deserializes JSON from a string to HCL
    ///
    /// Accepts: String
    ///
    /// Returns: Value (Any)
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = jsondecode("{\"key\": \"value\"}")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = { key = "value" }"#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn jsondecode(args: FuncArgs) -> FnRes {
        must_let!([Value::String(arg)] = &args[..]);

        serde_json::from_str(arg).map_err(|e| format!("Failed to deserialize JSON: {e}"))
    }

    #[test]
    fn test_jsondecode() {
        crate::parse(r#"hi = jsondecode()"#).expect_err("jsondecode() runs without args");
        crate::parse(r#"hi = jsondecode(1)"#).expect_err("jsondecode() runs with wrong-type args");
    }

    /// Serializes HCL from an object to JSON
    ///
    /// Accepts: Any
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = jsonencode({ key = "value" })"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "{\"key\":\"value\"}""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(Any)]
    pub fn jsonencode(args: FuncArgs) -> FnRes {
        must_let!([arg] = &args[..]);

        let jsonstring = serde_json::to_string(arg)
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            .trim()
            .to_string();
        Ok(Value::String(jsonstring))
    }

    /// Encode a string to base64
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = base64encode("hello")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "aGVsbG8=""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn base64encode(args: FuncArgs) -> FnRes {
        must_let!([Value::String(arg)] = &args[..]);

        let encoded = BASE64_STANDARD.encode(arg.as_bytes());
        Ok(Value::String(encoded))
    }

    /// Decode a base64 string to plain text
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = base64decode("aGVsbG8=")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "hello""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn base64decode(args: FuncArgs) -> FnRes {
        must_let!([Value::String(arg)] = &args[..]);

        let decoded = BASE64_STANDARD
            .decode(arg.as_bytes())
            .map_err(|e| format!("Failed to decode base64: {e}"))?;
        let decoded_str = String::from_utf8(decoded)
            .map_err(|e| format!("Failed to convert decoded bytes to string: {e}"))?;
        Ok(Value::String(decoded_str))
    }
}

#[ensan_proc_macro::ensan_internal_fn_mod(string_manipulation)]
/// This module contains string manipulation functions.
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
        must_let!([Value::String(arg)] = &args[..]);

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
        must_let!([Value::String(arg)] = &args[..]);

        Ok(Value::String(arg.to_uppercase()))
    }

    /// Split a string into a list of strings with a separator
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
        must_let!([Value::String(sep), Value::String(args)] = &args[..]);

        Ok(Value::Array(
            args.split(sep)
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
        must_let!([Value::String(sep), Value::Array(args)] = &args[..]);

        Ok(Value::String(
            args.iter().filter_map(Value::as_str).join(sep),
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

    /// Trim leading and trailing whitespace from a string
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = trimspace("  hello  ")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "hello""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn trimspace(args: FuncArgs) -> FnRes {
        must_let!([Value::String(s)] = &args[..]);
        Ok(s.trim().into())
    }

    /// Reverse a string
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Example:
    /// ```
    /// let eval = ensan::parse(r#"hi = strrev("hello")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "olleh""#).unwrap();
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn strrev(args: FuncArgs) -> FnRes {
        must_let!([Value::String(s)] = &args[..]);
        let reversed: String = s.chars().rev().collect();
        Ok(Value::String(reversed))
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
    /// Example:
    /// ```
    /// std::env::set_var("FOO", "bar"); // set environment variable
    ///
    /// let eval = ensan::parse(r#"hi = env("FOO")"#).unwrap();
    /// let expected = ensan::parse(r#"hi = "bar""#).unwrap();
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

#[ensan_proc_macro::ensan_internal_fn_mod(hashing)]
pub mod hashing {
    use super::*;

    // todo: add bcrypt function once we figure out how to do optional arguments

    /// Hash a string using the MD5 algorithm
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Note: MD5 is considered cryptographically broken and unsuitable for further use.
    /// This function is provided for compatibility with existing systems.
    ///
    /// Example:
    /// ```rust
    /// use md5::{Digest, Md5};
    ///
    /// let ex_str = "hello";
    ///
    /// let mut hasher = Md5::new();
    /// hasher.update(ex_str);
    /// let hash_result = format!("{:x}", hasher.finalize());
    ///
    /// let hcl_str = format!(r#"hi = md5("{}")"#, ex_str);
    ///
    /// let eval = ensan::parse(&hcl_str).unwrap();
    ///
    /// let expected = ensan::parse(&format!(r#"hi = "{}""#, hash_result)).unwrap();
    ///
    /// assert_eq!(eval, expected);
    ///
    /// ```
    ///
    #[ensan_fn(String)]
    pub fn md5(args: FuncArgs) -> FnRes {
        use md5::{Digest, Md5};
        must_let!([Value::String(s)] = &args[..]);
        let mut hasher = Md5::new();
        hasher.update(s);
        Ok(format!("{:x}", hasher.finalize()).into())
    }

    /// Hash a string using the SHA1 algorithm
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Note: SHA1 is considered cryptographically broken and unsuitable for further use.
    /// This function is provided for compatibility with existing systems.
    ///
    /// Example:
    ///
    /// ```rust
    /// use sha1::{Sha1, Digest};
    ///
    /// let ex_str = "hello";
    ///
    /// let mut hasher = Sha1::new();
    /// hasher.update(ex_str);
    /// let hash_result = format!("{:x}", hasher.finalize());
    ///
    /// let hcl_str = format!(r#"hi = sha1("{}")"#, ex_str);
    /// let eval = ensan::parse(&hcl_str).unwrap();
    ///
    /// let expected = ensan::parse(&format!(r#"hi = "{}""#, hash_result)).unwrap();
    ///
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn sha1(args: FuncArgs) -> FnRes {
        use sha1::{Digest, Sha1};
        must_let!([Value::String(s)] = &args[..]);
        let mut hasher = Sha1::new();
        hasher.update(s.as_bytes());
        Ok(format!("{:x}", hasher.finalize()).into())
    }

    /// Hash a string using the SHA256 algorithm
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Note: SHA256 is a widely used cryptographic hash function.
    ///
    /// Example:
    ///
    /// ```rust
    /// use sha2::{Sha256, Digest};
    ///
    /// let ex_str = "hello";
    ///
    /// let mut hasher = Sha256::new();
    /// hasher.update(ex_str);
    /// let hash_result = format!("{:x}", hasher.finalize());
    ///
    /// let hcl_str = format!(r#"hi = sha256("{}")"#, ex_str);
    /// let eval = ensan::parse(&hcl_str).unwrap();
    ///
    /// let expected = ensan::parse(&format!(r#"hi = "{}""#, hash_result)).unwrap();
    ///
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn sha256(args: FuncArgs) -> FnRes {
        use sha2::{Digest, Sha256};
        must_let!([Value::String(s)] = &args[..]);
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        Ok(format!("{:x}", hasher.finalize()).into())
    }

    /// Hash a string using the SHA512 algorithm
    ///
    /// Accepts: String
    ///
    /// Returns: String
    ///
    /// Note: SHA512 is a widely used cryptographic hash function.
    ///
    /// Example:
    ///
    /// ```rust
    /// use sha2::{Sha512, Digest};
    ///
    /// let ex_str = "hello";
    ///
    /// let mut hasher = Sha512::new();
    /// hasher.update(ex_str);
    /// let hash_result = format!("{:x}", hasher.finalize());
    ///
    /// let hcl_str = format!(r#"hi = sha512("{}")"#, ex_str);
    /// let eval = ensan::parse(&hcl_str).unwrap();
    ///
    /// let expected = ensan::parse(&format!(r#"hi = "{}""#, hash_result)).unwrap();
    ///
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String)]
    pub fn sha512(args: FuncArgs) -> FnRes {
        use sha2::{Digest, Sha512};
        must_let!([Value::String(s)] = &args[..]);
        let mut hasher = Sha512::new();
        hasher.update(s.as_bytes());
        Ok(format!("{:x}", hasher.finalize()).into())
    }
    // todo: make Nullable work when only one argument is passed?
    /// Hash a string using bcrypt
    ///
    /// Accepts: String, Nullable(Number)
    ///
    /// Returns: String
    ///
    /// Example:
    ///
    /// ```ignore // Test is ignored here because bcrypt is not deterministic, so we can't really compare the output
    /// use bcrypt::hash;
    ///
    /// let ex_str = "hello";
    ///
    /// let hash_result = hash(ex_str, 10).unwrap();
    ///
    /// let hcl_str = format!(r#"hi = bcrypt("{}", 10)"#, ex_str);
    /// let eval = ensan::parse(&hcl_str).unwrap();
    ///
    /// let expected = ensan::parse(&format!(r#"hi = "{}""#, hash_result)).unwrap();
    ///
    /// assert_eq!(eval, expected);
    /// ```
    #[ensan_fn(String, Nullable(Number))]
    pub fn bcrypt(args: FuncArgs) -> FnRes {
        use bcrypt::hash;
        // Ok, time to do this the old-fashioned way

        must_let!([Value::String(s), cost] = &args[..]);
        let cost = cost.as_u64().unwrap_or(10).try_into().unwrap();
        Ok(hash(s, cost)
            .map_err(|e| format!("Failed to hash string with bcrypt: {e}"))?
            .into())
    }
}

#[ensan_proc_macro::ensan_internal_fn_mod(uuid)]
pub mod uuid {
    use super::*;
    use ::uuid::Uuid;

    /// Generate a random UUID
    ///
    /// Accepts: None
    ///
    /// Returns: String
    ///
    #[ensan_fn()]
    pub fn uuidv4(_args: FuncArgs) -> FnRes {
        Ok(uuid::Uuid::new_v4().to_string().into())
    }

    /// Generate a UUIDv5 from a namespace and a name
    /// 
    /// Accepts: String, String
    /// 
    /// Returns: String
    /// 
    #[ensan_fn(String, String)]
    pub fn uuidv5(args: FuncArgs) -> FnRes {
        must_let!([Value::String(ns), Value::String(name)] = &args[..]);
        let ns = Uuid::parse_str(ns).map_err(|e| format!("Failed to parse UUID: {e}"))?;
        Ok(Uuid::new_v5(&ns, name.as_bytes()).to_string().into())
    }

}
