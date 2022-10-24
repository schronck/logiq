use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::{OsRng, RngCore};
use requiem::gate::Gate;
use requiem::token::LogicTree;
use requiem::TerminalId;

use std::collections::HashMap;
use std::str::FromStr;

fn gate_from_u32(num: u32) -> Gate {
    match num % 5 {
        0 => Gate::And,
        1 => Gate::Or,
        2 => Gate::Nand,
        3 => Gate::Nor,
        4 => Gate::Xor,
        _ => panic!("not possible"),
    }
}

fn generate_expression(rng: &mut OsRng, size: usize) -> String {
    let mut expression = String::from("0");
    let mut parentheses = 0;
    for i in 1..size {
        let next_u32 = rng.next_u32();
        let gate = gate_from_u32(next_u32);
        expression.push_str(&gate.to_string());
        if next_u32 % 5 == 0 {
            expression.push('(');
            parentheses += 1;
        }
        expression.push_str(&i.to_string());
        if next_u32 % 5 == 1 && parentheses > 0 {
            expression.push(')');
            parentheses -= 1;
        }
    }
    while parentheses > 0 {
        expression.push(')');
        parentheses -= 1;
    }
    expression
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    let mut rng = OsRng;
    let expression_10 = generate_expression(&mut rng, 10);
    let expression_100 = generate_expression(&mut rng, 100);
    let expression_1000 = generate_expression(&mut rng, 1000);

    group.bench_function("bench_10", |b| {
        b.iter(|| LogicTree::from_str(&expression_10).unwrap())
    });

    group.bench_function("bench_100", |b| {
        b.iter(|| LogicTree::from_str(&expression_100).unwrap())
    });

    group.bench_function("bench_1000", |b| {
        b.iter(|| LogicTree::from_str(&expression_1000).unwrap())
    });
}

fn bench_tree_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    let mut rng = OsRng;
    let expression_10 = generate_expression(&mut rng, 10);
    let expression_100 = generate_expression(&mut rng, 100);
    let expression_1000 = generate_expression(&mut rng, 1000);

    let tree_10 = LogicTree::from_str(&expression_10).unwrap();
    let tree_100 = LogicTree::from_str(&expression_100).unwrap();
    let tree_1000 = LogicTree::from_str(&expression_1000).unwrap();

    let map_10 = (0..expression_10.len())
        .map(|i| (i, (i % 2) != 0))
        .collect::<HashMap<TerminalId, bool>>();
    let map_100 = (0..expression_100.len())
        .map(|i| (i, (i % 2) != 0))
        .collect::<HashMap<TerminalId, bool>>();
    let map_1000 = (0..expression_1000.len())
        .map(|i| (i, (i % 2) != 0))
        .collect::<HashMap<TerminalId, bool>>();

    group.bench_function("bench_10", |b| {
        b.iter(|| tree_10.evaluate(&map_10).unwrap());
    });
    group.bench_function("bench_100", |b| {
        b.iter(|| tree_100.evaluate(&map_100).unwrap());
    });
    group.bench_function("bench_1000", |b| {
        b.iter(|| tree_1000.evaluate(&map_1000).unwrap());
    });
}

criterion_group!(benches, bench_parsing, bench_tree_evaluation,);
criterion_main!(benches);
