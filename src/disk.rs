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

// This handles reading/writing the disk files:
//     - [state.toml] -> [App] state
//     - [nodes.toml] -> [Manual Nodes] list
// The TOML format is used. This struct hierarchy
// directly translates into the TOML parser:
//   State/
//   ├─ Gupax/
//   │  ├─ ...
//   ├─ P2pool/
//   │  ├─ ...
//   ├─ Xmrig/
//   │  ├─ ...
//   ├─ Version/
//      ├─ ...

use std::{
	fs,
	fmt::Display,
	path::PathBuf,
	result::Result,
	sync::{Arc,Mutex},
	fmt::Write,
};
use serde::{Serialize,Deserialize};
use figment::Figment;
use figment::providers::{Format,Toml};
use crate::{
	constants::*,
	gupax::Ratio,
	Tab,
};
use log::*;

//---------------------------------------------------------------------------------------------------- Const
// State file
const ERROR: &str = "Disk error";
const PATH_ERROR: &str = "PATH for state directory could not be not found";
#[cfg(target_os = "windows")]
const DIRECTORY: &str = r#"Gupax\"#;
#[cfg(target_os = "macos")]
const DIRECTORY: &str = "Gupax/";
#[cfg(target_os = "linux")]
const DIRECTORY: &str = "gupax/";

#[cfg(target_os = "windows")]
pub const DEFAULT_P2POOL_PATH: &str = r"P2Pool\p2pool.exe";
#[cfg(target_os = "macos")]
pub const DEFAULT_P2POOL_PATH: &str = "p2pool/p2pool";
#[cfg(target_os = "windows")]
pub const DEFAULT_XMRIG_PATH: &str = r"XMRig\xmrig.exe";
#[cfg(target_os = "macos")]
pub const DEFAULT_XMRIG_PATH: &str = "xmrig/xmrig";

// Default to [/usr/bin/] for Linux distro builds.
#[cfg(target_os = "linux")]
#[cfg(not(feature = "distro"))]
pub const DEFAULT_P2POOL_PATH: &str = "p2pool/p2pool";
#[cfg(target_os = "linux")]
#[cfg(not(feature = "distro"))]
pub const DEFAULT_XMRIG_PATH: &str = "xmrig/xmrig";
#[cfg(feature = "distro")]
pub const DEFAULT_P2POOL_PATH: &str = "/usr/bin/p2pool";
#[cfg(feature = "distro")]
pub const DEFAULT_XMRIG_PATH: &str = "/usr/bin/xmrig";

//---------------------------------------------------------------------------------------------------- General functions for all [File]'s
// get_file_path()      | Return absolute path to OS data path + filename
// read_to_string()     | Convert the file at a given path into a [String]
// create_new()         | Write a default TOML Struct into the appropriate file (in OS data path)
// into_absolute_path() | Convert relative -> absolute path

pub fn get_gupax_data_path() -> Result<PathBuf, TomlError> {
	// Get OS data folder
	// Linux   | $XDG_DATA_HOME or $HOME/.local/share/gupax  | /home/alice/.local/state/gupax
	// macOS   | $HOME/Library/Application Support/Gupax     | /Users/Alice/Library/Application Support/Gupax
	// Windows | {FOLDERID_RoamingAppData}\Gupax             | C:\Users\Alice\AppData\Roaming\Gupax
	match dirs::data_dir() {
		Some(mut path) => {
			path.push(DIRECTORY);
			info!("OS | Data path ... {}", path.display());
			create_gupax_dir(&path)?;
			Ok(path)
		},
		None => { error!("OS | Data path ... FAIL"); Err(TomlError::Path(PATH_ERROR.to_string())) },
	}
}

pub fn create_gupax_dir(path: &PathBuf) -> Result<(), TomlError> {
	// Create directory
	match fs::create_dir_all(path) {
		Ok(_) => { info!("OS | Create data path ... OK"); Ok(()) },
		Err(e) => { error!("OS | Create data path ... FAIL ... {}", e); Err(TomlError::Io(e)) },
	}
}

// Convert a [File] path to a [String]
pub fn read_to_string(file: File, path: &PathBuf) -> Result<String, TomlError> {
	match fs::read_to_string(path) {
		Ok(string) => {
			info!("{:?} | Read ... OK", file);
			Ok(string)
		},
		Err(err) => {
			warn!("{:?} | Read ... FAIL", file);
			Err(TomlError::Io(err))
		},
	}
}

// Write str to console with [info!] surrounded by "---"
pub fn print_dash(toml: &str) {
	info!("{}", HORIZONTAL);
	for i in toml.lines() { info!("{}", i); }
	info!("{}", HORIZONTAL);
}

// Write str to console with [debug!] surrounded by "---"
pub fn print_dash_debug(toml: &str) {
	info!("{}", HORIZONTAL);
	for i in toml.lines() { debug!("{}", i); }
	info!("{}", HORIZONTAL);
}

// Turn relative paths into absolute paths
pub fn into_absolute_path(path: String) -> Result<PathBuf, TomlError> {
	let path = PathBuf::from(path);
	if path.is_relative() {
		let mut dir = std::env::current_exe()?;
		dir.pop();
		dir.push(path);
		Ok(dir)
	} else {
		Ok(path)
	}
}

//---------------------------------------------------------------------------------------------------- [State] Impl
impl Default for State {
	fn default() -> Self {
		Self::new()
	}
}

impl State {
	pub fn new() -> Self {
		let max_threads = num_cpus::get();
		let current_threads = if max_threads == 1 { 1 } else { max_threads / 2 };
		Self {
			gupax: Gupax::default(),
			p2pool: P2pool::default(),
			xmrig: Xmrig::with_threads(max_threads, current_threads),
			version: Arc::new(Mutex::new(Version::default())),
		}
	}

	// Convert [&str] to [State]
	pub fn from_str(string: &str) -> Result<Self, TomlError> {
		match toml::de::from_str(string) {
			Ok(state) => {
				info!("State | Parse ... OK");
				print_dash(string);
				Ok(state)
			}
			Err(err) => {
				warn!("State | String -> State ... FAIL ... {}", err);
				Err(TomlError::Deserialize(err))
			},
		}
	}

	// Conver [State] to [String]
	pub fn to_string(&self) -> Result<String, TomlError> {
		match toml::ser::to_string(self) {
			Ok(s) => Ok(s),
			Err(e) => { error!("State | Couldn't serialize default file: {}", e); Err(TomlError::Serialize(e)) },
		}
	}

	// Combination of multiple functions:
	//   1. Attempt to read file from path into [String]
	//      |_ Create a default file if not found
	//   2. Deserialize [String] into a proper [Struct]
	//      |_ Attempt to merge if deserialization fails
	pub fn get(path: &PathBuf) -> Result<Self, TomlError> {
		// Read
		let file = File::State;
		let string = match read_to_string(file, path) {
			Ok(string) => string,
			// Create
			_ => {
				Self::create_new(path)?;
				match read_to_string(file, path) {
					Ok(s) => s,
					Err(e) => return Err(e),
				}
			},
		};
		// Deserialize, attempt merge if failed
		match Self::from_str(&string) {
			Ok(s) => Ok(s),
			Err(_) => {
				warn!("State | Attempting merge...");
				match Self::merge(&string) {
					Ok(mut new) => { Self::save(&mut new, path)?; Ok(new) },
					Err(e)  => Err(e),
				}
			},
		}
	}

	// Completely overwrite current [state.toml]
	// with a new default version, and return [Self].
	pub fn create_new(path: &PathBuf) -> Result<Self, TomlError> {
		info!("State | Creating new default...");
		let new = Self::new();
		let string = Self::to_string(&new)?;
		fs::write(path, string)?;
		info!("State | Write ... OK");
		Ok(new)
	}

	// Save [State] onto disk file [gupax.toml]
	pub fn save(&mut self, path: &PathBuf) -> Result<(), TomlError> {
		info!("State | Saving to disk...");
		// Convert path to absolute
		self.gupax.absolute_p2pool_path = into_absolute_path(self.gupax.p2pool_path.clone())?;
		self.gupax.absolute_xmrig_path = into_absolute_path(self.gupax.xmrig_path.clone())?;
		let string = match toml::ser::to_string(&self) {
			Ok(string) => {
				info!("State | Parse ... OK");
				print_dash(&string);
				string
			},
			Err(err) => { error!("State | Couldn't parse TOML into string ... FAIL"); return Err(TomlError::Serialize(err)) },
		};
		match fs::write(path, string) {
			Ok(_) => { info!("State | Save ... OK"); Ok(()) },
			Err(err) => { error!("State | Couldn't overwrite TOML file ... FAIL"); Err(TomlError::Io(err)) },
		}
	}

	// Take [String] as input, merge it with whatever the current [default] is,
	// leaving behind old keys+values and updating [default] with old valid ones.
	pub fn merge(old: &str) -> Result<Self, TomlError> {
		let default = toml::ser::to_string(&Self::new()).unwrap();
		let new: Self = match Figment::from(Toml::string(&default)).merge(Toml::string(old)).extract() {
			Ok(new) => { info!("State | TOML merge ... OK"); new },
			Err(err) => { error!("State | Couldn't merge default + old TOML"); return Err(TomlError::Merge(err)) },
		};
		Ok(new)
	}
}

//---------------------------------------------------------------------------------------------------- [Node] Impl
impl Node {
	pub fn localhost() -> Self {
		Self {
			ip: "localhost".to_string(),
			rpc: "18081".to_string(),
			zmq: "18083".to_string(),
		}
	}

	pub fn new_vec() -> Vec<(String, Self)> {
		vec![("Local Monero Node".to_string(), Self::localhost())]
	}

	// Convert [String] to [Node] Vec
	pub fn from_str_to_vec(string: &str) -> Result<Vec<(String, Self)>, TomlError> {
		let nodes: toml::map::Map<String, toml::Value> = match toml::de::from_str(string) {
			Ok(map) => {
				info!("Node | Parse ... OK");
				map
			}
			Err(err) => {
				error!("Node | String parse ... FAIL ... {}", err);
				return Err(TomlError::Deserialize(err))
			},
		};
		let size = nodes.keys().len();
		let mut vec = Vec::with_capacity(size);
		for (key, values) in nodes.iter() {
			let ip = match values.get("ip") {
				Some(ip) => match ip.as_str() {
					Some(ip) => ip.to_string(),
					None => { error!("Node | [None] at [ip] parse"); return Err(TomlError::Parse("[None] at [ip] parse")) },
				},
				None => { error!("Node | [None] at [ip] parse"); return Err(TomlError::Parse("[None] at [ip] parse")) },
			};
			let rpc = match values.get("rpc") {
				Some(rpc) => match rpc.as_str() {
					Some(rpc) => rpc.to_string(),
					None => { error!("Node | [None] at [rpc] parse"); return Err(TomlError::Parse("[None] at [rpc] parse")) },
				},
				None => { error!("Node | [None] at [rpc] parse"); return Err(TomlError::Parse("[None] at [rpc] parse")) },
			};
			let zmq = match values.get("zmq") {
				Some(zmq) => match zmq.as_str() {
					Some(zmq) => zmq.to_string(),
					None => { error!("Node | [None] at [zmq] parse"); return Err(TomlError::Parse("[None] at [zmq] parse")) },
				},
				None => { error!("Node | [None] at [zmq] parse"); return Err(TomlError::Parse("[None] at [zmq] parse")) },
			};
			let node = Node {
				ip,
				rpc,
				zmq,
			};
			vec.push((key.clone(), node));
		}
		Ok(vec)
	}

	// Convert [Vec<(String, Self)>] into [String]
	// that can be written as a proper TOML file
	pub fn to_string(vec: &[(String, Self)]) -> Result<String, TomlError> {
		let mut toml = String::new();
		for (key, value) in vec.iter() {
			write!(
				toml,
				"[\'{}\']\nip = {:#?}\nrpc = {:#?}\nzmq = {:#?}\n\n",
				key,
				value.ip,
				value.rpc,
				value.zmq,
			)?;
		}
		Ok(toml)
	}

	// Combination of multiple functions:
	//   1. Attempt to read file from path into [String]
	//      |_ Create a default file if not found
	//   2. Deserialize [String] into a proper [Struct]
	//      |_ Attempt to merge if deserialization fails
	pub fn get(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
		// Read
		let file = File::Node;
		let string = match read_to_string(file, path) {
			Ok(string) => string,
			// Create
			_ => {
				Self::create_new(path)?;
				read_to_string(file, path)?
			},
		};
		// Deserialize, attempt merge if failed
		Self::from_str_to_vec(&string)
	}

	// Completely overwrite current [node.toml]
	// with a new default version, and return [Vec<String, Self>].
	pub fn create_new(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
		info!("Node | Creating new default...");
		let new = Self::new_vec();
		let string = Self::to_string(&Self::new_vec())?;
		fs::write(path, string)?;
		info!("Node | Write ... OK");
		Ok(new)
	}

	// Save [Node] onto disk file [node.toml]
	pub fn save(vec: &[(String, Self)], path: &PathBuf) -> Result<(), TomlError> {
		info!("Node | Saving to disk ... [{}]", path.display());
		let string = Self::to_string(vec)?;
		match fs::write(path, string) {
			Ok(_) => { info!("Node | Save ... OK"); Ok(()) },
			Err(err) => { error!("Node | Couldn't overwrite file"); Err(TomlError::Io(err)) },
		}
	}

//	pub fn merge(old: &String) -> Result<Self, TomlError> {
//		info!("Node | Starting TOML merge...");
//		let default = match toml::ser::to_string(&Self::new()) {
//			Ok(string) => { info!("Node | Default TOML parse ... OK"); string },
//			Err(err) => { error!("Node | Couldn't parse default TOML into string"); return Err(TomlError::Serialize(err)) },
//		};
//		let mut new: Self = match Figment::new().merge(Toml::string(&old)).merge(Toml::string(&default)).extract() {
//			Ok(new) => { info!("Node | TOML merge ... OK"); new },
//			Err(err) => { error!("Node | Couldn't merge default + old TOML"); return Err(TomlError::Merge(err)) },
//		};
//		// Attempt save
//		Self::save(&mut new)?;
//		Ok(new)
//	}
}

//---------------------------------------------------------------------------------------------------- [Pool] impl
impl Pool {
	pub fn p2pool() -> Self {
		Self {
			rig: GUPAX_VERSION_UNDERSCORE.to_string(),
			ip: "localhost".to_string(),
			port: "3333".to_string(),
		}
	}

	pub fn new_vec() -> Vec<(String, Self)> {
		vec![("Local P2Pool".to_string(), Self::p2pool())]
	}

	pub fn from_str_to_vec(string: &str) -> Result<Vec<(String, Self)>, TomlError> {
		let pools: toml::map::Map<String, toml::Value> = match toml::de::from_str(string) {
			Ok(map) => {
				info!("Pool | Parse ... OK");
				map
			}
			Err(err) => {
				error!("Pool | String parse ... FAIL ... {}", err);
				return Err(TomlError::Deserialize(err))
			},
		};
		let size = pools.keys().len();
		let mut vec = Vec::with_capacity(size);
		// We have to do [.as_str()] -> [.to_string()] to get rid of the \"...\" that gets added on.
		for (key, values) in pools.iter() {
			let rig = match values.get("rig") {
				Some(rig) => match rig.as_str() {
					Some(rig) => rig.to_string(),
					None => { error!("Pool | [None] at [rig] parse"); return Err(TomlError::Parse("[None] at [rig] parse")) },
				},
				None => { error!("Pool | [None] at [rig] parse"); return Err(TomlError::Parse("[None] at [rig] parse")) },
			};
			let ip = match values.get("ip") {
				Some(ip) => match ip.as_str() {
					Some(ip) => ip.to_string(),
					None => { error!("Pool | [None] at [ip] parse"); return Err(TomlError::Parse("[None] at [ip] parse")) },
				},
				None => { error!("Pool | [None] at [ip] parse"); return Err(TomlError::Parse("[None] at [ip] parse")) },
			};
			let port = match values.get("port") {
				Some(port) => match port.as_str() {
					Some(port) => port.to_string(),
					None => { error!("Pool | [None] at [port] parse"); return Err(TomlError::Parse("[None] at [port] parse")) },
				},
				None => { error!("Pool | [None] at [port] parse"); return Err(TomlError::Parse("[None] at [port] parse")) },
			};
			let pool = Pool {
				rig,
				ip,
				port,
			};
			vec.push((key.clone(), pool));
		}
		Ok(vec)
	}

	pub fn to_string(vec: &[(String, Self)]) -> Result<String, TomlError> {
		let mut toml = String::new();
		for (key, value) in vec.iter() {
			write!(
				toml,
				"[\'{}\']\nrig = {:#?}\nip = {:#?}\nport = {:#?}\n\n",
				key,
				value.rig,
				value.ip,
				value.port,
			)?;
		}
		Ok(toml)
	}

	pub fn get(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
		// Read
		let file = File::Pool;
		let string = match read_to_string(file, path) {
			Ok(string) => string,
			// Create
			_ => {
				Self::create_new(path)?;
				read_to_string(file, path)?
			},
		};
		// Deserialize
		Self::from_str_to_vec(&string)
	}

	pub fn create_new(path: &PathBuf) -> Result<Vec<(String, Self)>, TomlError> {
		info!("Pool | Creating new default...");
		let new = Self::new_vec();
		let string = Self::to_string(&Self::new_vec())?;
		fs::write(path, string)?;
		info!("Pool | Write ... OK");
		Ok(new)
	}

	pub fn save(vec: &[(String, Self)], path: &PathBuf) -> Result<(), TomlError> {
		info!("Pool | Saving to disk ... [{}]", path.display());
		let string = Self::to_string(vec)?;
		match fs::write(path, string) {
			Ok(_) => { info!("Pool | Save ... OK"); Ok(()) },
			Err(err) => { error!("Pool | Couldn't overwrite file"); Err(TomlError::Io(err)) },
		}
	}
}

//---------------------------------------------------------------------------------------------------- Custom Error [TomlError]
#[derive(Debug)]
pub enum TomlError {
	Io(std::io::Error),
	Path(String),
	Serialize(toml::ser::Error),
	Deserialize(toml::de::Error),
	Merge(figment::Error),
	Format(std::fmt::Error),
	Parse(&'static str),
}

impl Display for TomlError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		use TomlError::*;
		match self {
			Io(err)          => write!(f, "{}: IO | {}", ERROR, err),
			Path(err)        => write!(f, "{}: Path | {}", ERROR, err),
			Serialize(err)   => write!(f, "{}: Serialize | {}", ERROR, err),
			Deserialize(err) => write!(f, "{}: Deserialize | {}", ERROR, err),
			Merge(err)       => write!(f, "{}: Merge | {}", ERROR, err),
			Format(err)      => write!(f, "{}: Format | {}", ERROR, err),
			Parse(err)      => write!(f, "{}: Parse | {}", ERROR, err),
		}
	}
}

impl From<std::io::Error> for TomlError {
	fn from(err: std::io::Error) -> Self {
		TomlError::Io(err)
	}
}

impl From<std::fmt::Error> for TomlError {
	fn from(err: std::fmt::Error) -> Self {
		TomlError::Format(err)
	}
}

//---------------------------------------------------------------------------------------------------- [File] Enum (for matching which file)
#[derive(Clone,Copy,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum File {
	State, // state.toml -> Gupax state
	Node, // node.toml -> P2Pool manual node selector
	Pool, // pool.toml -> XMRig manual pool selector
}

//---------------------------------------------------------------------------------------------------- [Node] Struct
#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Node {
	pub ip: String,
	pub rpc: String,
	pub zmq: String,
}

//---------------------------------------------------------------------------------------------------- [Pool] Struct
#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Pool {
	pub rig: String,
	pub ip: String,
	pub port: String,
}

//---------------------------------------------------------------------------------------------------- [State] Struct
#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct State {
	pub gupax: Gupax,
	pub p2pool: P2pool,
	pub xmrig: Xmrig,
	pub version: Arc<Mutex<Version>>,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Gupax {
	pub simple: bool,
	pub auto_update: bool,
	pub auto_p2pool: bool,
	pub auto_xmrig: bool,
	pub ask_before_quit: bool,
	pub save_before_quit: bool,
	pub update_via_tor: bool,
	pub p2pool_path: String,
	pub xmrig_path: String,
	pub absolute_p2pool_path: PathBuf,
	pub absolute_xmrig_path: PathBuf,
	pub selected_width: u16,
	pub selected_height: u16,
	pub tab: Tab,
	pub ratio: Ratio,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct P2pool {
	pub simple: bool,
	pub mini: bool,
	pub auto_ping: bool,
	pub auto_select: bool,
	pub out_peers: u16,
	pub in_peers: u16,
	pub log_level: u8,
	pub node: crate::node::NodeEnum,
	pub arguments: String,
	pub address: String,
	pub name: String,
	pub ip: String,
	pub rpc: String,
	pub zmq: String,
	pub selected_index: usize,
	pub selected_name: String,
	pub selected_ip: String,
	pub selected_rpc: String,
	pub selected_zmq: String,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Xmrig {
	pub simple: bool,
	pub pause: u8,
	pub simple_rig: String,
	pub arguments: String,
	pub tls: bool,
	pub keepalive: bool,
	pub max_threads: usize,
	pub current_threads: usize,
	pub address: String,
	pub api_ip: String,
	pub api_port: String,
	pub name: String,
	pub rig: String,
	pub ip: String,
	pub port: String,
	pub selected_index: usize,
	pub selected_name: String,
	pub selected_rig: String,
	pub selected_ip: String,
	pub selected_port: String,
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Version {
	pub gupax: String,
	pub p2pool: String,
	pub xmrig: String,
}

//---------------------------------------------------------------------------------------------------- [State] Defaults
impl Default for Gupax {
	fn default() -> Self {
		Self {
			simple: true,
			auto_update: true,
			auto_p2pool: false,
			auto_xmrig: false,
			ask_before_quit: true,
			save_before_quit: true,
			#[cfg(not(target_os = "macos"))]
			update_via_tor: true,
			#[cfg(target_os = "macos")] // Arti library has issues on macOS
			update_via_tor: false,
			p2pool_path: DEFAULT_P2POOL_PATH.to_string(),
			xmrig_path: DEFAULT_XMRIG_PATH.to_string(),
			absolute_p2pool_path: into_absolute_path(DEFAULT_P2POOL_PATH.to_string()).unwrap(),
			absolute_xmrig_path: into_absolute_path(DEFAULT_XMRIG_PATH.to_string()).unwrap(),
			selected_width: APP_DEFAULT_WIDTH as u16,
			selected_height: APP_DEFAULT_HEIGHT as u16,
			ratio: Ratio::Width,
			tab: Tab::About,
		}
	}
}
impl Default for P2pool {
	fn default() -> Self {
		Self {
			simple: true,
			mini: true,
			auto_ping: true,
			auto_select: true,
			out_peers: 10,
			in_peers: 10,
			log_level: 3,
			node: crate::NodeEnum::C3pool,
			arguments: String::new(),
			address: String::with_capacity(96),
			name: "Local Monero Node".to_string(),
			ip: "localhost".to_string(),
			rpc: "18081".to_string(),
			zmq: "18083".to_string(),
			selected_index: 0,
			selected_name: "Local Monero Node".to_string(),
			selected_ip: "localhost".to_string(),
			selected_rpc: "18081".to_string(),
			selected_zmq: "18083".to_string(),
		}
	}
}
impl Xmrig {
	fn with_threads(max_threads: usize, current_threads: usize) -> Self {
		let xmrig = Self::default();
		Self {
			max_threads,
			current_threads,
			..xmrig
		}
	}
}
impl Default for Xmrig {
	fn default() -> Self {
		Self {
			simple: true,
			pause: 0,
			simple_rig: String::with_capacity(30),
			arguments: String::with_capacity(300),
			address: String::with_capacity(96),
			name: "Local P2Pool".to_string(),
			rig: GUPAX_VERSION_UNDERSCORE.to_string(),
			ip: "localhost".to_string(),
			port: "3333".to_string(),
			selected_index: 0,
			selected_name: "Local P2Pool".to_string(),
			selected_ip: "localhost".to_string(),
			selected_rig: GUPAX_VERSION_UNDERSCORE.to_string(),
			selected_port: "3333".to_string(),
			api_ip: "localhost".to_string(),
			api_port: "18088".to_string(),
			tls: false,
			keepalive: false,
			current_threads: 1,
			max_threads: 1,
		}
	}
}
impl Default for Version {
	fn default() -> Self {
		Self {
			gupax: GUPAX_VERSION.to_string(),
			p2pool: P2POOL_VERSION.to_string(),
			xmrig: XMRIG_VERSION.to_string(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
	#[test]
	fn serde_default_state() {
		let state = crate::State::new();
		let string = crate::State::to_string(&state).unwrap();
		crate::State::from_str(&string).unwrap();
	}
	#[test]
	fn serde_default_node() {
		let node = crate::Node::new_vec();
		let string = crate::Node::to_string(&node).unwrap();
		crate::Node::from_str_to_vec(&string).unwrap();
	}
	#[test]
	fn serde_default_pool() {
		let pool = crate::Pool::new_vec();
		let string = crate::Pool::to_string(&pool).unwrap();
		crate::Pool::from_str_to_vec(&string).unwrap();
	}

	#[test]
	fn serde_custom_state() {
		let state = r#"
			[gupax]
			simple = true
			auto_update = true
			auto_p2pool = false
			auto_xmrig = false
			ask_before_quit = true
			save_before_quit = true
			update_via_tor = true
			p2pool_path = "p2pool/p2pool"
			xmrig_path = "xmrig/xmrig"
			absolute_p2pool_path = "/home/hinto/p2pool/p2pool"
			absolute_xmrig_path = "/home/hinto/xmrig/xmrig"
			selected_width = 1280
			selected_height = 960
			tab = "About"
			ratio = "Width"

			[p2pool]
			simple = true
			mini = true
			auto_ping = true
			auto_select = true
			out_peers = 10
			in_peers = 450
			log_level = 3
			node = "Seth"
			arguments = ""
			address = "44hintoFpuo3ugKfcqJvh5BmrsTRpnTasJmetKC4VXCt6QDtbHVuixdTtsm6Ptp7Y8haXnJ6j8Gj2dra8CKy5ewz7Vi9CYW"
			name = "Local Monero Node"
			ip = "192.168.1.123"
			rpc = "18089"
			zmq = "18083"
			selected_index = 0
			selected_name = "Local Monero Node"
			selected_ip = "192.168.1.123"
			selected_rpc = "18089"
			selected_zmq = "18083"

			[xmrig]
			simple = true
			pause = 0
			simple_rig = ""
			arguments = ""
			tls = false
			keepalive = false
			max_threads = 32
			current_threads = 16
			address = ""
			api_ip = "localhost"
			api_port = "18088"
			name = "linux"
			rig = "Gupax"
			ip = "192.168.1.122"
			port = "3333"
			selected_index = 1
			selected_name = "linux"
			selected_rig = "Gupax"
			selected_ip = "192.168.1.122"
			selected_port = "3333"

			[version]
			gupax = "v1.0.0"
			p2pool = "v2.5"
			xmrig = "v6.18.0"
		"#;
		let state = crate::State::from_str(state).unwrap();
		crate::State::to_string(&state).unwrap();
	}

	#[test]
	fn serde_custom_node() {
		let node = r#"
			['Local Monero Node']
			ip = "localhost"
			rpc = "18081"
			zmq = "18083"

			['asdf-_. ._123']
			ip = "localhost"
			rpc = "11"
			zmq = "1234"

			['aaa     bbb']
			ip = "192.168.2.333"
			rpc = "1"
			zmq = "65535"
		"#;
		let node = crate::Node::from_str_to_vec(node).unwrap();
		crate::Node::to_string(&node).unwrap();
	}

	#[test]
	fn serde_custom_pool() {
		let pool = r#"
			['Local P2Pool']
			rig = "Gupax_v1.0.0"
			ip = "localhost"
			port = "3333"

			['aaa xx .. -']
			rig = "Gupax"
			ip = "192.168.22.22"
			port = "1"

			['           a']
			rig = "Gupax_v1.0.0"
			ip = "127.0.0.1"
			port = "65535"
		"#;
		let pool = crate::Pool::from_str_to_vec(pool).unwrap();
		crate::Pool::to_string(&pool).unwrap();
	}

	// Make sure we keep the user's old values that are still
	// valid but discard the ones that don't exist anymore.
	#[test]
	fn merge_state() {
		let bad_state = r#"
			[gupax]
			SETTING_THAT_DOESNT_EXIST_ANYMORE = 123123
			simple = false
			auto_update = true
			auto_p2pool = false
			auto_xmrig = false
			ask_before_quit = true
			save_before_quit = true
			update_via_tor = true
			p2pool_path = "p2pool/p2pool"
			xmrig_path = "xmrig/xmrig"
			absolute_p2pool_path = ""
			absolute_xmrig_path = ""
			selected_width = 0
			selected_height = 0
			tab = "About"
			ratio = "Width"

			[p2pool]
			SETTING_THAT_DOESNT_EXIST_ANYMORE = "String"
			simple = true
			mini = true
			auto_ping = true
			auto_select = true
			out_peers = 10
			in_peers = 450
			log_level = 6
			node = "Seth"
			arguments = ""
			address = "44hintoFpuo3ugKfcqJvh5BmrsTRpnTasJmetKC4VXCt6QDtbHVuixdTtsm6Ptp7Y8haXnJ6j8Gj2dra8CKy5ewz7Vi9CYW"
			name = "Local Monero Node"
			ip = "localhost"
			rpc = "18081"
			zmq = "18083"
			selected_index = 0
			selected_name = "Local Monero Node"
			selected_ip = "localhost"
			selected_rpc = "18081"
			selected_zmq = "18083"

			[xmrig]
			SETTING_THAT_DOESNT_EXIST_ANYMORE = true
			simple = true
			pause = 0
			simple_rig = ""
			arguments = ""
			tls = false
			keepalive = false
			max_threads = 32
			current_threads = 16
			address = ""
			api_ip = "localhost"
			api_port = "18088"
			name = "Local P2Pool"
			rig = "Gupax_v1.0.0"
			ip = "localhost"
			port = "3333"
			selected_index = 0
			selected_name = "Local P2Pool"
			selected_rig = "Gupax_v1.0.0"
			selected_ip = "localhost"
			selected_port = "3333"

			[version]
			gupax = "v1.0.0"
			p2pool = "v2.5"
			xmrig = "v6.18.0"
		"#.to_string();
		let merged_state = crate::State::merge(&bad_state).unwrap();
		let merged_state = crate::State::to_string(&merged_state).unwrap();
		println!("{}", merged_state);
		assert!(merged_state.contains("simple = false"));
		assert!(merged_state.contains("in_peers = 450"));
		assert!(merged_state.contains("log_level = 6"));
		assert!(merged_state.contains(r#"node = "Seth""#));
		assert!(!merged_state.contains("SETTING_THAT_DOESNT_EXIST_ANYMORE"));
		assert!(merged_state.contains("44hintoFpuo3ugKfcqJvh5BmrsTRpnTasJmetKC4VXCt6QDtbHVuixdTtsm6Ptp7Y8haXnJ6j8Gj2dra8CKy5ewz7Vi9CYW"));
	}
}
