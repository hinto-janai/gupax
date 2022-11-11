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

use crate::App;
use crate::constants::*;
use crate::state::*;
use crate::node::*;
use crate::node::NodeEnum::*;
use std::sync::{Arc,Mutex};
use std::thread;
use log::*;
use egui::{TextEdit,SelectableLabel,ComboBox,Label};
use egui::TextStyle::*;
use egui::FontFamily::Proportional;
use egui::{FontId,Button,Color32,RichText};
use regex::Regex;

impl P2pool {
	pub fn show(&mut self, og: &Arc<Mutex<State>>, online: bool, ping: &Arc<Mutex<Ping>>, addr_regex: &Regex, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
	let text_edit = height / 20.0;
	// Console
	ui.group(|ui| {
		let height = height / SPACE;
		let width = width - SPACE;
		ui.add_sized([width, height*3.0], TextEdit::multiline(&mut "".to_string()));
		ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(&mut "".to_string()), r#"Type a command (e.g "help" or "status") and press Enter"#));
	});

	let height = ui.available_height();
	// [Simple]
	if self.simple {
		// [Node]
		let height = height / 6.0;
		ui.spacing_mut().slider_width = width - 8.0;
		ui.spacing_mut().icon_width = width / 25.0;
		ui.vertical(|ui| {
		ui.horizontal(|ui| {
			// [Ping List]
			let id = og.lock().unwrap().p2pool.node;
			let ip = enum_to_ip(id);
			let mut ms = 0;
			let mut color = Color32::LIGHT_GRAY;
			for data in ping.lock().unwrap().nodes.iter() {
				if data.id == id {
					ms = data.ms;
					color = data.color;
					break
				}
			}
			let text = RichText::new(format!("⏺ {}ms | {} | {}", ms, id, ip)).color(color);
			ComboBox::from_id_source("nodes").selected_text(RichText::text_style(text, Monospace)).show_ui(ui, |ui| {
				for data in ping.lock().unwrap().nodes.iter() {
					let ms = crate::node::format_ms(data.ms);
					let id = crate::node::format_enum(data.id);
					let text = RichText::text_style(RichText::new(format!("⏺ {} | {} | {}", ms, id, data.ip)).color(data.color), Monospace);
					ui.selectable_value(&mut og.lock().unwrap().p2pool.node, data.id, text);
				}
			});
		});

		ui.add_space(5.0);

		ui.horizontal(|ui| {
		let width = (width/2.0)-4.0;
		// [Select fastest node]
		if ui.add_sized([width, height], Button::new("Select fastest node")).on_hover_text(P2POOL_SELECT_FASTEST).clicked() {
			let pinged = ping.lock().unwrap().pinged;
			let fastest = ping.lock().unwrap().fastest;
			if pinged && og.lock().unwrap().p2pool.node != fastest {
				og.lock().unwrap().p2pool.node = ping.lock().unwrap().fastest;
				og.lock().unwrap().save();
			}
		}
		// [Ping Button]
		ui.set_enabled(!ping.lock().unwrap().pinging);
		if ui.add_sized([width, height], Button::new("Ping community nodes")).on_hover_text(P2POOL_PING).clicked() {
			let ping = Arc::clone(&ping);
			let og_clone = Arc::clone(og);
			ping.lock().unwrap().pinging = true;
			thread::spawn(move|| {
				info!("Spawning ping thread...");
				let ping_result = crate::node::ping(ping.clone());
				ping.lock().unwrap().nodes = ping_result.nodes;
				ping.lock().unwrap().fastest = ping_result.fastest;
				if og_clone.lock().unwrap().p2pool.auto_select {
					og_clone.lock().unwrap().p2pool.node = ping_result.fastest;
					og_clone.lock().unwrap().save();
				}
			});
		}});

		ui.vertical(|ui| {
			let height = height / 2.0;
			let pinging = ping.lock().unwrap().pinging;
			ui.set_enabled(pinging);
			let prog = ping.lock().unwrap().prog.round();
			let msg = RichText::text_style(RichText::new(format!("{} ... {}%", ping.lock().unwrap().msg, prog)), Monospace);
			let height = height / 1.25;
			ui.add_space(5.0);
			ui.add_sized([width, height], Label::new(msg));
			ui.add_space(5.0);
			if pinging {
				ui.add_sized([width, height], egui::Spinner::new().size(height));
			} else {
				ui.add_sized([width, height], egui::Label::new("..."));
			}
			ui.add_sized([width, height], egui::ProgressBar::new(prog.round()/100.0));
			ui.add_space(5.0);
		});
		});

		ui.group(|ui| {
		ui.horizontal(|ui| {
			let width = (width/2.0)-(SPACE*1.5);
			// [Auto-node] + [Auto-select]
			let mut style = (*ctx.style()).clone();
			style.spacing.icon_width_inner = height/1.5;
			style.spacing.icon_width = height;
			style.spacing.icon_spacing = 20.0;
			ctx.set_style(style);
			ui.add_sized([width, height], egui::Checkbox::new(&mut self.auto_select, "Auto-select")).on_hover_text(P2POOL_AUTO_SELECT);
			ui.separator();
			ui.add_sized([width, height], egui::Checkbox::new(&mut self.auto_node, "Auto-node")).on_hover_text(P2POOL_AUTO_NODE);
		})});

		// [Address]
		let height = ui.available_height();
		ui.horizontal(|ui| {
			let width = width / 100.0;
			ui.add_sized([width*6.0, height], Label::new("Address"));
			if self.address.is_empty() {
				ui.add_sized([width, height], Label::new(RichText::new("➖").color(Color32::LIGHT_GRAY)));
			} else if self.address.len() == 95 && Regex::is_match(addr_regex, &self.address) {
				ui.add_sized([width, height], Label::new(RichText::new("✔").color(Color32::from_rgb(100, 230, 100))));
			} else {
				ui.add_sized([width, height], Label::new(RichText::new("❌").color(Color32::from_rgb(230, 50, 50))));
			}
			ui.spacing_mut().text_edit_width = (width*9.0)-(SPACE*2.5);
			ui.style_mut().override_text_style = Some(Monospace);
			ui.add_sized([ui.available_width(), text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.address), "4...")).on_hover_text(P2POOL_ADDRESS);
		});
//		ui.horizontal(|ui| {
//			ui.add_sized([width, height/2.0], Label::new("Address:"));
//			ui.add_sized([width, height], TextEdit::multiline(&mut self.address));
//		})});
	// [Advanced]
	} else {
		// TODO:
		// ping code
		// If ping was pressed, start thread
//		if self.ping {
//			self.ping = false;
//			self.pinging = Arc::new(Mutex::new(true));
//			let node_clone = Arc::clone(&self.node);
//			let pinging_clone = Arc::clone(&self.pinging);
//			thread::spawn(move|| {
//				let result = NodeStruct::ping();
//				*node_clone.lock().unwrap() = result.nodes;
//				*pinging_clone.lock().unwrap() = false;
//			});
//		}

		// If ping-ING, display stats
//		if *self.pinging.lock().unwrap() {
//			egui::CentralPanel::default().show(ctx, |ui| {
//				let width = ui.available_width();
//				let width = width - 10.0;
//				let height = ui.available_height();
//				init_text_styles(ctx, width);
//				ui.add_sized([width, height/2.0], Label::new(format!("In progress: {}", *self.pinging.lock().unwrap())));
//				ui.group(|ui| {
//					if ui.add_sized([width, height/10.0], egui::Button::new("Yes")).clicked() {
//						info!("Quit confirmation = yes ... goodbye!");
//						exit(0);
//					} else if ui.add_sized([width, height/10.0], egui::Button::new("No")).clicked() {
//						info!("Quit confirmation = no ... returning!");
//						self.show_confirmation_dialog = false;
//					}
//				});
//			});
//			return
//		}

		let width = width - 30.0;
		let mut style = (*ctx.style()).clone();
		let height = ui.available_height()/1.2;
		ui.horizontal(|ui| {
			ui.group(|ui| { ui.vertical(|ui| {
				ui.group(|ui| { ui.horizontal(|ui| {
					if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(self.mini == false, "P2Pool Main")).on_hover_text(P2POOL_MAIN).clicked() { self.mini = false; };
					if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(self.mini == true, "P2Pool Mini")).on_hover_text(P2POOL_MINI).clicked() { self.mini = true; };
				})});

				let width = width/4.0;
				style.spacing.slider_width = width*1.25;
				ctx.set_style(style);
				ui.horizontal(|ui| {
					ui.add_sized([width/8.0, height/5.0], egui::Label::new("Out peers [10-450]:"));
					ui.add_sized([width, height/5.0], egui::Slider::new(&mut self.out_peers, 10..=450)).on_hover_text(P2POOL_OUT);
				});

				ui.horizontal(|ui| {
					ui.add_sized([width/8.0, height/5.0], egui::Label::new("    In peers [10-450]:"));
					ui.add_sized([width, height/5.0], egui::Slider::new(&mut self.in_peers, 10..=450)).on_hover_text(P2POOL_IN);
				});

				ui.horizontal(|ui| {
					ui.add_sized([width/8.0, height/5.0], egui::Label::new("          Log level [0-6]:"));
					ui.add_sized([width, height/5.0], egui::Slider::new(&mut self.log_level, 0..=6)).on_hover_text(P2POOL_LOG);
				});
			})});

		ui.group(|ui| { ui.vertical(|ui| {
			ui.group(|ui| { ui.horizontal(|ui| {
				if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(self.simple == false, "Community Monero Node")).on_hover_text(P2POOL_COMMUNITY).clicked() { self.simple = false; };
				if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(self.simple == true, "Manual Monero Node")).on_hover_text(P2POOL_MANUAL).clicked() { self.simple = true; };
			})});
			ui.add_space(8.0);
			ui.horizontal(|ui| {
//			ui.add_sized([width/8.0, height/5.0],
			egui::ComboBox::from_label(self.node.to_string()).selected_text(RINO).show_ui(ui, |ui| {
					ui.selectable_value(&mut self.node, NodeEnum::Rino, RINO);
					ui.selectable_value(&mut self.node, NodeEnum::Seth, SETH);
					ui.selectable_value(&mut self.node, NodeEnum::Selsta1, SELSTA_1);
			});
//			);
			});

			if self.simple == false { ui.set_enabled(false); }
			let width = (width/4.0);
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/7.8], egui::Label::new("Monero Node IP:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut self.monerod);
			});
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/7.8], egui::Label::new("Monero Node RPC Port:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut self.rpc.to_string());
			});
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/7.8], egui::Label::new("Monero Node ZMQ Port:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut self.zmq.to_string());
			});

		})});

		});
		ui.group(|ui| {
		ui.horizontal(|ui| {
			ui.spacing_mut().text_edit_width = ui.available_width();
			ui.label("Address:");
			ui.text_edit_singleline(&mut self.address);
		})});
	}
	}
}
