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

    pub fn complex_fft_to_db_magnitude(&mut self, array: &Vec<Complex32>) -> Vec<(f32, f32)> {
        let resolution = self.get_fft_frequency_resolution(array.len());
        array
            .iter()
            .enumerate()
            .map(|(i, x)| {
                (
                    i as f32 * resolution,
                    (x.re.powi(2) + x.im.powi(2)).sqrt().log10() * 20.0f32,
                )
            })
            .collect()
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

    pub fn calculate_phase_radian(z: &Complex32) -> f32 {
        z.im.atan2(z.re)
    }

    pub fn get_fft_frequency_resolution(&self, fft_len: usize) -> f32 {
        self.samples_rate as f32 / (fft_len as f32)
    }
}
