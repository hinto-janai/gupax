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
use monero::util::address::Address;
use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use num_cpus;
use crate::State;
use crate::constants::*;

// Main data structure for the XMRig tab
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Xmrig {
	pub version: String,
	pub sha256: String,
	pub manual: bool,
	pub tls: bool,
	pub nicehash: bool,
	pub keepalive: bool,
	pub hugepages_jit: bool,
	pub threads: u16,
	pub priority: u8,
	pub pool: String,
	pub address: String,
//	pub pool: std::net::SocketAddr,
//	pub address: monero::util::address::Address,
	pub max_threads: u16,
	pub current_threads: u16,
}

impl Xmrig {
	pub fn new() -> Self {
		let max_threads = num_cpus::get().try_into().unwrap();
		let current_threads: u16;
		if max_threads == 1 { current_threads = 1 } else { current_threads = max_threads/2 }
		Self {
			version: String::from("v6.18.0"),
			sha256: String::from("asdf"),
			manual: false,
			tls: false,
			nicehash: false,
			keepalive: false,
			hugepages_jit: true,
			threads: 16,
			priority: 2,
			pool: String::from(""),
			address: String::from(""),
//			pool: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3333),
//			address: Address::from_str("44hintoFpuo3ugKfcqJvh5BmrsTRpnTasJmetKC4VXCt6QDtbHVuixdTtsm6Ptp7Y8haXnJ6j8Gj2dra8CKy5ewz7Vi9CYW").unwrap(),
			max_threads,
			current_threads,
		}
	}

	pub fn show(state: &mut Xmrig, ctx: &egui::Context, ui: &mut egui::Ui) {
		let height = ui.available_height() / 10.0;
		let mut width = ui.available_width() - 10.0;
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
					if ui.add_sized([width/2.0, height/6.0], egui::SelectableLabel::new(state.manual == false, "P2Pool Mode")).on_hover_text(XMRIG_P2POOL).clicked() { state.manual = false; };
					if ui.add_sized([width/2.0, height/6.0], egui::SelectableLabel::new(state.manual == true, "Manual Mode")).on_hover_text(XMRIG_MANUAL).clicked() { state.manual = true; };
				})});
				ui.group(|ui| { ui.horizontal(|ui| {
					let width = width - 58.0;
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut state.tls, "TLS Connection")).on_hover_text(XMRIG_TLS);
					ui.separator();
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut state.hugepages_jit, "Hugepages JIT")).on_hover_text(XMRIG_HUGEPAGES_JIT);
					ui.separator();
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut state.nicehash, "Nicehash")).on_hover_text(XMRIG_NICEHASH);
					ui.separator();
					ui.add_sized([width/4.0, height/6.0], egui::Checkbox::new(&mut state.keepalive, "Keepalive")).on_hover_text(XMRIG_KEEPALIVE);
				})});
			})});
		});

//		ui.group(|ui| {
			style.spacing.slider_width = ui.available_width()/1.25;
			ctx.set_style(style);
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new(format!("Threads [1-{}]:", state.max_threads)));
				ui.add_sized([width, height/8.0], egui::Slider::new(&mut state.current_threads, 1..=state.max_threads)).on_hover_text(XMRIG_THREADS);
			});

			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new("CPU Priority [0-5]:"));
				ui.add_sized([width, height/8.0], egui::Slider::new(&mut state.priority, 0..=5)).on_hover_text(XMRIG_PRIORITY);
			});
//		});

//		ui.group(|ui| {
			if state.manual == false { ui.set_enabled(false); }
			let width = (width/4.0);
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new("Pool IP:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut state.pool);
			});
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/8.0], egui::Label::new("Address:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut state.address);
			});
//		});
	}
}
