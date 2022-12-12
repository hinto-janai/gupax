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
pub fn show(sys: &Arc<Mutex<Sys>>, p2pool_api: &Arc<Mutex<PubP2poolApi>>, xmrig_api: &Arc<Mutex<PubXmrigApi>>, p2pool_img: &Arc<Mutex<ImgP2pool>>, xmrig_img: &Arc<Mutex<ImgXmrig>>, p2pool_online: bool, xmrig_online: bool, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
	let width = (width/3.0)-(SPACE*1.666);
	let min_height = height/1.14;
	let height = height/25.0;
	ui.horizontal(|ui| {
	// [Gupax]
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, height*2.0], Label::new(RichText::new("[Gupax]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("Gupax is online");
		// Uptime
		let sys = sys.lock().unwrap();
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_GUPAX_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", sys.gupax_uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_CPU_USAGE);
		ui.add_sized([width, height], Label::new(format!("{}", sys.gupax_cpu_usage)));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_MEMORY_USAGE);
		ui.add_sized([width, height], Label::new(format!("{}", sys.gupax_memory_used_mb)));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_USAGE);
		ui.add_sized([width, height], Label::new(format!("{}", sys.system_cpu_usage)));
		ui.add_sized([width, height], Label::new(RichText::new("System Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_MEMORY);
		ui.add_sized([width, height], Label::new(format!("{}", sys.system_memory)));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU Model").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_MODEL);
		ui.add_sized([width, height], Label::new(format!("{}", sys.system_cpu_model)));
		drop(sys);
	})});
	// [P2Pool]
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_enabled(p2pool_online);
		ui.set_min_height(min_height);
		ui.add_sized([width, height*2.0], Label::new(RichText::new("[P2Pool]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("P2Pool is online").on_disabled_hover_text("P2Pool is offline");
		// Uptime
		let api = p2pool_api.lock().unwrap();
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_P2POOL_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", api.uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("Shares Found").underline().color(BONE))).on_hover_text(STATUS_P2POOL_SHARES);
		ui.add_sized([width, height], Label::new(format!("{}", api.shares_found)));
		ui.add_sized([width, height], Label::new(RichText::new("Payouts").underline().color(BONE))).on_hover_text(STATUS_P2POOL_PAYOUTS);
		ui.add_sized([width, height], Label::new(format!("Total: {}", api.payouts)));
		ui.add_sized([width, height], Label::new(format!("[{}/hour] [{}/day] [{}/month]", api.payouts_hour, api.payouts_day, api.payouts_month)));
		ui.add_sized([width, height], Label::new(RichText::new("XMR Mined").underline().color(BONE))).on_hover_text(STATUS_P2POOL_XMR);
		ui.add_sized([width, height], Label::new(format!("Total: {} XMR", api.xmr)));
		ui.add_sized([width, height], Label::new(format!("[{}/hour] [{}/day] [{}/month]", api.xmr_hour, api.xmr_day, api.xmr_month)));
		ui.add_sized([width, height], Label::new(RichText::new("P2Pool Hashrate [15m/1h/24h]").underline().color(BONE))).on_hover_text(STATUS_P2POOL_HASHRATE);
		ui.add_sized([width, height], Label::new(format!("[{} H/s] [{} H/s] [{} H/s]", api.hashrate_15m, api.hashrate_1h, api.hashrate_24h)));
		ui.add_sized([width, height], Label::new(RichText::new("Miners Connected").underline().color(BONE))).on_hover_text(STATUS_P2POOL_CONNECTIONS);
		ui.add_sized([width, height], Label::new(format!("{}", api.connections)));
		ui.add_sized([width, height], Label::new(RichText::new("Effort").underline().color(BONE))).on_hover_text(STATUS_P2POOL_EFFORT);
		ui.add_sized([width, height], Label::new(format!("[Average: {}] [Current: {}]", api.average_effort, api.current_effort)));
	})});
	// [XMRig]
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_enabled(xmrig_online);
		ui.set_min_height(min_height);
		ui.add_sized([width, height*2.0], Label::new(RichText::new("[XMRig]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("XMRig is online").on_disabled_hover_text("XMRig is offline");
		// Uptime
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline()));
		ui.add_sized([width, height], Label::new(format!("{}", xmrig_api.lock().unwrap().uptime)));
	})});
	});
}
}
