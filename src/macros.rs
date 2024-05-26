#[macro_export]
macro_rules! add_hcl_fns {
    ($ctx:expr => $($f:ident[$($param:ident),*])+) => {
        $(
            $ctx.declare_func(stringify!($f), FuncDef::new($f, [$(ParamType::$param),*]));
        )+
    };
}
