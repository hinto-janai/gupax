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

// This handles reading/parsing the state file: [gupax.toml]
// The TOML format is used. This struct hierarchy directly
// translates into the TOML parser:
//   State/
//   ├─ Gupax/
//   │  ├─ ...
//   ├─ P2pool/
//   │  ├─ ...
//   ├─ Xmrig/
//   │  ├─ ...
//   ├─ Version/
//      ├─ ...

use std::{fs,env};
use std::fmt::Display;
use std::path::{Path,PathBuf};
use std::result::Result;
use serde_derive::{Serialize,Deserialize};
use crate::constants::HORIZONTAL;
use log::*;

//---------------------------------------------------------------------------------------------------- Impl
impl State {
	pub fn default() -> Self {
		use crate::constants::{P2POOL_VERSION,XMRIG_VERSION};
		let max_threads = num_cpus::get();
		let current_threads;
		if max_threads == 1 { current_threads = 1; } else { current_threads = max_threads / 2; }
		Self {
			gupax: Gupax {
				auto_update: true,
				auto_node: true,
				ask_before_quit: true,
				save_before_quit: true,
				p2pool_path: DEFAULT_P2POOL_PATH.to_string(),
				xmrig_path: DEFAULT_XMRIG_PATH.to_string(),
			},
			p2pool: P2pool {
				simple: true,
				mini: true,
				out_peers: 10,
				in_peers: 10,
				log_level: 3,
				node: crate::NodeEnum::C3pool,
				monerod: "localhost".to_string(),
				rpc: 18081,
				zmq: 18083,
				address: "".to_string(),
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
				address: "".to_string(),
			},
			version: Version {
				p2pool: P2POOL_VERSION.to_string(),
				xmrig: XMRIG_VERSION.to_string(),
			},
		}
	}

	pub fn get_path() -> Result<PathBuf, TomlError> {
		// Get OS data folder
		// Linux   | $XDG_DATA_HOME or $HOME/.local/share | /home/alice/.local/state
		// macOS   | $HOME/Library/Application Support    | /Users/Alice/Library/Application Support
		// Windows | {FOLDERID_RoamingAppData}            | C:\Users\Alice\AppData\Roaming
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
		path.push(FILENAME);
		info!("TOML path ... {}", path.display());
		Ok(path)
	}

	// Attempts to read [gupax.toml] or
	// attempts to create if not found.
	pub fn read_or_create(path: PathBuf) -> Result<String, TomlError> {
		// Attempt to read file, create default if not found
		match fs::read_to_string(&path) {
			Ok(string) => {
				info!("TOML read ... OK");
				Ok(string)
			},
			Err(err) => {
				warn!("TOML not found, attempting to create default");
				let default = match toml::ser::to_string(&State::default()) {
						Ok(o) => { info!("TOML serialization ... OK"); o },
						Err(e) => { error!("Couldn't serialize default TOML file: {}", e); return Err(TomlError::Serialize(e)) },
				};
				fs::write(&path, &default)?;
				info!("TOML write ... OK");
				Ok(default)
			},
		}
	}

	// Attempt to parse from String
	pub fn parse(string: String) -> Result<State, TomlError> {
		match toml::de::from_str(&string) {
			Ok(toml) => {
				info!("TOML parse ... OK");
				info!("{}", HORIZONTAL);
				for i in string.lines() { info!("{}", i); }
				info!("{}", HORIZONTAL);
				Ok(toml)
			},
			Err(err) => { error!("Couldn't parse TOML from string"); Err(TomlError::Deserialize(err)) },
		}
	}

	// Last three functions combined
	// get_path() -> read_or_create() -> parse()
	pub fn get() -> Result<State, TomlError> {
		Self::parse(Self::read_or_create(Self::get_path()?)?)
	}

	// Save [State] onto disk file [gupax.toml]
	pub fn save(&self) -> Result<(), TomlError> {
		let path = Self::get_path()?;
		info!("Starting TOML overwrite...");
		let string = match toml::ser::to_string(&self) {
			Ok(string) => {
				info!("TOML parse ... OK");
				info!("{}", HORIZONTAL);
				for i in string.lines() { info!("{}", i); }
				info!("{}", HORIZONTAL);
				string
			},
			Err(err) => { error!("Couldn't parse TOML into string"); return Err(TomlError::Serialize(err)) },
		};
		match fs::write(path, string) {
			Ok(_) => { info!("TOML overwrite ... OK"); Ok(()) },
			Err(err) => { error!("Couldn't overwrite TOML file"); return Err(TomlError::Io(err)) },
		}
	}
}

impl Display for TomlError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use TomlError::*;
		match self {
			Io(err) => write!(f, "{} | {}", ERROR, err),
			Path(err) => write!(f, "{} | {}", ERROR, err),
			Serialize(err) => write!(f, "{} | {}", ERROR, err),
			Deserialize(err) => write!(f, "{} | {}", ERROR, err),
		}
	}
}

impl From<std::io::Error> for TomlError {
	fn from(err: std::io::Error) -> Self {
		TomlError::Io(err)
	}
}

//---------------------------------------------------------------------------------------------------- Const
const FILENAME: &'static str = "gupax.toml";
const ERROR: &'static str = "TOML Error";
const PATH_ERROR: &'static str = "PATH for state directory could not be not found";
#[cfg(target_os = "windows")]
const DIRECTORY: &'static str = "Gupax";
#[cfg(target_os = "macos")]
const DIRECTORY: &'static str = "Gupax";
#[cfg(target_os = "linux")]
const DIRECTORY: &'static str = "gupax";
#[cfg(target_os = "windows")]
const DEFAULT_P2POOL_PATH: &'static str = r"P2Pool\p2pool.exe";
#[cfg(target_os = "macos")]
const DEFAULT_P2POOL_PATH: &'static str = "P2Pool/p2pool";
#[cfg(target_os = "linux")]
const DEFAULT_P2POOL_PATH: &'static str = "p2pool/p2pool";
#[cfg(target_os = "windows")]
const DEFAULT_XMRIG_PATH: &'static str = r"XMRig\xmrig.exe";
#[cfg(target_os = "macos")]
const DEFAULT_XMRIG_PATH: &'static str = "XMRig/xmrig";
#[cfg(target_os = "linux")]
const DEFAULT_XMRIG_PATH: &'static str = "xmrig/xmrig";

//---------------------------------------------------------------------------------------------------- Error Enum
#[derive(Debug)]
pub enum TomlError {
	Io(std::io::Error),
	Path(String),
	Serialize(toml::ser::Error),
	Deserialize(toml::de::Error),
}

//---------------------------------------------------------------------------------------------------- Structs
#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct State {
	pub gupax: Gupax,
	pub p2pool: P2pool,
	pub xmrig: Xmrig,
	pub version: Version,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Gupax {
	pub auto_update: bool,
	pub auto_node: bool,
	pub ask_before_quit: bool,
	pub save_before_quit: bool,
	pub p2pool_path: String,
	pub xmrig_path: String,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct P2pool {
	pub simple: bool,
	pub mini: bool,
	pub out_peers: u16,
	pub in_peers: u16,
	pub log_level: u8,
	pub node: crate::node::NodeEnum,
	pub monerod: String,
	pub rpc: u16,
	pub zmq: u16,
	pub address: String,
//	pub config: String,
//	pub args: String,
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
//	pub config: String,
//	pub args: String,
}

#[derive(Clone,Eq,PartialEq,Debug,Deserialize,Serialize)]
pub struct Version {
	pub p2pool: String,
	pub xmrig: String,
}
