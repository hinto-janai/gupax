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
use crate::constants::*;
use log::*;

//---------------------------------------------------------------------------------------------------- General functions for all [File]'s
// get_file_path()      | Return absolute path to OS data path + filename
// read_to_string()     | Convert the file at a given path into a [String]
// create_new()         | Write a default TOML Struct into the appropriate file (in OS data path)
// into_absolute_path() | Convert relative -> absolute path

pub fn get_os_data_path() -> Result<PathBuf, TomlError> {
	// Get OS data folder
	// Linux   | $XDG_DATA_HOME or $HOME/.local/share | /home/alice/.local/state
	// macOS   | $HOME/Library/Application Support    | /Users/Alice/Library/Application Support
	// Windows | {FOLDERID_RoamingAppData}            | C:\Users\Alice\AppData\Roaming
	let path = match dirs::data_dir() {
		Some(path) => {
			info!("OS | Data path ... OK");
			path
		},
		None => { error!("OS | Data path ... FAIL"); return Err(TomlError::Path(PATH_ERROR.to_string())) },
	};
	// Create directory
	fs::create_dir_all(&path)?;
	Ok(path)
}

pub fn get_file_path(file: File) -> Result<PathBuf, TomlError> {
	let name = File::name(&file);

	let mut path = match dirs::data_dir() {
		Some(mut path) => {
			path.push(DIRECTORY);
			info!("OS data path ... OK");
			path
		},
		None => { error!("Couldn't get OS PATH for data"); return Err(TomlError::Path(PATH_ERROR.to_string())) },
	};
	// Create directory
	fs::create_dir_all(&path)?;
	path.push(name);
	info!("{:?} | Path ... {}", file, path.display());
	Ok(path)
}

// Convert a [File] path to a [String]
pub fn read_to_string(file: File, path: &PathBuf) -> Result<String, TomlError> {
	match fs::read_to_string(&path) {
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

// Write [String] to console with [info!] surrounded by "---"
pub fn print_toml(toml: &String) {
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
		let current_threads;
		if max_threads == 1 { current_threads = 1; } else { current_threads = max_threads / 2; }
		Self {
			gupax: Gupax {
				auto_update: true,
				auto_node: true,
				ask_before_quit: true,
				save_before_quit: true,
				update_via_tor: true,
				p2pool_path: DEFAULT_P2POOL_PATH.to_string(),
				xmrig_path: DEFAULT_XMRIG_PATH.to_string(),
				absolute_p2pool_path: into_absolute_path(DEFAULT_P2POOL_PATH.to_string()).unwrap(),
				absolute_xmrig_path: into_absolute_path(DEFAULT_XMRIG_PATH.to_string()).unwrap(),
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
				selected_index: 1,
				selected_name: "Local Monero Node".to_string(),
				selected_ip: "localhost".to_string(),
				selected_rpc: "18081".to_string(),
				selected_zmq: "18083".to_string(),
			},
			xmrig: Xmrig {
				simple: true,
				tls: false,
				nicehash: false,
				keepalive: false,
				hugepages_jit: true,
				current_threads,
				max_threads,
				priority: 2,
				pool: "localhost:3333".to_string(),
				address: String::with_capacity(95),
			},
			version: Arc::new(Mutex::new(Version {
				p2pool: Arc::new(Mutex::new(P2POOL_VERSION.to_string())),
				xmrig: Arc::new(Mutex::new(XMRIG_VERSION.to_string())),
			})),
		}
	}

	// Convert [String] to [State]
	pub fn from_string(string: String) -> Result<Self, TomlError> {
		match toml::de::from_str(&string) {
			Ok(state) => {
				info!("State | Parse ... OK");
				print_toml(&string);
				Ok(state)
			}
			Err(err) => {
				error!("State | String -> State ... FAIL ... {}", err);
				Err(TomlError::Deserialize(err))
			},
		}
	}

	// Combination of multiple functions:
	//   1. Attempt to read file from path into [String]
	//      |_ Create a default file if not found
	//   2. Deserialize [String] into a proper [Struct]
	//      |_ Attempt to merge if deserialization fails
	pub fn get() -> Result<Self, TomlError> {
		// Read
		let file = File::State;
		let path = get_file_path(file)?;
		let string = match read_to_string(file, &path) {
			Ok(string) => string,
			// Create
			_ => {
				Self::create_new()?;
				read_to_string(file, &path)?
			},
		};
		// Deserialize
		Self::from_string(string)
	}

	// Completely overwrite current [state.toml]
	// with a new default version, and return [Self].
	pub fn create_new() -> Result<Self, TomlError> {
		info!("State | Creating new default...");
		let new = Self::new();
		let path = get_file_path(File::State)?;
		let string = match toml::ser::to_string(&new) {
				Ok(o) => { info!("State | Serialization ... OK"); o },
				Err(e) => { error!("State | Couldn't serialize default file: {}", e); return Err(TomlError::Serialize(e)) },
		};
		fs::write(&path, &string)?;
		info!("State | Write ... OK");
		Ok(new)
	}

	// Save [State] onto disk file [gupax.toml]
	pub fn save(&mut self) -> Result<(), TomlError> {
		info!("State | Saving to disk...");
		let path = get_file_path(File::State)?;
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
			Err(err) => { error!("State | Couldn't overwrite TOML file ... FAIL"); return Err(TomlError::Io(err)) },
		}
	}

	// Take [Self] as input, merge it with whatever the current [default] is,
	// leaving behind old keys+values and updating [default] with old valid ones.
	// Automatically overwrite current file.
	pub fn merge(old: &Self) -> Result<Self, TomlError> {
		info!("Starting TOML merge...");
		let old = match toml::ser::to_string(&old) {
			Ok(string) => { info!("Old TOML parse ... OK"); string },
			Err(err) => { error!("Couldn't parse old TOML into string"); return Err(TomlError::Serialize(err)) },
		};
		let default = match toml::ser::to_string(&Self::new()) {
			Ok(string) => { info!("Default TOML parse ... OK"); string },
			Err(err) => { error!("Couldn't parse default TOML into string"); return Err(TomlError::Serialize(err)) },
		};
		let mut new: Self = match Figment::new().merge(Toml::string(&old)).merge(Toml::string(&default)).extract() {
			Ok(new) => { info!("TOML merge ... OK"); new },
			Err(err) => { error!("Couldn't merge default + old TOML"); return Err(TomlError::Merge(err)) },
		};
		// Attempt save
		Self::save(&mut new)?;
		Ok(new)
	}
}

//---------------------------------------------------------------------------------------------------- [Node] Impl
impl Node {
	pub fn new() -> Self {
		Self {
			ip: String::new(),
			rpc: "18081".to_string(),
			zmq: "18083".to_string(),
		}
	}

	pub fn localhost() -> Self {
		Self {
			ip: "localhost".to_string(),
			rpc: "18081".to_string(),
			zmq: "18083".to_string(),
		}
	}

	pub fn new_vec() -> Vec<(String, Self)> {
		let mut vec = Vec::new();
		vec.push(("Local Monero Node".to_string(), Self::localhost()));
		vec
	}

	// Convert [String] to [Node] Vec
	pub fn from_string(string: String) -> Result<Vec<(String, Self)>, TomlError> {
		let nodes: toml::map::Map<String, toml::Value> = match toml::de::from_str(&string) {
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
//			println!("{:#?}", values.get("ip")); std::process::exit(0);
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
	pub fn into_string(vec: Vec<(String, Self)>) -> String {
		let mut toml = String::new();
		for (key, value) in vec.iter() {
			write!(
				toml,
				"[\'{}\']\nip = {:#?}\nrpc = {:#?}\nzmq = {:#?}\n",
				key,
				value.ip,
				value.rpc,
				value.zmq,
			);
		}
		toml
	}

	// Combination of multiple functions:
	//   1. Attempt to read file from path into [String]
	//      |_ Create a default file if not found
	//   2. Deserialize [String] into a proper [Struct]
	//      |_ Attempt to merge if deserialization fails
	pub fn get() -> Result<Vec<(String, Self)>, TomlError> {
		// Read
		let file = File::Node;
		let path = get_file_path(file)?;
		let string = match read_to_string(file, &path) {
			Ok(string) => string,
			// Create
			_ => {
				Self::create_new()?;
				read_to_string(file, &path)?
			},
		};
		// Deserialize
		Self::from_string(string)
	}

	// Completely overwrite current [node.toml]
	// with a new default version, and return [Vec<String, Self>].
	pub fn create_new() -> Result<Vec<(String, Self)>, TomlError> {
		info!("Node | Creating new default...");
		let new = Self::new_vec();
		let path = get_file_path(File::Node)?;
		let string = Self::into_string(Self::new_vec());
		fs::write(&path, &string)?;
		info!("Node | Write ... OK");
		Ok(new)
	}

//	// Save [State] onto disk file [gupax.toml]
//	pub fn save(&mut self) -> Result<(), TomlError> {
//		info!("Saving {:?} to disk...", self);
//		let path = get_file_path(File::State)?;
//		// Convert path to absolute
//		self.gupax.absolute_p2pool_path = into_absolute_path(self.gupax.p2pool_path.clone())?;
//		self.gupax.absolute_xmrig_path = into_absolute_path(self.gupax.xmrig_path.clone())?;
//		let string = match toml::ser::to_string(&self) {
//			Ok(string) => {
//				info!("TOML parse ... OK");
//				print_toml(&string);
//				string
//			},
//			Err(err) => { error!("Couldn't parse TOML into string"); return Err(TomlError::Serialize(err)) },
//		};
//		match fs::write(path, string) {
//			Ok(_) => { info!("TOML save ... OK"); Ok(()) },
//			Err(err) => { error!("Couldn't overwrite TOML file"); return Err(TomlError::Io(err)) },
//		}
//	}
//
//	// Take [Self] as input, merge it with whatever the current [default] is,
//	// leaving behind old keys+values and updating [default] with old valid ones.
//	// Automatically overwrite current file.
//	pub fn merge(old: &Self) -> Result<Self, TomlError> {
//		info!("Starting TOML merge...");
//		let old = match toml::ser::to_string(&old) {
//			Ok(string) => { info!("Old TOML parse ... OK"); string },
//			Err(err) => { error!("Couldn't parse old TOML into string"); return Err(TomlError::Serialize(err)) },
//		};
//		let default = match toml::ser::to_string(&Self::new()) {
//			Ok(string) => { info!("Default TOML parse ... OK"); string },
//			Err(err) => { error!("Couldn't parse default TOML into string"); return Err(TomlError::Serialize(err)) },
//		};
//		let mut new: Self = match Figment::new().merge(Toml::string(&old)).merge(Toml::string(&default)).extract() {
//			Ok(new) => { info!("TOML merge ... OK"); new },
//			Err(err) => { error!("Couldn't merge default + old TOML"); return Err(TomlError::Merge(err)) },
//		};
//		// Attempt save
//		Self::save(&mut new)?;
//		Ok(new)
//	}
}

//---------------------------------------------------------------------------------------------------- Custom Error [TomlError]
impl Display for TomlError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use TomlError::*;
		match self {
			Io(err)          => write!(f, "{}: {} | {}", ERROR, self, err),
			Path(err)        => write!(f, "{}: {} | {}", ERROR, self, err),
			Serialize(err)   => write!(f, "{}: {} | {}", ERROR, self, err),
			Deserialize(err) => write!(f, "{}: {} | {}", ERROR, self, err),
			Merge(err)       => write!(f, "{}: {} | {}", ERROR, self, err),
		}
	}
}

impl From<std::io::Error> for TomlError {
	fn from(err: std::io::Error) -> Self {
		TomlError::Io(err)
	}
}

//---------------------------------------------------------------------------------------------------- Const
// State file
const ERROR: &'static str = "Disk error";
const PATH_ERROR: &'static str = "PATH for state directory could not be not found";
#[cfg(target_os = "windows")]
const DIRECTORY: &'static str = r#"Gupax\"#;
#[cfg(target_os = "macos")]
const DIRECTORY: &'static str = "com.github.hinto-janaiyo.gupax/";
#[cfg(target_os = "linux")]
const DIRECTORY: &'static str = "gupax/";

#[cfg(target_os = "windows")]
pub const DEFAULT_P2POOL_PATH: &'static str = r"P2Pool\p2pool.exe";
#[cfg(target_family = "unix")]
pub const DEFAULT_P2POOL_PATH: &'static str = "p2pool/p2pool";
#[cfg(target_os = "windows")]
pub const DEFAULT_XMRIG_PATH: &'static str = r"XMRig\xmrig.exe";
#[cfg(target_family = "unix")]
pub const DEFAULT_XMRIG_PATH: &'static str = "xmrig/xmrig";

//---------------------------------------------------------------------------------------------------- Error Enum
#[derive(Debug)]
pub enum TomlError {
	Io(std::io::Error),
	Path(String),
	Serialize(toml::ser::Error),
	Deserialize(toml::de::Error),
	Merge(figment::Error),
}

//---------------------------------------------------------------------------------------------------- [File] Enum (for matching which file)
#[derive(Clone,Copy,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum File {
	State,
	Node,
}

impl File {
	fn name(&self) -> &'static str {
		match *self {
			Self::State => "state.toml",
			Self::Node => "node.toml",
		}
	}
}

//---------------------------------------------------------------------------------------------------- [Node] Struct
#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Node {
	pub ip: String,
	pub rpc: String,
	pub zmq: String,
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
	pub auto_update: bool,
	pub auto_node: bool,
	pub ask_before_quit: bool,
	pub save_before_quit: bool,
	pub update_via_tor: bool,
	pub p2pool_path: String,
	pub xmrig_path: String,
	pub absolute_p2pool_path: PathBuf,
	pub absolute_xmrig_path: PathBuf,
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
	pub selected_index: u16,
	pub selected_name: String,
	pub selected_ip: String,
	pub selected_rpc: String,
	pub selected_zmq: String,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Xmrig {
	pub simple: bool,
	pub tls: bool,
	pub nicehash: bool,
	pub keepalive: bool,
	pub hugepages_jit: bool,
	pub max_threads: usize,
	pub current_threads: usize,
	pub priority: u8,
	pub pool: String,
	pub address: String,
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Version {
	pub p2pool: Arc<Mutex<String>>,
	pub xmrig: Arc<Mutex<String>>,
}
