#[macro_export]
macro_rules! add_hcl_fns {
    ($ctx:expr => $($f:path[$($param:ident$(($inner:ident))?),*],)+) => {
        $(
            $ctx.declare_func(stringify!($f), hcl::eval::FuncDef::new($f, [$(hcl::eval::ParamType::$param$((Box::new(hcl::eval::ParamType::$inner)))?),*]));
        )+
    };
}

#[ensan_proc_macro::ensan_internal_fn_mod(my_new_fn)]
mod macro_test {
    use hcl::{eval::FuncArgs, Value};
    type FnRes = Result<Value, String>;

    #[ensan_fn(String)]
    pub fn yamldecode(args: FuncArgs) -> FnRes {
        let args = args.iter().next().ok_or("No arguments provided")?;
        serde_yml::from_str(&args.to_string())
            .map_err(|e| format!("Failed to deserialize YAML: {}", e))
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
    #[ensan_fn(String, Array(String))]
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
}
