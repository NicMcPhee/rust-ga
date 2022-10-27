use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use rust_ga::{bitstring::count_ones, population::Population};

fn simple(c: &mut Criterion) {
    let mut grp = c.benchmark_group("simple_grp");
    grp.sample_size(100);
    grp.measurement_time(Duration::from_secs(30));

    grp.bench_function("simple", |b| {
        b.iter(|| {
            let pop = Population::new_bitstring_population(1000, 128, count_ones);
            pop.simple_lexicase();
        })
    });

    grp.finish();
}

criterion_group!(simple_grp, simple);
criterion_main!(simple_grp);
