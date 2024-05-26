use hcl::{eval::{Context, Evaluate, FuncArgs, FuncDef, ParamType}, Value};

// #[hcl_func(ParamType::String)]
// fn yamldecode(args: FuncArgs) -> Result<Value, String>

// YAML de(ser)ialization functins
type Res<T> = Result<T, crate::Error>;

/// Serializes YAML from a string to HCL
fn yamldecode(args: FuncArgs) -> Result<Value, String> {
    let args = args.iter().next().ok_or("No arguments provided")?;
    Ok(serde_yml::from_str(&args.to_string())
        .map_err(|e| format!("Failed to deserialize YAML: {}", e))?)
}

/// Deserializes HCL from a string to YAML
fn yamlencode(args: FuncArgs) -> Result<Value, String> {
    let args = args.iter().next().ok_or("No arguments provided")?;

    Ok(Value::String(serde_yml::to_string(&args.to_string())
    .map_err(|e| format!("Failed to serialize YAML: {}", e))?))
}

/// Get value from environment variable
fn env(args: FuncArgs) -> Result<Value, String> {
    let args = args.iter().next().ok_or("No arguments provided")?;
    let key = args.to_string();
    Ok(std::env::var(&key)
        .map(Value::String)
        .map_err(|e| format!("Failed to get environment variable: {}", e))?)
}

#[derive(Debug, Clone)]
pub struct Engine<'a, S: AsRef<str>> {
    pub content: S,
    pub ctx: hcl::eval::Context<'a>,
}

impl<'a, S: AsRef<str>> From<S> for Engine<'a, S> {
    fn from(value: S) -> Self {
        Engine { content: value, ctx: Context::new() }
    }
}

#[derive(Debug, Clone)]
pub struct ValStore {
    pub key: String,
    pub val: Option<Box<ValStore>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HclTree {
    Root,
    // identifier, labels
    Block(Box<HclTree>, String, Vec<String>),
    Attr(Box<HclTree>, String)
}

impl HclTree {
    pub fn block(&self, ident: String, labels: Vec<String>) -> HclTree {
        HclTree::Block(Box::new(self.clone()), ident, labels)
    }
    pub fn attr(&self, key: String) -> HclTree {
        HclTree::Attr(Box::new(self.clone()), key)
    }
}


// ```hcl
// project "hai" "bai" {
//    field = "value"
// }
// project.hai.bai.field = "value"
// hai = {
///}

// ``` 
impl<'a, S: AsRef<str>> Engine<'a, S> {
    // fn _set_val_collect_idents(idents: &mut Vec<String>, mut inner: Box<HclTree>) {
    //     loop {
    //         match *inner {
    //             HclTree::Root => break,
    //             HclTree::Attr(x, name) => {
    //                 idents.push(name);
    //                 inner = x;
    //             },
    //             HclTree::Block(x, ident, labels) => {
    //                 for label in labels.into_iter().rev() {
    //                     idents.push(label);
    //                 }
    //                 idents.push(ident);
    //                 inner = x;
    //             }
    //         }
    //     }
    //     idents.reverse();
    // }
    // pub fn set_value(&mut self, name: HclTree, value: Value) {
    //     match name {
    //         HclTree::Root => panic!("Wrong use of HclTree::Root as ident"),
    //         HclTree::Attr(inner, name) => {
    //             let mut idents = vec![name];
    //             Self::_set_val_collect_idents(&mut idents, inner);
    //             for i in 0..idents.len()-1 {
    //                 self.ctx.declare_var(idents[0..i].join("."), hcl::value!({}));
    //             }
    //             self.ctx.declare_var(idents.join("."), value);
    //         },
    //         HclTree::Block(inner, ident, labels) => {
    //             let mut idents = labels;
    //             idents.reverse();
    //             idents.push(ident);
    //             Self::_set_val_collect_idents(&mut idents, inner);
    //             for i in 0..idents.len()-1 {
    //                 self.ctx.declare_var(idents[0..i].join("."), hcl::value!({}));
    //             }
    //             self.ctx.declare_var(idents.join("."), value);
    //         },
    //     }
    // }
        
    // pub fn add_structure_to_ctx(&mut self, structure: &mut hcl::Structure, root: HclTree) -> Res<()> {
    //     if let Some(attr) = structure.as_attribute_mut() {
    //         let value = attr.expr.evaluate(&self.ctx)?;
    //         attr.expr = value.clone().into();
    //         self.set_value(root.attr(attr.key.to_string()), value);
    //     } else if let Some(block) = structure.as_block_mut() {
    //         let newroot = root.block(block.identifier.to_string(), block.labels.iter().map(|l| l.as_str().to_string()).collect());
    //         for structure in &mut block.body {
    //             self.add_structure_to_ctx(structure, newroot.clone())?;
    //         }
    //     }
    //     Ok(())
    // }

    pub fn _parse_inner(&mut self, body: &mut hcl::Value) -> hcl::Value {
        if let Some(obj) = body.as_object_mut() {
            for val in obj.values_mut() {
                *val = self._parse_inner(val);
            }
        }
    }
    pub fn parse(&mut self) -> Res<hcl::Body> {
        let mut rawbody = hcl::parse(self.content.as_ref())?;
        crate::add_hcl_fns!(self.ctx =>
            yamldecode[String]
            yamlencode[String]
            env[String]
        );
        let mut body: hcl::Value = hcl::from_body(rawbody)?;
        self._parse_inner(&mut body);
        for structure in &mut rawbody {
            self.add_structure_to_ctx(structure, HclTree::Root)?;
        }
        Ok(rawbody)
    }
}

// a block wouuld equal to:
/// ```hcl
/// block "name" {
///     foo = "bar"
/// }
/// 
/// ```zzz
/// 
/// equals:
/// ```hcl
/// block = [
/// {
///  foo = "bar"
/// }
/// ]
/// ```

#[cfg(test)]
#[test]
fn test_eval_replacement() {
    let cfg = r#"
    // key = "value"
    arr = ["hai", "bai"]
    foo = arr[0]
    obj = {
        key = "value"
    }
    my_block "hai" {
        foo = "bar"
        baz = "nya"
    }

    nested_str = "${my_block.hai.foo}"
    "#;

    let expected = r#"
    arr = ["hai", "bai"]
    foo = "hai"
    obj = {
        key = "value"
    }
    my_block "hai" {
        foo = "bar"
        baz = "nya"
    }

    nested_str = "bar"
    "#;

    let mut engine = Engine::from(cfg);
    let body = engine.parse().unwrap();

    let mut engine2 = Engine::from(expected);
    let body2 = engine2.parse().unwrap();

    println!("{body:?}");

    assert_eq!(body, body2);

    // #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    // struct Config {
    //     key: String,
    //     foo: String,
    // }

    // let 

    // let val: Config = 
    // println!("{:?}", val);

    // assert_eq!(val, Config { key: "value".into(), foo: "value".into() });
    
}