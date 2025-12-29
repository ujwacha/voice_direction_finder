use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub struct TcpClient {
    route: String,
    stream: Option<TcpStream>,
    pub h: f64,
    pub k: f64,
    pub phi: f64,
    pub mic_dis: f64,
    pub del_t: f64,
    pub timestamp: u64,
}

impl TcpClient {
    pub fn new(route: String, h: f64, k: f64, phi: f64, mic_dis: f64) -> Self {
        let mut client = TcpClient {
            route,
            stream: None,
            h,
            k,
            phi,
            mic_dis,
            del_t: 0.0,
            timestamp: 0,
        };

        client.connect();
        client
    }

    fn connect(&mut self) {
        loop {
            match TcpStream::connect(&self.route) {
                Ok(stream) => {
                    let _ = stream.set_nodelay(true);
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
                    println!("Connected to {}", self.route);
                    self.stream = Some(stream);
                    break;
                }
                Err(e) => {
                    eprintln!("Connect failed: {e}, retrying...");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    pub fn send(&mut self) {
        let data = format!(
            "{},{},{},{},{},{}\n",
            self.timestamp, self.h, self.k, self.phi, self.mic_dis, self.del_t
        );

        if let Some(stream) = self.stream.as_mut() {
            if let Err(e) = stream
                .write_all(data.as_bytes())
                .and_then(|_| stream.flush())
            {
                eprintln!("Send error: {e}");
                self.stream = None; // drop broken stream
                self.connect(); // reconnect
            }
        } else {
            self.connect();
        }
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
