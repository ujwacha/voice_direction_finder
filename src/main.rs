use audio::StreamEncapsulate;
use cpal::traits::StreamTrait;
use eframe::NativeOptions;
use signal::SignalProcessor;
use std::sync::mpsc;
use std::thread;
use ui::Application;
// use voice_direction_finder::filter_with_cfar;
// use voice_direction_finder::find_peak_index;

use rustfft::num_complex::Complex32;

mod audio;
mod signal;
mod ui;

const DEVICE: &str = "default";

fn main() -> Result<(), eframe::Error> {
    let stream_encapsulate = StreamEncapsulate::new(DEVICE); // Spawned New Thread Here
    stream_encapsulate.stream.play().unwrap(); // Runs the thread

    let mut signal_processor = SignalProcessor::new(stream_encapsulate.samples_per_sec);

    println!(
        "The time resolution is: {}",
        signal_processor.get_time_resolution()
    );

    let angle_resolution = (signal_processor.get_time_resolution() * 343.0 / 0.055).asin();
    println!(
        "angle_resolution: {} degrees",
        angle_resolution * 180.0 / 3.1415
    );

    let (app_right_tx, app_right_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (app_left_tx, app_left_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (app_left_cfar_tx, app_left_cfar_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (app_right_cfar_tx, app_right_cfar_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (cross_correlation_tx, cross_correlation_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (phase_tx, phase_rx) = mpsc::sync_channel::<(f32, f32)>(1);

    thread::spawn(move || {
        // Signal Processing Thread
        loop {
            //println!("LOOPING FFT LOOP");

            if let Ok(mut left_data) = stream_encapsulate.left_rx.recv()
                && let Ok(mut right_data) = stream_encapsulate.right_rx.recv()
            {
                let right_fft = signal_processor.fft(&mut right_data);
                let right_magnitude_plot = signal_processor.complex_fft_to_db_magnitude(&right_fft);

                let left_fft = signal_processor.fft(&mut left_data);
                let left_magnitude_plot = signal_processor.complex_fft_to_db_magnitude(&left_fft);

                // cfar left
                let var: Vec<f32> = left_magnitude_plot.iter().map(|(_x, y)| *y).collect();
                let cfar_left = SignalProcessor::cfar(&var, 10, 4, 3.5);
                let cfar_left = signal_processor.add_frequency_resolution(cfar_left);

                // cfar right
                let var: Vec<f32> = right_magnitude_plot.iter().map(|(_x, y)| *y).collect();
                let cfar_right = SignalProcessor::cfar(&var, 10, 4, 3.5);
                let cfar_right = signal_processor.add_frequency_resolution(cfar_right);

                let fft_conj_mul: Vec<Complex32> = left_fft
                    .iter()
                    .zip(right_fft.iter().map(|x| x.conj()))
                    .map(|(x, y)| x * y)
                    .collect();

                // let signal_to_send = signal_processor.add_frequency_resolution(fft_conj_mul);

                // for gcc phat, you have to divide the magnetude to make it "unity"

                let mut fft_conj_mul: Vec<Complex32> = fft_conj_mul
                    .iter()
                    .map(|x| x / (x.re * x.re + x.im * x.im).sqrt())
                    .collect();

                let correlation = signal_processor.ifft(&mut fft_conj_mul); // this part is gcc phat

                //                let magnetude = signal_processor.complex_signal_to_magnitude(&correlation);
                let magnetude = signal_processor.fft_time_addition(&correlation);

                let (max_time, max_correlation) =
                    match signal_processor.parabolic_interpolate_peak_robust(&magnetude) {
                        Ok((a, b)) => (a, b),
                        Err(_) => continue,
                    };

                // now fit a quardratic equation to get a better number

                let _ = app_right_tx.try_send(right_magnitude_plot);
                let _ = app_left_tx.try_send(left_magnitude_plot);
                let _ = app_left_cfar_tx.try_send(cfar_left);
                let _ = app_right_cfar_tx.try_send(cfar_right);
                let _ = cross_correlation_tx.try_send(magnetude);

                //let _ = phase_tx.try_send((phase_left, phase_right));
                let _ = phase_tx.try_send((max_correlation, max_time));
            }
        }
    });

    // Blocks
    eframe::run_native(
        "AudioDir",
        NativeOptions::default(),
        Box::new(move |cc| {
            Result::Ok(Box::new(Application::new(
                cc,
                app_right_rx,
                app_left_rx,
                app_right_cfar_rx,
                app_left_cfar_rx,
                phase_rx,
                cross_correlation_rx,
                &stream_encapsulate.samples_per_sec,
            )))
        }),
    )?;

    Ok(())
}
