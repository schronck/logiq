use criterion::{criterion_group, criterion_main, Criterion};
use logiq::{eval, parse, Gate};
use rand_core::{OsRng, RngCore};

fn gate_from_u32(num: u32) -> Gate {
    match num % 6 {
        0 => Gate::And,
        1 => Gate::Or,
        2 => Gate::Nand,
        3 => Gate::Nor,
        4 => Gate::Xor,
        5 => Gate::Not,
        _ => unreachable!(),
    }
}

fn generate_expression(rng: &mut OsRng, size: usize) -> String {
    let mut expression = String::new();

    if size == 0 {
        return 0.to_string();
    }

    let i = rng.next_u32();
    let gate = gate_from_u32(i);

    expression.push('(');

    if gate != Gate::Not {
        expression.push_str(&((i + 69) % (size as u32)).to_string());
    }

    expression.push_str(&format!(" {gate} "));

    expression.push_str(&generate_expression(rng, size - 1));

    expression.push(')');

    expression
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    let mut rng = OsRng;

    let expression_10 = generate_expression(&mut rng, 10);
    let expression_100 = generate_expression(&mut rng, 100);
    let expression_1000 = generate_expression(&mut rng, 1000);

    group.bench_function("bench_10", |b| b.iter(|| parse(&expression_10).unwrap()));

    group.bench_function("bench_100", |b| b.iter(|| parse(&expression_100).unwrap()));

    group.bench_function("bench_1000", |b| {
        b.iter(|| parse(&expression_1000).unwrap())
    });
}

fn bench_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval");
    let mut rng = OsRng;

    let expr_10 = generate_expression(&mut rng, 10);
    let expr_100 = generate_expression(&mut rng, 100);
    let expr_1000 = generate_expression(&mut rng, 1000);

    let bool_10 = (0..expr_10.len())
        .map(|i| (i % 2) != 0)
        .collect::<Vec<bool>>();

    let bool_100 = (0..expr_100.len())
        .map(|i| (i % 2) != 0)
        .collect::<Vec<bool>>();

    let bool_1000 = (0..expr_1000.len())
        .map(|i| (i % 2) != 0)
        .collect::<Vec<bool>>();

    group.bench_function("bench_10", |b| {
        b.iter(|| eval(&expr_10, &bool_10).unwrap());
    });

    group.bench_function("bench_100", |b| {
        b.iter(|| eval(&expr_100, &bool_100).unwrap());
    });

    group.bench_function("bench_1000", |b| {
        b.iter(|| eval(&expr_1000, &bool_1000).unwrap());
    });
}

criterion_group!(benches, bench_parsing, bench_eval);
criterion_main!(benches);
