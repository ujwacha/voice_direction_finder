use audio::StreamEncapsulate;
use cpal::traits::StreamTrait;
use eframe::NativeOptions;
use signal::SignalProcessor;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread;
use ui::Application;
use voice_direction_finder::filter_with_cfar;
use voice_direction_finder::find_peak_index;

mod audio;
mod signal;
mod ui;

const DEVICE: &str = "pulse";

fn main() -> Result<(), eframe::Error> {
    let stream_encapsulate = StreamEncapsulate::new(DEVICE); // Spawned New Thread Here
    stream_encapsulate.stream.play().unwrap(); // Runs the thread

    let mut signal_processor = SignalProcessor::new(stream_encapsulate.samples_per_sec);

    let (app_right_tx, app_right_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (app_left_tx, app_left_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (app_left_cfar_tx, app_left_cfar_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
    let (app_right_cfar_tx, app_right_cfar_rx) = mpsc::sync_channel::<Vec<(f32, f32)>>(1);
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
                let cfar_left = SignalProcessor::cfar(&var, 4, 4, 3.5);
                let cfar_left = signal_processor.add_frequency_resolution(cfar_left);

                // cfar right
                let var: Vec<f32> = right_magnitude_plot.iter().map(|(_x, y)| *y).collect();
                let cfar_right = SignalProcessor::cfar(&var, 4, 4, 3.5);
                let cfar_right = signal_processor.add_frequency_resolution(cfar_right);

                let filtered_right = filter_with_cfar(&right_magnitude_plot, &cfar_right);
                let res_right = signal_processor.get_fft_frequency_resolution(filtered_right.len());
                let target_index_right =
                    find_peak_index((1400.0, 1600.0), &filtered_right, res_right).unwrap();

                // println!("Freq: {}", target_index_right as f32 * res_right);

                let filtered_left = filter_with_cfar(&left_magnitude_plot, &cfar_left);
                let res_left = signal_processor.get_fft_frequency_resolution(filtered_left.len());
                let target_index_left =
                    find_peak_index((1400.0, 1600.0), &filtered_left, res_left).unwrap();

                // println!("Freq: {}", target_index_left as f32 * res_left);
                // // let phase_right =

                let phase_right = SignalProcessor::calculate_phase_radian(
                    right_fft.get(target_index_right).unwrap(),
                );

                let phase_left = SignalProcessor::calculate_phase_radian(
                    left_fft.get(target_index_left).unwrap(),
                );

                // let phase_diff = phase_left - phase_right;

                // println!("({})", phase_diff);

                let _ = app_right_tx.try_send(right_magnitude_plot);
                let _ = app_left_tx.try_send(left_magnitude_plot);
                let _ = app_left_cfar_tx.try_send(cfar_left);
                let _ = app_right_cfar_tx.try_send(cfar_right);
                let _ = phase_tx.try_send((phase_left, phase_right));
            }
        }
    });

    let native_options = NativeOptions::default();
    // Blocks
    eframe::run_native(
        "AudioDir",
        native_options,
        Box::new(move |cc| {
            Result::Ok(Box::new(Application::new(
                cc,
                app_right_rx,
                app_left_rx,
                app_right_cfar_rx,
                app_left_cfar_rx,
                phase_rx,
                &stream_encapsulate.samples_per_sec,
            )))
        }),
    )?;

    Ok(())
}
