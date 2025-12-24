use std::io::prelude::*;
use std::{net::TcpStream, vec};

pub struct TCP_Client {
    pub stream: TcpStream,
    pub h: f64,
    pub k: f64,
    pub phi: f64,
    pub mic_dis: f64,
    pub del_t: f64,
    pub timestamp: u64,
}

impl TCP_Client {
    pub fn new(route: String, h: f64, k: f64, phi: f64, mic_dis: f64) -> Self {
        let stream = TcpStream::connect(route).expect("Cannot connect");

        TCP_Client {
            stream,
            h,
            k,
            phi,
            mic_dis,
            del_t: 0.0,
            timestamp: 0,
        }
    }

    pub fn send(&mut self) {
        let data_string = format!(
            "{},{},{},{},{},{}\n",
            self.timestamp, self.h, self.k, self.phi, self.mic_dis, self.del_t
        );
        self.stream.write(data_string.as_bytes()).unwrap();
        // let mut i = 0;
        // loop {
        //     let data = format!("Hello{i}\n");
        //     self.stream.write_all(data.as_bytes()).unwrap();
        //     i += 1;
        // }
    }
}

pub fn find_peak_index(
    min_max_range: (f32, f32),
    fft_db_array: &Vec<(f32, f32)>,
    angular_resolution: f32,
) -> Option<usize> {
    let (min, max) = min_max_range;

    let (min_ind, max_ind) = (
        (min / angular_resolution) as usize,
        (max / angular_resolution) as usize,
    );

    // I'll just use for loop
    let mut max_value: f32 = 0.0;
    let mut max_index: usize = min_ind;

    for i in min_ind..=max_ind {
        let (_cur_freq, cur_val) = fft_db_array.get(i)?;

        if *cur_val > max_value {
            max_value = *cur_val;
            max_index = i;
        }
    }

    Some(max_index)
}

pub fn filter_with_cfar(
    fft_db_array: &Vec<(f32, f32)>,
    cfar_db_array: &Vec<(f32, f32)>,
) -> Vec<(f32, f32)> {
    fft_db_array
        .iter()
        .zip(cfar_db_array)
        .map(|((x1, y1), (x2, y2))| {
            assert!(
                (x1 - x2).abs() < 0.01,
                "CFAR filtering Frequency Assertion Failled"
            );

            let value = if y1 > y2 {
                // cfar value low
                *y1
            } else {
                0.0
            };

            (*x1, value)
        })
        .collect()
}

/// Version that works with f32 for better performance when precision requirements are lower
pub fn angle_wrap_f32(angle: f32) -> f32 {
    use std::f32::consts::PI;

    let mut wrapped = angle % (2.0 * PI);

    if wrapped >= PI {
        wrapped -= 2.0 * PI;
    } else if wrapped < -PI {
        wrapped += 2.0 * PI;
    }

    wrapped
}
