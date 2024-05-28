//! # Ensan Expression Engine (for HCL)
//!
//! This module contains the [`Engine`] implementation.
//! You may use the engine to parse hcl-formatted strings.
//!
//! # Examples
//! ```
//! use ensan::Engine;
//! let cfg = r#"
//! // key = "value"
//! arr = ["hai", "bai"]
//! foo = arr[0]
//! obj = {
//!     key = "value"
//! }
//! my_block "hai" "bai" {
//!     foo = "bar"
//!     baz = "nya"
//! }
//!
//! nested_str = "${my_block.hai.bai.foo}"
//! "#;
//!
//! let expected = r#"
//! arr = ["hai", "bai"]
//! foo = "hai"
//! obj = {
//!     key = "value"
//! }
//! my_block "hai" "bai" {
//!     foo = "bar"
//!     baz = "nya"
//! }
//!
//! nested_str = "bar"
//! "#;
//!
//! let mut engine = Engine::new();
//! let body = engine.parse(cfg).unwrap();
//! let body2 = engine.parse(expected).unwrap();
//!
//! println!("{body:?}");
//!
//! assert_eq!(body, body2);
//! ```

use core::borrow::BorrowMut;
use hcl::{
    eval::{Context, Evaluate},
    Value,
};

/// Internal result type
type Res<T> = Result<T, crate::Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VarScope {
    Var(String, Value),
    Scope(String, VarScopes),
}

impl VarScope {
    #[must_use]
    #[inline]
    pub fn get_scope_mut(&mut self, scope: &str) -> Option<&mut VarScopes> {
        match self {
            Self::Scope(key, varscopes) if key == scope => Some(varscopes),
            _ => None,
        }
    }
    #[must_use]
    #[inline]
    pub fn get_scope_ref(&self, scope: &str) -> Option<&VarScopes> {
        match self {
            Self::Scope(key, varscopes) if key == scope => Some(varscopes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VarScopes(Vec<VarScope>);

impl From<Vec<VarScope>> for VarScopes {
    #[inline]
    fn from(value: Vec<VarScope>) -> Self {
        Self(value)
    }
}

impl VarScopes {
    /// List all variables that reside in the given scope.
    ///
    /// This function is extremely inefficient as it recursively look up all the variables inside
    /// the scope. If you pass in a scope of len 3, it'll call itself until it consumes all scopes
    /// and performs a `flat_map` operation.
    #[must_use]
    pub fn list_in_scope_mut<'a>(
        &'a mut self,
        scope: &'a [&str],
    ) -> Box<dyn Iterator<Item = &'_ mut VarScope> + 'a> {
        let [first, remaining @ ..] = scope else {
            return Box::new(self.0.iter_mut());
        };
        Box::new(
            self.0
                .iter_mut()
                .filter_map(|varscope| varscope.get_scope_mut(first))
                .flat_map(|varscopes| varscopes.list_in_scope_mut(remaining)),
        )
    }
    /// List all variables that reside in the given scope.
    ///
    /// This function is extremely inefficient as it recursively look up all the variables inside
    /// the scope. If you pass in a scope of len 3, it'll call itself until it consumes all scopes
    /// and performs a `flat_map` operation.
    #[must_use]
    pub fn list_in_scope_ref<'a>(
        &'a self,
        scope: &'a [impl AsRef<str>],
    ) -> Box<dyn Iterator<Item = &'_ VarScope> + 'a> {
        let [first, remaining @ ..] = scope else {
            return Box::new(self.0.iter());
        };
        Box::new(
            self.0
                .iter()
                .filter_map(|varscope| varscope.get_scope_ref(first.as_ref()))
                .flat_map(|varscopes| varscopes.list_in_scope_ref(remaining)),
        )
    }
    #[must_use]
    pub fn to_hcl_value(&self) -> Value {
        let mut indexmap = hcl::Map::new();
        self.0.iter().for_each(|x| match x {
            VarScope::Var(k, v) => _ = indexmap.insert(k.clone(), v.clone()),
            VarScope::Scope(k, v) => _ = indexmap.insert(k.clone(), v.to_hcl_value()),
        });
        Value::Object(indexmap)
    }
    pub fn populate_hcl_ctx(&self, ctx: &mut Context, scope: &[impl AsRef<str>]) {
        self.list_in_scope_ref(scope)
            .for_each(|varscopes| match varscopes {
                VarScope::Var(k, v) => ctx.declare_var(k.to_string(), v.to_owned()),
                VarScope::Scope(k, v) => ctx.declare_var(k.to_string(), v.to_hcl_value()),
            });
    }
    #[must_use]
    pub fn to_hcl_ctx(&self, scope: &[impl AsRef<str>]) -> Context {
        let mut ctx = Context::new();
        self.populate_hcl_ctx(&mut ctx, scope);
        ctx
    }
    /// Create a new variable and set the value.
    ///
    /// This function assumes the variable *does NOT exist*.
    /// However, if you call this twice with another `value`, the old value would be overwritten
    /// during `to_hcl_value()` and `to_hcl_ctx()`.
    pub fn set(&mut self, scope: &[String], key: String, value: Value) {
        let [first, rest @ ..] = scope else {
            self.0.push(VarScope::Var(key, value));
            return;
        };
        if let Some(s) = self.0.iter_mut().find_map(|s| s.get_scope_mut(first)) {
            s.set(rest, key, value);
        } else {
            let mut new = Self::default();
            new.set(rest, key, value); // TODO: optimizations
            self.0.push(VarScope::Scope(first.to_string(), new));
        }
    }
}

/// Engine for parsing hcl strings
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Engine<'a> {
    pub ctx_init: Context<'a>,
    /// (variable) scope during parsing
    pub scope: Vec<String>,
    /// variable list
    pub varlist: VarScopes,
}

impl Engine<'_> {
    #[must_use]
    pub fn new() -> Self {
        let mut ctx_init = Context::new();
        Self::init_ctx(&mut ctx_init);
        Self {
            ctx_init,
            ..Default::default()
        }
    }
    // NOTE: since 0.1.2 we are only calling this once then we just clone `self.ctx_init`
    // everytime we want a new context. This is because `ctx.declare_func()` is actually pretty
    // expensive. According to my benchmarks using flamegraph, during execution of
    // [`Self::parse_struct()`], 46% of the time it would be inside `init_ctx()`.
    fn init_ctx(ctx: &mut Context) {
        // We are going to import each function module here
        // todo: Make even more robust thing here or we can separate loading of each module by feature flags
        crate::functions::ensan_builtin_fns(ctx);
        crate::functions::string_manipulation(ctx);
        crate::functions::yaml(ctx);
    }

    fn parse_block(&mut self, block: &mut hcl::Block) -> Res<()> {
        let old_scope_len = self.scope.len();
        {
            self.scope.reserve(1 + block.labels.len());
            self.scope.push(block.identifier.to_string());
            self.scope
                .extend(block.labels.iter().map(|bl| bl.to_owned().into_inner()));
            for structure in &mut block.body {
                self.parse_struct(structure)?;
            }
        }
        self.scope.drain(old_scope_len..);
        Ok(())
    }

    fn parse_struct(&mut self, structure: &mut hcl::Structure) -> Res<()> {
        let mut ctx = self.ctx_init.clone();
        self.varlist.populate_hcl_ctx(&mut ctx, &self.scope);
        if let Some(attr) = structure.as_attribute_mut() {
            let val = attr.expr.evaluate(&ctx)?;
            self.varlist
                .set(&self.scope, attr.key.to_string(), val.clone());
            *attr.expr.borrow_mut() = val.into(); // NOTE: this is where we need &mut structure
        } else if let Some(block) = structure.as_block_mut() {
            self.parse_block(block)?;
        };
        Ok(())
    }

    /// Parse the string from hcl to an [`hcl::Body`] object.
    ///
    /// # Errors
    /// The following scenarios would terminate the function immediately:
    /// - failure to evalutate an hcl expression
    /// - syntax error
    pub fn parse(&mut self, content: impl AsRef<str>) -> Res<hcl::Body> {
        let mut body = hcl::parse(content.as_ref())?;
        for structure in &mut body {
            self.parse_struct(structure)?;
        }
        Ok(body)
    }
}
