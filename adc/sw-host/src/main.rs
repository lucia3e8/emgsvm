use std::io::{self, BufRead, BufReader};
use std::time::Duration;
use binrw::{BinRead, io::Cursor, binread};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::time::Instant;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
struct Frame {
    #[br(temp)]
    _discard: u8,
    status: u8,
    micros: u32,
    values: [i32; 8],
}

const FRAME_COUNT: usize = 1024;

#[derive(Debug, Clone)]
struct StatusBits {
    lock: bool,
    f_resync: bool,
    reg_map: bool,
    crc_err: bool,
    crc_type: bool,
    reset: bool,
    wlength: u8,
}

impl StatusBits {
    fn wlength_str(&self) -> &'static str {
        match self.wlength {
            0b00 => "16-bit",
            0b01 => "24-bit",
            0b10 => "32-bit (zero pad)",
            0b11 => "32-bit (sign ext)",
            _ => "Unknown",
        }
    }

    fn crc_type_str(&self) -> &'static str {
        if self.crc_type {
            "16-bit ANSI"
        } else {
            "16-bit CCITT"
        }
    }
}

pub struct AdcGui {
    // Shared data buffer with the serial reader thread
    data_buffer: Arc<Mutex<VecDeque<Frame>>>,

    // Local copy for plotting to avoid holding the lock
    plot_data: Vec<VecDeque<f32>>,

    // Configuration
    max_points: usize,

    // Recording state
    is_recording: bool,

    // For frame rate limiting
    last_update: Instant,

    // Latest status byte
    latest_status: Option<u8>,
}

impl AdcGui {
    pub fn new(data_buffer: Arc<Mutex<VecDeque<Frame>>>) -> Self {
        Self {
            data_buffer,
            plot_data: vec![VecDeque::with_capacity(1000); 8],
            max_points: 1000, // Keep last 1000 points per channel
            is_recording: false,
            last_update: Instant::now(),
            latest_status: None,
        }
    }

    fn decode_status(&self, status: u8) -> StatusBits {
        StatusBits {
            lock: (status & 0x80) != 0,
            f_resync: (status & 0x40) != 0,
            reg_map: (status & 0x20) != 0,
            crc_err: (status & 0x10) != 0,
            crc_type: (status & 0x08) != 0,
            reset: (status & 0x04) != 0,
            wlength: status & 0x03,
        }
    }

    fn update_plot_data(&mut self) {
        // Try to acquire lock without blocking
        if let Ok(buffer) = self.data_buffer.try_lock() {
            // Process all new frames
            for frame in buffer.iter() {
                // Update latest status
                self.latest_status = Some(frame.status);
                
                for (ch_idx, &value) in frame.values.iter().enumerate() {
                    // Convert i32 to f32 for plotting
                    self.plot_data[ch_idx].push_back(value as f32);

                    // Keep buffer size limited
                    if self.plot_data[ch_idx].len() > self.max_points {
                        self.plot_data[ch_idx].pop_front();
                    }
                }
            }
        }
    }
}

impl eframe::App for AdcGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint continuously for live updates
        ctx.request_repaint();

        // Update plot data from shared buffer
        self.update_plot_data();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading("ADC Live Monitor");

            // Control panel
            ui.horizontal(|ui| {
                if ui.button(if self.is_recording { "⏹ Stop Recording" } else { "⏺ Start Recording" }).clicked() {
                    self.is_recording = !self.is_recording;
                    if self.is_recording {
                        // TODO: Start recording to file
                        println!("Recording started");
                    } else {
                        // TODO: Stop recording
                        println!("Recording stopped");
                    }
                }

                ui.separator();

                ui.label(format!("Buffer size: {} samples", self.max_points));

                if ui.button("Clear").clicked() {
                    for channel in &mut self.plot_data {
                        channel.clear();
                    }
                }
            });

            ui.separator();

            // Status register display
            if let Some(status) = self.latest_status {
                let status_bits = self.decode_status(status);
                
                ui.group(|ui| {
                    ui.label("STATUS Register:");
                    ui.horizontal(|ui| {
                        // Show raw value
                        ui.label(format!("Raw: 0x{:02X}", status));
                        ui.separator();
                        
                        // Show decoded bits
                        if status_bits.lock {
                            ui.colored_label(egui::Color32::RED, "🔒 LOCKED");
                        }
                        if status_bits.f_resync {
                            ui.colored_label(egui::Color32::YELLOW, "⚠️ RESYNC");
                        }
                        if status_bits.reg_map {
                            ui.colored_label(egui::Color32::YELLOW, "⚠️ REG_MAP");
                        }
                        if status_bits.crc_err {
                            ui.colored_label(egui::Color32::RED, "❌ CRC_ERR");
                        }
                        if status_bits.reset {
                            ui.colored_label(egui::Color32::LIGHT_BLUE, "🔄 RESET");
                        }
                        
                        ui.separator();
                        ui.label(format!("CRC: {}", status_bits.crc_type_str()));
                        ui.separator();
                        ui.label(format!("Word Length: {}", status_bits.wlength_str()));
                    });
                });
            }

            ui.separator();

            // Plot
            let plot_height = ui.available_height() - 20.0;

            Plot::new("adc_plot")
                .height(plot_height)
                .legend(egui_plot::Legend::default())
                .show(ui, |plot_ui| {
                    // Plot each channel
                    for (ch_idx, channel_data) in self.plot_data.iter().enumerate() {
                        if !channel_data.is_empty() {
                            let points: PlotPoints = channel_data
                                .iter()
                                .enumerate()
                                .map(|(i, &y)| [i as f64, y as f64])
                                .collect();

                            let line = Line::new(points)
                                .name(format!("Channel {}", ch_idx))
                                .color(channel_color(ch_idx));

                            plot_ui.line(line);
                        }
                    }
                });

            // Status bar
            ui.separator();
            ui.horizontal(|ui| {
                let fps = 1.0 / self.last_update.elapsed().as_secs_f32();
                self.last_update = Instant::now();

                ui.label(format!("FPS: {:.1}", fps));
                ui.separator();

                // Show latest values
                if !self.plot_data[0].is_empty() {
                    ui.label("Latest values:");
                    for (ch_idx, channel) in self.plot_data.iter().enumerate() {
                        if let Some(&value) = channel.back() {
                            ui.colored_label(
                                channel_color(ch_idx),
                                format!("CH{}: {:.3}", ch_idx, value)
                            );
                        }
                    }
                }
            });
        });

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.is_recording = !self.is_recording;
            println!("Recording toggled via keyboard: {}", self.is_recording);
        }
    }
}

fn channel_color(index: usize) -> egui::Color32 {
    match index {
        0 => egui::Color32::from_rgb(255, 0, 0),     // Red
        1 => egui::Color32::from_rgb(0, 255, 0),     // Green
        2 => egui::Color32::from_rgb(0, 0, 255),     // Blue
        3 => egui::Color32::from_rgb(255, 255, 0),   // Yellow
        4 => egui::Color32::from_rgb(255, 0, 255),   // Magenta
        5 => egui::Color32::from_rgb(0, 255, 255),   // Cyan
        6 => egui::Color32::from_rgb(255, 128, 0),   // Orange
        7 => egui::Color32::from_rgb(128, 0, 255),   // Purple
        _ => egui::Color32::GRAY,
    }
}

// Example of how to run the GUI
pub fn run_gui(data_buffer: Arc<Mutex<VecDeque<Frame>>>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("ADC Monitor"),
        ..Default::default()
    };

    eframe::run_native(
        "ADC Monitor",
        options,
        Box::new(|_cc| Ok(Box::new(AdcGui::new(data_buffer)))),
    )
}

fn main() -> io::Result<()> {
    let port_name = "/dev/ttyACM0";
    let state = Arc::new(Mutex::new(VecDeque::new()));

    let state_clone_serial = state.clone();
    std::thread::spawn(move || {
        let state = state_clone_serial;
        loop {
            let port = loop {
                match serialport::new(port_name, 115_200)
                    .timeout(Duration::from_millis(100))
                    .open()
                {
                    Ok(port) => break port,
                    Err(_) => {
                        eprintln!("waiting for {}", port_name);
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            };

            eprintln!("Connected to {}", port_name);

            let reader = BufReader::new(port);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if let Some(data) = line.strip_prefix("[INFO simsamadc]: db64:") {
                            match BASE64.decode(data.trim()) {
                                Ok(decoded) => {
                                    match Frame::read(&mut Cursor::new(&decoded)) {
                                        Ok(frame) => {
                                            let mut state = state.lock().unwrap();
                                            state.push_back(frame);
                                            if state.len() > FRAME_COUNT {
                                                state.pop_front();
                                            }
                                        }
                                        Err(e) => eprintln!("Failed to parse frame: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("Failed to decode base64: {}", e),
                            }
                        } else {
                            eprintln!("{}", line);
                        }
                    }
                    Err(e) => {
                        eprintln!("Serial port error: {:?}", e);
                        break;
                    }
                }
            }
        }
    });

    run_gui(state.clone()).expect("Failed to run GUI");

    Ok(())
}
