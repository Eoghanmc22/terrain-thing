use criterion::{Criterion, criterion_group, criterion_main, black_box};
use pprof::criterion::{Output, PProfProfiler};
use web_terrain::SlopeMode;

pub fn build_chunk_benchmark(c: &mut Criterion, slope_mode: SlopeMode) {
    let name = format!("build_chunk_{:?}", &slope_mode);
    let ctx = web_terrain::init(slope_mode, 10);

    c.bench_function(&name,
                     |b| b.iter(
                         || web_terrain::build_chunk_internal(
                             &|x, y, z, block| { black_box((x, y, z, block)); },
                             black_box(0), black_box(0), ctx)));

    web_terrain::drop_ctx(ctx);
}

pub fn build_chunk_benchmark_full(c: &mut Criterion) {
    build_chunk_benchmark(c, SlopeMode::FULL);
}

pub fn build_chunk_benchmark_von_neumann(c: &mut Criterion) {
    build_chunk_benchmark(c, SlopeMode::VonNeumann);
}

pub fn build_chunk_benchmark_m_von_neumann(c: &mut Criterion) {
    build_chunk_benchmark(c, SlopeMode::MVonNeumann);
}


criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)));
    targets = build_chunk_benchmark_full, build_chunk_benchmark_von_neumann, build_chunk_benchmark_m_von_neumann
}
criterion_main!(benches);