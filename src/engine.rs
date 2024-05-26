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
//! my_block "hai" {
//!     foo = "bar"
//!     baz = "nya"
//! }
//!
//! nested_str = "${my_block.hai.foo}"
//! "#;
//!
//! let expected = r#"
//! arr = ["hai", "bai"]
//! foo = "hai"
//! obj = {
//!     key = "value"
//! }
//! my_block "hai" {
//!     foo = "bar"
//!     baz = "nya"
//! }
//!
//! nested_str = "bar"
//! "#;
//!
//! let mut engine = Engine::from(cfg);
//! let body = engine.parse().unwrap();
//!
//! let mut engine2 = Engine::from(expected);
//! let body2 = engine2.parse().unwrap();
//!
//! println!("{body:?}");
//!
//! assert_eq!(body, body2);
//! ```

use hcl::{
    eval::{Context, Evaluate},
    Value,
};
use std::borrow::BorrowMut;
// #[hcl_func(ParamType::String)]
// fn yamldecode(args: FuncArgs) -> Result<Value, String>

// YAML de(ser)ialization functins
type Res<T> = Result<T, crate::Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarScope {
    Var(String, Value),
    Scope(String, VarScopes),
}

impl VarScope {
    #[must_use]
    pub fn get_scope_mut(&mut self, scope: &str) -> Option<&mut VarScopes> {
        match self {
            Self::Scope(key, varscopes) if key == scope => Some(varscopes),
            _ => None,
        }
    }
    #[must_use]
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
    fn from(value: Vec<VarScope>) -> Self {
        Self(value)
    }
}

impl VarScopes {
    #[must_use]
    pub fn as_vec_mut(&mut self) -> Vec<&mut VarScope> {
        self.0.iter_mut().collect()
    }
    #[must_use]
    pub fn as_vec_ref(&self) -> Vec<&VarScope> {
        self.0.iter().collect()
    }
    #[must_use]
    pub fn list_in_scope_mut(&mut self, scope: &[&str]) -> Vec<&mut VarScope> {
        let [first, remaining @ ..] = scope else {
            return self.as_vec_mut();
        };
        self.0
            .iter_mut()
            .filter_map(|varscope| varscope.get_scope_mut(first))
            .flat_map(|varscopes| varscopes.list_in_scope_mut(remaining))
            .collect()
    }
    #[must_use]
    pub fn list_in_scope_ref(&self, scope: &[&str]) -> Vec<&VarScope> {
        let [first, remaining @ ..] = scope else {
            return self.as_vec_ref();
        };
        self.0
            .iter()
            .filter_map(|varscope| varscope.get_scope_ref(first))
            .flat_map(|varscopes| varscopes.list_in_scope_ref(remaining))
            .collect()
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
    #[must_use]
    pub fn to_hcl_ctx(&self, scope: &[impl AsRef<str>]) -> Context {
        let mut ctx = Context::new();
        self.list_in_scope_ref(&scope.iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .into_iter()
            .for_each(|varscopes| match varscopes {
                VarScope::Var(k, v) => ctx.declare_var(k.to_string(), v.to_owned()),
                VarScope::Scope(k, v) => ctx.declare_var(k.to_string(), v.to_hcl_value()),
            });
        ctx
    }
    pub fn set(&mut self, scope: &[String], key: String, value: Value) {
        let [first, rest @ ..] = scope else {
            self.0.push(VarScope::Var(key, value));
            return;
        };
        if let Some(s) = self.0.iter_mut().find_map(|s| s.get_scope_mut(first)) {
            s.set(rest, key, value);
        } else {
            let mut new = VarScopes::default();
            new.set(rest, key, value); // TODO: optimizations
            self.0.push(VarScope::Scope(first.to_string(), new));
        }
    }
}

/// Engine for parsing hcl strings
#[derive(Debug, Clone)]
pub struct Engine<S: AsRef<str>> {
    pub content: S,
    scope: Vec<String>,
    pub varlist: VarScopes,
}

impl<S: AsRef<str>> From<S> for Engine<S> {
    fn from(value: S) -> Self {
        Engine {
            content: value,
            scope: vec![],
            varlist: Default::default(),
        }
    }
}
impl<S: AsRef<str>> Engine<S> {
    fn init_ctx(&self, ctx: &mut Context) {
        use crate::functions;
        crate::add_hcl_fns!(ctx =>
            functions::yamlencode[String],
            functions::yamldecode[String],
            functions::env[String],
            functions::lower[String],
            functions::upper[String],
            functions::split[String, String],
            functions::join[String, Array(String)],
            functions::strlen[String],
        );
    }
    fn parse_block(&mut self, block: &mut hcl::Block) -> Res<()> {
        let old_scope_len = self.scope.len();
        {
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
        let mut ctx = self.varlist.to_hcl_ctx(&self.scope);
        self.init_ctx(&mut ctx);
        if let Some(attr) = structure.as_attribute_mut() {
            let val = attr.expr.evaluate(&ctx)?;
            self.varlist
                .set(&self.scope, attr.key.to_string(), val.clone());
            *attr.expr.borrow_mut() = val.into();
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
    pub fn parse(&mut self) -> Res<hcl::Body> {
        let mut body = hcl::parse(self.content.as_ref())?;
        for structure in &mut body {
            self.parse_struct(structure)?;
        }
        Ok(body)
    }
}
