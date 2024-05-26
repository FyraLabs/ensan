use hcl::{
    eval::{Context, Evaluate, FuncArgs, FuncDef, ParamType},
    Value,
};

type FnRes = Result<Value, String>;

/// Serializes YAML from a string to HCL
///
/// Accepts: String
///
/// Returns: Value (Any)
///
/// Example: yamldecode("key: value") => { key = "value" }
fn yamldecode(args: FuncArgs) -> Result<Value, String> {
    let args = args.iter().next().ok_or("No arguments provided")?;
    Ok(serde_yml::from_str(&args.to_string())
        .map_err(|e| format!("Failed to deserialize YAML: {}", e))?)
}

/// Deserializes HCL from a string to YAML
///
/// Accepts: String
///
/// Returns: String
///
/// Example: yamlencode({ key = "value" }) => "key: value"
fn yamlencode(args: FuncArgs) -> Result<Value, String> {
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
/// Example: env("HOME") => "/home/user"
fn env(args: FuncArgs) -> Result<Value, String> {
    let args = args.iter().next().ok_or("No arguments provided")?;
    let key = args.to_string();
    Ok(std::env::var(&key)
        .map(Value::String)
        .map_err(|e| format!("Failed to get environment variable: {}", e))?)
}

/// Make all characters in a string lowercase
///
/// Accepts: String
///
/// Returns: String
///
/// Example: lower("HELLO") => "hello"
fn lower(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    Ok(Value::String(args.to_string().to_lowercase()))
}

/// Make all characters in a string uppercase
///
/// Accepts: String
///
/// Returns: String
///
/// Example: upper("hello") => "HELLO"
fn upper(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    Ok(Value::String(args.to_string().to_uppercase()))
}

/// Split a string into a list of string with a separator
///
/// Accepts: String, String
///
/// Returns: [String]
///
/// Example: split(",", "a,b,c") => ["a", "b", "c"]
fn split(args: FuncArgs) -> FnRes {
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
/// Example: join(",", ["a", "b", "c"]) => "a,b,c"
fn join(args: FuncArgs) -> FnRes {
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
/// Example: strlen("hello") => 5
fn strlen(args: FuncArgs) -> FnRes {
    let args = args.iter().next().ok_or("No arguments provided")?;
    let len = args.to_string().len();
    Ok(len.into())
}
