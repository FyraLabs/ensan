use criterion::{criterion_group, criterion_main, Criterion};
use ensan::Engine;

macro_rules! bench_group {
    ($group:ident => $($f:ident)*) => {
        fn $group(c: &mut Criterion) {
            let mut en = Engine::new();
            $(
                c.bench_function(stringify!($f), |b| b.iter(|| $f(&mut en)));
            )*
        }
    }
}

fn ref_attr_in_blk(en: &mut Engine) {
    let _ = en
        .parse(
            r#"
        var "foo" { bar = "baz" }
        test = var.foo.bar
    "#,
        )
        .unwrap();
    en.clean_up();
}
fn ref_attr_in_10_blks(en: &mut Engine) {
    let _ = en.parse(
        r#"
        b1 = { b2 = { b3 = { b4 = { b5 = { b6 = { b7 = { b8 = { b9 = { b10 = { bar = "baz" }}}}}}}}}}
        test = b1.b2.b3.b4.b5.b6.b7.b8.b9.b10.bar
    "#,
    )
    .unwrap();
    en.clean_up();
}
fn ref_attr_nblks_3_lbls(en: &mut Engine) {
    let _ = en
        .parse(
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
    en.clean_up();
}

bench_group!(criterion_refs => ref_attr_in_blk ref_attr_in_10_blks ref_attr_nblks_3_lbls);

criterion_group!(engine_benches, criterion_refs);
criterion_main!(engine_benches);
