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
    phase_rx: Receiver<VecDeque<f32>>,
    cross_correlation_rx: Receiver<Vec<(f32, f32)>>,
    sample_rate: u32,
}

impl Application {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        right_rx: Receiver<Vec<(f32, f32)>>,
        left_rx: Receiver<Vec<(f32, f32)>>,
        right_cfar_rx: Receiver<Vec<(f32, f32)>>,
        left_cfar_rx: Receiver<Vec<(f32, f32)>>,
        phase_rx: Receiver<VecDeque<f32>>,
        cross_correlation_rx: Receiver<Vec<(f32, f32)>>,
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
            cross_correlation_rx: cross_correlation_rx,
            sample_rate: *sample_rate,
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(right) = self.right_rx.recv()
            && let Ok(left) = self.left_rx.recv()
            && let Ok(left_cfar) = self.left_cfar_rx.recv()
            && let Ok(right_cfar) = self.right_cfar_rx.recv()
            && let Ok(cross_correlation) = self.cross_correlation_rx.recv()
            && let Ok(phases) = self.phase_rx.recv()
        {
            // self.add_element_in_queue(phases);

            let (high, _) = left.last().unwrap();
            let (high_cross, _) = cross_correlation.last().unwrap();
            let (low_cross, _) = cross_correlation.get(0).unwrap();
            let max_cross = cross_correlation
                .iter()
                .map(|(_x, y)| y)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap();

            egui::CentralPanel::default().show(ctx, |ui| {
                // Top panel for Left microphone
                egui::TopBottomPanel::top("left_mic")
                    .exact_height(ui.available_height() * 0.2)
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

                        egui::SidePanel::left("right_panel_mic")
                            .exact_width(ui.available_width() * 0.5)
                            .show_inside(ui, |ui| {
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

                        egui::SidePanel::left("angle_panel_mic")
                            .exact_width(ui.available_width())
                            .show_inside(ui, |ui| {
                                let root = EguiBackend::new(ui).into_drawing_area();
                                root.fill(&RGBColor(35, 35, 40)).unwrap();

                                let to_plot: Vec<(f32, f32)> = phases
                                    .iter()
                                    .map(|b| (b * 343.0 / 0.055).asin())
                                    .enumerate()
                                    .map(|(a, b)| (a as f32, b))
                                    .collect();

                                let mut chart2 = ChartBuilder::on(&root)
                                    .margin(8)
                                    .x_label_area_size(35)
                                    .y_label_area_size(45)
                                    .build_cartesian_2d(
                                        0.0f32..to_plot.len() as f32,
                                        -1.5f32..1.5f32,
                                    )
                                    .unwrap();

                                chart2
                                    .configure_mesh()
                                    .x_desc("time")
                                    .y_desc("angle")
                                    .label_style(("sans-serif", 13, &WHITE))
                                    .axis_style(&RGBColor(150, 150, 150))
                                    .draw()
                                    .unwrap();

                                chart2
                                    .draw_series(LineSeries::new(
                                        to_plot.iter().cloned(),
                                        &RGBColor(80, 150, 255),
                                    ))
                                    .unwrap();

                                root.present().unwrap();
                            });
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
                                .build_cartesian_2d(
                                    *low_cross..*high_cross,
                                    -*max_cross..*max_cross,
                                )
                                .unwrap();

                            chart
                                .configure_mesh()
                                .x_desc("Time ")
                                .y_desc("Cross Correlation")
                                .label_style(("sans-serif", 13, &WHITE))
                                .axis_style(&RGBColor(150, 150, 150))
                                .draw()
                                .unwrap();

                            chart
                                .draw_series(LineSeries::new(
                                    cross_correlation.iter().cloned(),
                                    &RGBColor(255, 80, 80).mix(0.7),
                                ))
                                .unwrap();

                            root.present().unwrap();
                        });

                    egui::SidePanel::right("right_panel")
                        .exact_width(ui.available_width())
                        .show_inside(ui, |ui| {
                            let root = EguiBackend::new(ui).into_drawing_area();
                            root.fill(&RGBColor(35, 35, 40)).unwrap();

                            let mut chart = ChartBuilder::on(&root)
                                .margin(8)
                                .x_label_area_size(35)
                                .y_label_area_size(45)
                                .build_cartesian_2d(-1.0f32..1.0f32, -1.0f32..1.0f32)
                                .unwrap();

                            chart
                                .configure_mesh()
                                .x_desc("X")
                                .y_desc("Y")
                                .label_style(("sans-serif", 13, &WHITE))
                                .axis_style(&RGBColor(150, 150, 150))
                                .draw()
                                .unwrap();

                            let time_delay = phases.get(phases.len() - 1).unwrap();
                            let angle = (time_delay * 343.0 / 0.055).asin();

                            // println!("angle: {}", angle * 180.0 / 3.1415);

                            let mut vec: Vec<(f32, f32)> = Vec::new();
                            vec.push((0.0, 0.0));
                            vec.push((angle.sin(), -angle.cos()));

                            chart
                                .draw_series(LineSeries::new(
                                    //phase_shift,
                                    vec.iter().cloned(),
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
