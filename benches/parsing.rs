use boolean_expression::BDD;
use borsh::{BorshDeserialize, BorshSerialize};
use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::{OsRng, RngCore};
use requiem::evaluator::BDDData;
use requiem::gate::Gate;
use requiem::token::TokenTree;
use requiem::TerminalId;

use std::collections::HashMap;
use std::str::FromStr;

fn generate_expression(rng: &mut OsRng, size: usize) -> String {
    let mut expression = String::from("0");
    for i in 1..size {
        let gate = Gate::from(rng.next_u32() % 5);
        expression.push_str(&gate.to_string());
        expression.push_str(&i.to_string());
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
        b.iter(|| TokenTree::from_str(&expression_10).unwrap())
    });

    group.bench_function("bench_100", |b| {
        b.iter(|| TokenTree::from_str(&expression_100).unwrap())
    });

    group.bench_function("bench_1000", |b| {
        b.iter(|| TokenTree::from_str(&expression_1000).unwrap())
    });
}

fn bench_bdd_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("bdd_build");
    let mut rng = OsRng;
    let expression_10 = generate_expression(&mut rng, 10);
    let expression_20 = generate_expression(&mut rng, 20);
    let expression_30 = generate_expression(&mut rng, 30);

    println!("GENERATING LARGE BDD...");
    let expression_large = generate_expression(&mut rng, 35);
    let now = std::time::Instant::now();
    let bdd = BDDData::from_str(&expression_large).unwrap();
    println!(
        "BDD LABELS: {}, ELAPSED: {} ms",
        bdd.bdd.labels().len(),
        now.elapsed().as_millis()
    );

    group.bench_function("bench_10", |b| {
        b.iter(|| BDDData::from_str(&expression_10).unwrap())
    });

    group.bench_function("bench_20", |b| {
        b.iter(|| BDDData::from_str(&expression_20).unwrap())
    });

    group.bench_function("bench_30", |b| {
        b.iter(|| BDDData::from_str(&expression_30).unwrap())
    });
}

fn bench_bdd_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("bdd_serialize");
    let mut rng = OsRng;
    let expression_10 = generate_expression(&mut rng, 10);
    let expression_20 = generate_expression(&mut rng, 20);
    let expression_30 = generate_expression(&mut rng, 30);

    let bdd_10 = BDDData::from_str(&expression_10).unwrap();
    let bdd_20 = BDDData::from_str(&expression_20).unwrap();
    let bdd_30 = BDDData::from_str(&expression_30).unwrap();

    let ser_short = bdd_10.bdd.try_to_vec().unwrap();
    let ser_medium = bdd_20.bdd.try_to_vec().unwrap();
    let ser_long = bdd_30.bdd.try_to_vec().unwrap();

    println!("SERIALIZED BDD LEN:");
    println!("FEW REQUIREMENTS: {}", ser_short.len());
    println!("MED REQUIREMENTS: {}", ser_medium.len());
    println!("LOT REQUIREMENTS: {}", ser_long.len());

    group.bench_function("bench_ser_10", |b| {
        b.iter(|| bdd_10.bdd.try_to_vec().unwrap());
    });
    group.bench_function("bench_ser_20", |b| {
        b.iter(|| bdd_20.bdd.try_to_vec().unwrap());
    });
    group.bench_function("bench_ser_30", |b| {
        b.iter(|| bdd_30.bdd.try_to_vec().unwrap());
    });
    group.bench_function("bench_de_10", |b| {
        b.iter(|| BDD::<TerminalId>::try_from_slice(&ser_short).unwrap());
    });
    group.bench_function("bench_de_20", |b| {
        b.iter(|| BDD::<TerminalId>::try_from_slice(&ser_medium).unwrap());
    });
    group.bench_function("bench_de_30", |b| {
        b.iter(|| BDD::<TerminalId>::try_from_slice(&ser_long).unwrap());
    });
}

fn bench_bdd_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bdd_evaluate");
    let mut rng = OsRng;
    let expression_10 = generate_expression(&mut rng, 10);
    let expression_20 = generate_expression(&mut rng, 20);
    let expression_30 = generate_expression(&mut rng, 30);

    let bdd_10 = BDDData::from_str(&expression_10).unwrap();
    let bdd_20 = BDDData::from_str(&expression_20).unwrap();
    let bdd_30 = BDDData::from_str(&expression_30).unwrap();

    let map_10 = (0..expression_10.len())
        .map(|i| (i, (i % 2) != 0))
        .collect::<HashMap<usize, bool>>();
    let map_20 = (0..expression_20.len())
        .map(|i| (i, (i % 2) != 0))
        .collect::<HashMap<usize, bool>>();
    let map_30 = (0..expression_30.len())
        .map(|i| (i, (i % 2) != 0))
        .collect::<HashMap<usize, bool>>();

    group.bench_function("bench_eval_10", |b| {
        b.iter(|| bdd_10.bdd.evaluate(bdd_10.root_bdd_func, &map_10));
    });
    group.bench_function("bench_eval_20", |b| {
        b.iter(|| bdd_20.bdd.evaluate(bdd_20.root_bdd_func, &map_20));
    });
    group.bench_function("bench_eval_30", |b| {
        b.iter(|| bdd_30.bdd.evaluate(bdd_30.root_bdd_func, &map_30));
    });
}

criterion_group!(
    benches,
    bench_parsing,
    bench_bdd_build,
    bench_bdd_serialization,
    bench_bdd_evaluation
);
criterion_main!(benches);
