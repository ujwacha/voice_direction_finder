use eframe::egui;
use eframe::egui::{Color32, Visuals};
use egui_plotter::EguiBackend;
use plotters::prelude::*;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

pub struct Application {
    right_rx: Receiver<Vec<(f32, f32)>>,
    left_rx: Receiver<Vec<(f32, f32)>>,
    right_cfar_rx: Receiver<Vec<(f32, f32)>>,
    left_cfar_rx: Receiver<Vec<(f32, f32)>>,
    phase_rx: Receiver<(f32, f32)>,
    phase_queue: VecDeque<(f32, f32)>,
    sample_rate: u32,
}

impl Application {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        right_rx: Receiver<Vec<(f32, f32)>>,
        left_rx: Receiver<Vec<(f32, f32)>>,
        right_cfar_rx: Receiver<Vec<(f32, f32)>>,
        left_cfar_rx: Receiver<Vec<(f32, f32)>>,
        phase_rx: Receiver<(f32, f32)>,
        sample_rate: &u32,
    ) -> Self {
        let context = &cc.egui_ctx;
        context.set_visuals(Visuals::dark());

        Application {
            right_rx,
            left_rx,
            right_cfar_rx,
            left_cfar_rx,
            phase_rx,
            phase_queue: VecDeque::new(),
            sample_rate: *sample_rate,
        }
    }

    fn add_element_in_queue(&mut self, phases: (f32, f32)) {
        self.phase_queue.push_back(phases);

        if self.phase_queue.len() > 120 {
            self.phase_queue.pop_front();
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(right) = self.right_rx.recv()
            && let Ok(left) = self.left_rx.recv()
            && let Ok(left_cfar) = self.left_cfar_rx.recv()
            && let Ok(right_cfar) = self.right_cfar_rx.recv()
            && let Ok(phases) = self.phase_rx.recv()
        {
            // make values
            //            let resolution = self.get_fft_frequency_resolution(right.len());
            //           let resolution = self.get_fft_frequency_resolution(left.len());

            self.add_element_in_queue(phases);

            let (high, _) = left.last().unwrap();

            egui::CentralPanel::default().show(ctx, |ui| {
                // Top panel for Left microphone
                egui::TopBottomPanel::top("left_mic")
                    .exact_height(ui.available_height() * 0.33)
                    .show_inside(ui, |ui| {
                        ui.add_space(3.0);

                        let root = EguiBackend::new(ui).into_drawing_area();
                        root.fill(&RGBColor(35, 35, 40)).unwrap();

                        let mut chart = ChartBuilder::on(&root)
                            .margin(8)
                            .x_label_area_size(35)
                            .y_label_area_size(45)
                            .build_cartesian_2d(0.0f32..*high, 0f32..60f32)
                            .unwrap();

                        chart
                            .configure_mesh()
                            .x_desc("Frequency (Hz)")
                            .y_desc("Magnitude (dB)")
                            .label_style(("sans-serif", 13, &WHITE))
                            .axis_style(&RGBColor(150, 150, 150))
                            .draw()
                            .unwrap();

                        chart
                            .draw_series(LineSeries::new(
                                left.iter().cloned(),
                                &RGBColor(255, 80, 80),
                            ))
                            .unwrap();

                        chart
                            .draw_series(LineSeries::new(
                                left_cfar.iter().cloned(),
                                &RGBColor(148, 255, 139),
                            ))
                            .unwrap();

                        root.present().unwrap();
                    });

                // Bottom panel for Right microphone
                egui::TopBottomPanel::bottom("right_mic")
                    .exact_height(ui.available_height() * 0.5)
                    .show_inside(ui, |ui| {
                        ui.add_space(3.0);

                        let root = EguiBackend::new(ui).into_drawing_area();
                        root.fill(&RGBColor(35, 35, 40)).unwrap();

                        let mut chart2 = ChartBuilder::on(&root)
                            .margin(8)
                            .x_label_area_size(35)
                            .y_label_area_size(45)
                            .build_cartesian_2d(0.0f32..*high, 0f32..60f32)
                            .unwrap();

                        chart2
                            .configure_mesh()
                            .x_desc("Frequency (Hz)")
                            .y_desc("Magnitude (dB)")
                            .label_style(("sans-serif", 13, &WHITE))
                            .axis_style(&RGBColor(150, 150, 150))
                            .draw()
                            .unwrap();

                        chart2
                            .draw_series(LineSeries::new(
                                right.iter().cloned(),
                                &RGBColor(80, 150, 255),
                            ))
                            .unwrap();

                        chart2
                            .draw_series(LineSeries::new(
                                right_cfar.iter().cloned(),
                                &RGBColor(148, 255, 139),
                            ))
                            .unwrap();

                        root.present().unwrap();
                    });

                // Middle panel - Combined view
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.add_space(3.0);

                    egui::SidePanel::left("left_panel")
                        .exact_width(ui.available_width() * 0.65)
                        .show_inside(ui, |ui| {
                            let root = EguiBackend::new(ui).into_drawing_area();
                            root.fill(&RGBColor(35, 35, 40)).unwrap();

                            let mut chart = ChartBuilder::on(&root)
                                .margin(8)
                                .x_label_area_size(35)
                                .y_label_area_size(45)
                                .build_cartesian_2d(0.0f32..150.0f32, -6.5f32..6.5f32)
                                .unwrap();

                            chart
                                .configure_mesh()
                                .x_desc("Frequency (Hz)")
                                .y_desc("Magnitude (dB)")
                                .label_style(("sans-serif", 13, &WHITE))
                                .axis_style(&RGBColor(150, 150, 150))
                                .draw()
                                .unwrap();

                            let phase_left = self
                                .phase_queue
                                .iter()
                                .map(|(x, _y)| x)
                                .enumerate()
                                .map(|(x, y)| (x as f32, *y));

                            chart
                                .draw_series(LineSeries::new(
                                    phase_left,
                                    &RGBColor(255, 80, 80).mix(0.7),
                                ))
                                .unwrap();

                            let phase_right = self
                                .phase_queue
                                .iter()
                                .map(|(_x, y)| y)
                                .enumerate()
                                .map(|(x, y)| (x as f32, *y));

                            chart
                                .draw_series(LineSeries::new(
                                    phase_right,
                                    &RGBColor(80, 150, 255).mix(0.7),
                                ))
                                .unwrap();

                            root.present().unwrap();
                        });

                    egui::SidePanel::right("right_panel")
                        .exact_width(ui.available_width())
                        .show_inside(ui, |ui| {
                            let root = EguiBackend::new(ui).into_drawing_area();
                            root.fill(&RGBColor(35, 35, 40)).unwrap();

                            // let mut chart = ChartBuilder::on(&root)
                            //     .margin(8)
                            //     .x_label_area_size(35)
                            //     .y_label_area_size(45)
                            //     .build_cartesian_2d(-1.0f32..1.0f32, -1.0f32..1.0f32)
                            //     .unwrap();

                            let mut chart = ChartBuilder::on(&root)
                                .margin(8)
                                .x_label_area_size(35)
                                .y_label_area_size(45)
                                .build_cartesian_2d(0.0f32..150.0f32, -7.5f32..7.5f32)
                                .unwrap();

                            chart
                                .configure_mesh()
                                .x_desc("X")
                                .y_desc("Y")
                                .label_style(("sans-serif", 13, &WHITE))
                                .axis_style(&RGBColor(150, 150, 150))
                                .draw()
                                .unwrap();

                            let phase_shift = self
                                .phase_queue
                                .iter()
                                .map(|(x, y)| x - y)
                                .enumerate()
                                .map(|(x, y)| (x as f32, y));

                            chart
                                .draw_series(LineSeries::new(
                                    phase_shift,
                                    &RGBColor(80, 150, 255).mix(0.7),
                                ))
                                .unwrap();

                            root.present().unwrap();
                        });
                });
            });
        }

        ctx.request_repaint();
    }
}
