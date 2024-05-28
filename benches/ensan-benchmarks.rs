use criterion::{criterion_group, criterion_main, Criterion};

macro_rules! bench_group {
    ($group:ident => $($f:ident)*) => {
        fn $group(c: &mut Criterion) { $(
            c.bench_function(stringify!($f), |b| b.iter(|| $f()));
        )* }
    }
}

fn ref_attr_in_blk() {
    let _ = ensan::parse(
        r#"
        var "foo" { bar = "baz" }
        test = var.foo.bar
    "#,
    )
    .unwrap();
}
fn ref_attr_in_10_blks() {
    let _ = ensan::parse(
        r#"
        b1 = { b2 = { b3 = { b4 = { b5 = { b6 = { b7 = { b8 = { b9 = { b10 = { bar = "baz" }}}}}}}}}}
        test = b1.b2.b3.b4.b5.b6.b7.b8.b9.b10.bar
    "#,
    )
    .unwrap();
}
fn ref_attr_nblks_3_lbls() {
    let _ = ensan::parse(
        r#"
        blk "one" "two" "three" {
            bar = "baz"
            again "four" "five" "six" {
                foo = "?"
                hai = "bai"
            }
            hai = again.four.five.six.hai
            nya "idk" "anymore" {}
            another = bar
        }
        wow = blk.one.two.three.again.four.five.six.foo
        "#,
    )
    .unwrap();
}

bench_group!(criterion_refs => ref_attr_in_blk ref_attr_in_10_blks ref_attr_nblks_3_lbls);

criterion_group!(engine_benches, criterion_refs);
criterion_main!(engine_benches);
