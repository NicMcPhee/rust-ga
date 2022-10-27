use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_ga::{
    args::{Args, RunModel, TargetProblem},
    do_main,
};

const DEFAULT_ARGS: Args = Args {
    run_model: RunModel::Parallel,
    target_problem: TargetProblem::Hiff,
    population_size: 1000,
    bit_length: 128,
    num_generations: 100,
};

fn benchmark_run_count_ones_serial(c: &mut Criterion) {
    let args = Args {
        run_model: RunModel::Serial,
        target_problem: TargetProblem::CountOnes,
        ..DEFAULT_ARGS
    };
    c.bench_function("Run main() serially on Count Ones", |b| {
        b.iter(|| do_main(black_box(args)))
    });
}

fn benchmark_run_count_ones_parallel(c: &mut Criterion) {
    let args = Args {
        run_model: RunModel::Parallel,
        target_problem: TargetProblem::CountOnes,
        ..DEFAULT_ARGS
    };
    c.bench_function("Run main() in parallel on Count Ones", |b| {
        b.iter(|| do_main(black_box(args)))
    });
}

fn benchmark_run_hiff_serial(c: &mut Criterion) {
    let args = Args {
        run_model: RunModel::Serial,
        target_problem: TargetProblem::Hiff,
        ..DEFAULT_ARGS
    };
    c.bench_function("Run main() serially on HIFF", |b| {
        b.iter(|| do_main(black_box(args)))
    });
}

fn benchmark_run_hiff_parallel(c: &mut Criterion) {
    let args = Args {
        run_model: RunModel::Parallel,
        target_problem: TargetProblem::Hiff,
        ..DEFAULT_ARGS
    };
    c.bench_function("Run main() in parallel on HIFF", |b| {
        b.iter(|| do_main(black_box(args)))
    });
}

criterion_group!(
    run_benches,
    benchmark_run_count_ones_serial,
    benchmark_run_count_ones_parallel,
    benchmark_run_hiff_serial,
    benchmark_run_hiff_parallel
);
criterion_main!(run_benches);
