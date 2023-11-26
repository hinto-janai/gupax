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
	node::*,
	helper::*,
	macros::*,
};
use egui::{
	TextEdit,SelectableLabel,ComboBox,Label,Button,
	Color32,RichText,Slider,Checkbox,ProgressBar,Spinner,
	TextStyle::*,Hyperlink
};
use std::sync::{Arc,Mutex};
use regex::Regex;
use log::*;
use crate::regex::{
	REGEXES,
};

impl crate::disk::P2pool {
	#[inline(always)]
	pub fn show(&mut self, node_vec: &mut Vec<(String, Node)>, _og: &Arc<Mutex<State>>, ping: &Arc<Mutex<Ping>>, process: &Arc<Mutex<Process>>, api: &Arc<Mutex<PubP2poolApi>>, buffer: &mut String, width: f32, height: f32, _ctx: &egui::Context, ui: &mut egui::Ui) {
	let text_edit = height / 25.0;
	//---------------------------------------------------------------------------------------------------- [Simple] Console
	debug!("P2Pool Tab | Rendering [Console]");
	ui.group(|ui| {
	if self.simple {
		let height = height / 2.8;
		let width = width - SPACE;
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
		egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
			ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
			egui::ScrollArea::vertical().stick_to_bottom(true).max_width(width).max_height(height).auto_shrink([false; 2]).show_viewport(ui, |ui, _| {
				ui.add_sized([width, height], TextEdit::multiline(&mut lock!(api).output.as_str()));
			});
		});
		ui.separator();
		let response = ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(buffer), r#"Type a command (e.g "help" or "status") and press Enter"#)).on_hover_text(P2POOL_INPUT);
		// If the user pressed enter, dump buffer contents into the process STDIN
		if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
			response.request_focus();                  // Get focus back
			let buffer = std::mem::take(buffer);       // Take buffer
			let mut process = lock!(process); // Lock
			if process.is_alive() { process.input.push(buffer); } // Push only if alive
		}
	}
	});

	//---------------------------------------------------------------------------------------------------- Args
	if !self.simple {
		debug!("P2Pool Tab | Rendering [Arguments]");
		ui.group(|ui| { ui.horizontal(|ui| {
			let width = (width/10.0) - SPACE;
			ui.add_sized([width, text_edit], Label::new("Command arguments:"));
			ui.add_sized([ui.available_width(), text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.arguments), r#"--wallet <...> --host <...>"#)).on_hover_text(P2POOL_ARGUMENTS);
			self.arguments.truncate(1024);
		})});
		ui.set_enabled(self.arguments.is_empty());
	}

	//---------------------------------------------------------------------------------------------------- Address
	debug!("P2Pool Tab | Rendering [Address]");
	ui.group(|ui| {
		let width = width - SPACE;
		ui.spacing_mut().text_edit_width = (width)-(SPACE*3.0);
		let text;
		let color;
		let len = format!("{:02}", self.address.len());
		if self.address.is_empty() {
			text = format!("Monero Address [{}/95] ➖", len);
			color = Color32::LIGHT_GRAY;
		} else if Regexes::addr_ok(&self.address) {
			text = format!("Monero Address [{}/95] ✔", len);
			color = Color32::from_rgb(100, 230, 100);
		} else {
			text = format!("Monero Address [{}/95] ❌", len);
			color = Color32::from_rgb(230, 50, 50);
		}
		ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
		ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.address), "4...")).on_hover_text(P2POOL_ADDRESS);
		self.address.truncate(95);
	});

	//---------------------------------------------------------------------------------------------------- Simple
	let height = ui.available_height();
	if self.simple {
		// [Node]
		let height = height / 6.5;
		ui.spacing_mut().slider_width = width - 8.0;
		ui.spacing_mut().icon_width = width / 25.0;

		// [Auto-select] if we haven't already.
		// Using [Arc<Mutex<Ping>>] as an intermediary here
		// saves me the hassle of wrapping [state: State] completely
		// and [.lock().unwrap()]ing it everywhere.
		// Two atomic bools = enough to represent this data
		debug!("P2Pool Tab | Running [auto-select] check");
		if self.auto_select {
			let mut ping = lock!(ping);
			// If we haven't auto_selected yet, auto-select and turn it off
			if ping.pinged && !ping.auto_selected {
				self.node = ping.fastest.to_string();
				ping.auto_selected = true;
			}
			drop(ping);
		}

		ui.vertical(|ui| {
		ui.horizontal(|ui| {

			debug!("P2Pool Tab | Rendering [Ping List]");
			// [Ping List]
			let mut ms = 0;
			let mut color = Color32::LIGHT_GRAY;
			if lock!(ping).pinged {
				for data in lock!(ping).nodes.iter() {
 					if data.ip == self.node {
						ms = data.ms;
						color = data.color;
						break
					}
				}
			}
			debug!("P2Pool Tab | Rendering [ComboBox] of Remote Nodes");
			let ip_location = crate::node::format_ip_location(&self.node, false);
			let text = RichText::new(format!(" ⏺ {}ms | {}", ms, ip_location)).color(color);
			ComboBox::from_id_source("remote_nodes").selected_text(text).show_ui(ui, |ui| {
				for data in lock!(ping).nodes.iter() {
					let ms = crate::node::format_ms(data.ms);
					let ip_location = crate::node::format_ip_location(data.ip, true);
					let text = RichText::new(format!(" ⏺ {} | {}", ms, ip_location)).color(data.color);
					ui.selectable_value(&mut self.node, data.ip.to_string(), text);
				}
			});
		});

		ui.add_space(5.0);

		debug!("P2Pool Tab | Rendering [Select fastest ... Ping] buttons");
		ui.horizontal(|ui| {
			let width = (width/5.0)-6.0;
			// [Select random node]
			if ui.add_sized([width, height], Button::new("Select random node")).on_hover_text(P2POOL_SELECT_RANDOM).clicked() {
				self.node = RemoteNode::get_random(&self.node);
			}
			// [Select fastest node]
			if ui.add_sized([width, height], Button::new("Select fastest node")).on_hover_text(P2POOL_SELECT_FASTEST).clicked() && lock!(ping).pinged {
				self.node = lock!(ping).fastest.to_string();
			}
			// [Ping Button]
			ui.add_enabled_ui(!lock!(ping).pinging, |ui| {
				if ui.add_sized([width, height], Button::new("Ping remote nodes")).on_hover_text(P2POOL_PING).clicked() {
					Ping::spawn_thread(ping);
				}
			});
			// [Last <-]
			if ui.add_sized([width, height], Button::new("⬅ Last")).on_hover_text(P2POOL_SELECT_LAST).clicked() {
				let ping = lock!(ping);
				match ping.pinged {
					true  => self.node = RemoteNode::get_last_from_ping(&self.node, &ping.nodes),
					false => self.node = RemoteNode::get_last(&self.node),
				}
				drop(ping);
			}
			// [Next ->]
			if ui.add_sized([width, height], Button::new("Next ➡")).on_hover_text(P2POOL_SELECT_NEXT).clicked() {
				let ping = lock!(ping);
				match ping.pinged {
					true  => self.node = RemoteNode::get_next_from_ping(&self.node, &ping.nodes),
					false => self.node = RemoteNode::get_next(&self.node),
				}
				drop(ping);
			}
		});

		ui.vertical(|ui| {
			let height = height / 2.0;
			let pinging = lock!(ping).pinging;
			ui.set_enabled(pinging);
			let prog = lock!(ping).prog.round();
			let msg = RichText::new(format!("{} ... {}%", lock!(ping).msg, prog));
			let height = height / 1.25;
			ui.add_space(5.0);
			ui.add_sized([width, height], Label::new(msg));
			ui.add_space(5.0);
			if pinging {
				ui.add_sized([width, height], Spinner::new().size(height));
			} else {
				ui.add_sized([width, height], Label::new("..."));
			}
			ui.add_sized([width, height], ProgressBar::new(prog.round()/100.0));
			ui.add_space(5.0);
		});
		});

		debug!("P2Pool Tab | Rendering [Auto-*] buttons");
		ui.group(|ui| {
		ui.horizontal(|ui| {
			let width = (width/3.0)-(SPACE*1.75);
			// [Auto-node]
			ui.add_sized([width, height], Checkbox::new(&mut self.auto_select, "Auto-select")).on_hover_text(P2POOL_AUTO_SELECT);
			ui.separator();
			// [Auto-node]
			ui.add_sized([width, height], Checkbox::new(&mut self.auto_ping, "Auto-ping")).on_hover_text(P2POOL_AUTO_NODE);
			ui.separator();
			// [Backup host]
			ui.add_sized([width, height], Checkbox::new(&mut self.backup_host, "Backup host")).on_hover_text(P2POOL_BACKUP_HOST_SIMPLE);
		})});

		debug!("P2Pool Tab | Rendering warning text");
		ui.add_sized([width, height/2.0], Hyperlink::from_label_and_url("WARNING: It is recommended to run/use your own Monero Node (hover for details)", "https://github.com/hinto-janai/gupax#running-a-local-monero-node")).on_hover_text(P2POOL_COMMUNITY_NODE_WARNING);

	//---------------------------------------------------------------------------------------------------- Advanced
	} else {
		debug!("P2Pool Tab | Rendering [Node List] elements");
		let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
		// [Monero node IP/RPC/ZMQ]
		ui.horizontal(|ui| {
		ui.group(|ui| {
			let width = width/10.0;
			ui.vertical(|ui| {
			ui.spacing_mut().text_edit_width = width*3.32;
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.name.len());
				if self.name.is_empty() {
					text = format!("Name [ {}/30 ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.name.is_match(&self.name) {
					text = format!("Name [ {}/30 ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!("Name [ {}/30 ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.name).on_hover_text(P2POOL_NAME);
				self.name.truncate(30);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:03}", self.ip.len());
				if self.ip.is_empty() {
					text = format!("  IP [{}/255]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if self.ip == "localhost" || REGEXES.ipv4.is_match(&self.ip) || REGEXES.domain.is_match(&self.ip) {
					text = format!("  IP [{}/255]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!("  IP [{}/255]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.ip).on_hover_text(P2POOL_NODE_IP);
				self.ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.rpc.len();
				if self.rpc.is_empty() {
					text = format!(" RPC [  {}/5  ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.port.is_match(&self.rpc) {
					text = format!(" RPC [  {}/5  ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!(" RPC [  {}/5  ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.rpc).on_hover_text(P2POOL_RPC_PORT);
				self.rpc.truncate(5);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.zmq.len();
				if self.zmq.is_empty() {
					text = format!(" ZMQ [  {}/5  ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if REGEXES.port.is_match(&self.zmq) {
					text = format!(" ZMQ [  {}/5  ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!(" ZMQ [  {}/5  ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.zmq).on_hover_text(P2POOL_ZMQ_PORT);
				self.zmq.truncate(5);
			});
		});

		ui.vertical(|ui| {
			let width = ui.available_width();
			ui.add_space(1.0);
			// [Manual node selection]
			ui.spacing_mut().slider_width = width - 8.0;
			ui.spacing_mut().icon_width = width / 25.0;
			// [Ping List]
			debug!("P2Pool Tab | Rendering [Node List]");
			let text = RichText::new(format!("{}. {}", self.selected_index+1, self.selected_name));
			ComboBox::from_id_source("manual_nodes").selected_text(text).show_ui(ui, |ui| {
				let mut n = 0;
				for (name, node) in node_vec.iter() {
					let text = RichText::new(format!("{}. {}\n     IP: {}\n    RPC: {}\n    ZMQ: {}", n+1, name, node.ip, node.rpc, node.zmq));
					if ui.add(SelectableLabel::new(self.selected_name == *name, text)).clicked() {
						self.selected_index = n;
						let node = node.clone();
						self.selected_name = name.clone();
						self.selected_ip = node.ip.clone();
						self.selected_rpc = node.rpc.clone();
						self.selected_zmq = node.zmq.clone();
						self.name = name.clone();
						self.ip = node.ip;
						self.rpc = node.rpc;
						self.zmq = node.zmq;
					}
					n += 1;
				}
			});
			// [Add/Save]
			let node_vec_len = node_vec.len();
			let mut exists = false;
			let mut save_diff = true;
			let mut existing_index = 0;
			for (name, node) in node_vec.iter() {
				if *name == self.name {
					exists = true;
					if self.ip == node.ip && self.rpc == node.rpc && self.zmq == node.zmq {
						save_diff = false;
					}
					break
				}
				existing_index += 1;
			}
			ui.horizontal(|ui| {
				let text = if exists { LIST_SAVE } else { LIST_ADD };
				let text = format!("{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000", text, self.selected_index+1, self.selected_name, node_vec_len);
				// If the node already exists, show [Save] and mutate the already existing node
				if exists {
					ui.set_enabled(!incorrect_input && save_diff);
					if ui.add_sized([width, text_edit], Button::new("Save")).on_hover_text(text).clicked() {
						let node = Node {
							ip: self.ip.clone(),
							rpc: self.rpc.clone(),
							zmq: self.zmq.clone(),
						};
						node_vec[existing_index].1 = node;
						self.selected_index = existing_index;
						self.selected_ip = self.ip.clone();
						self.selected_rpc = self.rpc.clone();
						self.selected_zmq = self.zmq.clone();
						info!("Node | S | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", existing_index+1, self.name, self.ip, self.rpc, self.zmq);
					}
				// Else, add to the list
				} else {
					ui.set_enabled(!incorrect_input && node_vec_len < 1000);
					if ui.add_sized([width, text_edit], Button::new("Add")).on_hover_text(text).clicked() {
						let node = Node {
							ip: self.ip.clone(),
							rpc: self.rpc.clone(),
							zmq: self.zmq.clone(),
						};
						node_vec.push((self.name.clone(), node));
						self.selected_index = node_vec_len;
						self.selected_name = self.name.clone();
						self.selected_ip = self.ip.clone();
						self.selected_rpc = self.rpc.clone();
						self.selected_zmq = self.zmq.clone();
						info!("Node | A | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", node_vec_len, self.name, self.ip, self.rpc, self.zmq);
					}
				}
			});
			// [Delete]
			ui.horizontal(|ui| {
				ui.set_enabled(node_vec_len > 1);
				let text = format!("{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000", LIST_DELETE, self.selected_index+1, self.selected_name, node_vec_len);
				if ui.add_sized([width, text_edit], Button::new("Delete")).on_hover_text(text).clicked() {
					let new_name;
					let new_node;
					match self.selected_index {
						0 => {
							new_name = node_vec[1].0.clone();
							new_node = node_vec[1].1.clone();
							node_vec.remove(0);
						}
						_ => {
							node_vec.remove(self.selected_index);
							self.selected_index -= 1;
							new_name = node_vec[self.selected_index].0.clone();
							new_node = node_vec[self.selected_index].1.clone();
						}
					};
					self.selected_name = new_name.clone();
					self.selected_ip = new_node.ip.clone();
					self.selected_rpc = new_node.rpc.clone();
					self.selected_zmq = new_node.zmq.clone();
					self.name = new_name;
					self.ip = new_node.ip;
					self.rpc = new_node.rpc;
					self.zmq = new_node.zmq;
					info!("Node | D | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", self.selected_index, self.selected_name, self.selected_ip, self.selected_rpc, self.selected_zmq);
				}
			});
			ui.horizontal(|ui| {
				ui.set_enabled(!self.name.is_empty() || !self.ip.is_empty() || !self.rpc.is_empty() || !self.zmq.is_empty());
				if ui.add_sized([width, text_edit], Button::new("Clear")).on_hover_text(LIST_CLEAR).clicked() {
					self.name.clear();
					self.ip.clear();
					self.rpc.clear();
					self.zmq.clear();
				}
			});
		});
		});
		});
		ui.add_space(5.0);

		debug!("P2Pool Tab | Rendering [Main/Mini/Peers/Log] elements");
		// [Main/Mini]
		ui.horizontal(|ui| {
		let height = height/4.0;
		ui.group(|ui| { ui.horizontal(|ui| {
			let width = (width/4.0)-SPACE;
			let height = height + 6.0;
			if ui.add_sized([width, height], SelectableLabel::new(!self.mini, "P2Pool Main")).on_hover_text(P2POOL_MAIN).clicked() { self.mini = false; }
			if ui.add_sized([width, height], SelectableLabel::new(self.mini, "P2Pool Mini")).on_hover_text(P2POOL_MINI).clicked() { self.mini = true; }
		})});
		// [Out/In Peers] + [Log Level]
		ui.group(|ui| { ui.vertical(|ui| {
			let text = (ui.available_width()/10.0)-SPACE;
			let width = (text*8.0)-SPACE;
			let height = height/3.0;
			ui.style_mut().spacing.slider_width = width/1.1;
			ui.style_mut().spacing.interact_size.y = height;
			ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
			ui.horizontal(|ui| {
				ui.add_sized([text, height], Label::new("Out peers [10-450]:"));
				ui.add_sized([width, height], Slider::new(&mut self.out_peers, 10..=450)).on_hover_text(P2POOL_OUT);
				ui.add_space(ui.available_width()-4.0);
			});
			ui.horizontal(|ui| {
				ui.add_sized([text, height], Label::new(" In peers [10-450]:"));
				ui.add_sized([width, height], Slider::new(&mut self.in_peers, 10..=450)).on_hover_text(P2POOL_IN);
			});
			ui.horizontal(|ui| {
				ui.add_sized([text, height], Label::new("   Log level [0-6]:"));
				ui.add_sized([width, height], Slider::new(&mut self.log_level, 0..=6)).on_hover_text(P2POOL_LOG);
			});
		})});
		});

		debug!("P2Pool Tab | Rendering Backup host button");
		ui.group(|ui| {
			let width = width - SPACE;
			let height = ui.available_height() / 3.0;
			// [Backup host]
			ui.add_sized([width, height], Checkbox::new(&mut self.backup_host, "Backup host")).on_hover_text(P2POOL_BACKUP_HOST_ADVANCED);
		});
	}
	}
}
