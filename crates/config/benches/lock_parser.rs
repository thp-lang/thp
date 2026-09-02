use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use thp_config::parse_lock;

fn lock_with_unparsed_extension() -> Vec<u8> {
    let payload = "[".repeat(64 * 1024);
    format!(
        "THP-LOCK 1\n\
         fingerprint {}\n\
         profile common\n\
         memory.limit 134217728\n\
         request.post_max_size 8388608\n\
         time.max_input 60\n\
         time.max_execution 30\n\
         extension example {}\n\
         {}\n\
         end-profile\n\
         end-lock\n",
        "0".repeat(64),
        payload.len(),
        payload
    )
    .into_bytes()
}

fn benchmark_lock_parser(criterion: &mut Criterion) {
    let bytes = lock_with_unparsed_extension();
    criterion.bench_function("parse core with 64 KiB opaque extension", |bencher| {
        bencher.iter(|| {
            let parsed = parse_lock(black_box(&bytes)).expect("benchmark fixture must parse");
            black_box(parsed.common.runtime);
        });
    });
}

criterion_group!(benches, benchmark_lock_parser);
criterion_main!(benches);
