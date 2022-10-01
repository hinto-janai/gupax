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
use crate::constants::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
	Rino,
	Seth,
	Selsta,
}

impl Node {
	pub fn ip(self) -> String {
		match self {
			Node::Rino => String::from("node.community.rino.io:18081"),
			Node::Seth => String::from("node.sethforprivacy.com:18089"),
			Node::Selsta => String::from("selsta1.featherwallet.net:18081"),
		}
	}
	pub fn name(self) -> String {
		match self {
			Node::Rino => String::from("Rino"),
			Node::Seth => String::from("Seth"),
			Node::Selsta => String::from("Selsta"),
		}
	}
}

// Main data structure for the P2Pool tab
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct P2pool {
	pub version: String,
	pub sha256: String,
	pub manual: bool,
	pub mini: bool,
	pub rpc: String,
	pub zmq: String,
//	pub rpc: u16,
//	pub zmq: u16,
	pub out_peers: u16,
	pub in_peers: u16,
	pub log: u8,
	pub monerod: String,
//	pub monerod: std::net::SocketAddr,
	pub community: Node,
	pub address: String,
//	pub address: monero::util::address::Address,
}

impl P2pool {
	pub fn new() -> Self {
		Self {
			version: String::from("v2.4"),
			sha256: String::from("asdf"),
			manual: false,
			mini: true,
			rpc: String::from(""),
			zmq: String::from(""),
			out_peers: 10,
			in_peers: 10,
			log: 3,
//			monerod: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 18081),
			monerod: String::from(""),
			address: String::from(""),
			community: Node::Rino,
//			address: Address::from_str("44hintoFpuo3ugKfcqJvh5BmrsTRpnTasJmetKC4VXCt6QDtbHVuixdTtsm6Ptp7Y8haXnJ6j8Gj2dra8CKy5ewz7Vi9CYW").unwrap(),
		}
	}

	pub fn show(state: &mut P2pool, ctx: &egui::Context, ui: &mut egui::Ui) {
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
					if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(state.mini == false, "P2Pool Main")).on_hover_text(P2POOL_MAIN).clicked() { state.mini = false; };
					if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(state.mini == true, "P2Pool Mini")).on_hover_text(P2POOL_MINI).clicked() { state.mini = true; };
				})});

				let width = (width/4.0);
				style.spacing.slider_width = width*1.25;
				ctx.set_style(style);
				ui.horizontal(|ui| {
					ui.add_sized([width/8.0, height/5.0], egui::Label::new("Out peers [10-450]:"));
					ui.add_sized([width, height/5.0], egui::Slider::new(&mut state.out_peers, 10..=450)).on_hover_text(P2POOL_OUT);
				});

				ui.horizontal(|ui| {
					ui.add_sized([width/8.0, height/5.0], egui::Label::new("    In peers [10-450]:"));
					ui.add_sized([width, height/5.0], egui::Slider::new(&mut state.in_peers, 10..=450)).on_hover_text(P2POOL_IN);
				});

				ui.horizontal(|ui| {
					ui.add_sized([width/8.0, height/5.0], egui::Label::new("          Log level [0-6]:"));
					ui.add_sized([width, height/5.0], egui::Slider::new(&mut state.log, 0..=6)).on_hover_text(P2POOL_LOG);
				});
			})});

		ui.group(|ui| { ui.vertical(|ui| {
			ui.group(|ui| { ui.horizontal(|ui| {
				if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(state.manual == false, "Community Monero Node")).on_hover_text(P2POOL_COMMUNITY).clicked() { state.manual = false; };
				if ui.add_sized([width/4.0, height/5.0], egui::SelectableLabel::new(state.manual == true, "Manual Monero Node")).on_hover_text(P2POOL_MANUAL).clicked() { state.manual = true; };
			})});
			ui.add_space(8.0);
			ui.horizontal(|ui| {
//			ui.add_sized([width/8.0, height/5.0],
			egui::ComboBox::from_label(Node::name(state.community.clone())).selected_text(Node::ip(state.community.clone())).show_ui(ui, |ui| {
					ui.selectable_value(&mut state.community, Node::Rino, Node::ip(Node::Rino));
					ui.selectable_value(&mut state.community, Node::Seth, Node::ip(Node::Seth));
					ui.selectable_value(&mut state.community, Node::Selsta, Node::ip(Node::Selsta));
			});
//			);
			});

			if state.manual == false { ui.set_enabled(false); }
			let width = (width/4.0);
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/7.8], egui::Label::new("Monero Node IP:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut state.monerod);
			});
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/7.8], egui::Label::new("Monero Node RPC Port:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut state.rpc);
			});
			ui.horizontal(|ui| {
				ui.add_sized([width/8.0, height/7.8], egui::Label::new("Monero Node ZMQ Port:"));
				ui.spacing_mut().text_edit_width = ui.available_width() - 35.0;
				ui.text_edit_singleline(&mut state.zmq);
			});

		})});

		});
		ui.group(|ui| {
		ui.horizontal(|ui| {
			ui.spacing_mut().text_edit_width = ui.available_width();
			ui.label("Address:");
			ui.text_edit_singleline(&mut state.address);
		})});
	}
}
