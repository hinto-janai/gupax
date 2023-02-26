// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022 hinto-janai
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
	human::*,
	constants::*,
	gupax::Ratio,
	Tab,
	xmr::*,
	macros::*,
};
use log::*;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

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

// File names
pub const STATE_TOML: &str = "state.toml";
pub const NODE_TOML: &str = "node.toml";
pub const POOL_TOML: &str = "pool.toml";

// P2Pool API
// Lives within the Gupax OS data directory.
// ~/.local/share/gupax/p2pool/
// ├─ payout_log  // Raw log lines of payouts received
// ├─ payout      // Single [u64] representing total payouts
// ├─ xmr         // Single [u64] representing total XMR mined in atomic units
#[cfg(target_os = "windows")]
pub const GUPAX_P2POOL_API_DIRECTORY:    &str = r"p2pool\";
#[cfg(target_family = "unix")]
pub const GUPAX_P2POOL_API_DIRECTORY:    &str = "p2pool/";
pub const GUPAX_P2POOL_API_LOG:       &str = "log";
pub const GUPAX_P2POOL_API_PAYOUT: &str = "payout";
pub const GUPAX_P2POOL_API_XMR:    &str = "xmr";
pub const GUPAX_P2POOL_API_FILE_ARRAY: [&str; 3] = [
	GUPAX_P2POOL_API_LOG,
	GUPAX_P2POOL_API_PAYOUT,
	GUPAX_P2POOL_API_XMR,
];

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
			let mut gupax_p2pool_dir = path.clone();
			gupax_p2pool_dir.push(GUPAX_P2POOL_API_DIRECTORY);
			create_gupax_p2pool_dir(&gupax_p2pool_dir)?;
			Ok(path)
		},
		None => { error!("OS | Data path ... FAIL"); Err(TomlError::Path(PATH_ERROR.to_string())) },
	}
}

pub fn set_unix_750_perms(path: &PathBuf) -> Result<(), TomlError> {
	#[cfg(target_os = "windows")]
	return Ok(());
	#[cfg(target_family = "unix")]
	match fs::set_permissions(path, fs::Permissions::from_mode(0o750)) {
		Ok(_) => { info!("OS | Unix 750 permissions on path [{}] ... OK", path.display()); Ok(()) },
		Err(e) => { error!("OS | Unix 750 permissions on path [{}] ... FAIL ... {}", path.display(), e); Err(TomlError::Io(e)) },
	}
}

pub fn set_unix_660_perms(path: &PathBuf) -> Result<(), TomlError> {
	#[cfg(target_os = "windows")]
	return Ok(());
	#[cfg(target_family = "unix")]
	match fs::set_permissions(path, fs::Permissions::from_mode(0o660)) {
		Ok(_) => { info!("OS | Unix 660 permissions on path [{}] ... OK", path.display()); Ok(()) },
		Err(e) => { error!("OS | Unix 660 permissions on path [{}] ... FAIL ... {}", path.display(), e); Err(TomlError::Io(e)) },
	}
}

pub fn get_gupax_p2pool_path(os_data_path: &PathBuf) -> PathBuf {
	let mut gupax_p2pool_dir = os_data_path.clone();
	gupax_p2pool_dir.push(GUPAX_P2POOL_API_DIRECTORY);
	gupax_p2pool_dir
}

pub fn create_gupax_dir(path: &PathBuf) -> Result<(), TomlError> {
	// Create Gupax directory
	match fs::create_dir_all(path) {
		Ok(_) => info!("OS | Create data path ... OK"),
		Err(e) => { error!("OS | Create data path ... FAIL ... {}", e); return Err(TomlError::Io(e)) },
	}
	set_unix_750_perms(path)
}

pub fn create_gupax_p2pool_dir(path: &PathBuf) -> Result<(), TomlError> {
	// Create Gupax directory
	match fs::create_dir_all(path) {
		Ok(_) => { info!("OS | Create Gupax-P2Pool API path [{}] ... OK", path.display()); Ok(()) },
		Err(e) => { error!("OS | Create Gupax-P2Pool API path [{}] ... FAIL ... {}", path.display(), e); Err(TomlError::Io(e)) },
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
			status: Status::default(),
			gupax: Gupax::default(),
			p2pool: P2pool::default(),
			xmrig: Xmrig::with_threads(max_threads, current_threads),
			version: arc_mut!(Version::default()),
		}
	}

	pub fn update_absolute_path(&mut self) -> Result<(), TomlError> {
		self.gupax.absolute_p2pool_path = into_absolute_path(self.gupax.p2pool_path.clone())?;
		self.gupax.absolute_xmrig_path = into_absolute_path(self.gupax.xmrig_path.clone())?;
		Ok(())
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

//---------------------------------------------------------------------------------------------------- Gupax-P2Pool API
#[derive(Clone,Debug)]
pub struct GupaxP2poolApi {
	pub log: String,           // Log file only containing full payout lines
	pub log_rev: String,       // Same as above but reversed based off lines
	pub payout: HumanNumber,   // Human-friendly display of payout count
	pub payout_u64: u64,       // [u64] version of above
	pub payout_ord: PayoutOrd, // Ordered Vec of payouts, see [PayoutOrd]
	pub payout_low: String,    // A pre-allocated/computed [String] of the above Vec from low payout to high
	pub payout_high: String,   // Same as above but high -> low
	pub xmr: AtomicUnit,       // XMR stored as atomic units
	pub path_log: PathBuf,     // Path to [log]
	pub path_payout: PathBuf,  // Path to [payout]
	pub path_xmr: PathBuf,     // Path to [xmr]
}

impl Default for GupaxP2poolApi { fn default() -> Self { Self::new() } }

impl GupaxP2poolApi {
	//---------------------------------------------------------------------------------------------------- Init, these pretty much only get called once
	pub fn new() -> Self {
		Self {
			log: String::new(),
			log_rev: String::new(),
			payout: HumanNumber::unknown(),
			payout_u64: 0,
			payout_ord: PayoutOrd::new(),
			payout_low: String::new(),
			payout_high: String::new(),
			xmr: AtomicUnit::new(),
			path_xmr: PathBuf::new(),
			path_payout: PathBuf::new(),
			path_log: PathBuf::new(),
		}
	}

	pub fn fill_paths(&mut self, gupax_p2pool_dir: &PathBuf) {
		let mut path_log    = gupax_p2pool_dir.clone();
		let mut path_payout = gupax_p2pool_dir.clone();
		let mut path_xmr    = gupax_p2pool_dir.clone();
		path_log.push(GUPAX_P2POOL_API_LOG);
		path_payout.push(GUPAX_P2POOL_API_PAYOUT);
		path_xmr.push(GUPAX_P2POOL_API_XMR);
		*self = Self {
			path_log,
			path_payout,
			path_xmr,
			..std::mem::take(self)
		};
	}

	pub fn create_all_files(gupax_p2pool_dir: &PathBuf) -> Result<(), TomlError> {
		use std::io::Write;
		for file in GUPAX_P2POOL_API_FILE_ARRAY {
			let mut path = gupax_p2pool_dir.clone();
			path.push(file);
			if path.exists() {
				info!("GupaxP2poolApi | [{}] already exists, skipping...", path.display());
				continue
			}
			match std::fs::File::create(&path) {
				Ok(mut f)  => {
					match file {
						GUPAX_P2POOL_API_PAYOUT|GUPAX_P2POOL_API_XMR => writeln!(f, "0")?,
						_ => (),
					}
					info!("GupaxP2poolApi | [{}] create ... OK", path.display());
				},
				Err(e) => { warn!("GupaxP2poolApi | [{}] create ... FAIL: {}", path.display(), e); return Err(TomlError::Io(e)) },
			}
		}
		Ok(())
	}

	pub fn read_all_files_and_update(&mut self) -> Result<(), TomlError> {
		let payout_u64 = match read_to_string(File::Payout, &self.path_payout)?.trim().parse::<u64>() {
			Ok(o)  => o,
			Err(e) => { warn!("GupaxP2poolApi | [payout] parse error: {}", e); return Err(TomlError::Parse("payout")) }
		};
 		let xmr = match read_to_string(File::Xmr, &self.path_xmr)?.trim().parse::<u64>() {
			Ok(o)  => AtomicUnit::from_u64(o),
			Err(e) => { warn!("GupaxP2poolApi | [xmr] parse error: {}", e); return Err(TomlError::Parse("xmr")) }
		};
		let payout = HumanNumber::from_u64(payout_u64);
		let log    = read_to_string(File::Log, &self.path_log)?;
		self.payout_ord.update_from_payout_log(&log);
		self.update_payout_strings();
		*self = Self {
			log,
			payout,
			payout_u64,
			xmr,
			..std::mem::take(self)
		};
		self.update_log_rev();
		Ok(())
	}

	// Completely delete the [p2pool] folder and create defaults.
	pub fn create_new(path: &PathBuf) -> Result<(), TomlError> {
		info!("GupaxP2poolApi | Deleting old folder at [{}]...", path.display());
		std::fs::remove_dir_all(&path)?;
		info!("GupaxP2poolApi | Creating new default folder at [{}]...", path.display());
		create_gupax_p2pool_dir(&path)?;
		Self::create_all_files(&path)?;
		Ok(())
	}

	//---------------------------------------------------------------------------------------------------- Live, functions that actually update/write live stats
	pub fn update_log_rev(&mut self) {
		let mut log_rev = String::with_capacity(self.log.len());
		for line in self.log.lines().rev() {
			log_rev.push_str(line);
			log_rev.push('\n');
		}
		self.log_rev = log_rev;
	}

	pub fn format_payout(date: &str, atomic_unit: &AtomicUnit, block: &HumanNumber) -> String {
		format!("{} | {} XMR | Block {}", date, atomic_unit, block)
	}

	pub fn append_log(&mut self, formatted_log_line: &str) {
		self.log.push_str(formatted_log_line);
		self.log.push('\n');
	}

	pub fn append_head_log_rev(&mut self, formatted_log_line: &str) {
		self.log_rev = format!("{}\n{}", formatted_log_line, self.log_rev);
	}

	pub fn update_payout_low(&mut self) {
		self.payout_ord.sort_payout_low_to_high();
		self.payout_low = self.payout_ord.to_string();
	}

	pub fn update_payout_high(&mut self) {
		self.payout_ord.sort_payout_high_to_low();
		self.payout_high = self.payout_ord.to_string();
	}

	pub fn update_payout_strings(&mut self) {
		self.update_payout_low();
		self.update_payout_high();
	}

	// Takes the (date, atomic_unit, block) and updates [self] and the [PayoutOrd]
	pub fn add_payout(&mut self, formatted_log_line: &str, date: String, atomic_unit: AtomicUnit, block: HumanNumber) {
		self.append_log(formatted_log_line);
		self.append_head_log_rev(formatted_log_line);
		self.payout_u64 += 1;
		self.payout = HumanNumber::from_u64(self.payout_u64);
		self.xmr = self.xmr.add_self(atomic_unit);
		self.payout_ord.push(date, atomic_unit, block);
		self.update_payout_strings();
	}

	pub fn write_to_all_files(&self, formatted_log_line: &str) -> Result<(), TomlError> {
		Self::disk_overwrite(&self.payout_u64.to_string(), &self.path_payout)?;
		Self::disk_overwrite(&self.xmr.to_string(), &self.path_xmr)?;
		Self::disk_append(formatted_log_line, &self.path_log)?;
		Ok(())
	}

	pub fn disk_append(formatted_log_line: &str, path: &PathBuf) -> Result<(), TomlError> {
		use std::io::Write;
		let mut file = match fs::OpenOptions::new().append(true).create(true).open(path) {
			Ok(f) => f,
			Err(e) => { error!("GupaxP2poolApi | Append [{}] ... FAIL: {}", path.display(), e); return Err(TomlError::Io(e)) },
		};
		match writeln!(file, "{}", formatted_log_line) {
			Ok(_) => { debug!("GupaxP2poolApi | Append [{}] ... OK", path.display()); Ok(()) },
			Err(e) => { error!("GupaxP2poolApi | Append [{}] ... FAIL: {}", path.display(), e); Err(TomlError::Io(e)) },
		}
	}

	pub fn disk_overwrite(string: &str, path: &PathBuf) -> Result<(), TomlError> {
		use std::io::Write;
		let mut file = match fs::OpenOptions::new().write(true).truncate(true).create(true).open(path) {
			Ok(f) => f,
			Err(e) => { error!("GupaxP2poolApi | Overwrite [{}] ... FAIL: {}", path.display(), e); return Err(TomlError::Io(e)) },
		};
		match writeln!(file, "{}", string) {
			Ok(_) => { debug!("GupaxP2poolApi | Overwrite [{}] ... OK", path.display()); Ok(()) },
			Err(e) => { error!("GupaxP2poolApi | Overwrite [{}] ... FAIL: {}", path.display(), e); Err(TomlError::Io(e)) },
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
	// State files
	State,       // state.toml   | Gupax state
	Node,        // node.toml    | P2Pool manual node selector
	Pool,        // pool.toml    | XMRig manual pool selector

	// Gupax-P2Pool API
	Log,    // log    | Raw log lines of P2Pool payouts received
	Payout, // payout | Single [u64] representing total payouts
	Xmr,    // xmr    | Single [u64] representing total XMR mined in atomic units
}

//---------------------------------------------------------------------------------------------------- [Submenu] enum for [Status] tab
#[derive(Clone,Copy,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum Submenu {
	Processes,
	P2pool,
}

impl Default for Submenu {
	fn default() -> Self {
		Self::Processes
	}
}

impl Display for Submenu {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		use Submenu::*;
		match self {
			P2pool => write!(f, "P2Pool"),
			_ => write!(f, "{:?}", self),
		}
	}
}

//---------------------------------------------------------------------------------------------------- [PayoutView] enum for [Status/P2Pool] tab
// The enum buttons for selecting which "view" to sort the payout log in.
#[derive(Clone,Copy,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum PayoutView {
	Latest,   // Shows the most recent logs first
	Oldest,   // Shows the oldest logs first
	Biggest,  // Shows highest to lowest payouts
	Smallest, // Shows lowest to highest payouts
}

impl PayoutView {
	fn new() -> Self {
		Self::Latest
	}
}

impl Default for PayoutView {
	fn default() -> Self {
		Self::new()
	}
}

impl Display for PayoutView {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

//---------------------------------------------------------------------------------------------------- [Hash] enum for [Status/P2Pool]
#[derive(Clone,Copy,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub enum Hash {
	Hash,
	Kilo,
	Mega,
	Giga,
}

impl Default for Hash {
	fn default() -> Self {
		Self::Hash
	}
}

impl Hash {
	pub fn convert_to_hash(f: f64, from: Self) -> f64 {
		match from {
			Self::Hash => f,
			Self::Kilo => f * 1_000.0,
			Self::Mega => f * 1_000_000.0,
			Self::Giga => f * 1_000_000_000.0,
		}
	}

	pub fn convert(f: f64, og: Self, new: Self) -> f64 {
		match og {
			Self::Hash => {
				match new {
					Self::Hash => f,
					Self::Kilo => f / 1_000.0,
					Self::Mega => f / 1_000_000.0,
					Self::Giga => f / 1_000_000_000.0,
				}
			},
			Self::Kilo => {
				match new {
					Self::Hash => f * 1_000.0,
					Self::Kilo => f,
					Self::Mega => f / 1_000.0,
					Self::Giga => f / 1_000_000.0,
				}
			},
			Self::Mega => {
				match new {
					Self::Hash => f * 1_000_000.0,
					Self::Kilo => f * 1_000.0,
					Self::Mega => f,
					Self::Giga => f / 1_000.0,
				}
			},
			Self::Giga => {
				match new {
					Self::Hash => f * 1_000_000_000.0,
					Self::Kilo => f * 1_000_000.0,
					Self::Mega => f * 1_000.0,
					Self::Giga => f,
				}
			},
		}
	}
}

impl Display for Hash {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Hash::Hash => write!(f, "Hash"),
			_ => write!(f, "{:?}hash", self),
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
	pub status: Status,
	pub gupax: Gupax,
	pub p2pool: P2pool,
	pub xmrig: Xmrig,
	pub version: Arc<Mutex<Version>>,
}

#[derive(Clone,PartialEq,Debug,Deserialize,Serialize)]
pub struct Status {
	pub submenu: Submenu,
	pub payout_view: PayoutView,
	pub monero_enabled: bool,
	pub manual_hash: bool,
	pub hashrate: f64,
	pub hash_metric: Hash,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Gupax {
	pub simple: bool,
	pub auto_update: bool,
	pub auto_p2pool: bool,
	pub auto_xmrig: bool,
//	pub auto_monero: bool,
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
	pub node: String,
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
impl Default for Status {
	fn default() -> Self {
		Self {
			submenu: Submenu::default(),
			payout_view: PayoutView::default(),
			monero_enabled: false,
			manual_hash: false,
			hashrate: 1.0,
			hash_metric: Hash::default(),
		}
	}
}

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
			node: crate::RemoteNode::new().to_string(),
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

			[status]
			submenu = "P2pool"
			payout_view = "Oldest"
			monero_enabled = true
			manual_hash = false
			hashrate = 1241.23
			hash_metric = "Hash"

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

	#[test]
	fn create_and_serde_gupax_p2pool_api() {
		use crate::disk::GupaxP2poolApi;
		use crate::regex::P2poolRegex;
		use crate::xmr::PayoutOrd;
		use crate::xmr::AtomicUnit;

		// Get API dir, fill paths.
		let mut api = GupaxP2poolApi::new();
		let mut path = crate::disk::get_gupax_data_path().unwrap();
		path.push(crate::disk::GUPAX_P2POOL_API_DIRECTORY);
		GupaxP2poolApi::fill_paths(&mut api, &path);
		println!("{:#?}", api);

		// Create, write some fake data.
		GupaxP2poolApi::create_all_files(&path).unwrap();
		api.log        = "NOTICE  2022-01-27 01:30:23.1377 P2Pool You received a payout of 0.000000000001 XMR in block 2642816".to_string();
		api.payout_u64 = 1;
		api.xmr        = AtomicUnit::from_u64(2);
		let (date, atomic_unit, block) = PayoutOrd::parse_raw_payout_line(&api.log, &P2poolRegex::new());
		let formatted_log_line = GupaxP2poolApi::format_payout(&date, &atomic_unit, &block);
		GupaxP2poolApi::write_to_all_files(&api, &formatted_log_line).unwrap();
		println!("AFTER WRITE: {:#?}", api);

		// Read
		GupaxP2poolApi::read_all_files_and_update(&mut api).unwrap();
		println!("AFTER READ: {:#?}", api);

		// Assert that the file read mutated the internal struct correctly.
		assert_eq!(api.payout_u64, 1);
		assert_eq!(api.xmr.to_u64(), 2);
		assert!(!api.payout_ord.is_empty());
		assert!(api.log.contains("2022-01-27 01:30:23.1377 | 0.000000000001 XMR | Block 2,642,816"));
	}

	#[test]
	fn convert_hash() {
		use crate::disk::Hash;
		let hash = 1.0;
		assert_eq!(Hash::convert(hash, Hash::Hash, Hash::Hash), 1.0);
		assert_eq!(Hash::convert(hash, Hash::Hash, Hash::Kilo), 0.001);
		assert_eq!(Hash::convert(hash, Hash::Hash, Hash::Mega), 0.000_001);
		assert_eq!(Hash::convert(hash, Hash::Hash, Hash::Giga), 0.000_000_001);
		let hash = 1.0;
		assert_eq!(Hash::convert(hash, Hash::Kilo, Hash::Hash), 1_000.0);
		assert_eq!(Hash::convert(hash, Hash::Kilo, Hash::Kilo), 1.0);
		assert_eq!(Hash::convert(hash, Hash::Kilo, Hash::Mega), 0.001);
		assert_eq!(Hash::convert(hash, Hash::Kilo, Hash::Giga), 0.000_001);
		let hash = 1.0;
		assert_eq!(Hash::convert(hash, Hash::Mega, Hash::Hash), 1_000_000.0);
		assert_eq!(Hash::convert(hash, Hash::Mega, Hash::Kilo), 1_000.0);
		assert_eq!(Hash::convert(hash, Hash::Mega, Hash::Mega), 1.0);
		assert_eq!(Hash::convert(hash, Hash::Mega, Hash::Giga), 0.001);
		let hash = 1.0;
		assert_eq!(Hash::convert(hash, Hash::Giga, Hash::Hash), 1_000_000_000.0);
		assert_eq!(Hash::convert(hash, Hash::Giga, Hash::Kilo), 1_000_000.0);
		assert_eq!(Hash::convert(hash, Hash::Giga, Hash::Mega), 1_000.0);
		assert_eq!(Hash::convert(hash, Hash::Giga, Hash::Giga), 1.0);
	}
}
