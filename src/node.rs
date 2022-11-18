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
//use hyper::{
//	Client,Body,Request,
//	header::{HeaderValue,LOCATION},
//};

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
pub const FEATHER_1: &'static str = "selsta1.featherwallet.net:18081";
pub const FEATHER_2: &'static str = "selsta2.featherwallet.net:18081";
pub const MAJESTICBANK_IS: &'static str = "node.majesticbank.is:18089";
pub const MAJESTICBANK_SU: &'static str = "node.majesticbank.su:18089";
pub const MONERUJO: &'static str = "nodex.monerujo.io:18081";
pub const RINO: &'static str = "node.community.rino.io:18081";
pub const SETH: &'static str = "node.sethforprivacy.com:18089";
pub const SUPPORTXMR: &'static str = "node.supportxmr.com:18081";
pub const SUPPORTXMR_IR: &'static str = "node.supportxmr.ir:18081";
pub const SINGAPORE: &'static str = "singapore.node.xmr.pm:18089";
pub const XMRVSBEAST: &'static str = "p2pmd.xmrvsbeast.com:18081";

pub const NODE_IPS: [&'static str; 16] = [
	C3POOL,CAKE,CAKE_EU,CAKE_UK,CAKE_US,FEATHER_1,FEATHER_2,MAJESTICBANK_IS,MAJESTICBANK_SU,
	MONERUJO,RINO,SETH,SUPPORTXMR,SUPPORTXMR_IR,SINGAPORE,XMRVSBEAST,
];

#[derive(Copy,Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum NodeEnum {
	C3pool,Cake,CakeEu,CakeUk,CakeUs,MajesticBankIs,MajesticBankSu,Monerujo,
	Rino,Feather1,Feather2,Seth,SupportXmr,SupportXmrIr,Singapore,XmrVsBeast,
}

impl std::fmt::Display for NodeEnum {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", self)
	}
}

//---------------------------------------------------------------------------------------------------- Node data
#[derive(Debug, Clone)]
pub struct NodeData {
	pub id: NodeEnum,
	pub ip: &'static str,
	pub ms: u128,
	pub color: Color32,
}

impl NodeData {
	pub fn new_vec() -> Vec<Self> {
		let mut vec = Vec::new();
		for ip in NODE_IPS {
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
use crate::NodeEnum::*;
// Function for returning IP/Enum
pub fn ip_to_enum(ip: &'static str) -> NodeEnum {
	match ip {
		C3POOL          => C3pool,
		CAKE            => Cake,
		CAKE_EU         => CakeEu,
		CAKE_UK         => CakeUk,
		CAKE_US         => CakeUs,
		FEATHER_1       => Feather1,
		FEATHER_2       => Feather2,
		MAJESTICBANK_IS => MajesticBankIs,
		MAJESTICBANK_SU => MajesticBankSu,
		MONERUJO        => Monerujo,
		RINO            => Rino,
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
		Feather1       => FEATHER_1,
		Feather2       => FEATHER_2,
		MajesticBankIs => MAJESTICBANK_IS,
		MajesticBankSu => MAJESTICBANK_SU,
		Monerujo       => MONERUJO,
		Rino           => RINO,
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
//   - Send [get_info] JSON-RPC request over HTTP to all IPs
//   - Measure each request in milliseconds
//   - Timeout on requests over 5 seconds
//   - Add data to appropriate struct
//   - Sorting fastest to lowest is automatic (fastest nodes return ... the fastest)
//
// This used to be done 3x linearly but after testing, sending a single
// JSON-RPC call to all IPs asynchronously resulted in the same data.
//
// <300ms  = GREEN
// <1000ms = YELLOW
// >1000ms = RED
// timeout = BLACK
// default = GRAY
#[tokio::main]
pub async fn ping(ping: Arc<Mutex<Ping>>, og: Arc<Mutex<State>>) -> Result<(), anyhow::Error> {
	// Start ping
	ping.lock().unwrap().pinging = true;
	ping.lock().unwrap().prog = 0.0;
	let percent = (100.0 / ((NODE_IPS.len()) as f32)).floor();

	// Create HTTP client
	let info = format!("{}", "Creating HTTP Client");
	ping.lock().unwrap().msg = info;
	let client: hyper::client::Client<hyper::client::HttpConnector> = hyper::Client::builder()
		.build(hyper::client::HttpConnector::new());

	// Random User Agent
	let rand_user_agent = crate::Pkg::get_user_agent();
	// Handle vector
	let mut handles = vec![];
	let node_vec = Arc::new(Mutex::new(Vec::new()));

	for ip in NODE_IPS {
		let client = client.clone();
		let ping = Arc::clone(&ping);
		let node_vec = Arc::clone(&node_vec);
		let request = hyper::Request::builder()
			.method("POST")
			.uri("http://".to_string() + ip + "/json_rpc")
			.header("User-Agent", rand_user_agent)
			.body(hyper::Body::from(r#"{"jsonrpc":"2.0","id":"0","method":"get_info"}"#))
			.unwrap();
		let handle = tokio::spawn(async move { response(client, request, ip, ping, percent, node_vec).await });
		handles.push(handle);
	}

	for handle in handles {
		handle.await;
	}
	let node_vec = node_vec.lock().unwrap().clone();

	let info = format!("Fastest node: {}ms ... {} @ {}", node_vec[0].ms, node_vec[0].id, node_vec[0].ip);
	info!("Ping | {}", info);
	let mut ping = ping.lock().unwrap();
		ping.fastest = node_vec[0].id;
		ping.nodes = node_vec;
		ping.prog = 100.0;
		ping.msg = info;
		ping.pinging = false;
		ping.pinged = true;
		ping.auto_selected = false;
		drop(ping);
	Ok(())
}

async fn response(client: hyper::client::Client<hyper::client::HttpConnector>, request: hyper::Request<hyper::Body>, ip: &'static str, ping: Arc<Mutex<Ping>>, percent: f32, node_vec: Arc<Mutex<Vec<NodeData>>>) {
	let id = ip_to_enum(ip);
	let now = Instant::now();
	let ms;
	match tokio::time::timeout(Duration::from_secs(5), client.request(request)).await {
		Ok(_) => ms = now.elapsed().as_millis(),
		Err(e) => { warn!("Ping | {}: {} ... FAIL ... {}", id, ip, e); ms = 5000; },
	};
	let color;
	if ms < 300 {
		color = Color32::from_rgb(100, 230, 100); // GREEN
	} else if ms < 1000 {
		color = Color32::from_rgb(230, 230, 100); // YELLOW
	} else if ms < 5000 {
		color = Color32::from_rgb(230, 50, 50); // RED
	} else {
		color = Color32::BLACK;
	}
	let info = format!("{}ms ... {}: {}", ms, id, ip);
	info!("Ping | {}", info);
	let mut ping = ping.lock().unwrap();
	ping.msg = info;
	ping.prog += percent;
	drop(ping);
	node_vec.lock().unwrap().push(NodeData { id: ip_to_enum(ip), ip, ms, color, });
}
