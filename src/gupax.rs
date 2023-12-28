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

use crate::State;
use egui::{
	TextEdit,
	TextStyle,
	TextStyle::Monospace,
	Checkbox,ProgressBar,Spinner,Button,Label,Slider,
	SelectableLabel,
	RichText,
	Vec2,
};
use crate::{
	constants::*,
	update::*,
	ErrorState,
	Restart,
	Tab,
	macros::*,
};
use std::{
	thread,
	sync::{Arc,Mutex},
	path::Path,
};
use log::*;
use serde::{Serialize,Deserialize};

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
		arc_mut!(Self {
			thread: false,
			picked_p2pool: false,
			picked_xmrig: false,
			p2pool_path: String::new(),
			xmrig_path: String::new(),
		})
	}
}

#[derive(Debug,Clone)]
pub enum FileType {
	P2pool,
	Xmrig,
}

//---------------------------------------------------------------------------------------------------- Ratio Lock
// Enum for the lock ratio in the advanced tab.
#[derive(Clone,Copy,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum Ratio {
	Width,
	Height,
	None,
}

//---------------------------------------------------------------------------------------------------- Gupax
impl crate::disk::Gupax {
	#[inline(always)] // called once
	pub fn show(
		&mut self,
		og: &Arc<Mutex<State>>,
		state_path: &Path,
		update: &Arc<Mutex<Update>>,
		file_window: &Arc<Mutex<FileWindow>>,
		error_state: &mut ErrorState,
		restart: &Arc<Mutex<Restart>>,
		width: f32,
		height: f32,
		frame: &mut eframe::Frame,
		_ctx: &egui::Context,
		ui: &mut egui::Ui
	) {
		// Update button + Progress bar
		debug!("Gupax Tab | Rendering [Update] button + progress bar");
		ui.group(|ui| {
				let button = if self.simple { height/5.0 } else { height/15.0 };
				let height = if self.simple { height/5.0 } else { height/10.0 };
				let width = width - SPACE;
				let updating = *lock2!(update,updating);
				ui.vertical(|ui| {
					// If [Gupax] is being built for a Linux distro,
					// disable built-in updating completely.
					#[cfg(feature = "distro")]
					ui.set_enabled(false);
					#[cfg(feature = "distro")]
					ui.add_sized([width, button], Button::new("Updates are disabled")).on_disabled_hover_text(DISTRO_NO_UPDATE);
					#[cfg(not(feature = "distro"))]
					ui.set_enabled(!updating);
					#[cfg(not(feature = "distro"))]
					if ui.add_sized([width, button], Button::new("Check for updates")).on_hover_text(GUPAX_UPDATE).clicked() {
						Update::spawn_thread(og, self, state_path, update, error_state, restart);
					}
				});
				ui.vertical(|ui| {
					ui.set_enabled(updating);
					let prog = *lock2!(update,prog);
					let msg = format!("{}\n{}{}", *lock2!(update,msg), prog, "%");
					ui.add_sized([width, height*1.4], Label::new(RichText::new(msg)));
					let height = height/2.0;
					if updating {
						ui.add_sized([width, height], Spinner::new().size(height));
					} else {
						ui.add_sized([width, height], Label::new("..."));
					}
					ui.add_sized([width, height], ProgressBar::new(lock2!(update,prog).round() / 100.0));
				});
		});

		debug!("Gupax Tab | Rendering bool buttons");
		ui.horizontal(|ui| {
			ui.group(|ui| {
					let width = (width - SPACE*12.0)/6.0;
					let height = if self.simple { height/10.0 } else { height/15.0 };
					ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
					ui.add_sized([width, height], Checkbox::new(&mut self.update_via_tor, "Update via Tor")).on_hover_text(GUPAX_UPDATE_VIA_TOR);
					ui.separator();
					ui.add_sized([width, height], Checkbox::new(&mut self.auto_update, "Auto-Update")).on_hover_text(GUPAX_AUTO_UPDATE);
					ui.separator();
					ui.add_sized([width, height], Checkbox::new(&mut self.auto_p2pool, "Auto-P2Pool")).on_hover_text(GUPAX_AUTO_P2POOL);
					ui.separator();
					ui.add_sized([width, height], Checkbox::new(&mut self.auto_xmrig, "Auto-XMRig")).on_hover_text(GUPAX_AUTO_XMRIG);
					ui.separator();
					ui.add_sized([width, height], Checkbox::new(&mut self.ask_before_quit, "Ask before quit")).on_hover_text(GUPAX_ASK_BEFORE_QUIT);
					ui.separator();
					ui.add_sized([width, height], Checkbox::new(&mut self.save_before_quit, "Save before quit")).on_hover_text(GUPAX_SAVE_BEFORE_QUIT);
			});
		});

		if self.simple { return }

		debug!("Gupax Tab | Rendering P2Pool/XMRig path selection");
		// P2Pool/XMRig binary path selection
		let height = height/28.0;
		let text_edit = (ui.available_width()/10.0)-SPACE;
		ui.group(|ui| {
		ui.add_sized([ui.available_width(), height/2.0], Label::new(RichText::new("P2Pool/XMRig PATHs").underline().color(LIGHT_GRAY))).on_hover_text("Gupax is online");
		ui.separator();
		ui.horizontal(|ui| {
			if self.p2pool_path.is_empty() {
				ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ➖").color(LIGHT_GRAY))).on_hover_text(P2POOL_PATH_EMPTY);
			} else if !Self::path_is_file(&self.p2pool_path) {
				ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ❌").color(RED))).on_hover_text(P2POOL_PATH_NOT_FILE);
			} else if !crate::update::check_p2pool_path(&self.p2pool_path) {
				ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ❌").color(RED))).on_hover_text(P2POOL_PATH_NOT_VALID);
			} else {
				ui.add_sized([text_edit, height], Label::new(RichText::new("P2Pool Binary Path ✔").color(GREEN))).on_hover_text(P2POOL_PATH_OK);
			}
			ui.spacing_mut().text_edit_width = ui.available_width() - SPACE;
			ui.set_enabled(!lock!(file_window).thread);
			if ui.button("Open").on_hover_text(GUPAX_SELECT).clicked() {
				Self::spawn_file_window_thread(file_window, FileType::P2pool);
			}
			ui.add_sized([ui.available_width(), height], TextEdit::singleline(&mut self.p2pool_path)).on_hover_text(GUPAX_PATH_P2POOL);
		});
		ui.horizontal(|ui| {
			if self.xmrig_path.is_empty() {
				ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ➖").color(LIGHT_GRAY))).on_hover_text(XMRIG_PATH_EMPTY);
			} else if !Self::path_is_file(&self.xmrig_path) {
				ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ❌").color(RED))).on_hover_text(XMRIG_PATH_NOT_FILE);
			} else if !crate::update::check_xmrig_path(&self.xmrig_path) {
				ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ❌").color(RED))).on_hover_text(XMRIG_PATH_NOT_VALID);
			} else {
				ui.add_sized([text_edit, height], Label::new(RichText::new(" XMRig Binary Path ✔").color(GREEN))).on_hover_text(XMRIG_PATH_OK);
			}
			ui.spacing_mut().text_edit_width = ui.available_width() - SPACE;
			ui.set_enabled(!lock!(file_window).thread);
			if ui.button("Open").on_hover_text(GUPAX_SELECT).clicked() {
				Self::spawn_file_window_thread(file_window, FileType::Xmrig);
			}
			ui.add_sized([ui.available_width(), height], TextEdit::singleline(&mut self.xmrig_path)).on_hover_text(GUPAX_PATH_XMRIG);
		});
		});
		let mut guard = lock!(file_window);
		if guard.picked_p2pool { self.p2pool_path = guard.p2pool_path.clone(); guard.picked_p2pool = false; }
		if guard.picked_xmrig { self.xmrig_path = guard.xmrig_path.clone(); guard.picked_xmrig = false; }
		drop(guard);

		let height = ui.available_height()/6.0;

		// Saved [Tab]
		debug!("Gupax Tab | Rendering [Tab] selector");
		ui.group(|ui| {
			let width = (width/5.0)-(SPACE*1.93);
			ui.add_sized([ui.available_width(), height/2.0], Label::new(RichText::new("Default Tab").underline().color(LIGHT_GRAY))).on_hover_text(GUPAX_TAB);
			ui.separator();
			ui.horizontal(|ui| {
			if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::About, "About")).on_hover_text(GUPAX_TAB_ABOUT).clicked() { self.tab = Tab::About; }
			ui.separator();
			if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::Status, "Status")).on_hover_text(GUPAX_TAB_STATUS).clicked() { self.tab = Tab::Status; }
			ui.separator();
			if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::Gupax, "Gupax")).on_hover_text(GUPAX_TAB_GUPAX).clicked() { self.tab = Tab::Gupax; }
			ui.separator();
			if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::P2pool, "P2Pool")).on_hover_text(GUPAX_TAB_P2POOL).clicked() { self.tab = Tab::P2pool; }
			ui.separator();
			if ui.add_sized([width, height], SelectableLabel::new(self.tab == Tab::Xmrig, "XMRig")).on_hover_text(GUPAX_TAB_XMRIG).clicked() { self.tab = Tab::Xmrig; }
		})});

		// Gupax App resolution sliders
		debug!("Gupax Tab | Rendering resolution sliders");
		ui.group(|ui| {
		ui.add_sized([ui.available_width(), height/2.0], Label::new(RichText::new("Width/Height Adjust").underline().color(LIGHT_GRAY))).on_hover_text(GUPAX_ADJUST);
		ui.separator();
		ui.vertical(|ui| {
			let width = width/10.0;
			ui.spacing_mut().icon_width = width / 25.0;
			ui.spacing_mut().slider_width = width*7.6;
			match self.ratio {
				Ratio::None => (),
				Ratio::Width => {
					let width = self.selected_width as f64;
					let height = (width / 1.333).round();
					self.selected_height = height as u16;
				},
				Ratio::Height => {
					let height = self.selected_height as f64;
					let width = (height * 1.333).round();
					self.selected_width = width as u16;
				},
			}
			let height = height/3.5;
			ui.horizontal(|ui| {
				ui.set_enabled(self.ratio != Ratio::Height);
				ui.add_sized([width, height], Label::new(format!(" Width [{}-{}]:", APP_MIN_WIDTH as u16, APP_MAX_WIDTH as u16)));
				ui.add_sized([width, height], Slider::new(&mut self.selected_width, APP_MIN_WIDTH as u16..=APP_MAX_WIDTH as u16)).on_hover_text(GUPAX_WIDTH);
			});
			ui.horizontal(|ui| {
				ui.set_enabled(self.ratio != Ratio::Width);
				ui.add_sized([width, height], Label::new(format!("Height [{}-{}]:", APP_MIN_HEIGHT as u16, APP_MAX_HEIGHT as u16)));
				ui.add_sized([width, height], Slider::new(&mut self.selected_height, APP_MIN_HEIGHT as u16..=APP_MAX_HEIGHT as u16)).on_hover_text(GUPAX_HEIGHT);
			});
			ui.horizontal(|ui| {
				ui.add_sized([width, height], Label::new(format!("Scaling [{APP_MIN_SCALE}..{APP_MAX_SCALE}]:")));
				ui.add_sized([width, height], Slider::new(&mut self.selected_scale, APP_MIN_SCALE..=APP_MAX_SCALE).step_by(0.1)).on_hover_text(GUPAX_SCALE);
			});
		});
		ui.style_mut().override_text_style = Some(egui::TextStyle::Button);
		ui.separator();
		// Width/Height locks
		ui.horizontal(|ui| {
			use Ratio::*;
			let width = (width/4.0)-(SPACE*1.5);
			if ui.add_sized([width, height], SelectableLabel::new(self.ratio == Width, "Lock to width")).on_hover_text(GUPAX_LOCK_WIDTH).clicked() { self.ratio = Width; }
			ui.separator();
			if ui.add_sized([width, height], SelectableLabel::new(self.ratio == Height, "Lock to height")).on_hover_text(GUPAX_LOCK_HEIGHT).clicked() { self.ratio = Height; }
			ui.separator();
			if ui.add_sized([width, height], SelectableLabel::new(self.ratio == None, "No lock")).on_hover_text(GUPAX_NO_LOCK).clicked() { self.ratio = None; }
			if ui.add_sized([width, height], Button::new("Set")).on_hover_text(GUPAX_SET).clicked() {
				let size = Vec2::new(self.selected_width as f32, self.selected_height as f32);
				ui.ctx().send_viewport_cmd(egui::viewport::ViewportCommand::InnerSize(size));
			}
		})});
	}

	// Checks if a path is a valid path to a file.
	pub fn path_is_file(path: &str) -> bool {
		let path = path.to_string();
		match crate::disk::into_absolute_path(path) {
			Ok(path) => path.is_file(),
			_ => false,
		}
	}

	#[cold]
	#[inline(never)]
	fn spawn_file_window_thread(file_window: &Arc<Mutex<FileWindow>>, file_type: FileType) {
		use FileType::*;
		let name = match file_type {
			P2pool => "P2Pool",
			Xmrig  => "XMRig",
		};
		let file_window = file_window.clone();
		lock!(file_window).thread = true;
		thread::spawn(move|| {
			match rfd::FileDialog::new().set_title(&format!("Select {} Binary for Gupax", name)).pick_file() {
				Some(path) => {
					info!("Gupax | Path selected for {} ... {}", name, path.display());
					match file_type {
						P2pool => { lock!(file_window).p2pool_path = path.display().to_string(); lock!(file_window).picked_p2pool = true; },
						Xmrig  => { lock!(file_window).xmrig_path = path.display().to_string(); lock!(file_window).picked_xmrig = true; },
					};
				},
				None => info!("Gupax | No path selected for {}", name),
			};
			lock!(file_window).thread = false;
		});
	}
}
