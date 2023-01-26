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
	constants::*,
	macros::*,
};
use serde::{Serialize,Deserialize};
use rand::{thread_rng, Rng};
use std::time::{Instant,Duration};
use std::sync::{Arc,Mutex};
use egui::Color32;
use log::*;
use hyper::{
	client::HttpConnector,
	Client,Body,Request,
};

//---------------------------------------------------------------------------------------------------- Node list
// Remote Monero Nodes with ZMQ enabled, sourced from: [https://github.com/hinto-janaiyo/monero-nodes]
// The format is an array of tuples consisting of: (ARRAY_INDEX, IP, LOCATION, RPC_PORT, ZMQ_PORT)

pub const REMOTE_NODES: [(usize, &str, &str, &str, &str); 22] = [
	(0,  "monero.10z.com.ar",       "🇦🇷 AR - Buenos Aires F.D.",         "18089", "18084"),
	(1,  "escom.sadovo.com",        "🇧🇬 BG - Plovdiv",                   "18089", "18084"),
	(2,  "monero2.10z.com.ar",      "🇧🇷 BR - São Paulo",                 "18089", "18083"),
	(3,  "monero1.heitechsoft.com", "🇨🇦 CA - Ontario",                   "18081", "18084"),
	(4,  "node.monerodevs.org",     "🇨🇦 CA - Quebec",                    "18089", "18084"),
	(5,  "de.poiuty.com",           "🇩🇪 DE - Berlin",                    "18081", "18084"),
	(6,  "m1.poiuty.com",           "🇩🇪 DE - Berlin",                    "18081", "18084"),
	(7,  "p2pmd.xmrvsbeast.com",    "🇩🇪 DE - Hesse",                     "18081", "18083"),
	(8,  "fbx.tranbert.com",        "🇫🇷 FR - Île-de-France",             "18089", "18084"),
	(9,  "reynald.ro",              "🇫🇷 FR - Île-de-France",             "18089", "18084"),
	(10, "node2.monerodevs.org",    "🇫🇷 FR - Occitanie",                 "18089", "18084"),
	(11, "monero.homeqloud.com",    "🇬🇷 GR - East Macedonia and Thrace", "18089", "18083"),
	(12, "home.allantaylor.kiwi",   "🇳🇿 NZ - Canterbury",                "18089", "18083"),
	(13, "ru.poiuty.com",           "🇷🇺 RU - Kuzbass",                   "18081", "18084"),
	(14, "radishfields.hopto.org",  "🇺🇸 US - Colorado",                  "18081", "18084"),
	(15, "xmrbandwagon.hopto.org",  "🇺🇸 US - Colorado",                  "18081", "18084"),
	(16, "xmr.spotlightsound.com",  "🇺🇸 US - Kansas",                    "18081", "18084"),
	(17, "xmrnode.facspro.net",     "🇺🇸 US - Nebraska",                  "18089", "18084"),
	(18, "jameswillhoite.com",      "🇺🇸 US - Ohio",                      "18089", "18084"),
	(19, "moneronode.ddns.net",     "🇺🇸 US - Pennsylvania",              "18089", "18084"),
	(20, "node.richfowler.net",     "🇺🇸 US - Pennsylvania",              "18089", "18084"),
	(21, "bunkernet.ddns.net",      "🇿🇦 ZA - Western Cape",              "18089", "18084"),
];

pub const REMOTE_NODE_LENGTH: usize = REMOTE_NODES.len();
pub const REMOTE_NODE_MAX_CHARS: usize = 24; // monero1.heitechsoft.com

pub struct RemoteNode {
	pub index: usize,
	pub ip: &'static str,
	pub flag: &'static str,
	pub rpc: &'static str,
	pub zmq: &'static str,
}

impl Default for RemoteNode {
	fn default() -> Self {
		Self::new()
	}
}

impl RemoteNode {
	pub fn new() -> Self {
		let (index, ip, flag, rpc, zmq) = REMOTE_NODES[0];
		Self {
			index,
			ip,
			flag,
			rpc,
			zmq,
		}
	}

	// Returns a default if IP is not found.
	pub fn from_ip(from_ip: &str) -> Self {
		for (index, ip, flag, rpc, zmq) in REMOTE_NODES {
			if from_ip == ip {
				return Self { index, ip, flag, rpc, zmq }
			}
		}
		Self::new()
	}

	// Returns a default if index is not found in the const array.
	pub fn from_index(index: usize) -> Self {
		if index > REMOTE_NODE_LENGTH {
			Self::new()
		} else {
			let (index, ip, flag, rpc, zmq) = REMOTE_NODES[index];
			Self { index, ip, flag, rpc, zmq }
		}
	}

	pub fn from_tuple(t: (usize, &'static str, &'static str, &'static str, &'static str)) -> Self {
		let (index, ip, flag, rpc, zmq) = (t.0, t.1, t.2, t.3, t.4);
		Self { index, ip, flag, rpc, zmq }
	}

	// monero1.heitechsoft.com = 24 max length
	pub fn format_ip(&self) -> String {
		match self.ip.len() {
			1  => format!("{}                       ", self.ip),
			2  => format!("{}                      ", self.ip),
			3  => format!("{}                     ", self.ip),
			4  => format!("{}                    ", self.ip),
			5  => format!("{}                   ", self.ip),
			6  => format!("{}                  ", self.ip),
			7  => format!("{}                 ", self.ip),
			8  => format!("{}                ", self.ip),
			9  => format!("{}               ", self.ip),
			10 => format!("{}              ", self.ip),
			11 => format!("{}             ", self.ip),
			12 => format!("{}            ", self.ip),
			13 => format!("{}           ", self.ip),
			14 => format!("{}          ", self.ip),
			15 => format!("{}         ", self.ip),
			16 => format!("{}        ", self.ip),
			17 => format!("{}       ", self.ip),
			18 => format!("{}      ", self.ip),
			19 => format!("{}     ", self.ip),
			20 => format!("{}    ", self.ip),
			21 => format!("{}   ", self.ip),
			22 => format!("{}  ", self.ip),
			23 => format!("{} ", self.ip),
			_  => format!("{}", self.ip),
		}
	}

	// Return a random node (that isn't the one already selected).
	pub fn get_random(&self) -> Self {
		let mut rand = thread_rng().gen_range(0..REMOTE_NODE_LENGTH);
		while rand == self.index {
			rand = thread_rng().gen_range(0..REMOTE_NODE_LENGTH);
		}
		Self::from_index(rand)
	}

	// Return the node [-1] of this one (wraps around)
	pub fn get_last(&self) -> Self {
		let index = self.index;
		if index == 0 {
			Self::from_index(REMOTE_NODE_LENGTH-1)
		} else {
			Self::from_index(index-1)
		}
	}

	// Return the node [+1] of this one (wraps around)
	pub fn get_next(&self) -> Self {
		let index = self.index;
		if index == REMOTE_NODE_LENGTH-1 {
			Self::from_index(0)
		} else {
			Self::from_index(index+1)
		}
	}

	// This returns relative to the ping.
	pub fn get_last_from_ping(&self, nodes: &Vec<NodeData>) -> Self {
		let mut found = false;
		let mut last = self.ip;
		for data in nodes {
			if found { return Self::from_ip(last) }
			if self.ip == data.ip { found = true; } else { last = data.ip; }
		}
		Self::from_ip(last)
	}

	pub fn get_next_from_ping(&self, nodes: &Vec<NodeData>) -> Self {
		let mut found = false;
		for data in nodes {
			if found { return Self::from_ip(data.ip) }
			if self.ip == data.ip { found = true; }
		}
		*self
	}
}

impl std::fmt::Display for RemoteNode {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", self.ip)
	}
}

//---------------------------------------------------------------------------------------------------- Node data
#[derive(Debug, Clone)]
pub struct NodeData {
	pub ip: &'static str,
	pub ms: u128,
	pub color: Color32,
}

impl NodeData {
	pub fn new_vec() -> Vec<Self> {
		let mut vec = Vec::new();
		for tuple in REMOTE_NODES {
			vec.push(Self {
				ip,
				ms: 0,
				color: Color32::LIGHT_GRAY,
			});
		}
		vec
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

//---------------------------------------------------------------------------------------------------- Ping data
#[derive(Debug)]
pub struct Ping {
	pub nodes: Vec<NodeData>,
	pub fastest: &'static str,
	pub pinging: bool,
	pub msg: String,
	pub prog: f32,
	pub pinged: bool,
	pub auto_selected: bool,
}

impl Default for Ping {
	fn default() -> Self {
		Self::new()
	}
}

impl Ping {
	pub fn new() -> Self {
		Self {
			nodes: NodeData::new_vec(),
			fastest: REMOTE_NODES[0].1,
			pinging: false,
			msg: "No ping in progress".to_string(),
			prog: 0.0,
			pinged: false,
			auto_selected: true,
		}
	}

	//---------------------------------------------------------------------------------------------------- Main Ping function
	// Intermediate function for spawning thread
	pub fn spawn_thread(ping: &Arc<Mutex<Self>>) {
		info!("Spawning ping thread...");
		let ping = Arc::clone(ping);
		std::thread::spawn(move|| {
			let now = Instant::now();
			match Self::ping(&ping) {
				Ok(msg) => {
					info!("Ping ... OK");
					lock!(ping).msg = msg;
					lock!(ping).pinged = true;
					lock!(ping).auto_selected = false;
					lock!(ping).prog = 100.0;
				},
				Err(err) => {
					error!("Ping ... FAIL ... {}", err);
					lock!(ping).pinged = false;
					lock!(ping).msg = err.to_string();
				},
			}
			info!("Ping ... Took [{}] seconds...", now.elapsed().as_secs_f32());
			lock!(ping).pinging = false;
		});
	}

	// This is for pinging the remote nodes to
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
	// <200ms  = GREEN
	// <500ms = YELLOW
	// >500ms = RED
	// timeout = BLACK
	// default = GRAY
	#[tokio::main]
	pub async fn ping(ping: &Arc<Mutex<Self>>) -> Result<String, anyhow::Error> {
		// Start ping
		let ping = Arc::clone(ping);
		lock!(ping).pinging = true;
		lock!(ping).prog = 0.0;
		let percent = (100.0 / (REMOTE_NODE_LENGTH as f32)).floor();

		// Create HTTP client
		let info = "Creating HTTP Client".to_string();
		lock!(ping).msg = info;
		let client: Client<HttpConnector> = Client::builder()
			.build(HttpConnector::new());

		// Random User Agent
		let rand_user_agent = crate::Pkg::get_user_agent();
		// Handle vector
		let mut handles = Vec::with_capacity(REMOTE_NODE_LENGTH);
		let node_vec = arc_mut!(Vec::with_capacity(REMOTE_NODE_LENGTH));

		for (index, ip, location, rpc, zmq) in REMOTE_NODES {
			let client = client.clone();
			let ping = Arc::clone(&ping);
			let node_vec = Arc::clone(&node_vec);
			let request = Request::builder()
				.method("POST")
				.uri("http://".to_string() + ip + "/json_rpc")
				.header("User-Agent", rand_user_agent)
				.body(hyper::Body::from(r#"{"jsonrpc":"2.0","id":"0","method":"get_info"}"#))
				.unwrap();
			let handle = tokio::task::spawn(async move { Self::response(client, request, ip, rpc, ping, percent, node_vec).await; });
			handles.push(handle);
		}

		for handle in handles {
			handle.await?;
		}

		let node_vec = std::mem::take(&mut *lock!(node_vec));
		let fastest_info = format!("Fastest node: {}ms ... {}", node_vec[0].ms, node_vec[0].ip);

		let info = "Cleaning up connections".to_string();
		info!("Ping | {}...", info);
		let mut ping = lock!(ping);
			ping.fastest = node_vec[0].ip;
			ping.nodes = node_vec;
			ping.msg = info;
			drop(ping);
		Ok(fastest_info)
	}

	async fn response(client: Client<HttpConnector>, request: Request<Body>, ip: &'static str, rpc: &'static str, ping: Arc<Mutex<Self>>, percent: f32, node_vec: Arc<Mutex<Vec<NodeData>>>) {
		let ms;
		let info;
		let now = Instant::now();
		match tokio::time::timeout(Duration::from_secs(5), client.request(request)).await {
			Ok(_) => {
				ms = now.elapsed().as_millis();
				info = format!("{}ms ... {}", ms, ip);
				info!("Ping | {}", info)
			},
			Err(_) => {
				ms = 5000;
				info = format!("{}ms ... {}", ms, ip);
				warn!("Ping | {}", info)
			},
		};
		let color;
		if ms < 200 {
			color = GREEN;
		} else if ms < 500 {
			color = YELLOW;
		} else if ms < 5000 {
			color = RED;
		} else {
			color = BLACK;
		}
		let mut ping = lock!(ping);
		ping.msg = info;
		ping.prog += percent;
		drop(ping);
		lock!(node_vec).push(NodeData { ip, ms, color, });
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
	#[test]
	fn validate_node_ips() {
		for (_, ip, _, _, _) in crate::REMOTE_NODES {
			assert!(ip.len() < 255);
			assert!(ip.is_ascii());
			assert!(ip.ends_with(":18081") || ip.ends_with(":18089"));
		}
	}

	#[test]
	fn spacing() {
		for (_, ip, _, _, _) in crate::REMOTE_NODES {
			assert!(crate::format_ip(ip).len() <= crate::REMOTE_NODE_MAX_CHARS);
		}
	}
}
