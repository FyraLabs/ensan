#![cfg(test)]

#[test]
fn test_nested_block_with_3_labels() {
    let mut en = crate::Engine::new();
    let hcl = r#"
        blk "one" "two" "three" {
            bar = "baz"
            another = bar
            again "four" "five" "six" {
                foo = "?"
                hai = "bai"
            }
            hai = again.four.five.six.hai
        }
        wow = blk.one.two.three.again.four.five.six.foo
        "#;
    let expect = r#"
        blk "one" "two" "three" {
            bar = "baz"
            another = "baz" 
            again "four" "five" "six" {
                foo = "?"
                hai = "bai"
            }
            hai = "bai"
        }
        wow = "?"
        "#;
    assert_eq!(en.parse(hcl).unwrap(), en.clean_up().parse(expect).unwrap());
}
