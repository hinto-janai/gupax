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
};
use log::*;

//---------------------------------------------------------------------------------------------------- Const
// State file
const ERROR: &str = "Disk error";
const PATH_ERROR: &str = "PATH for state directory could not be not found";
#[cfg(target_os = "windows")]
const DIRECTORY: &'static str = r#"Gupax\"#;
#[cfg(target_os = "macos")]
const DIRECTORY: &'static str = "Gupax/";
#[cfg(target_os = "linux")]
const DIRECTORY: &str = "gupax/";

#[cfg(target_os = "windows")]
pub const DEFAULT_P2POOL_PATH: &'static str = r"P2Pool\p2pool.exe";
#[cfg(target_family = "unix")]
pub const DEFAULT_P2POOL_PATH: &str = "p2pool/p2pool";
#[cfg(target_os = "windows")]
pub const DEFAULT_XMRIG_PATH: &'static str = r"XMRig\xmrig.exe";
#[cfg(target_family = "unix")]
pub const DEFAULT_XMRIG_PATH: &str = "xmrig/xmrig";

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
pub fn print_toml(toml: &str) {
	info!("{}", HORIZONTAL);
	for i in toml.lines() { info!("{}", i); }
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
impl State {
	pub fn new() -> Self {
		let max_threads = num_cpus::get();
		let current_threads = if max_threads == 1 { 1 } else { max_threads / 2 };
		Self {
			gupax: Gupax {
				simple: true,
				auto_update: true,
				auto_node: true,
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
			},
			p2pool: P2pool {
				simple: true,
				mini: true,
				auto_node: true,
				auto_select: true,
				out_peers: 10,
				in_peers: 10,
				log_level: 3,
				node: crate::NodeEnum::C3pool,
				arguments: String::new(),
				address: String::with_capacity(95),
				name: "Local Monero Node".to_string(),
				ip: "localhost".to_string(),
				rpc: "18081".to_string(),
				zmq: "18083".to_string(),
				selected_index: 0,
				selected_name: "Local Monero Node".to_string(),
				selected_ip: "localhost".to_string(),
				selected_rpc: "18081".to_string(),
				selected_zmq: "18083".to_string(),
			},
			xmrig: Xmrig {
				simple: true,
				pause: 0,
				config: String::with_capacity(100),
				address: String::with_capacity(95),
				name: "Local P2Pool".to_string(),
				rig: "Gupax".to_string(),
				ip: "localhost".to_string(),
				port: "3333".to_string(),
				selected_index: 0,
				selected_name: "Local P2Pool".to_string(),
				selected_ip: "localhost".to_string(),
				selected_rig: "Gupax".to_string(),
				selected_port: "3333".to_string(),
				api_ip: "localhost".to_string(),
				api_port: "18088".to_string(),
				tls: true,
				keepalive: true,
				current_threads,
				max_threads,
			},
			version: Arc::new(Mutex::new(Version {
				gupax: GUPAX_VERSION.to_string(),
				p2pool: P2POOL_VERSION.to_string(),
				xmrig: XMRIG_VERSION.to_string(),
			})),
		}
	}

	// Convert [&str] to [State]
	pub fn from_str(string: &str) -> Result<Self, TomlError> {
		match toml::de::from_str(string) {
			Ok(state) => {
				info!("State | Parse ... OK");
				print_toml(string);
				Ok(state)
			}
			Err(err) => {
				warn!("State | String -> State ... FAIL ... {}", err);
				Err(TomlError::Deserialize(err))
			},
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
				Self::merge(string, path)
			},
		}
	}

	// Completely overwrite current [state.toml]
	// with a new default version, and return [Self].
	pub fn create_new(path: &PathBuf) -> Result<Self, TomlError> {
		info!("State | Creating new default...");
		let new = Self::new();
		let string = match toml::ser::to_string(&new) {
				Ok(o) => o,
				Err(e) => { error!("State | Couldn't serialize default file: {}", e); return Err(TomlError::Serialize(e)) },
		};
		fs::write(path, &string)?;
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
				print_toml(&string);
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
	// Automatically overwrite current file.
	pub fn merge(old: String, path: &PathBuf) -> Result<Self, TomlError> {
		let default = match toml::ser::to_string(&Self::new()) {
			Ok(string) => { info!("State | Default TOML parse ... OK"); string },
			Err(err) => { error!("State | Couldn't parse default TOML into string"); return Err(TomlError::Serialize(err)) },
		};
		let mut new: Self = match Figment::new().merge(Toml::string(&old)).merge(Toml::string(&default)).extract() {
			Ok(new) => { info!("State | TOML merge ... OK"); new },
			Err(err) => { error!("State | Couldn't merge default + old TOML"); return Err(TomlError::Merge(err)) },
		};
		// Attempt save
		Self::save(&mut new, path)?;
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
			let node = Node {
				ip: values.get("ip").unwrap().as_str().unwrap().to_string(),
				rpc: values.get("rpc").unwrap().as_str().unwrap().to_string(),
				zmq: values.get("zmq").unwrap().as_str().unwrap().to_string(),
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
		fs::write(path, &string)?;
		info!("Node | Write ... OK");
		Ok(new)
	}

	// Save [Node] onto disk file [node.toml]
	pub fn save(vec: &[(String, Self)], path: &PathBuf) -> Result<(), TomlError> {
		info!("Node | Saving to disk...");
		let string = Self::to_string(vec)?;
		match fs::write(path, string) {
			Ok(_) => { info!("TOML save ... OK"); Ok(()) },
			Err(err) => { error!("Couldn't overwrite TOML file"); Err(TomlError::Io(err)) },
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
			rig: "Gupax".to_string(),
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
		for (key, values) in pools.iter() {
			let pool = Pool {
				rig: values.get("rig").unwrap().as_str().unwrap().to_string(),
				ip: values.get("ip").unwrap().as_str().unwrap().to_string(),
				port: values.get("port").unwrap().as_str().unwrap().to_string(),
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
		fs::write(path, &string)?;
		info!("Pool | Write ... OK");
		Ok(new)
	}

	pub fn save(vec: &[(String, Self)], path: &PathBuf) -> Result<(), TomlError> {
		info!("Pool | Saving to disk...");
		let string = Self::to_string(vec)?;
		match fs::write(path, string) {
			Ok(_) => { info!("TOML save ... OK"); Ok(()) },
			Err(err) => { error!("Couldn't overwrite TOML file"); Err(TomlError::Io(err)) },
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
	pub auto_node: bool,
	pub ask_before_quit: bool,
	pub save_before_quit: bool,
	pub update_via_tor: bool,
	pub p2pool_path: String,
	pub xmrig_path: String,
	pub absolute_p2pool_path: PathBuf,
	pub absolute_xmrig_path: PathBuf,
	pub selected_width: u16,
	pub selected_height: u16,
	pub ratio: Ratio,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct P2pool {
	pub simple: bool,
	pub mini: bool,
	pub auto_node: bool,
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
	pub config: String,
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
