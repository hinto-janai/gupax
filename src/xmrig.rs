// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022-2023 hinto-janai
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

use crate::{
	Regexes,
	constants::*,
	disk::*,
	Process,
	PubXmrigApi,
	macros::*,
};
use egui::{
	TextEdit,SelectableLabel,ComboBox,Label,Button,RichText,Slider,Checkbox,
	TextStyle::*,
};
use std::{
	sync::{Arc,Mutex},
};
use regex::Regex;
use log::*;

impl crate::disk::Xmrig {
	#[inline(always)]
	pub fn show(&mut self, pool_vec: &mut Vec<(String, Pool)>, regex: &Regexes, process: &Arc<Mutex<Process>>, api: &Arc<Mutex<PubXmrigApi>>, buffer: &mut String, width: f32, height: f32, _ctx: &egui::Context, ui: &mut egui::Ui) {
	let text_edit = height / 25.0;
	//---------------------------------------------------------------------------------------------------- [Simple] Console
	debug!("XMRig Tab | Rendering [Console]");
	ui.group(|ui| {
	if self.simple {
		let height = height / 1.5;
		let width = width - SPACE;
		ui.style_mut().override_text_style = Some(Monospace);
		egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
			ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
			egui::ScrollArea::vertical().stick_to_bottom(true).max_width(width).max_height(height).auto_shrink([false; 2]).show_viewport(ui, |ui, _| {
				ui.add_sized([width, height], TextEdit::multiline(&mut lock!(api).output.as_str()));
			});
		});
	//---------------------------------------------------------------------------------------------------- [Advanced] Console
	} else {
		let height = height / 2.8;
		let width = width - SPACE;
		ui.style_mut().override_text_style = Some(Monospace);
		egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
			ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
			egui::ScrollArea::vertical().stick_to_bottom(true).max_width(width).max_height(height).auto_shrink([false; 2]).show_viewport(ui, |ui, _| {
				ui.add_sized([width, height], TextEdit::multiline(&mut lock!(api).output.as_str()));
			});
		});
		ui.separator();
		let response = ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(buffer), r#"Commands: [h]ashrate, [p]ause, [r]esume, re[s]ults, [c]onnection"#)).on_hover_text(XMRIG_INPUT);
		// If the user pressed enter, dump buffer contents into the process STDIN
		if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
			response.request_focus();                  // Get focus back
			let buffer = std::mem::take(buffer);       // Take buffer
			let mut process = lock!(process); // Lock
			if process.is_alive() { process.input.push(buffer); } // Push only if alive
		}
	}
	});

	//---------------------------------------------------------------------------------------------------- Arguments
	if !self.simple {
		debug!("XMRig Tab | Rendering [Arguments]");
		ui.group(|ui| { ui.horizontal(|ui| {
			let width = (width/10.0) - SPACE;
			ui.style_mut().override_text_style = Some(Monospace);
			ui.add_sized([width, text_edit], Label::new("Command arguments:"));
			ui.add_sized([ui.available_width(), text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.arguments), r#"--url <...> --user <...> --config <...>"#)).on_hover_text(XMRIG_ARGUMENTS);
			self.arguments.truncate(1024);
		})});
		ui.set_enabled(self.arguments.is_empty());
	//---------------------------------------------------------------------------------------------------- Address
		debug!("XMRig Tab | Rendering [Address]");
		ui.group(|ui| {
			let width = width - SPACE;
			ui.spacing_mut().text_edit_width = (width)-(SPACE*3.0);
			ui.style_mut().override_text_style = Some(Monospace);
			let text;
			let color;
			let len = format!("{:02}", self.address.len());
			if self.address.is_empty() {
				text = format!("Monero Address [{}/95] ➖", len);
				color = LIGHT_GRAY;
			} else if self.address.len() == 95 && Regex::is_match(&regex.address, &self.address) && ! self.address.contains('0') && ! self.address.contains('O') && ! self.address.contains('l') {
				text = format!("Monero Address [{}/95] ✔", len);
				color = GREEN;
			} else {
				text = format!("Monero Address [{}/95] ❌", len);
				color = RED;
			}
			ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
			ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.address), "4...")).on_hover_text(XMRIG_ADDRESS);
			self.address.truncate(95);
		});
	}

	//---------------------------------------------------------------------------------------------------- Threads
	if self.simple { ui.add_space(SPACE); }
	debug!("XMRig Tab | Rendering [Threads]");
	ui.vertical(|ui| {
		let width = width/10.0;
		ui.spacing_mut().icon_width = width / 25.0;
		ui.horizontal(|ui| {
			ui.spacing_mut().slider_width = width*8.35;
			ui.add_sized([width, text_edit], Label::new(format!("Threads [1-{}]:", self.max_threads)));
			ui.add_sized([width, text_edit], Slider::new(&mut self.current_threads, 1..=self.max_threads)).on_hover_text(XMRIG_THREADS);
		});
		#[cfg(not(target_os = "linux"))] // Pause on active isn't supported on Linux
		ui.horizontal(|ui| {
			ui.spacing_mut().slider_width = width*7.7;
			ui.add_sized([width, text_edit], Label::new(format!("Pause on active [0-255]:")));
			ui.add_sized([width, text_edit], Slider::new(&mut self.pause, 0..=255)).on_hover_text(format!("{} [{}] seconds.", XMRIG_PAUSE, self.pause));
		});
	});

	//---------------------------------------------------------------------------------------------------- Simple
	if self.simple {
//		ui.group(|ui|

//		});
	} else {
		debug!("XMRig Tab | Rendering [Pool List] elements");
		let width = ui.available_width() - 10.0;
		let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
		// [Pool IP/Port]
		ui.horizontal(|ui| {
		ui.group(|ui| {
			let width = width/10.0;
			ui.vertical(|ui| {
			ui.style_mut().override_text_style = Some(Monospace);
			ui.spacing_mut().text_edit_width = width*3.32;
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.name.len());
				if self.name.is_empty() {
					text = format!("Name [ {}/30 ]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if Regex::is_match(&regex.name, &self.name) {
					text = format!("Name [ {}/30 ]✔", len);
					color = GREEN;
				} else {
					text = format!("Name [ {}/30 ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.name).on_hover_text(XMRIG_NAME);
				self.name.truncate(30);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:03}", self.ip.len());
				if self.ip.is_empty() {
					text = format!("  IP [{}/255]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if self.ip == "localhost" || Regex::is_match(&regex.ipv4, &self.ip) || Regex::is_match(&regex.domain, &self.ip) {
					text = format!("  IP [{}/255]✔", len);
					color = GREEN;
				} else {
					text = format!("  IP [{}/255]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.ip).on_hover_text(XMRIG_IP);
				self.ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.port.len();
				if self.port.is_empty() {
					text = format!("Port [  {}/5  ]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if Regex::is_match(&regex.port, &self.port) {
					text = format!("Port [  {}/5  ]✔", len);
					color = GREEN;
				} else {
					text = format!("Port [  {}/5  ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.port).on_hover_text(XMRIG_PORT);
				self.port.truncate(5);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.rig.len());
				if self.rig.is_empty() {
					text = format!(" Rig [ {}/30 ]➖", len);
					color = LIGHT_GRAY;
				} else if Regex::is_match(&regex.name, &self.rig) {
					text = format!(" Rig [ {}/30 ]✔", len);
					color = GREEN;
				} else {
					text = format!(" Rig [ {}/30 ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.rig).on_hover_text(XMRIG_RIG);
				self.rig.truncate(30);
			});
		});

		ui.vertical(|ui| {
			let width = ui.available_width();
			ui.add_space(1.0);
			// [Manual node selection]
			ui.spacing_mut().slider_width = width - 8.0;
			ui.spacing_mut().icon_width = width / 25.0;
			// [Node List]
			debug!("XMRig Tab | Rendering [Node List] ComboBox");
			let text = RichText::new(format!("{}. {}", self.selected_index+1, self.selected_name));
			ComboBox::from_id_source("manual_pool").selected_text(RichText::text_style(text, Monospace)).show_ui(ui, |ui| {
				let mut n = 0;
				for (name, pool) in pool_vec.iter() {
					let text = RichText::text_style(RichText::new(format!("{}. {}\n     IP: {}\n   Port: {}\n    Rig: {}", n+1, name, pool.ip, pool.port, pool.rig)), Monospace);
					if ui.add(SelectableLabel::new(self.selected_name == *name, text)).clicked() {
						self.selected_index = n;
						let pool = pool.clone();
						self.selected_name = name.clone();
						self.selected_rig = pool.rig.clone();
						self.selected_ip = pool.ip.clone();
						self.selected_port = pool.port.clone();
						self.name = name.clone();
						self.rig = pool.rig;
						self.ip = pool.ip;
						self.port = pool.port;
					}
					n += 1;
				}
			});
			// [Add/Save]
			let pool_vec_len = pool_vec.len();
			let mut exists = false;
			let mut save_diff = true;
			let mut existing_index = 0;
			for (name, pool) in pool_vec.iter() {
				if *name == self.name {
					exists = true;
					if self.rig == pool.rig && self.ip == pool.ip && self.port == pool.port {
						save_diff = false;
					}
					break
				}
				existing_index += 1;
			}
			ui.horizontal(|ui| {
				let text = if exists { LIST_SAVE } else { LIST_ADD };
				let text = format!("{}\n    Currently selected pool: {}. {}\n    Current amount of pools: {}/1000", text, self.selected_index+1, self.selected_name, pool_vec_len);
				// If the pool already exists, show [Save] and mutate the already existing pool
				if exists {
					ui.set_enabled(!incorrect_input && save_diff);
					if ui.add_sized([width, text_edit], Button::new("Save")).on_hover_text(text).clicked() {
						let pool = Pool {
							rig: self.rig.clone(),
							ip: self.ip.clone(),
							port: self.port.clone(),
						};
						pool_vec[existing_index].1 = pool;
						self.selected_name = self.name.clone();
						self.selected_rig = self.rig.clone();
						self.selected_ip = self.ip.clone();
						self.selected_port = self.port.clone();
						info!("Node | S | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig: \"{}\"]", existing_index+1, self.name, self.ip, self.port, self.rig);
					}
				// Else, add to the list
				} else {
					ui.set_enabled(!incorrect_input && pool_vec_len < 1000);
					if ui.add_sized([width, text_edit], Button::new("Add")).on_hover_text(text).clicked() {
						let pool = Pool {
							rig: self.rig.clone(),
							ip: self.ip.clone(),
							port: self.port.clone(),
						};
						pool_vec.push((self.name.clone(), pool));
						self.selected_index = pool_vec_len;
						self.selected_name = self.name.clone();
						self.selected_rig = self.rig.clone();
						self.selected_ip = self.ip.clone();
						self.selected_port = self.port.clone();
						info!("Node | A | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig: \"{}\"]", pool_vec_len, self.name, self.ip, self.port, self.rig);
					}
				}
			});
			// [Delete]
			ui.horizontal(|ui| {
				ui.set_enabled(pool_vec_len > 1);
				let text = format!("{}\n    Currently selected pool: {}. {}\n    Current amount of pools: {}/1000", LIST_DELETE, self.selected_index+1, self.selected_name, pool_vec_len);
				if ui.add_sized([width, text_edit], Button::new("Delete")).on_hover_text(text).clicked() {
					let new_name;
					let new_pool;
					match self.selected_index {
						0 => {
							new_name = pool_vec[1].0.clone();
							new_pool = pool_vec[1].1.clone();
							pool_vec.remove(0);
						}
						_ => {
							pool_vec.remove(self.selected_index);
							self.selected_index -= 1;
							new_name = pool_vec[self.selected_index].0.clone();
							new_pool = pool_vec[self.selected_index].1.clone();
						}
					};
					self.selected_name = new_name.clone();
					self.selected_rig = new_pool.rig.clone();
					self.selected_ip = new_pool.ip.clone();
					self.selected_port = new_pool.port.clone();
					self.name = new_name;
					self.rig = new_pool.rig;
					self.ip = new_pool.ip;
					self.port = new_pool.port;
					info!("Node | D | [index: {}, name: \"{}\", ip: \"{}\", port: {}, rig\"{}\"]", self.selected_index, self.selected_name, self.selected_ip, self.selected_port, self.selected_rig);
				}
			});
			ui.horizontal(|ui| {
				ui.set_enabled(!self.name.is_empty() || !self.ip.is_empty() || !self.port.is_empty());
				if ui.add_sized([width, text_edit], Button::new("Clear")).on_hover_text(LIST_CLEAR).clicked() {
					self.name.clear();
					self.rig.clear();
					self.ip.clear();
					self.port.clear();
				}
			});
		});
		});
		});
		ui.add_space(5.0);

		debug!("XMRig Tab | Rendering [API] TextEdits");
		// [HTTP API IP/Port]
		ui.group(|ui| { ui.horizontal(|ui| {
		ui.vertical(|ui| {
			let width = width/10.0;
			ui.style_mut().override_text_style = Some(Monospace);
			ui.spacing_mut().text_edit_width = width*2.39;
			// HTTP API
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:03}", self.api_ip.len());
				if self.api_ip.is_empty() {
					text = format!("HTTP API IP   [{}/255]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if self.api_ip == "localhost" || Regex::is_match(&regex.ipv4, &self.api_ip) || Regex::is_match(&regex.domain, &self.api_ip) {
					text = format!("HTTP API IP   [{}/255]✔", len);
					color = GREEN;
				} else {
					text = format!("HTTP API IP   [{}/255]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.api_ip).on_hover_text(XMRIG_API_IP);
				self.api_ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.api_port.len();
				if self.api_port.is_empty() {
					text = format!("HTTP API Port [  {}/5  ]➖", len);
					color = LIGHT_GRAY;
					incorrect_input = true;
				} else if Regex::is_match(&regex.port, &self.api_port) {
					text = format!("HTTP API Port [  {}/5  ]✔", len);
					color = GREEN;
				} else {
					text = format!("HTTP API Port [  {}/5  ]❌", len);
					color = RED;
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.api_port).on_hover_text(XMRIG_API_PORT);
				self.api_port.truncate(5);
			});
		});

		ui.separator();

		debug!("XMRig Tab | Rendering [TLS/Keepalive] buttons");
		ui.vertical(|ui| {
			// TLS/Keepalive
			ui.horizontal(|ui| {
				let width = (ui.available_width()/2.0)-11.0;
				let height = text_edit*2.0;
//				let mut style = (*ctx.style()).clone();
//				style.spacing.icon_width_inner = width / 8.0;
//				style.spacing.icon_width = width / 6.0;
//				style.spacing.icon_spacing = 20.0;
//				ctx.set_style(style);
				ui.add_sized([width, height], Checkbox::new(&mut self.tls, "TLS Connection")).on_hover_text(XMRIG_TLS);
				ui.separator();
				ui.add_sized([width, height], Checkbox::new(&mut self.keepalive, "Keepalive")).on_hover_text(XMRIG_KEEPALIVE);
			});
		});
		});
		});
	}
	}
}
