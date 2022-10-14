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

use serde_derive::{Serialize,Deserialize};
use std::time::{Instant,Duration};
use std::collections::HashMap;
use std::error::Error;
use std::thread;
use egui::Color32;
use log::*;

// Community Monerod nodes. All of these have ZMQ on 18083.
// Adding/removing nodes will need changes to pretty
// much all the code in this file, and the code that
// handles the actual Enum selector in the P2Pool tab.
pub const C3POOL: &'static str = "node.c3pool.com:18081";
pub const CAKE: &'static str = "xmr-node.cakewallet.com:18081";
pub const CAKE_EU: &'static str = "xmr-node-eu.cakewallet.com:18081";
pub const CAKE_UK: &'static str = "xmr-node-uk.cakewallet.com:18081";
pub const CAKE_US: &'static str = "xmr-node-usa-east.cakewallet.com:18081";
pub const MONERUJO: &'static str = "nodex.monerujo.io:18081";
pub const RINO: &'static str = "node.community.rino.io:18081";
pub const SELSTA: &'static str = "selsta1.featherwallet.net:18081";
pub const SETH: &'static str = "node.sethforprivacy.com:18089";
pub const SUPPORTXMR: &'static str = "node.supportxmr.com:18081";
pub const SUPPORTXMR_IR: &'static str = "node.supportxmr.ir:18081";
pub const XMRVSBEAST: &'static str = "p2pmd.xmrvsbeast.com:18081";

#[derive(Debug)]
pub struct NodeStruct {
	c3pool: Data, cake: Data, cake_eu: Data, cake_uk: Data, cake_us: Data, monerujo: Data,
	rino: Data, selsta: Data, seth: Data, supportxmr: Data, supportxmr_ir: Data, xmrvsbeast: Data,
}

#[derive(Debug)]
pub struct Data {
	pub ms: u128,
	pub color: Color32,
	pub id: NodeEnum,
	pub ip: String,
}

#[derive(Copy,Clone,Debug,Deserialize,Serialize)]
pub enum NodeEnum {
	C3pool, Cake, CakeEu, CakeUk, CakeUs, Monerujo, Rino,
	Selsta, Seth, SupportXmr, SupportXmrIr, XmrVsBeast,
}

#[derive(Debug)]
pub struct PingResult {
	pub node: NodeStruct,
	pub fastest: NodeEnum,
}

impl NodeStruct {
	pub fn default() -> Self {
		use crate::NodeEnum::*;
		Self {
			c3pool:        Data { ms: 0, color: Color32::GRAY, id: C3pool, ip: C3POOL.to_string(), },
			cake:          Data { ms: 0, color: Color32::GRAY, id: Cake, ip: CAKE.to_string(), },
			cake_eu:       Data { ms: 0, color: Color32::GRAY, id: CakeEu, ip: CAKE_EU.to_string(), },
			cake_uk:       Data { ms: 0, color: Color32::GRAY, id: CakeUk, ip: CAKE_UK.to_string(), },
			cake_us:       Data { ms: 0, color: Color32::GRAY, id: CakeUs, ip: CAKE_US.to_string(), },
			monerujo:      Data { ms: 0, color: Color32::GRAY, id: Monerujo, ip: MONERUJO.to_string(), },
			rino:          Data { ms: 0, color: Color32::GRAY, id: Rino, ip: RINO.to_string(), },
			selsta:        Data { ms: 0, color: Color32::GRAY, id: Selsta, ip: SELSTA.to_string(), },
			seth:          Data { ms: 0, color: Color32::GRAY, id: Seth, ip: SETH.to_string(), },
			supportxmr:    Data { ms: 0, color: Color32::GRAY, id: SupportXmr, ip: SUPPORTXMR.to_string(), },
			supportxmr_ir: Data { ms: 0, color: Color32::GRAY, id: SupportXmrIr, ip: SUPPORTXMR_IR.to_string(), },
			xmrvsbeast:    Data { ms: 0, color: Color32::GRAY, id: XmrVsBeast, ip: XMRVSBEAST.to_string(), },
		}
	}

	// Return array of all IPs
	fn array() -> [&'static str; 12] {
		[
			C3POOL,CAKE,CAKE_EU,CAKE_UK,CAKE_US,MONERUJO,RINO,
			SELSTA,SETH,SUPPORTXMR,SUPPORTXMR_IR,XMRVSBEAST,
		]
	}

	// This is for pinging the community nodes to
	// find the fastest/slowest one for the user.
	// The process:
	//   - Send 3x [get_info] JSON-RPC requests over HTTP
	//   - Measure each request in milliseconds as [u128]
	//   - Timeout on requests over 5 seconds
	//   - Calculate average time
	//   - Add data to appropriate struct
	//   - Sort fastest to lowest
	//   - Return [PingResult(NodeStruct, NodeEnum)] (data and fastest node)
	//
	// This is done linearly since per IP since
	// multi-threading might affect performance.
	//
	// <300ms  = GREEN
	// <1000ms = YELLOW
	// >1000ms = RED
	// timeout = BLACK
	// default = GRAY
	pub fn ping() -> PingResult {
		use crate::NodeEnum::*;
		info!("Starting community node pings...");
		let mut node = NodeStruct::default();
		let mut get_info = HashMap::new();
		get_info.insert("jsonrpc", "2.0");
		get_info.insert("id", "0");
		get_info.insert("method", "get_info");
		let mut vec: Vec<(u128, NodeEnum)> = Vec::new();
		let fastest = false;
		let timeout_sec = Duration::from_millis(5000);

		for ip in Self::array().iter() {
			let id = match *ip {
				C3POOL        => C3pool,
				CAKE          => Cake,
				CAKE_EU       => CakeEu,
				CAKE_UK       => CakeUk,
				CAKE_US       => CakeUs,
				MONERUJO      => Monerujo,
				RINO          => Rino,
				SELSTA        => Selsta,
				SETH          => Seth,
				SUPPORTXMR    => SupportXmr,
				SUPPORTXMR_IR => SupportXmrIr,
//				XMRVSBEAST    => XmrVsBeast,
				_ => XmrVsBeast,
			};
			let mut timeout = false;
			let mut mid = Duration::new(0, 0);
			let max = 3;
			for i in 1..=max {
				let client = reqwest::blocking::ClientBuilder::new();
				let client = reqwest::blocking::ClientBuilder::timeout(client, timeout_sec);
				let client = reqwest::blocking::ClientBuilder::build(client).unwrap();
				let http = "http://".to_owned() + &**ip + "/json_rpc";
				let now = Instant::now();
				match client.post(http).json(&get_info).send() {
					Ok(r) => mid += now.elapsed(),
					Err(err) => {
						error!("Timeout on [{}] (over 3 seconds)", ip);
						mid += timeout_sec;
						timeout = true;
					},
				};
			}
			let ms = mid.as_millis() / 3;
			vec.push((ms, id));
			info!("{}ms ... {}", ms, ip);
			let color: Color32;
			if timeout == true {
				color = Color32::BLACK
			} else if ms >= 1000 {
				color = Color32::LIGHT_RED
			} else if ms >= 300 {
				color = Color32::LIGHT_YELLOW
			} else {
				color = Color32::LIGHT_GREEN
			}
			match id {
				C3pool       => { node.c3pool.ms = ms; node.c3pool.color = color; },
				Cake         => { node.cake.ms = ms; node.cake.color = color; },
				CakeEu       => { node.cake_eu.ms = ms; node.cake_eu.color = color; },
				CakeUk       => { node.cake_uk.ms = ms; node.cake_uk.color = color; },
				CakeUs       => { node.cake_us.ms = ms; node.cake_us.color = color; },
				Monerujo     => { node.monerujo.ms = ms; node.monerujo.color = color; },
				Rino         => { node.rino.ms = ms; node.rino.color = color; },
				Selsta       => { node.selsta.ms = ms; node.selsta.color = color; },
				Seth         => { node.seth.ms = ms; node.seth.color = color; },
				SupportXmr   => { node.supportxmr.ms = ms; node.supportxmr.color = color; },
				SupportXmrIr => { node.supportxmr_ir.ms = ms; node.supportxmr_ir.color = color; },
				XmrVsBeast   => { node.xmrvsbeast.ms = ms; node.xmrvsbeast.color = color; },
			}
		}
		let mut best_ms: u128 = vec[0].0;
		let mut fastest: NodeEnum = vec[0].1;
		for (ms, id) in vec.iter() {
			if ms < &best_ms {
				fastest = *id;
				best_ms = *ms;
			}
		}
		// These values have weird behavior.
		// The values don't update if not printed beforehand,
		// so the match below on [fastest] gets funky.
		info!("Fastest node ... {:#?} @ {:#?}ms", fastest, best_ms);
		let ip = match fastest {
			C3pool       => C3POOL,
			Cake         => CAKE,
			CakeEu       => CAKE_EU,
			CakeUk       => CAKE_UK,
			CakeUs       => CAKE_US,
			Monerujo     => MONERUJO,
			Rino         => RINO,
			Selsta       => SELSTA,
			Seth         => SETH,
			SupportXmr   => SUPPORTXMR,
			SupportXmrIr => SUPPORTXMR_IR,
			XmrVsBeast   => XMRVSBEAST,
		};
		info!("Using IP ... {}", ip);
		info!("Community node ping ... OK");
		PingResult { node, fastest, }
	}
}
