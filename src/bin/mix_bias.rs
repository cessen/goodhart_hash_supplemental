use lib::{
    avalanche_chart::{
        compute_avalanche_chart, generate_counting, generate_random, generate_single_1_bit,
    },
    mixers::{aquahash, cityhash128, fnv1a, goodhart, meowhash, metrohash128, murmur3, xxhash3},
};

/// (name, mixing_function, input_size_in_bytes, output_size_in_bytes, digest_size_in_bytes, rounds)
///
/// The rounds is how many rounds to use in computing the avalanche statistics.
/// Higher numbers will give lower variance in the results, but also be slower.
/// For mixers that are already slow, you may want to reduce the number of
/// rounds.
const MIXERS: &[(&str, &dyn Fn(&[u8], &mut [u8]), usize, usize, usize, usize)] = &[
    (
        "Goodhart mixer, 12 rounds",
        &goodhart::mix_input,
        goodhart::IN_SIZE_BYTES,
        goodhart::OUT_SIZE_BYTES,
        goodhart::DIGEST_SIZE_BYTES,
        128 * 128,
    ),
    (
        "MeowHash v0.5 absorber",
        &meowhash::mix_input,
        meowhash::IN_SIZE_BYTES,
        meowhash::OUT_SIZE_BYTES,
        meowhash::DIGEST_SIZE_BYTES,
        128 * 64,
    ),
    (
        "Murmur3 accumulator",
        &murmur3::mix_input,
        murmur3::IN_SIZE_BYTES,
        murmur3::OUT_SIZE_BYTES,
        murmur3::DIGEST_SIZE_BYTES,
        128 * 128,
    ),
    (
        "CityHash128 accumulator",
        &cityhash128::mix_input,
        cityhash128::IN_SIZE_BYTES,
        cityhash128::OUT_SIZE_BYTES,
        cityhash128::DIGEST_SIZE_BYTES,
        128 * 64,
    ),
    (
        "MetroHash128 accumulator",
        &metrohash128::mix_input,
        metrohash128::IN_SIZE_BYTES,
        metrohash128::OUT_SIZE_BYTES,
        metrohash128::DIGEST_SIZE_BYTES,
        128 * 128,
    ),
    (
        "xxhash3 accumulator",
        &xxhash3::mix_input,
        xxhash3::IN_SIZE_BYTES,
        xxhash3::OUT_SIZE_BYTES,
        xxhash3::DIGEST_SIZE_BYTES,
        128 * 64,
    ),
    (
        "AquaHash accumulator",
        &aquahash::mix_input,
        aquahash::IN_SIZE_BYTES,
        aquahash::OUT_SIZE_BYTES,
        aquahash::DIGEST_SIZE_BYTES,
        128 * 64,
    ),
    (
        "FNV1a (128-bit) accumulator",
        &fnv1a::mix_input,
        fnv1a::IN_SIZE_BYTES,
        fnv1a::OUT_SIZE_BYTES,
        fnv1a::DIGEST_SIZE_BYTES,
        128 * 128,
    ),
];

fn main() {
    for (name, mixer, in_size, out_size, digest_size, rounds) in MIXERS.iter().copied() {
        println!("\n================================");
        println!("{}", name);
        println!("\nPattern: random");
        let chart = compute_avalanche_chart(
            generate_random,
            mixer,
            in_size,
            out_size,
            digest_size,
            rounds,
        );
        chart.print_report();
        chart.write_png(&format!("{} - random.png", name)).unwrap();

        println!("\nPattern: counting");
        let chart = compute_avalanche_chart(
            generate_counting,
            mixer,
            in_size,
            out_size,
            digest_size,
            1 << 12,
        );
        chart.print_report();
        chart
            .write_png(&format!("{} - counting.png", name))
            .unwrap();

        println!("\nPattern: single-bit");
        let chart = compute_avalanche_chart(
            generate_single_1_bit,
            mixer,
            in_size,
            out_size,
            digest_size,
            out_size * 8,
        );
        chart.print_report();
        chart
            .write_png(&format!("{} - single-bit.png", name))
            .unwrap();
    }
}
