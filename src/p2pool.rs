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
use crate::state::P2pool;
use crate::node::NodeEnum;
use crate::node::{RINO,SETH,SELSTA};

//	pub simple: bool,
//	pub mini: bool,
//	pub out_peers: u8,
//	pub in_peers: u8,
//	pub log_level: u8,
//	pub node: crate::node::NodeEnum,
//	pub monerod: String,
//	pub rpc: u16,
//	pub zmq: u16,
//	pub address: String,


impl P2pool {
	pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		// TODO:
		// ping code
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

		let height = ui.available_height() / 10.0;
		let mut width = ui.available_width() - 50.0;
		ui.group(|ui| {
			ui.add_sized([width, height*4.0], egui::TextEdit::multiline(&mut "".to_owned()));
			ui.add_sized([width, 30.0], egui::TextEdit::singleline(&mut "".to_owned()));
		});

		width = width - 30.0;
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
					ui.selectable_value(&mut self.node, NodeEnum::Selsta, SELSTA);
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
