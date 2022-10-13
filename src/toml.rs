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
//   Toml/
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
use serde_derive::{Serialize,Deserialize};
use log::*;

//---------------------------------------------------------------------------------------------------- Impl
// Since [State] is already used in [main.rs] to represent
// working state, [Toml] is used to disk state.
impl Toml {
	pub fn default() -> Self {
		use crate::constants::{P2POOL_VERSION,XMRIG_VERSION};
		Self {
			gupax: Gupax {
				auto_update: true,
				ask_before_quit: true,
				p2pool_path: DEFAULT_P2POOL_PATH.to_string(),
				xmrig_path: DEFAULT_XMRIG_PATH.to_string(),
			},
			p2pool: P2pool {
				simple: true,
				mini: true,
				out_peers: 10,
				in_peers: 10,
				log_level: 3,
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
				threads: 1,
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

	pub fn get() -> Result<Toml, TomlError> {
		// Get OS data folder
		// Linux   | $XDG_DATA_HOME or $HOME/.local/share | /home/alice/.local/state
		// macOS   | $HOME/Library/Application Support    | /Users/Alice/Library/Application Support
		// Windows | {FOLDERID_RoamingAppData}            | C:\Users\Alice\AppData\Roaming
		let mut path = match dirs::data_dir() {
			Some(mut path) => {
				path.push(DIRECTORY);
				info!("{}, OS data path ... OK", path.display());
				path
			},
			None => { error!("Couldn't get OS PATH for data"); return Err(TomlError::Path(PATH_ERROR.to_string())) },
		};

		// Create directory
		fs::create_dir_all(&path)?;

		// Attempt to read file, create default if not found
		path.push(FILENAME);
		let file = match fs::read_to_string(&path) {
			Ok(file) => file,
			Err(err) => {
				error!("TOML not found, attempting to create default");
				let default = match toml::ser::to_string(&Toml::default()) {
						Ok(o) => { info!("TOML serialization ... OK"); o },
						Err(e) => { error!("Couldn't serialize default TOML file: {}", e); return Err(TomlError::Serialize(e)) },
				};
				fs::write(&path, default)?;
				info!("TOML write ... OK");
				fs::read_to_string(&path)?
			},
		};
		info!("TOML read ... OK");

		// Attempt to parse, return Result
		match toml::from_str(&file) {
			Ok(file) => { info!("TOML parse ... OK"); Ok(file) },
			Err(err) => { error!("Couldn't parse TOML file"); Err(TomlError::Parse(err)) },
		}
	}
}

impl Display for TomlError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use TomlError::*;
		match self {
			Io(err) => write!(f, "{} | {}", ERROR, err),
			Path(err) => write!(f, "{} | {}", ERROR, err),
			Parse(err) => write!(f, "{} | {}", ERROR, err),
			Serialize(err) => write!(f, "{} | {}", ERROR, err),
		}
	}
}

impl From<std::io::Error> for TomlError {
	fn from(err: std::io::Error) -> Self {
		TomlError::Io(err)
	}
}

fn main() {
	let state = match Toml::get() {
		Ok(state) => { println!("OK"); state },
		Err(err) => panic!(),
	};
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
	Parse(toml::de::Error),
	Serialize(toml::ser::Error),
}

//---------------------------------------------------------------------------------------------------- Structs
#[derive(Debug,Deserialize,Serialize)]
pub struct Toml {
	gupax: Gupax,
	p2pool: P2pool,
	xmrig: Xmrig,
	version: Version,
}

#[derive(Debug,Deserialize,Serialize)]
struct Gupax {
	auto_update: bool,
	ask_before_quit: bool,
	p2pool_path: String,
	xmrig_path: String,
}

#[derive(Debug,Deserialize,Serialize)]
struct P2pool {
	simple: bool,
	mini: bool,
	out_peers: u8,
	in_peers: u8,
	log_level: u8,
	monerod: String,
	rpc: u16,
	zmq: u16,
	address: String,
}

#[derive(Debug,Deserialize,Serialize)]
struct Xmrig {
	simple: bool,
	tls: bool,
	nicehash: bool,
	keepalive: bool,
	threads: u16,
	priority: u8,
	pool: String,
	address: String,
}

#[derive(Debug,Deserialize,Serialize)]
struct Version {
	p2pool: String,
	xmrig: String,
}
