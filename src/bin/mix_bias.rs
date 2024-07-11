use lib::{
    avalanche_chart::{
        compute_avalanche_chart, generate_counting, generate_random, generate_single_1_bit,
    },
    mixers::{aquahash, cityhash128, fnv1a, goodhart, meowhash, metrohash128, murmur3, xxhash3},
};

struct Mixer<'a> {
    name: &'a str,
    mix_function: &'a dyn Fn(&[u8], &mut [u8]),
    input_size: usize,  // In bytes.
    output_size: usize, // In bytes.
    digest_size: usize, // In bytes.
}

const MIXERS: &[Mixer] = &[
    Mixer {
        name: "Goodhart mixer, 12 rounds",
        mix_function: &goodhart::mix_input,
        input_size: goodhart::IN_SIZE_BYTES,
        output_size: goodhart::OUT_SIZE_BYTES,
        digest_size: goodhart::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "MeowHash v0.5 absorber",
        mix_function: &meowhash::mix_input,
        input_size: meowhash::IN_SIZE_BYTES,
        output_size: meowhash::OUT_SIZE_BYTES,
        digest_size: meowhash::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "Murmur3 accumulator",
        mix_function: &murmur3::mix_input,
        input_size: murmur3::IN_SIZE_BYTES,
        output_size: murmur3::OUT_SIZE_BYTES,
        digest_size: murmur3::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "CityHash128 accumulator",
        mix_function: &cityhash128::mix_input,
        input_size: cityhash128::IN_SIZE_BYTES,
        output_size: cityhash128::OUT_SIZE_BYTES,
        digest_size: cityhash128::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "MetroHash128 accumulator",
        mix_function: &metrohash128::mix_input,
        input_size: metrohash128::IN_SIZE_BYTES,
        output_size: metrohash128::OUT_SIZE_BYTES,
        digest_size: metrohash128::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "xxhash3 accumulator",
        mix_function: &xxhash3::mix_input,
        input_size: xxhash3::IN_SIZE_BYTES,
        output_size: xxhash3::OUT_SIZE_BYTES,
        digest_size: xxhash3::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "AquaHash accumulator",
        mix_function: &aquahash::mix_input,
        input_size: aquahash::IN_SIZE_BYTES,
        output_size: aquahash::OUT_SIZE_BYTES,
        digest_size: aquahash::DIGEST_SIZE_BYTES,
    },
    Mixer {
        name: "FNV1a (128-bit) accumulator",
        mix_function: &fnv1a::mix_input,
        input_size: fnv1a::IN_SIZE_BYTES,
        output_size: fnv1a::OUT_SIZE_BYTES,
        digest_size: fnv1a::DIGEST_SIZE_BYTES,
    },
];

fn main() {
    // for (name, mixer, in_size, out_size, digest_size, rounds) in MIXERS.iter().copied() {
    for mixer in MIXERS.iter() {
        println!("\n================================");
        println!("{}", mixer.name);
        println!("\nPattern: random");
        let chart = compute_avalanche_chart(
            generate_random,
            mixer.mix_function,
            mixer.input_size,
            mixer.output_size,
            mixer.digest_size,
            1 << 16,
        );
        chart.print_report();
        chart
            .write_png(&format!("{} - random.png", mixer.name))
            .unwrap();

        println!("\nPattern: counting");
        let chart = compute_avalanche_chart(
            generate_counting,
            mixer.mix_function,
            mixer.input_size,
            mixer.output_size,
            mixer.digest_size,
            1 << 16,
        );
        chart.print_report();
        chart
            .write_png(&format!("{} - counting.png", mixer.name))
            .unwrap();

        // NOTE: because this test has a small, fixed number of rounds by its
        // nature, the generated statistics should be interpreted a little
        // differently. In particular, even a very good mixing function is
        // unlikely to achieve "perfect" avalanche by this measure, purely
        // because it's impossible to collect enough samples to reduce variance
        // enough.
        println!("\nPattern: single-bit");
        let chart = compute_avalanche_chart(
            generate_single_1_bit,
            mixer.mix_function,
            mixer.input_size,
            mixer.output_size,
            mixer.digest_size,
            mixer.output_size * 8,
        );
        chart.print_report();
        chart
            .write_png(&format!("{} - single-bit.png", mixer.name))
            .unwrap();
    }
}
