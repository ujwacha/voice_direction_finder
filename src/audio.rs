use cpal::{Device, traits::DeviceTrait, traits::HostTrait};
use rustfft::num_complex::Complex32;
use std::sync::mpsc::{self, Receiver};

pub struct StreamEncapsulate {
    pub stream: cpal::Stream,
    pub left_rx: Receiver<Vec<Complex32>>,
    pub right_rx: Receiver<Vec<Complex32>>,
    pub samples_per_sec: u32,
}

impl StreamEncapsulate {
    pub fn new(device_name: &str) -> Self {
        let host = cpal::default_host();

        let mut input: Vec<Device> = host
            .input_devices()
            .unwrap()
            .filter(|x| {
                let name = x.name().unwrap();
                dbg!(&name);
                name == device_name
            })
            .collect();

        let input = input.pop().expect("No Input Vector");
        println!("Input Device: {}", input.name().expect("No Name For Input"));

        // let input = host.default_input_device().unwrap();

        // for config in input.supported_input_configs().unwrap() {
        //     //config.buffer_size()
        //     dbg!(config);
        // }

        let config = input
            .default_input_config()
            .expect("No Default Input Configuration")
            .config();

        // config.buffer_size = BufferSize::Fixed(44100);
        // config.buffer_size = BufferSize::Fixed(10000);
        // dbg!(host.input_devices());

        dbg!(&config);

        let samples_per_sec = config.sample_rate.0;

        dbg!(samples_per_sec);

        let (tx_right, rx_right) = mpsc::sync_channel::<Vec<Complex32>>(100);
        let (tx_left, rx_left) = mpsc::sync_channel::<Vec<Complex32>>(100);

        let stream = input
            .build_input_stream(
                &config,
                move |x: &[f32], _a: &cpal::InputCallbackInfo| {
                    // runs in another thread
                    let even_left: Vec<Complex32> =
                        x.iter().step_by(2).map(|x| Complex32::from(x)).collect();
                    let odd_right: Vec<Complex32> = x
                        .iter()
                        .skip(1)
                        .step_by(2)
                        .map(|x| Complex32::from(x))
                        .collect();

                    // println!(
                    //     "ADC: {}\nCal: {}",
                    //     _a.timestamp()
                    //         .capture
                    //         .duration_since(&StreamInstant::new(0, 0))
                    //         .unwrap(),
                    //     _a.timestamp()
                    //         .callback
                    //         .duration_since(&StreamInstant::new(0, 0))
                    //         .unwrap()
                    // );

                    // dbg!(&_a);

                    // drop data if the FFT is not fast enough in reciever
                    if let Ok(_) = tx_left.try_send(even_left) {}
                    if let Ok(_) = tx_right.try_send(odd_right) {}
                },
                |err| {
                    // runs in another thread
                    // Callback Here
                    eprint!("[ERROR]: {err}");
                },
                None,
            )
            .expect("Couldn't Create the Stream");

        StreamEncapsulate {
            stream: stream,
            left_rx: rx_left,
            right_rx: rx_right,
            samples_per_sec,
        }
    }
}
