use std::{fs::File, path::Path};

use nanorand::{Rng, WyRand};

pub struct Stats {
    pub input_bit_len: usize,
    pub output_bit_len: usize,
    pub digest_bit_len: usize,

    // The number of samples accumulated.  Or put another way, the number of
    // rounds used to generate the chart.
    pub sample_count: usize,

    // `input_bit_len * output_bit_len` long.  Each element is a count of the
    // number of bit flips for a given in/out bit pairing.
    pub avalanche_chart: Vec<u32>,

    // For every input bit, the BIC quadrants for each pair of output bits.
    pub bic_chart: Vec<[u32; 4]>,
}

impl Stats {
    pub fn new(
        input_bit_len: usize,
        output_bit_len: usize,
        digest_bit_len: usize,
        do_avalanche: bool,
        do_bic: bool,
    ) -> Self {
        Self {
            input_bit_len: input_bit_len,
            output_bit_len: output_bit_len,
            digest_bit_len: digest_bit_len,
            sample_count: 0,
            avalanche_chart: if do_avalanche {
                vec![0; input_bit_len * output_bit_len]
            } else {
                Vec::new()
            },
            bic_chart: if do_bic {
                vec![[0; 4]; input_bit_len * output_bit_len * (output_bit_len - 1)]
            } else {
                Vec::new()
            },
        }
    }

    pub fn accumulate(&mut self, in_bit: usize, out_bit: usize, flipped: bool) {
        self.avalanche_chart[in_bit * self.output_bit_len + out_bit] += flipped as u32;
    }

    pub fn get(&self, in_bit: usize, out_bit: usize) -> u32 {
        self.avalanche_chart[in_bit * self.output_bit_len + out_bit]
    }

    pub fn get_row(&self, in_bit: usize) -> &[u32] {
        let start = in_bit * self.output_bit_len;
        let end = start + self.output_bit_len;
        &self.avalanche_chart[start..end]
    }

    pub fn row_diffusion(&self, in_bit: usize) -> f64 {
        let norm = 1.0 / self.sample_count as f64;
        self.get_row(in_bit)
            .iter()
            .map(|&flips| 1.0 - p_to_bias(flips as f64 * norm))
            .sum()
    }

    pub fn row_entropy(&self, in_bit: usize) -> f64 {
        let norm = 1.0 / self.sample_count as f64;
        self.get_row(in_bit)
            .iter()
            .map(|&flips| p_to_entropy(flips as f64 * norm))
            .sum()
    }

    pub fn average_bias(&self) -> f64 {
        let norm = 1.0 / self.sample_count as f64;

        let bias_sum: f64 = self
            .avalanche_chart
            .iter()
            .map(|&flips| p_to_bias(flips as f64 * norm))
            .sum();
        bias_sum / self.avalanche_chart.len() as f64
    }

    pub fn min_bias(&self) -> f64 {
        let norm = 1.0 / self.sample_count as f64;

        let mut min_bias = 0.0f64;
        for &flips in &self.avalanche_chart {
            let bias = p_to_bias(flips as f64 * norm);
            min_bias = min_bias.min(bias);
        }
        min_bias
    }

    pub fn max_bias(&self) -> f64 {
        let norm = 1.0 / self.sample_count as f64;

        let mut max_bias = 0.0f64;
        for &flips in &self.avalanche_chart {
            let bias = p_to_bias(flips as f64 * norm);
            max_bias = max_bias.max(bias);
        }
        max_bias
    }

    pub fn min_input_bit_diffusion(&self) -> f64 {
        let mut min_diffusion = f64::INFINITY;
        for i in 0..self.input_bit_len {
            min_diffusion = min_diffusion.min(self.row_diffusion(i));
        }
        min_diffusion
    }

    pub fn avg_input_bit_diffusion(&self) -> f64 {
        let mut avg_diffusion = 0.0f64;
        for i in 0..self.input_bit_len {
            avg_diffusion += self.row_diffusion(i);
        }
        avg_diffusion / self.input_bit_len as f64
    }

    pub fn max_input_bit_diffusion(&self) -> f64 {
        let mut max_diffusion = 0.0f64;
        for i in 0..self.input_bit_len {
            max_diffusion = max_diffusion.max(self.row_diffusion(i));
        }
        max_diffusion
    }

    pub fn min_input_bit_entropy(&self) -> f64 {
        let mut min_entropy = f64::INFINITY;
        for i in 0..self.input_bit_len {
            min_entropy = min_entropy.min(self.row_entropy(i));
        }
        min_entropy
    }

    pub fn avg_input_bit_entropy(&self) -> f64 {
        let mut avg_entropy = 0.0f64;
        for i in 0..self.input_bit_len {
            avg_entropy += self.row_entropy(i);
        }
        avg_entropy / self.input_bit_len as f64
    }

    pub fn max_input_bit_entropy(&self) -> f64 {
        let mut max_entropy = 0.0f64;
        for i in 0..self.input_bit_len {
            max_entropy = max_entropy.max(self.row_entropy(i));
        }
        max_entropy
    }

    pub fn row_bic_avg_deviation(&self, in_bit_idx: usize) -> f64 {
        let stride = self.output_bit_len * (self.output_bit_len - 1);
        let start = in_bit_idx * stride;
        let end = start + stride;
        let bic = &self.bic_chart[start..end];

        let mut sum = 0.0;
        for [a, b, c, d] in bic.iter() {
            let min = *a.min(b).min(c).min(d);
            let max = *a.max(b).max(c).max(d);

            sum += (max - min) as f64 / max as f64;
        }
        sum / (stride * self.sample_count) as f64
    }

    pub fn min_bic_deviation(&self) -> f64 {
        let mut n = 999.0_f64;
        for i in 0..self.input_bit_len {
            n = n.min(self.row_bic_avg_deviation(i));
        }
        n
    }

    pub fn avg_bic_deviation(&self) -> f64 {
        let mut n = 0.0;
        for i in 0..self.input_bit_len {
            n += self.row_bic_avg_deviation(i);
        }
        n / self.input_bit_len as f64
    }

    pub fn max_bic_deviation(&self) -> f64 {
        let mut n = 0.0_f64;
        for i in 0..self.input_bit_len {
            n = n.max(self.row_bic_avg_deviation(i));
        }
        n
    }

    pub fn row_bic_avg_sorted_quadrants(&self, in_bit_idx: usize) -> [f64; 4] {
        let stride = self.output_bit_len * (self.output_bit_len - 1);
        let start = in_bit_idx * stride;
        let end = start + stride;
        let bic = &self.bic_chart[start..end];

        let mut sum = [0; 4];
        for mut quadrants in bic.iter().copied() {
            quadrants.sort_unstable();
            sum[0] += quadrants[0];
            sum[1] += quadrants[1];
            sum[2] += quadrants[2];
            sum[3] += quadrants[3];
        }

        [
            sum[0] as f64 / (stride * self.sample_count) as f64,
            sum[1] as f64 / (stride * self.sample_count) as f64,
            sum[2] as f64 / (stride * self.sample_count) as f64,
            sum[3] as f64 / (stride * self.sample_count) as f64,
        ]
    }

    pub fn avg_bic_sorted_quadrants(&self) -> [f64; 4] {
        let mut sum = [0.0; 4];
        for i in 0..self.input_bit_len {
            let n = self.row_bic_avg_sorted_quadrants(i);
            sum[0] += n[0];
            sum[1] += n[1];
            sum[2] += n[2];
            sum[3] += n[3];
        }

        sum[0] /= self.input_bit_len as f64;
        sum[1] /= self.input_bit_len as f64;
        sum[2] /= self.input_bit_len as f64;
        sum[3] /= self.input_bit_len as f64;

        sum
    }

    pub fn worst_bic_sorted_quadrants(&self) -> [f64; 4] {
        let mut worst = [999.0f64, 999.0, 999.0, 0.0];
        for i in 0..self.input_bit_len {
            let n = self.row_bic_avg_sorted_quadrants(i);
            worst[0] = worst[0].min(n[0]);
            worst[1] = worst[1].min(n[1]);
            worst[2] = worst[2].min(n[2]);
            worst[3] = worst[3].max(n[3]);
        }
        worst
    }

    pub fn best_bic_sorted_quadrants(&self) -> [f64; 4] {
        let mut best = [0.0; 4];
        for i in 0..self.input_bit_len {
            let n = self.row_bic_avg_sorted_quadrants(i);
            if n[0] > best[0] {
                best = n;
            }
        }
        best
    }

    pub fn print_report(&self) {
        if !self.avalanche_chart.is_empty() {
            println!(
                "    Bias:
        Min: {:0.2}
        Avg: {:0.2}
        Max: {:0.2}
    Input Bit Diffusion (digest size = {} bits):
        Min: {:0.1} bits
        Avg: {:0.1} bits 
        Max: {:0.1} bits
    Input Bit Diffusion Entropy (digest size = {} bits):
        Min: {:0.1} bits
        Avg: {:0.1} bits
        Max: {:0.1} bits",
                self.min_bias(),
                self.average_bias(),
                self.max_bias(),
                self.digest_bit_len,
                self.min_input_bit_diffusion(),
                self.avg_input_bit_diffusion(),
                self.max_input_bit_diffusion(),
                self.digest_bit_len,
                self.min_input_bit_entropy(),
                self.avg_input_bit_entropy(),
                self.max_input_bit_entropy(),
            );
        }

        if !self.bic_chart.is_empty() {
            let worst = self.worst_bic_sorted_quadrants();
            let avg = self.avg_bic_sorted_quadrants();
            let best = self.best_bic_sorted_quadrants();

            println!(
                "    BIC quadrants (sorted):
        Wrst: [{:0.4}, {:0.4}, {:0.4}, {:0.4}]
         Avg: [{:0.4}, {:0.4}, {:0.4}, {:0.4}]
        Best: [{:0.4}, {:0.4}, {:0.4}, {:0.4}]",
                worst[0],
                worst[1],
                worst[2],
                worst[3],
                avg[0],
                avg[1],
                avg[2],
                avg[3],
                best[0],
                best[1],
                best[2],
                best[3],
            );
        }
    }

    pub fn write_avalanche_png<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut pixels = Vec::new();

        for bit in self.avalanche_chart.iter().copied() {
            let v = (bit * 255 / self.sample_count as u32).min(255) as u8;
            pixels.extend_from_slice(&[v, v, v, 255]);
        }

        png_encode_mini::write_rgba_from_u8(
            &mut File::create(path.as_ref())?,
            &pixels,
            self.output_bit_len as u32,
            self.input_bit_len as u32,
        )?;

        Ok(())
    }
}

/// Computes an avalanche chart for a given mix/absorb function, using a provided
/// input generator.
///
/// - `generate_input`: function that takes a seed and generates an input block.
///   The result should be deterministic based on the seed.  Note that the seed
///   starts from zero, and simply increments each round.
/// - `mix`: function that takes input and mixes it to produce an output. Note
///   that any data in the passed output parameter should *not* be used by this
///   function, and should instead be ignored and overwritten.  In other words,
///   it is purely an out paramater, not an in-out parameter.
/// - `input_size`: size of `mix`'s input, in bytes.
/// - `output_size`: size of `mix`'s output, in bytes.
/// - `digest_size`: the size in bytes of the digest of the hash function `mix`
///   is a component of.  This is not actually used in any computations, and is
///   just provided as information in the final printouts.
/// - `rounds`: how many test rounds to perform to produce the estimated chart.
pub fn compute_stats<F1, F2>(
    generate_input: F1,
    mix: F2,
    input_size: usize,
    output_size: usize,
    digest_size: usize,
    rounds: usize,
    do_avalanche: bool,
    do_bic: bool,
) -> Stats
where
    F1: Fn(usize, &mut [u8]),
    F2: Fn(&[u8], &mut [u8]),
{
    let mut chart = Stats::new(
        input_size * 8,
        output_size * 8,
        digest_size * 8,
        do_avalanche,
        do_bic,
    );

    let mut input = vec![0u8; input_size];
    let mut output = vec![0u8; output_size];
    let mut input_tweaked = vec![0u8; input_size];
    let mut output_tweaked = vec![0u8; output_size];

    for round in 0..rounds {
        use std::io::Write;
        print!(
            "\r                                                  \rRound {}/{}",
            round, rounds
        );
        let _ = std::io::stdout().flush();

        generate_input(round, &mut input[..]);

        mix(&input[..], &mut output[..]);
        for in_bit_idx in 0..(input_size * 8) {
            input_tweaked.copy_from_slice(&mut input[..]);
            input_tweaked[in_bit_idx / 8] ^= 1 << (in_bit_idx % 8);
            mix(&input_tweaked[..], &mut output_tweaked[..]);

            // Avalanche.
            if do_avalanche {
                for out_bit_idx in 0..(output_size * 8) {
                    let i = out_bit_idx / 8;
                    let mask = 1 << (out_bit_idx % 8);
                    let flipped = (output[i] & mask) != (output_tweaked[i] & mask);

                    chart.accumulate(in_bit_idx, out_bit_idx, flipped);
                }
            }

            // Bit independence criterion.
            if do_bic {
                for i in 0..(output_size * 8) {
                    for j in 0..(output_size * 8 - 1) {
                        let i_b = (i + j + 1) % (output_size * 8);

                        let byte_a = i / 8;
                        let mask_a = 1 << (i % 8);
                        let byte_b = i_b / 8;
                        let mask_b = 1 << (i_b % 8);

                        let flipped_a =
                            (output[byte_a] & mask_a) != (output_tweaked[byte_a] & mask_a);
                        let flipped_b =
                            (output[byte_b] & mask_b) != (output_tweaked[byte_b] & mask_b);

                        let both = flipped_a && flipped_b;
                        let neither = !flipped_a && !flipped_b;
                        let only_left = flipped_a && !flipped_b;
                        let only_right = !flipped_a && flipped_b;

                        let stride = (output_size * 8) * (output_size * 8 - 1);
                        let k = (in_bit_idx * stride) + (i * (output_size * 8 - 1)) + j;

                        chart.bic_chart[k][0] += both as u32;
                        chart.bic_chart[k][1] += neither as u32;
                        chart.bic_chart[k][2] += only_left as u32;
                        chart.bic_chart[k][3] += only_right as u32;
                    }
                }
            }
        }

        chart.sample_count += 1;
    }

    print!("\r                                                  \r");

    chart
}

pub fn p_to_bias(p: f64) -> f64 {
    (p * 2.0 - 1.0).abs()
}

pub fn p_to_entropy(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        0.0
    } else {
        let q = 1.0 - p;
        -(p * p.log2()) - (q * q.log2())
    }
}

//-------------------------------------------------------------

/// Generates a random byte stream.
pub fn generate_random(seed: usize, bytes: &mut [u8]) {
    let mut rng = WyRand::new_seed(mix64(seed as u64));
    rng.fill_bytes(bytes);
}

/// Generates a byte stream with all zero bits except one.
pub fn generate_single_1_bit(index: usize, bytes: &mut [u8]) {
    let bit_idx = index % (bytes.len() * 8);
    let i = bit_idx / 8;
    let byte = 1 << (bit_idx % 8);
    bytes.fill(0);
    bytes[i] = byte;
}

/// Generates a byte stream with roughly `n` random bits set to one.
pub fn generate_n_random_bits(seed: usize, bytes: &mut [u8], n: usize) {
    let mut rng = WyRand::new_seed(mix64(seed as u64 ^ mix64(n as u64)));

    bytes.fill(0);
    for _ in 0..n {
        let n = rng.generate_range(0..(bytes.len() * 8));
        let byte_idx = n / 8;
        let byte_mask = 1 << (n % 8);
        bytes[byte_idx] |= byte_mask;
    }
}

/// Generates a byte stream with the lowest bits simply counting up as an
/// incrementing integer.
pub fn generate_counting(index: usize, bytes: &mut [u8]) {
    bytes[0..8].copy_from_slice(&u64::to_le_bytes(index as u64));
    bytes[8..].fill(0);
}

/// Generates a byte stream with the lowest bits simply iterating through gray
/// codes in order.
pub fn generate_gray_code(index: usize, bytes: &mut [u8]) {
    let gray_code = index ^ (index >> 1);
    bytes[0..8].copy_from_slice(&u64::to_le_bytes(gray_code as u64));
    bytes[8..].fill(0);
}

/// Generates all combinations of setting zero bits, then one bit, then two
/// bits, and so on, in that order.
#[allow(dead_code)]
pub fn generate_bit_combinations(index: usize, bytes: &mut [u8]) {
    let bit_len = bytes.len() * 8;

    // Compute the number of bits and the sub-index into that subsequence, for the
    // given index.
    let mut n = if bit_len < std::mem::size_of::<usize>() {
        // Wrap to the total number of combinations if that's less than usize.
        index % (1 << bit_len)
    } else {
        index
    };
    let mut bits = 0;
    let mut combos = binomial(bit_len, bits);
    while n >= combos {
        n -= combos;
        bits += 1;
        combos = binomial(bit_len, bits);
    }

    // Generate the bits for the computed subsequence and sub-index.
    bytes.fill(0);
    let mut t = bit_len;
    while t > 0 && bits > 0 {
        let y = if t > bits { binomial(t - 1, bits) } else { 0 };

        if n >= y {
            let byte_idx = (t - 1) / 8;
            let byte_mask = 1 << ((t - 1) % 8);
            bytes[byte_idx] |= byte_mask;

            n -= y;
            bits -= 1;
        }

        t -= 1;
    }
}

const fn binomial(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }

    if k == 0 {
        1
    } else if k > (n / 2) {
        binomial(n, n - k)
    } else {
        n * binomial(n - 1, k - 1) / k
    }
}

/// 64-bit bijective bit mixer.
fn mix64(mut n: u64) -> u64 {
    // Break zero sensitivity.
    n ^= 0x7be355f7c2e736d2;

    // http://zimbry.blogspot.ch/2011/09/better-bit-mixing-improving-on.html
    // (variant "Mix13")
    n ^= n >> 30;
    n = n.wrapping_mul(0xbf58476d1ce4e5b9);
    n ^= n >> 27;
    n = n.wrapping_mul(0x94d049bb133111eb);
    n ^= n >> 31;

    n
}
