use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use num_bigint::BigInt;
use twenty_first::shared_math::{
    prime_field_element_big::{PrimeFieldBig, PrimeFieldElementBig},
    rescue_prime_stark::RescuePrime,
    rpsss::RPSSS,
    stark::Stark,
};

pub fn get_tutorial_stark<'a>(field: &'a PrimeFieldBig) -> (Stark<'a>, RescuePrime<'a>) {
    let expansion_factor = 4;
    let colinearity_checks_count = 2;
    let rescue_prime = RescuePrime::from_tutorial(&field);
    let register_count = rescue_prime.m;
    let cycles_count = rescue_prime.steps_count + 1;
    let transition_constraints_degree = 2;
    let generator =
        PrimeFieldElementBig::new(85408008396924667383611388730472331217u128.into(), &field);

    (
        Stark::new(
            &field,
            expansion_factor,
            colinearity_checks_count,
            register_count,
            cycles_count,
            transition_constraints_degree,
            generator,
        ),
        rescue_prime,
    )
}

fn rpsss_bench_sign(c: &mut Criterion) {
    let modulus: BigInt = (407u128 * (1 << 119) + 1).into();
    let field = PrimeFieldBig::new(modulus);
    let (stark, rp) = get_tutorial_stark(&field);
    let rpsss = RPSSS {
        field: field.clone(),
        stark: stark.clone(),
        rp,
    };
    let document_string: String = "Hello Neptune!".to_string();
    let document: Vec<u8> = document_string.clone().into_bytes();

    // Calculate the index, AKA preprocessing
    let (transition_zerofier, transition_zerofier_mt, _transition_zerofier_mt_root) =
        stark.preprocess();

    let (sk, _pk) = rpsss.keygen();
    let mut group_sign = c.benchmark_group("rpsss_bench_sign");
    group_sign
        .bench_with_input(
            BenchmarkId::from_parameter("rpsss_bench_sign"),
            &1,
            |b, _| {
                b.iter(|| {
                    rpsss.sign(
                        &sk,
                        &document,
                        transition_zerofier.clone(),
                        transition_zerofier_mt.clone(),
                    )
                });
            },
        )
        .sample_size(10);
    group_sign.finish();
}

criterion_group!(benches, rpsss_bench_sign);
criterion_main!(benches);
