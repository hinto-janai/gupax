// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022 hinto-janaiyo
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use eframe::{egui,NativeOptions};
use egui::{Vec2,Pos2};
use std::process::exit;
use std::thread;
use egui::color::Color32;
use egui::Stroke;
use egui::FontId;
use egui::FontFamily::Proportional;
use egui::TextStyle::{Body,Button,Heading,Monospace,Name,Small};
use egui::RichText;
use egui::Label;
use regex::Regex;
use egui_extras::RetainedImage;
use log::*;
use env_logger::Builder;
use env_logger::WriteStyle;
use std::io::Write;
use std::time::Instant;

mod constants;
mod toml;
mod about;
mod status;
mod gupax;
mod p2pool;
mod xmrig;
use {constants::*,crate::toml::*,about::*,status::*,gupax::*,p2pool::*,xmrig::*};

// The state of the outer [App].
// See the [State] struct for the
// actual inner state of the settings.
pub struct App {
	version: String,
	name_version: String,
	tab: Tab,
	changed: bool,
	os: &'static str,
	current_threads: u16,
	max_threads: u16,
	resolution: Vec2,
	banner: RetainedImage,
	p2pool: bool,
	xmrig: bool,
	state: State,
	og: State,
	allowed_to_close: bool,
	show_confirmation_dialog: bool,
}

impl App {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let version = String::from("v0.0.1");
		let name_version = String::from("Gupax v0.0.1");
		let tab = Tab::default();
		let max_threads = num_cpus::get().try_into().unwrap();
		let current_threads: u16;
		let changed = false;
		let os = OS;
		if max_threads != 1 {
			current_threads = max_threads / 2
		} else {
			current_threads = 1
		}
		let resolution = cc.integration_info.window_info.size;
		init_text_styles(&cc.egui_ctx, resolution[0] as f32);
		let banner = match RetainedImage::from_image_bytes("banner.png", BYTES_BANNER) {
			Ok(banner) => { info!("Banner loading ... OK"); banner },
			Err(err) => { error!("{}", err); panic!("{}", err); },
		};
		let mut state = State::new();
		let mut og = State::new();
		info!("Frame resolution ... {:#?}", resolution);
		Self {
			version,
			name_version,
			tab,
			current_threads,
			max_threads,
			changed,
			resolution,
			os,
			banner,
			p2pool: false,
			xmrig: false,
			state,
			og,
			allowed_to_close: false,
			show_confirmation_dialog: false,
		}
	}
}

// Inner state holding all
// mutable tab structs.
#[derive(Clone, Debug, Eq, PartialEq)]
struct State {
	gupax: Gupax,
	p2pool: P2pool,
	xmrig: Xmrig,
}

impl State {
	fn new() -> Self {
		Self {
			gupax: Gupax::new(),
			p2pool: P2pool::new(),
			xmrig: Xmrig::new(),
		}
	}

	fn save(new: State) -> Self {
		Self {
			gupax: new.gupax,
			p2pool: new.p2pool,
			xmrig: new.xmrig,
		}
	}
}

// The tabs inside [App].
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tab {
	About,
	Status,
	Gupax,
	P2pool,
	Xmrig,
}
impl Default for Tab {
    fn default() -> Self {
        Self::About
    }
}

fn init_text_styles(ctx: &egui::Context, width: f32) {
	let scale = width / 26.666;
	let mut style = (*ctx.style()).clone();
	style.text_styles = [
//		(Small, FontId::new(10.0, Proportional)),
//		(Body, FontId::new(25.0, Proportional)),
//		(Button, FontId::new(25.0, Proportional)),
//		(Monospace, FontId::new(25.0, Proportional)),
//		(Heading, FontId::new(30.0, Proportional)),
//		(Name("Tab".into()), FontId::new(50.0, Proportional)),
//		(Name("Bottom".into()), FontId::new(25.0, Proportional)),
		(Small, FontId::new(scale/3.0, Proportional)),
		(Body, FontId::new(scale/2.0, Proportional)),
		(Button, FontId::new(scale/2.0, Proportional)),
		(Monospace, FontId::new(scale/2.0, Proportional)),
		(Heading, FontId::new(scale/1.5, Proportional)),
		(Name("Tab".into()), FontId::new(scale*1.2, Proportional)),
		(Name("Bottom".into()), FontId::new(scale/2.0, Proportional)),
	].into();
//	style.spacing.slider_width = scale;
//	style.spacing.text_edit_width = scale;
//	style.spacing.button_padding = Vec2::new(scale/2.0, scale/2.0);
	ctx.set_style(style);
	ctx.set_pixels_per_point(1.0);
	ctx.request_repaint();
}

fn init_logger() {
	use env_logger::fmt::Color;
	Builder::new().format(|buf, record| {
		let level;
		let mut style = buf.style();
		match record.level() {
			Level::Error => { style.set_color(Color::Red); level = "ERROR" },
			Level::Warn => { style.set_color(Color::Yellow); level = "WARN" },
			Level::Info => { style.set_color(Color::White); level = "INFO" },
			Level::Debug => { style.set_color(Color::Blue); level = "DEBUG" },
			Level::Trace => { style.set_color(Color::Magenta); level = "TRACE" },
		};
		writeln!(
			buf,
			"[{}] [{}] [{}:{}] {}",
			style.set_bold(true).value(level),
			buf.style().set_dimmed(true).value(chrono::Local::now().format("%F %T%.3f")),
			buf.style().set_dimmed(true).value(record.file().unwrap_or("???")),
			buf.style().set_dimmed(true).value(record.line().unwrap_or(0)),
			record.args(),
		)
	}).filter_level(LevelFilter::Info).write_style(WriteStyle::Always).parse_default_env().format_timestamp_millis().init();
	info!("init_logger() ... OK");
}

fn init_options() -> NativeOptions {
	let mut options = eframe::NativeOptions::default();
	options.min_window_size = Option::from(Vec2::new(1280.0, 720.0));
	options.max_window_size = Option::from(Vec2::new(3180.0, 2160.0));
	options.initial_window_size = Option::from(Vec2::new(1280.0, 720.0));
	options.follow_system_theme = false;
	options.default_theme = eframe::Theme::Dark;
	let icon = image::load_from_memory(BYTES_ICON).expect("Failed to read icon bytes").to_rgba8();
	let (icon_width, icon_height) = icon.dimensions();
	options.icon_data = Some(eframe::IconData {
		rgba: icon.into_raw(),
		width: icon_width,
		height: icon_height,
	});
	info!("init_options() ... OK");
	options
}

fn main() {
	init_logger();
	let options = init_options();
	let toml = Toml::get();
	info!("Printing gupax.toml...");
	eprintln!("{:#?}", toml);
	let now = Instant::now();
	eframe::run_native(
		"Gupax",
		options,
		Box::new(|cc| Box::new(App::new(cc))),
	);
}

impl eframe::App for App {
	fn on_close_event(&mut self) -> bool {
		self.show_confirmation_dialog = true;
		self.allowed_to_close
	}
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		if self.show_confirmation_dialog {
			// Show confirmation dialog:
			egui::CentralPanel::default().show(ctx, |ui| {
				let width = ui.available_width();
				let width = width - 10.0;
				let height = ui.available_height();
				init_text_styles(ctx, width);
				ui.add_sized([width, height/2.0], Label::new("Are you sure you want to quit?"));
				ui.group(|ui| {
					if ui.add_sized([width, height/10.0], egui::Button::new("Yes")).clicked() {
						exit(0);
					} else if ui.add_sized([width, height/10.0], egui::Button::new("No")).clicked() {
						self.show_confirmation_dialog = false;
					}
				});
			});
			return
		}
		// Top: Tabs
		egui::CentralPanel::default().show(ctx, |ui| {
			init_text_styles(ctx, ui.available_width());
			let width = (ui.available_width() - 90.0) / 5.0;
			let height = ui.available_height() / 10.0;
		    ui.add_space(4.0);
			ui.style_mut().override_text_style = Some(Name("Tab".into()));
			ui.horizontal(|ui| {
				ui.style_mut().visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(100, 100, 100);
				ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(255, 120, 120);
				ui.style_mut().visuals.selection.stroke = Stroke {
					width: 5.0,
					color: Color32::from_rgb(255, 255, 255),
				};
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::About, "About")).clicked() { self.tab = Tab::About; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::Status, "Status")).clicked() { self.tab = Tab::Status; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::Gupax, "Gupax")).clicked() { self.tab = Tab::Gupax; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::P2pool, "P2Pool")).clicked() { self.tab = Tab::P2pool; }
				ui.separator();
				if ui.add_sized([width, height], egui::SelectableLabel::new(self.tab == Tab::Xmrig, "XMRig")).clicked() { self.tab = Tab::Xmrig; }
			});
			ui.add_space(3.0);
			ui.separator();
//		});


		let height = height / 2.0;
		// Bottom: app info + state/process buttons
		egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
			ui.style_mut().override_text_style = Some(Name("Bottom".into()));
			ui.horizontal(|ui| {
				ui.group(|ui| {
					let width = width / 2.0;
					ui.add_sized([width, height], Label::new(&self.name_version));
					ui.separator();
					ui.add_sized([width, height], Label::new(self.os));
					ui.separator();
					ui.add_sized([width/1.5, height], Label::new("P2Pool"));
					if self.p2pool == true {
						ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(100, 230, 100))));
					} else {
						ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(230, 50, 50))));
					}
					ui.separator();
					ui.add_sized([width/1.5, height], Label::new("XMRig"));
					if self.xmrig == true {
							ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(100, 230, 100))));
					} else {
							ui.add_sized([width/4.0, height], Label::new(RichText::new("⏺").color(Color32::from_rgb(230, 50, 50))));
					}
				});

				ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
					ui.group(|ui| {
						if self.state == self.og {
							ui.set_enabled(false)
						}
						let width = width / 2.0;
						if ui.add_sized([width, height], egui::Button::new("Save")).on_hover_text("Save changes").clicked() { self.og = self.state.clone(); }
						if ui.add_sized([width, height], egui::Button::new("Reset")).on_hover_text("Reset changes").clicked() { self.state = self.og.clone(); }
					});

					let width = (ui.available_width() / 3.0) - 6.2;
					match self.tab {
						Tab::P2pool => {
							ui.group(|ui| {
								if self.p2pool == true {
									if ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart P2Pool").clicked() { self.p2pool = false; }
									if ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop P2Pool").clicked() { self.p2pool = false; }
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start P2Pool");
									});
								} else {
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart P2Pool");
										ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop P2Pool");
									});
									if ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start P2Pool").clicked() { self.p2pool = true; }
								}
							});
						}
						Tab::Xmrig => {
							ui.group(|ui| {
								if self.xmrig == true {
									if ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart XMRig").clicked() { self.xmrig = false; }
									if ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop XMRig").clicked() { self.xmrig = false; }
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start XMRig");
									});
								} else {
									ui.add_enabled_ui(false, |ui| {
										ui.add_sized([width, height], egui::Button::new("⟲")).on_hover_text("Restart XMRig");
										ui.add_sized([width, height], egui::Button::new("⏹")).on_hover_text("Stop XMRig");
									});
									if ui.add_sized([width, height], egui::Button::new("⏺")).on_hover_text("Start XMRig").clicked() { self.xmrig = true; }
								}
							});
						}
						_ => (),
					}
				});
			});
		});

		// Central: tab contents
		// Docs say to add central last, don't think it matters here but whatever:
		// [https://docs.rs/egui/latest/egui/containers/panel/struct.TopBottomPanel.html]
//        egui::TopBottomPanel::bottom("2").show(ctx, |ui| {
			ui.style_mut().override_text_style = Some(egui::TextStyle::Body);
	        match self.tab {
	            Tab::About => {
					About::show(self, ctx, ui);
	            }
	            Tab::Status => {
					Status::show(self, ctx, ui);
	            }
	            Tab::Gupax => {
					Gupax::show(&mut self.state.gupax, ctx, ui);
	            }
	            Tab::P2pool => {
					P2pool::show(&mut self.state.p2pool, ctx, ui);
	            }
	            Tab::Xmrig => {
					Xmrig::show(&mut self.state.xmrig, ctx, ui);
	            }
	        }
//        });

		});
	}
}
