const HCL: &str = r#"
    b1 = { b2 = { b3 = { b4 = { b5 = { b6 = { b7 = { b8 = { b9 = { b10 = { bar = "baz" }}}}}}}}}}
    test = b1.b2.b3.b4.b5.b6.b7.b8.b9.b10.bar
"#;

fn main() {
    let mut engine = ensan::Engine::new();
    for _ in 0..5999 {
        let x = std::hint::black_box(engine.parse(HCL)).unwrap();
        drop(std::hint::black_box(x));
    }
}
