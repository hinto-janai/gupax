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

use crate::{
	Helper,
	PubP2poolApi,
	PubXmrigApi,
	ImgP2pool,
	ImgXmrig,
	constants::*,
	Sys,
};
use std::sync::{Arc,Mutex};
use egui::{
	containers::*,
	Label,RichText,TextStyle
};

// Main data structure for the Status tab
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {}

impl Status {
pub fn show(sys: &Arc<Mutex<Sys>>, p2pool_api: &Arc<Mutex<PubP2poolApi>>, xmrig_api: &Arc<Mutex<PubXmrigApi>>, p2pool_img: &Arc<Mutex<ImgP2pool>>, xmrig_img: &Arc<Mutex<ImgXmrig>>, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
	let width = (width/3.0)-(SPACE*1.666);
	let min_height = height/1.14;
	let height = height/20.0;
	ui.horizontal(|ui| {
	// [Gupax]
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, height*2.0], Label::new(RichText::new("[Gupax]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into()))));
		// Uptime
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_GUPAX_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", sys.lock().unwrap().gupax_uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_CPU_USAGE);
		ui.add_sized([width, height], Label::new(format!("{}", sys.lock().unwrap().gupax_cpu_usage)));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_MEMORY_USAGE);
		ui.add_sized([width, height], Label::new(format!("{}", sys.lock().unwrap().gupax_memory_used_mb)));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_USAGE);
		ui.add_sized([width, height], Label::new(format!("{}", sys.lock().unwrap().system_cpu_usage)));
		ui.add_sized([width, height], Label::new(RichText::new("System Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_MEMORY);
		ui.add_sized([width, height], Label::new(format!("{}", sys.lock().unwrap().system_memory)));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU Model").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_MODEL);
		ui.add_sized([width, height], Label::new(format!("{}", sys.lock().unwrap().system_cpu_model)));
	})});
	// [P2Pool]
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, height*2.0], Label::new(RichText::new("[P2Pool]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into()))));
		// Uptime
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline()));
		ui.add_sized([width, height], Label::new(format!("{}", p2pool_api.lock().unwrap().uptime)));
	})});
	// [XMRig]
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, height*2.0], Label::new(RichText::new("[XMRig]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into()))));
		// Uptime
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline()));
		ui.add_sized([width, height], Label::new(format!("{}", xmrig_api.lock().unwrap().uptime)));
	})});
	});
}
}
