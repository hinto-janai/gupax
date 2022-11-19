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

use crate::State;
use egui::{
	TextStyle::Monospace,
	RichText,
	Label,
	Color32,
};
use crate::constants::*;
use crate::disk::{Gupax,Version};
use crate::update::*;
use std::thread;
use std::sync::{Arc,Mutex};
use log::*;

//---------------------------------------------------------------------------------------------------- FileWindow
// Struct for writing/reading the path state.
// The opened file picker is started in a new
// thread so main() needs to be in sync.
pub struct FileWindow {
	thread: bool, // Is there already a FileWindow thread?
	picked_p2pool: bool, // Did the user pick a path for p2pool?
	picked_xmrig: bool, // Did the user pick a path for xmrig?
	p2pool_path: String, // The picked p2pool path
	xmrig_path: String, // The picked p2pool path
}

impl FileWindow {
	pub fn new() -> Arc<Mutex<Self>> {
		Arc::new(Mutex::new(Self {
			thread: false,
			picked_p2pool: false,
			picked_xmrig: false,
			p2pool_path: String::new(),
			xmrig_path: String::new(),
		}))
	}
}

//---------------------------------------------------------------------------------------------------- Gupax
impl Gupax {
	pub fn show(&mut self, og: &Arc<Mutex<State>>, state_ver: &Arc<Mutex<Version>>, update: &Arc<Mutex<Update>>, file_window: &Arc<Mutex<FileWindow>>, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
		// Update button + Progress bar
		ui.group(|ui| {
				// These are in unnecessary [ui.vertical()]'s
				// because I need to use [ui.set_enabled]s, but I can't
				// find a way to use a [ui.xxx()] with [ui.add_sized()].
				// I have to pick one. This one seperates them though.
				let height = height/8.0;
				let width = width - SPACE;
				let updating = *update.lock().unwrap().updating.lock().unwrap();
				ui.vertical(|ui| {
					ui.set_enabled(!updating);
					if ui.add_sized([width, height], egui::Button::new("Check for updates")).on_hover_text(GUPAX_UPDATE).clicked() {
						update.lock().unwrap().path_p2pool = og.lock().unwrap().gupax.absolute_p2pool_path.display().to_string();
						update.lock().unwrap().path_xmrig = og.lock().unwrap().gupax.absolute_xmrig_path.display().to_string();
						update.lock().unwrap().tor = og.lock().unwrap().gupax.update_via_tor;
						let og = Arc::clone(&og);
						let state_ver = Arc::clone(&state_ver);
						let update = Arc::clone(&update);
						let update_thread = Arc::clone(&update);
						thread::spawn(move|| {
							info!("Spawning update thread...");
							match Update::start(update_thread, og.clone(), state_ver.clone()) {
								Err(e) => {
									info!("Update ... FAIL ... {}", e);
									*update.lock().unwrap().msg.lock().unwrap() = format!("{} | {}", MSG_FAILED, e);
								},
								_ => {
									info!("Update | Saving state...");
									match State::save(&mut og.lock().unwrap()) {
										Ok(_) => info!("Update ... OK"),
										Err(e) => {
											warn!("Update | Saving state ... FAIL ... {}", e);
											*update.lock().unwrap().msg.lock().unwrap() = format!("Saving new versions into state failed");
										},
									};
								}
							};
							*update.lock().unwrap().updating.lock().unwrap() = false;
						});
					}
				});
				ui.vertical(|ui| {
					ui.set_enabled(updating);
					let prog = *update.lock().unwrap().prog.lock().unwrap();
					let msg = format!("{}\n{}{}", *update.lock().unwrap().msg.lock().unwrap(), prog, "%");
					ui.add_sized([width, height*1.4], egui::Label::new(RichText::text_style(RichText::new(msg), Monospace)));
					let height = height/2.0;
					if updating {
						ui.add_sized([width, height], egui::Spinner::new().size(height));
					} else {
						ui.add_sized([width, height], egui::Label::new("..."));
					}
					ui.add_sized([width, height], egui::ProgressBar::new(update.lock().unwrap().prog.lock().unwrap().round() / 100.0));
				});
		});

		ui.horizontal(|ui| {
			ui.group(|ui| {
					let width = (width - SPACE*7.5)/4.0;
					let height = height/8.0;
					let mut style = (*ctx.style()).clone();
					style.spacing.icon_width_inner = width / 8.0;
					style.spacing.icon_width = width / 6.0;
					style.spacing.icon_spacing = 20.0;
					ctx.set_style(style);
					ui.add_sized([width, height], egui::Checkbox::new(&mut self.auto_update, "Auto-update")).on_hover_text(GUPAX_AUTO_UPDATE);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut self.update_via_tor, "Update via Tor")).on_hover_text(GUPAX_UPDATE_VIA_TOR);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut self.ask_before_quit, "Ask before quit")).on_hover_text(GUPAX_ASK_BEFORE_QUIT);
					ui.separator();
					ui.add_sized([width, height], egui::Checkbox::new(&mut self.save_before_quit, "Save before quit")).on_hover_text(GUPAX_SAVE_BEFORE_QUIT);
			});
		});
		ui.add_space(SPACE);

		ui.style_mut().override_text_style = Some(Monospace);
		let height = height/20.0;
		let text_edit = (ui.available_width()/10.0)-SPACE;
		ui.horizontal(|ui| {
			if self.p2pool_path.is_empty() {
				ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ➖").color(Color32::LIGHT_GRAY)));
			} else {
				match crate::disk::into_absolute_path(self.p2pool_path.clone()) {
					Ok(path) => {
						if path.is_file() {
							ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ✔").color(Color32::from_rgb(100, 230, 100))))
						} else {
							ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ❌").color(Color32::from_rgb(230, 50, 50))))
						}
					},
					_ => ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ❌").color(Color32::from_rgb(230, 50, 50)))),
				};
			}
			ui.spacing_mut().text_edit_width = ui.available_width() - SPACE;
			ui.set_enabled(!file_window.lock().unwrap().thread);
			if ui.button("Open").on_hover_text(GUPAX_SELECT).clicked() {
				file_window.lock().unwrap().thread = true;
				let file_window = Arc::clone(file_window);
				thread::spawn(move|| {
					match rfd::FileDialog::new().set_title("Select P2Pool Binary for Gupax").pick_file() {
						Some(path) => {
							info!("Gupax | [{}] path selected for P2Pool", path.display());
							file_window.lock().unwrap().p2pool_path = path.display().to_string();
							file_window.lock().unwrap().picked_p2pool = true;
						},
						None => info!("Gupax | No path selected for P2Pool"),
					};
					file_window.lock().unwrap().thread = false;
				});
			}
			ui.text_edit_singleline(&mut self.p2pool_path).on_hover_text(GUPAX_PATH_P2POOL);
		});
		ui.horizontal(|ui| {
			if self.xmrig_path.is_empty() {
				ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ➖").color(Color32::LIGHT_GRAY)));
			} else {
				match crate::disk::into_absolute_path(self.xmrig_path.clone()) {
					Ok(path) => {
						if path.is_file() {
							ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ✔").color(Color32::from_rgb(100, 230, 100))))
						} else {
							ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ❌").color(Color32::from_rgb(230, 50, 50))))
						}
					},
					_ => ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ❌").color(Color32::from_rgb(230, 50, 50)))),
				};
			}
			ui.spacing_mut().text_edit_width = ui.available_width() - SPACE;
			ui.set_enabled(!file_window.lock().unwrap().thread);
			if ui.button("Open").on_hover_text(GUPAX_SELECT).clicked() {
				file_window.lock().unwrap().thread = true;
				let file_window = Arc::clone(file_window);
				thread::spawn(move|| {
					match rfd::FileDialog::new().set_title("Select XMRig Binary for Gupax").pick_file() {
						Some(path) => {
							info!("Gupax | [{}] path selected for XMRig", path.display());
							file_window.lock().unwrap().xmrig_path = path.display().to_string();
							file_window.lock().unwrap().picked_xmrig = true;
						},
						None => info!("Gupax | No path selected for XMRig"),
					};
					file_window.lock().unwrap().thread = false;
				});
			}
			ui.text_edit_singleline(&mut self.xmrig_path).on_hover_text(GUPAX_PATH_XMRIG);
		});
		let mut guard = file_window.lock().unwrap();
		if guard.picked_p2pool { self.p2pool_path = guard.p2pool_path.clone(); guard.picked_p2pool = false; }
		if guard.picked_xmrig { self.xmrig_path = guard.xmrig_path.clone(); guard.picked_xmrig = false; }
		drop(guard);
	}
}
