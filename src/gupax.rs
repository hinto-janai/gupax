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

use std::path::Path;
use crate::App;
use egui::WidgetType::Button;
use crate::constants::*;
use crate::state::Gupax;

impl Gupax {
	pub fn show(state: &mut Gupax, ctx: &egui::Context, ui: &mut egui::Ui) {
		let height = ui.available_height();
		let width = ui.available_width();
		let half_height = height / 6.0;
		let half_width = width / 2.0;

		ui.horizontal(|ui| {
			ui.group(|ui| {
					ui.vertical(|ui| {
						ui.add_sized([half_width - 8.0, half_height], egui::Button::new("Check for updates")).on_hover_text(GUPAX_UPDATE);
						ui.set_enabled(false);
						ui.add_sized([half_width - 8.0, half_height], egui::Button::new("Upgrade")).on_hover_text("asdf");
					});
			});

			ui.group(|ui| {
					let mut style = (*ctx.style()).clone();
					style.spacing.icon_width_inner = ui.available_height() / 6.0;
					style.spacing.icon_width = ui.available_height() / 4.0;
					style.spacing.icon_spacing = ui.available_width() / 20.0;
					ctx.set_style(style);
					let half_width = (half_width/2.0)-15.0;
					ui.vertical(|ui| {
						ui.add_sized([half_width, half_height], egui::Checkbox::new(&mut state.auto_update, "Auto-update")).on_hover_text(GUPAX_AUTO_UPDATE);
						ui.add_sized([half_width, half_height], egui::Checkbox::new(&mut state.ask_before_quit, "Ask before quitting")).on_hover_text(GUPAX_ASK_BEFORE_QUIT);
					});
					ui.vertical(|ui| {
						ui.add_sized([half_width, half_height], egui::Checkbox::new(&mut state.auto_node, "Auto-node")).on_hover_text(GUPAX_AUTO_NODE);
						ui.add_sized([half_width, half_height], egui::Checkbox::new(&mut state.save_before_quit, "Save before quitting")).on_hover_text(GUPAX_SAVE_BEFORE_QUIT);
					});
			});
		});
		ui.add_space(10.0);

		ui.horizontal(|ui| {
			ui.label("P2Pool binary path:");
			ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
			ui.text_edit_singleline(&mut state.p2pool_path).on_hover_text(GUPAX_PATH_P2POOL);
		});
		ui.horizontal(|ui| {
			ui.label("XMRig binary path: ");
			ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
			ui.text_edit_singleline(&mut state.xmrig_path).on_hover_text(GUPAX_PATH_XMRIG);
		});
	}
}
