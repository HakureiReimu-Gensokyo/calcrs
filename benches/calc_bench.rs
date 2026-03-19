use calcrs::{eval::Context, evaluate};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_simple(c: &mut Criterion) {
    let mut ctx = Context::new();
    c.bench_function("simple  1+2*3-4/2", |b| {
        b.iter(|| evaluate(black_box("1+2*3-4/2"), &mut ctx).ok());
    });
}

fn bench_power_right_assoc(c: &mut Criterion) {
    let mut ctx = Context::new();
    c.bench_function("power   2^3^2  (right-assoc = 512)", |b| {
        b.iter(|| evaluate(black_box("2^3^2"), &mut ctx).ok());
    });
}

fn bench_nested_parens(c: &mut Criterion) {
    let mut ctx = Context::new();
    c.bench_function("nested  ((3+4)*(2-1))/(7%3)", |b| {
        b.iter(|| evaluate(black_box("((3+4)*(2-1))/(7%3)"), &mut ctx).ok());
    });
}

fn bench_trig_and_log(c: &mut Criterion) {
    let mut ctx = Context::new();
    c.bench_function("trig+log  sin(pi/4)+cos(pi/3)*log10(100)", |b| {
        b.iter(|| {
            evaluate(
                black_box("sin(pi/4)+cos(pi/3)*log10(100)"),
                &mut ctx,
            )
            .ok()
        });
    });
}

fn bench_sci_notation(c: &mut Criterion) {
    let mut ctx = Context::new();
    c.bench_function("sci  1.5e10 * 2.7e-3", |b| {
        b.iter(|| evaluate(black_box("1.5e10 * 2.7e-3"), &mut ctx).ok());
    });
}

fn bench_hex_binary(c: &mut Criterion) {
    let mut ctx = Context::new();
    c.bench_function("radix  0xFF + 0b1010_1010", |b| {
        b.iter(|| evaluate(black_box("0xFF + 0b1010_1010"), &mut ctx).ok());
    });
}

fn bench_assign_reference(c: &mut Criterion) {
    let mut ctx = Context::new();
    // Pre-assign so the bench only measures read+compute path
    evaluate("x = 42", &mut ctx).ok();
    c.bench_function("var-ref  x * 2 + 1", |b| {
        b.iter(|| evaluate(black_box("x * 2 + 1"), &mut ctx).ok());
    });
}

fn bench_deep_chain(c: &mut Criterion) {
    let mut ctx = Context::new();
    // 12 operations — exercises the Pratt loop under load
    c.bench_function("chain  1+2+3+4+5+6+7+8+9+10+11+12", |b| {
        b.iter(|| {
            evaluate(black_box("1+2+3+4+5+6+7+8+9+10+11+12"), &mut ctx).ok()
        });
    });
}

criterion_group!(
    benches,
    bench_simple,
    bench_power_right_assoc,
    bench_nested_parens,
    bench_trig_and_log,
    bench_sci_notation,
    bench_hex_binary,
    bench_assign_reference,
    bench_deep_chain,
);
criterion_main!(benches);
