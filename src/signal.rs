use rustfft::FftPlanner;
use rustfft::num_complex::Complex32;

pub struct SignalProcessor {
    planner: FftPlanner<f32>,
    samples_rate: u32,
}

impl SignalProcessor {
    pub fn new(samples_rate: u32) -> Self {
        SignalProcessor {
            planner: FftPlanner::new(),
            samples_rate,
        }
    }

    pub fn fft(&mut self, array: &mut Vec<Complex32>) -> Vec<Complex32> {
        let fft = self.planner.plan_fft_forward(array.len());
        fft.process(array);
        return array.clone();
    }

    pub fn ifft(&mut self, array: &mut Vec<Complex32>) -> Vec<Complex32> {
        let len = array.len();
        let fft = self.planner.plan_fft_inverse(array.len());
        fft.process(array);
        // normalize
        for i in 0..array.len() {
            // I can't do that with iter
            array[i] /= len as f32;
        }
        return array.clone();
    }

    pub fn complex_fft_to_db_magnitude(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
        let resolution = self.get_fft_frequency_resolution(array.len());
        array
            .iter()
            .enumerate()
            .map(|(i, x)| {
                (
                    i as f32 * resolution,
                    (x.re.powi(2) + x.im.powi(2)).sqrt().log10() * 10.0f32,
                )
            })
            .collect()
    }

    pub fn complex_signal_to_magnitude(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
        let resolution = self.get_time_resolution();
        array
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f32 * resolution, (x.re.powi(2) + x.im.powi(2)).sqrt()))
            .collect()
    }

    pub fn complex_signal_to_real_only(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
        let resolution = self.get_time_resolution();
        array
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f32 * resolution, x.re))
            .collect()
    }

    // pub fn fft_time_addition(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
    //     let resolution = self.get_time_resolution();

    //     // split the array in halves

    //     let (front, back) = array.split_at(array.len() / 2);

    //     // map

    //     let front: Vec<(f32, f32)> = front
    //         .iter()
    //         .enumerate()
    //         .map(|(i, x)| (i as f32 * resolution, x.re))
    //         .collect();

    //     let back: Vec<(f32, f32)> = back
    //         .iter()
    //         .enumerate()
    //         .map(|(i, x)| (-1.0 * ((i + 1) as f32 * resolution), x.re))
    //         .collect();

    //     let full_array = back
    //         .iter()
    //         .chain(front.iter())
    //         .map(|(x, y)| (*x, *y))
    //         .collect();

    //     return full_array;
    // }

    pub fn fft_time_addition(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
        let resolution = self.get_time_resolution();
        let n = array.len();

        // Perform FFT shift: second half -> first half, first half -> second half
        let mut shifted = vec![Complex32::new(0.0, 0.0); n];
        let half = n / 2;

        if n % 2 == 0 {
            // Even length (N=8): [0,1,2,3,4,5,6,7] -> [4,5,6,7,0,1,2,3]
            shifted[..half].copy_from_slice(&array[half..]); // Second half to first
            shifted[half..].copy_from_slice(&array[..half]); // First half to second
        } else {
            // Odd length (N=7): [0,1,2,3,4,5,6] -> [4,5,6,0,1,2,3]
            let first_part_len = (n + 1) / 2; // 4 for N=7
            let second_part_len = n / 2; // 3 for N=7

            // Copy second part first (indices first_part_len to end)
            shifted[..second_part_len].copy_from_slice(&array[first_part_len..]);
            // Copy first part second (indices 0 to first_part_len)
            shifted[second_part_len..].copy_from_slice(&array[..first_part_len]);
        }

        // Now assign proper time values: from -N/2 to N/2-1 for even, or similar for odd
        shifted
            .iter()
            .enumerate()
            .map(|(i, x)| {
                // Calculate time: (i - N/2) * resolution, but careful with integer division
                let time = (i as f32 - n as f32 / 2.0) * resolution;
                (time, x.re)
            })
            .collect()
    }

    pub fn parabolic_interpolate_peak_robust(
        &self,
        magnetude: &[(f32, f32)],
    ) -> Result<(f32, f32), &'static str> {
        if magnetude.len() < 3 {
            return Err("Need at least 3 points for interpolation");
        }

        // Find peak index
        let (max_index, (_, max_val)) = magnetude.iter().enumerate().fold(
            (0, (0.0, f32::NEG_INFINITY)),
            |(max_i, (max_t, max_v)), (i, &(t, v))| {
                if v > max_v {
                    (i, (t, v))
                } else {
                    (max_i, (max_t, max_v))
                }
            },
        );

        // Check peak is not at edges
        if max_index == 0 || max_index == magnetude.len() - 1 {
            return Err("Peak at boundary, cannot interpolate");
        }

        let (t_left, y_left) = magnetude[max_index - 1];
        let (t_center, y_center) = magnetude[max_index];
        let (t_right, y_right) = magnetude[max_index + 1];

        // Verify this is actually a peak
        if y_center <= y_left || y_center <= y_right {
            return Err("Not a valid peak (neighbors are higher)");
        }

        // Calculate time step (should be uniform)
        let time_step1 = t_center - t_left;
        let time_step2 = t_right - t_center;

        if (time_step1 - time_step2).abs() > 1e-6 {
            return Err("Non-uniform time spacing");
        }

        let time_step = time_step1;

        // Parabolic interpolation
        let denominator = y_left - 2.0 * y_center + y_right;

        if denominator.abs() < 1e-10 {
            return Err("Denominator too small for interpolation");
        }

        let offset = (time_step / 2.0) * (y_left - y_right) / denominator;

        // Constrain offset to reasonable range (±0.5 samples)
        let offset = offset.clamp(-time_step * 0.5, time_step * 0.5);

        let peak_time = t_center + offset;

        // Calculate interpolated value
        let t = offset; // Time from center
        let peak_value = y_center
            + 0.5 * (y_right - y_left) * t / time_step
            + 0.5 * (y_left - 2.0 * y_center + y_right) * t * t / (time_step * time_step);

        Ok((peak_time, peak_value))
    }

    pub fn complex_fft_to_phase_radians(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
        let resolution = self.get_fft_frequency_resolution(array.len());
        array
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f32 * resolution, x.im.atan2(x.re)))
            .collect()
    }

    pub fn cfar(db_fft_array: &Vec<f32>, gap: usize, refrence: usize, bias: f32) -> Vec<f32> {
        let mut ret_vec = vec![0.0; db_fft_array.len()];
        for i in 0..db_fft_array.len() {
            // first refrence
            let mut sum: f32 = 0.0;
            let mut len = 0;

            for j in (i as i32 - gap as i32 - refrence as i32)..(i as i32 - gap as i32) {
                if j > 0 {
                    sum += db_fft_array[j as usize];
                    len += 1;
                }
            }

            for j in (i + gap)..(i + gap + refrence) {
                if j < db_fft_array.len() {
                    sum += db_fft_array[j];
                    len += 1;
                }
            }

            let len = if len == 0 { refrence } else { len };

            let avg = sum / (2.0 * len as f32);

            let biased = bias * avg;

            ret_vec[i] = biased;
        }

        ret_vec
    }

    pub fn add_frequency_resolution(&self, array: Vec<f32>) -> Vec<(f32, f32)> {
        let resolution = self.get_fft_frequency_resolution(array.len());

        array
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f32 * resolution, *y))
            .collect()
    }

    pub fn add_time_resolution(&self, array: Vec<f32>) -> Vec<(f32, f32)> {
        let resolution = self.get_time_resolution();

        array
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f32 * resolution, *y))
            .collect()
    }

    pub fn calculate_phase_radian(z: &Complex32) -> f32 {
        z.im.atan2(z.re)
    }

    pub fn get_fft_frequency_resolution(&self, fft_len: usize) -> f32 {
        self.samples_rate as f32 / (fft_len as f32)
    }

    pub fn get_time_resolution(&self) -> f32 {
        1.0f32 / self.samples_rate as f32
    }
}
