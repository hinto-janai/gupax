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
use serde::{Serialize,Deserialize};
use std::time::{Instant,Duration};
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use egui::Color32;
use log::*;
use reqwest::blocking::ClientBuilder;

//---------------------------------------------------------------------------------------------------- Node list
// Community Monerod nodes. All of these have ZMQ on 18083.
// Adding/removing nodes will need changes to pretty
// much all the code in this file, and the code that
// handles the actual Enum selector in the P2Pool tab.
pub const C3POOL: &'static str = "node.c3pool.com:18081";
pub const CAKE: &'static str = "xmr-node.cakewallet.com:18081";
pub const CAKE_EU: &'static str = "xmr-node-eu.cakewallet.com:18081";
pub const CAKE_UK: &'static str = "xmr-node-uk.cakewallet.com:18081";
pub const CAKE_US: &'static str = "xmr-node-usa-east.cakewallet.com:18081";
pub const MAJESTICBANK_IS: &'static str = "node.majesticbank.is:18089";
pub const MAJESTICBANK_SU: &'static str = "node.majesticbank.su:18089";
pub const MONERUJO: &'static str = "nodex.monerujo.io:18081";
pub const RINO: &'static str = "node.community.rino.io:18081";
pub const SELSTA_1: &'static str = "selsta1.featherwallet.net:18081";
pub const SELSTA_2: &'static str = "selsta2.featherwallet.net:18081";
pub const SETH: &'static str = "node.sethforprivacy.com:18089";
pub const SUPPORTXMR: &'static str = "node.supportxmr.com:18081";
pub const SUPPORTXMR_IR: &'static str = "node.supportxmr.ir:18081";
pub const SINGAPORE: &'static str = "singapore.node.xmr.pm:18089";
pub const XMRVSBEAST: &'static str = "p2pmd.xmrvsbeast.com:18081";

pub const NODE_IPS: [&'static str; 16] = [
	C3POOL,CAKE,CAKE_EU,CAKE_UK,CAKE_US,MAJESTICBANK_IS,MAJESTICBANK_SU,MONERUJO,
	RINO,SELSTA_1,SELSTA_2,SETH,SUPPORTXMR,SUPPORTXMR_IR,SINGAPORE,XMRVSBEAST,
];

#[derive(Copy,Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum NodeEnum {
	C3pool,Cake,CakeEu,CakeUk,CakeUs,MajesticBankIs,MajesticBankSu,Monerujo,
	Rino,Selsta1,Selsta2,Seth,SupportXmr,SupportXmrIr,Singapore,XmrVsBeast,
}

impl std::fmt::Display for NodeEnum {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", self)
	}
}

//---------------------------------------------------------------------------------------------------- Node data
#[derive(Debug)]
pub struct NodeData {
	pub id: NodeEnum,
	pub ip: &'static str,
	pub ms: u128,
	pub color: Color32,
}

impl NodeData {
	pub fn new_vec() -> Vec<Self> {
		let mut vec = Vec::new();
		for ip in NODE_IPS.iter() {
			vec.push(Self {
				id: ip_to_enum(ip),
				ip,
				ms: 0,
				color: Color32::LIGHT_GRAY,
			});
		}
		vec
	}
}

//---------------------------------------------------------------------------------------------------- Ping data
#[derive(Debug)]
pub struct Ping {
	pub nodes: Vec<NodeData>,
	pub fastest: NodeEnum,
	pub pinging: bool,
	pub msg: String,
	pub prog: f32,
	pub pinged: bool,
	pub auto_selected: bool,
}

impl Ping {
	pub fn new() -> Self {
		Self {
			nodes: NodeData::new_vec(),
			fastest: NodeEnum::C3pool,
			pinging: false,
			msg: "No ping in progress".to_string(),
			prog: 0.0,
			pinged: false,
			auto_selected: true,
		}
	}
}

//---------------------------------------------------------------------------------------------------- IP <-> Enum functions
// Function for returning IP/Enum
pub fn ip_to_enum(ip: &'static str) -> NodeEnum {
	match ip {
		C3POOL          => C3pool,
		CAKE            => Cake,
		CAKE_EU         => CakeEu,
		CAKE_UK         => CakeUk,
		CAKE_US         => CakeUs,
		MAJESTICBANK_IS => MajesticBankIs,
		MAJESTICBANK_SU => MajesticBankSu,
		MONERUJO        => Monerujo,
		RINO            => Rino,
		SELSTA_1        => Selsta1,
		SELSTA_2        => Selsta2,
		SETH            => Seth,
		SINGAPORE       => Singapore,
		SUPPORTXMR      => SupportXmr,
		SUPPORTXMR_IR   => SupportXmrIr,
		_               => XmrVsBeast,
	}
}

pub fn enum_to_ip(node: NodeEnum) -> &'static str {
	match node {
		C3pool         => C3POOL,
		Cake           => CAKE,
		CakeEu         => CAKE_EU,
		CakeUk         => CAKE_UK,
		CakeUs         => CAKE_US,
		MajesticBankIs => MAJESTICBANK_IS,
		MajesticBankSu => MAJESTICBANK_SU,
		Monerujo       => MONERUJO,
		Rino           => RINO,
		Selsta1        => SELSTA_1,
		Selsta2        => SELSTA_2,
		Seth           => SETH,
		Singapore      => SINGAPORE,
		SupportXmr     => SUPPORTXMR,
		SupportXmrIr   => SUPPORTXMR_IR,
		_              => XMRVSBEAST
	}
}

// 5000 = 4 max length
pub fn format_ms(ms: u128) -> String {
	match ms.to_string().len() {
		1 => format!("{}ms   ", ms),
		2 => format!("{}ms  ", ms),
		3 => format!("{}ms ", ms),
		_ => format!("{}ms", ms),
	}
}

// MajesticBankIs = 14 max length
pub fn format_enum(id: NodeEnum) -> String {
	match id.to_string().len() {
		1  => format!("{}             ", id),
		2  => format!("{}            ", id),
		3  => format!("{}           ", id),
		4  => format!("{}          ", id),
		5  => format!("{}         ", id),
		6  => format!("{}        ", id),
		7  => format!("{}       ", id),
		8  => format!("{}      ", id),
		9  => format!("{}     ", id),
		10 => format!("{}    ", id),
		11 => format!("{}   ", id),
		12 => format!("{}  ", id),
		13 => format!("{} ", id),
		_  => format!("{}", id),
	}
}

//---------------------------------------------------------------------------------------------------- Main Ping function
// This is for pinging the community nodes to
// find the fastest/slowest one for the user.
// The process:
//   - Send 3 [get_info] JSON-RPC requests over HTTP
//   - Measure each request in milliseconds as [u128]
//   - Timeout on requests over 5 seconds
//   - Calculate average time
//   - Add data to appropriate struct
//   - Sort fastest to lowest
//   - Return [PingResult] (data and fastest node)
//
// This is done linearly since per IP since
// multi-threading might affect performance.
//
// <300ms  = GREEN
// <1000ms = YELLOW
// >1000ms = RED
// timeout = BLACK
// default = GRAY
use crate::NodeEnum::*;
pub fn ping(ping: Arc<Mutex<Ping>>, og: Arc<Mutex<State>>) {
	// Start ping
	ping.lock().unwrap().pinging = true;
	ping.lock().unwrap().prog = 0.0;
	let info = format!("{}", "Creating HTTPS Client");
	info!("Ping | {}", info);
	ping.lock().unwrap().msg = info;
	let percent = (100 / (NODE_IPS.len() - 1)) as f32 / 3.0;

	// Create Node vector
	let mut nodes = Vec::new();

	// Create JSON request
	let mut get_info = HashMap::new();
	get_info.insert("jsonrpc", "2.0");
	get_info.insert("id", "0");
	get_info.insert("method", "get_info");

	// Misc Settings
	let mut vec: Vec<(NodeEnum, u128)> = Vec::new();

	// Create HTTP Client
	let timeout_sec = Duration::from_millis(5000);
	let client = ClientBuilder::new();
	let client = ClientBuilder::timeout(client, timeout_sec);
	let client = ClientBuilder::build(client).unwrap();

	for ip in NODE_IPS.iter() {
		// Match IP
		let id = match *ip {
			C3POOL          => C3pool,
			CAKE            => Cake,
			CAKE_EU         => CakeEu,
			CAKE_UK         => CakeUk,
			CAKE_US         => CakeUs,
			MAJESTICBANK_IS => MajesticBankIs,
			MAJESTICBANK_SU => MajesticBankSu,
			MONERUJO        => Monerujo,
			RINO            => Rino,
			SELSTA_1        => Selsta1,
			SELSTA_2        => Selsta2,
			SETH            => Seth,
			SINGAPORE       => Singapore,
			SUPPORTXMR      => SupportXmr,
			SUPPORTXMR_IR   => SupportXmrIr,
			_ => XmrVsBeast,
		};
		// Misc
		let mut timeout = 0;
		let mut mid = Duration::new(0, 0);

		// Start JSON-RPC request
		for i in 1..=3 {
			ping.lock().unwrap().msg = format!("{}: {} [{}/3]", id, ip, i);
			let now = Instant::now();
			let http = "http://".to_string() + &**ip + "/json_rpc";
			match client.post(http).json(&get_info).send() {
				Ok(_) => mid += now.elapsed(),
				Err(_) => {
					mid += timeout_sec;
					timeout += 1;
					let error = format!("Timeout [{}/3] ... {:#?} ... {}", timeout, id, ip);
					error!("Ping | {}", error);
					ping.lock().unwrap().msg = error;
				},
			};
			ping.lock().unwrap().prog += percent;
		}

		// Calculate average
		let ms = mid.as_millis() / 3;
		vec.push((id, ms));
		let info = format!("{}ms ... {}: {}", ms, id, ip);
		info!("Ping | {}", info);
		ping.lock().unwrap().msg = format!("{}", info);
		let color: Color32;
		if timeout == 3 {
			color = Color32::BLACK;
		} else if ms >= 1000 {
			// RED
			color = Color32::from_rgb(230, 50, 50);
		} else if ms >= 300 {
			// YELLOW
			color = Color32::from_rgb(230, 230, 100);
		} else {
			// GREEN
			color = Color32::from_rgb(100, 230, 100);
		}
		nodes.push(NodeData { id, ip, ms, color })
	}

	let percent = (100.0 - ping.lock().unwrap().prog) / 2.0;
	ping.lock().unwrap().prog += percent;
	ping.lock().unwrap().msg = "Calculating fastest node".to_string();
	// Calculate fastest out of all nodes
	let mut fastest: NodeEnum = vec[0].0;
	let mut best_ms: u128 = vec[0].1;
	for (id, ms) in vec.iter() {
		if ms < &best_ms {
			fastest = *id;
			best_ms = *ms;
		}
	}
	let info = format!("Fastest node: {}ms ... {} @ {}", best_ms, fastest, enum_to_ip(fastest));
	info!("Ping | {}", info);
	let mut guard = ping.lock().unwrap();
		guard.nodes = nodes;
		guard.fastest = fastest;
		guard.prog = 100.0;
		guard.msg = info;
		guard.pinging = false;
		guard.pinged = true;
		drop(guard);
	if og.lock().unwrap().p2pool.auto_select {
		ping.lock().unwrap().auto_selected = false;
	}
	info!("Ping ... OK");
}
