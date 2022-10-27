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
use crate::state::{Gupax,Version};
use crate::update::*;
use std::thread;
use std::sync::{Arc,Mutex};
use log::*;

impl Gupax {
	pub fn show(state: &mut Gupax, width: f32, height: f32, update: &mut Update, version: Version, ctx: &egui::Context, ui: &mut egui::Ui) {
		// Update button + Progress bar
		ui.group(|ui| {
				// These are in unnecessary [ui.vertical()]'s
				// because I need to use [ui.set_enabled]s, but I can't
				// find a way to use a [ui.xxx()] with [ui.add_sized()].
				// I have to pick one. This one seperates them though.
				let height = height/6.0;
				let width = width - SPACE;
				let updating = *update.updating.lock().unwrap();
				ui.vertical(|ui| {
					ui.set_enabled(!updating);
					if ui.add_sized([width, height], egui::Button::new("Check for updates")).on_hover_text(GUPAX_UPDATE).clicked() {
						update.path_p2pool = state.absolute_p2pool_path.display().to_string();
						update.path_xmrig = state.absolute_xmrig_path.display().to_string();
						update.tor = state.update_via_tor;
						let u = Arc::new(Mutex::new(update.clone()));
						let u = Arc::clone(&u);
						let u2 = Arc::new(Mutex::new(update.clone()));
						let u2 = Arc::clone(&u);
						thread::spawn(move|| {
							info!("Spawning update thread...");
							match Update::start(u, version) {
								Err(e) => {
									info!("Update | {} ... FAIL", e);
									*u2.lock().unwrap().msg.lock().unwrap() = MSG_FAILED.to_string();
									*u2.lock().unwrap().updating.lock().unwrap() = false;
								},
								_ => {
									info!("Update ... OK");
									*u2.lock().unwrap().msg.lock().unwrap() = MSG_SUCCESS.to_string();
									*u2.lock().unwrap().prog.lock().unwrap() = 100;
									*u2.lock().unwrap().updating.lock().unwrap() = false;
								},
							}
						});
					}
				});
				ui.vertical(|ui| {
					ui.set_enabled(updating);
					let height = height/2.0;
					let msg = format!("{}{}{}{}", *update.msg.lock().unwrap(), " ... ", *update.prog.lock().unwrap(), "%");
					ui.add_sized([width, height], egui::Label::new(msg));
					if updating { ui.add_sized([width, height], egui::Spinner::new().size(height)); }
					ui.add_sized([width, height], egui::ProgressBar::new(*update.prog.lock().unwrap() as f32 / 100.0));
				});
		});

		ui.horizontal(|ui| {
			ui.group(|ui| {
					let width = (width - SPACE*10.0)/5.0;
					let height = height/2.0;
					let mut style = (*ctx.style()).clone();
					style.spacing.icon_width_inner = height / 6.0;
					style.spacing.icon_width = height / 4.0;
					style.spacing.icon_spacing = width / 20.0;
					ctx.set_style(style);
					let height = height/2.0;
					ui.add_sized([width, height], egui::Checkbox::new(&mut state.auto_update, "Auto-update")).on_hover_text(GUPAX_AUTO_UPDATE);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut state.auto_node, "Auto-node")).on_hover_text(GUPAX_AUTO_NODE);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut state.update_via_tor, "Update via Tor")).on_hover_text(GUPAX_UPDATE_VIA_TOR);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut state.ask_before_quit, "Ask before quit")).on_hover_text(GUPAX_ASK_BEFORE_QUIT);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut state.save_before_quit, "Save before quit")).on_hover_text(GUPAX_SAVE_BEFORE_QUIT);
			});
		});
		ui.add_space(SPACE);

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
