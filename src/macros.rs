#[macro_export]
macro_rules! add_hcl_fns {
    ($ctx:expr => $($f:path[$($param:ident$(($inner:ident))?),*],)+) => {
        $(
            $ctx.declare_func(stringify!($f), hcl::eval::FuncDef::new($f, [$(hcl::eval::ParamType::$param$((Box::new(hcl::eval::ParamType::$inner)))?),*]));
        )+
    };
}
