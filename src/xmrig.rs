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

use crate::constants::*;
use crate::disk::Xmrig;

impl Xmrig {
	pub fn show(&mut self, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
		let height = ui.available_height() / 10.0;
		let width = ui.available_width() - 10.0;
		ui.group(|ui| {
			ui.add_sized([width, height*4.0], egui::TextEdit::multiline(&mut "".to_owned()));
			ui.add_sized([width, 30.0], egui::TextEdit::singleline(&mut "".to_owned()));
		});

		let mut style = (*ctx.style()).clone();
		let height = ui.available_height()/1.2;
		let width = width - 15.0;
		ui.horizontal(|ui| {
			ui.group(|ui| { ui.vertical(|ui| {
				ui.group(|ui| { ui.horizontal(|ui| {
					if ui.add_sized([width/2.0, height/6.0], egui::SelectableLabel::new(self.simple == false, "P2Pool Mode")).on_hover_text(XMRIG_P2POOL).clicked() { self.simple = false; };
					if ui.add_sized([width/2.0, height/6.0], egui::SelectableLabel::new(self.simple == true, "Manual Mode")).on_hover_text(XMRIG_MANUAL).clicked() { self.simple = true; };
				})});
				ui.group(|ui| { ui.horizontal(|ui| {
					let width = width - 58.0;
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut self.tls, "TLS Connection")).on_hover_text(XMRIG_TLS);
					ui.separator();
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut self.hugepages_jit, "Hugepages JIT")).on_hover_text(XMRIG_HUGEPAGES_JIT);
					ui.separator();
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut self.nicehash, "Nicehash")).on_hover_text(XMRIG_NICEHASH);
					ui.separator();
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut self.keepalive, "Keepalive")).on_hover_text(XMRIG_KEEPALIVE);
				})});
			})});
		});

//		ui.group(|ui| {
			style.spacing.slider_width = ui.available_width()/1.25;
			ctx.set_style(style);
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new(format!("Threads [1-{}]:", self.max_threads)));
				ui.add_sized([width, height/8.0], egui::Slider::new(&mut self.current_threads, 1..=self.max_threads)).on_hover_text(XMRIG_THREADS);
			});

			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new("CPU Priority [0-5]:"));
				ui.add_sized([width, height/8.0], egui::Slider::new(&mut self.priority, 0..=5)).on_hover_text(XMRIG_PRIORITY);
			});
//		});

//		ui.group(|ui| {
			if self.simple == false { ui.set_enabled(false); }
			let width = width/4.0;
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new("Pool IP:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut self.pool);
			});
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new("Address:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut self.address);
			});
//		});
	}
}
