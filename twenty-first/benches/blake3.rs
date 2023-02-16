use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use itertools::Itertools;
use rand::Rng;

use twenty_first::shared_math::tip5::STATE_SIZE;

fn bench_10(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3/hash_10");

    let size = 10;
    group.sample_size(100);

    let mut rng = rand::thread_rng();
    let single_element: [u8; STATE_SIZE * 8] = (0..STATE_SIZE * 8)
        .into_iter()
        .map(|_| rng.sample(rand::distributions::Standard))
        .collect_vec()
        .try_into()
        .unwrap();

    group.bench_function(BenchmarkId::new("Blake3 / Hash 10", size), |bencher| {
        bencher.iter(|| blake3::hash(&single_element))
    });
}

criterion_group!(benches, bench_10);
criterion_main!(benches);
