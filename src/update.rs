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
//// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// This file contains all (most) of the code for updating.
// The main [Update] struct contains meta update information.
// It is held by the top [App] struct. Each package also gets
// a [Pkg] struct that only lasts as long as the download.
//
// An update is triggered by either:
//     a. user clicks update on [Gupax] tab
//     b. auto-update at startup

//---------------------------------------------------------------------------------------------------- Imports
use serde_derive::{Serialize,Deserialize};
use tokio::task::JoinHandle;
use std::time::Duration;
use std::sync::{Arc,Mutex};
use std::os::unix::fs::OpenOptionsExt;
use std::io::{Read,Write};
//use crate::{Name::*,State};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use anyhow::{anyhow,Error};
use arti_hyper::*;
use arti_client::{TorClient,TorClientConfig};
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tls_api::{TlsConnector, TlsConnectorBuilder};
use hyper::header::HeaderValue;
use hyper::{Client,Body,Request};
use hyper_tls::HttpsConnector;
use arti_hyper::*;
use log::*;
use crate::update::Name::*;
use std::path::PathBuf;
use crate::state::Version;
use crate::constants::GUPAX_VERSION;

// use tls_api_native_tls::{TlsConnector,TlsConnectorBuilder};

//---------------------------------------------------------------------------------------------------- Constants
// Package naming schemes:
// gupax  | gupax-vX.X.X-(windows|macos|linux)-x64.(zip|tar.gz)
// p2pool | p2pool-vX.X.X-(windows|macos|linux)-x64.(zip|tar.gz)
// xmrig  | xmrig-X.X.X-(msvc-win64|macos-x64|linux-static-x64).(zip|tar.gz)
//
// Download link = PREFIX + Version (found at runtime) + SUFFIX + Version + EXT
// Example: https://github.com/hinto-janaiyo/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-standalone-x64
//
// Exceptions (there are always exceptions...):
//   - XMRig doesn't have a [v], so it is [xmrig-6.18.0-...]
//   - XMRig separates the hash and signature
//   - P2Pool hashes are in UPPERCASE
//   - Gupax will be downloaded as a standalone binary (no decompression/extraction needed)

const GUPAX_METADATA: &'static str = "https://api.github.com/repos/hinto-janaiyo/gupax/releases/latest";
const P2POOL_METADATA: &'static str = "https://api.github.com/repos/SChernykh/p2pool/releases/latest";
const XMRIG_METADATA: &'static str = "https://api.github.com/repos/xmrig/xmrig/releases/latest";

const GUPAX_PREFIX: &'static str = "https://github.com/hinto-janaiyo/gupax/releases/download/";
const P2POOL_PREFIX: &'static str = "https://github.com/SChernykh/p2pool/releases/download/";
const XMRIG_PREFIX: &'static str = "https://github.com/xmrig/xmrig/releases/download/";

const GUPAX_SUFFIX: &'static str = "/gupax-";
const P2POOL_SUFFIX: &'static str = "/p2pool-";
const XMRIG_SUFFIX: &'static str = "/xmrig-";

const GUPAX_HASH: &'static str = "SHA256SUMS";
const P2POOL_HASH: &'static str = "sha256sums.txt.asc";
const XMRIG_HASH: &'static str = "SHA256SUMS";

#[cfg(target_os = "windows")]
const GUPAX_EXTENSION: &'static str = "-windows-standalone-x64.exe";
#[cfg(target_os = "windows")]
const P2POOL_EXTENSION: &'static str = "-windows-x64.zip";
#[cfg(target_os = "windows")]
const XMRIG_EXTENSION: &'static str = "-msvc-win64.zip";

#[cfg(target_os = "macos")]
const GUPAX_EXTENSION: &'static str = "-macos-standalone-x64";
#[cfg(target_os = "macos")]
const P2POOL_EXTENSION: &'static str = "-macos-x64.tar.gz";
#[cfg(target_os = "macos")]
const XMRIG_EXTENSION: &'static str = "-macos-x64.tar.gz";

#[cfg(target_os = "linux")]
const GUPAX_EXTENSION: &'static str = "-linux-standalone-x64";
#[cfg(target_os = "linux")]
const P2POOL_EXTENSION: &'static str = "-linux-x64.tar.gz";
#[cfg(target_os = "linux")]
const XMRIG_EXTENSION: &'static str = "-linux-static-x64.tar.gz";

// Some fake Curl/Wget user-agents because GitHub API requires one and a Tor browser
// user-agent might be fingerprintable without all the associated headers.
const FAKE_USER_AGENT: [&'static str; 50] = [
	"Wget/1.16.3",
	"Wget/1.17",
	"Wget/1.17.1",
	"Wget/1.18",
	"Wget/1.18",
	"Wget/1.19",
	"Wget/1.19.1",
	"Wget/1.19.2",
	"Wget/1.19.3",
	"Wget/1.19.4",
	"Wget/1.19.5",
	"Wget/1.20",
	"Wget/1.20.1",
	"Wget/1.20.2",
	"Wget/1.20.3",
	"Wget/1.21",
	"Wget/1.21.1",
	"Wget/1.21.2",
	"Wget/1.21.3",
	"curl/7.64.1",
	"curl/7.65.0",
	"curl/7.65.1",
	"curl/7.65.2",
	"curl/7.65.3",
	"curl/7.66.0",
	"curl/7.67.0",
	"curl/7.68.0",
	"curl/7.69.0",
	"curl/7.69.1",
	"curl/7.70.0",
	"curl/7.70.1",
	"curl/7.71.0",
	"curl/7.71.1",
	"curl/7.72.0",
	"curl/7.73.0",
	"curl/7.74.0",
	"curl/7.75.0",
	"curl/7.76.0",
	"curl/7.76.1",
	"curl/7.77.0",
	"curl/7.78.0",
	"curl/7.79.0",
	"curl/7.79.1",
	"curl/7.80.0",
	"curl/7.81.0",
	"curl/7.82.0",
	"curl/7.83.0",
	"curl/7.83.1",
	"curl/7.84.0",
	"curl/7.85.0",
];

const MSG_NONE: &'static str = "No update in progress";
const MSG_START: &'static str = "Starting update";
const MSG_TMP: &'static str = "Creating temporary directory";
const MSG_TOR: &'static str = "Creating Tor+HTTPS client";
const MSG_HTTPS: &'static str = "Creating HTTPS client";
const MSG_METADATA: &'static str = "Fetching package metadata";
const MSG_ARCHIVE: &'static str = "Downloading packages";

// These two are sequential and not async so no need for a constant message.
// The package in question will be known at runtime, so that will be printed.
//const MSG_EXTRACT: &'static str = "Extracting packages";
//const MSG_UPGRADE: &'static str = "Upgrading packages";

//---------------------------------------------------------------------------------------------------- Update struct/impl
// Contains values needed during update
// Progress bar structure:
// 10% | Create tmp directory and pkg list
// 15% | Create Tor/HTTPS client
// 15% | Download Metadata (x3)
// 30% | Download Archive (x3)
// 15% | Extract (x3)
// 15% | Upgrade (x3)

#[derive(Clone)]
pub struct Update {
	pub path_gupax: String, // Full path to current gupax
	pub path_p2pool: String, // Full path to current p2pool
	pub path_xmrig: String, // Full path to current xmrig
	pub tmp_dir: String, // Full path to temporary directory
	pub updating: Arc<Mutex<bool>>, // Is an update in progress?
	pub prog: Arc<Mutex<u8>>, // Holds the 0-100% progress bar number
	pub msg: Arc<Mutex<String>>, // Message to display on [Gupax] tab while updating
	pub tor: bool, // Is Tor enabled or not?
}

impl Update {
	// Takes in current paths from [State]
	pub fn new(path_p2pool: PathBuf, path_xmrig: PathBuf, tor: bool) -> Self {
		Self {
			path_gupax: crate::get_exe().unwrap(),
			path_p2pool: path_p2pool.display().to_string(),
			path_xmrig: path_xmrig.display().to_string(),
			tmp_dir: "".to_string(),
			updating: Arc::new(Mutex::new(false)),
			prog: Arc::new(Mutex::new(0)),
			msg: Arc::new(Mutex::new(MSG_NONE.to_string())),
			tor,
		}
	}

	// Get a temporary random folder
	// for package download contents
	// Will look like [/tmp/gupax_A1m98FN3fa/] on Unix
	pub fn get_tmp_dir() -> String {
		let rand_string: String = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(10)
			.map(char::from)
			.collect();
		let tmp = std::env::temp_dir();
		let tmp = format!("{}{}{}{}", tmp.display(), "/gupax_", rand_string, "/");
		tmp
	}

	// Get a HTTPS client. Uses [Arti] if Tor is enabled.
	// The base type looks something like [hyper::Client<...>].
	// This is then wrapped with the custom [ClientEnum] type to implement
	// returning either a [Tor+TLS|TLS-only] client AT RUNTIME BASED ON USER SETTINGS
	//     tor == true?  => return Tor client
	//     tor == false? => return normal TLS client
	//
	// Since functions that take generic INPUT are much easier to implement,
	// [get_response()] just takes a [hyper::Client<C>], which is passed to
	// it via deconstructing this [ClientEnum] with a match, like so:
	//     ClientEnum::Tor(T)   => get_reponse(... T ...)
	//     ClientEnum::Https(H) => get_reponse(... H ...)
	//
	pub async fn get_client(tor: bool) -> Result<ClientEnum, anyhow::Error> {
		if tor {
			let tor = TorClient::create_bootstrapped(TorClientConfig::default()).await?;
			let tls = tls_api_native_tls::TlsConnector::builder()?.build()?;
		    let connector = ArtiHttpConnector::new(tor, tls);
			let client = ClientEnum::Tor(Client::builder().build(connector));
			return Ok(client)
		} else {
			let mut connector = hyper_tls::HttpsConnector::new();
			connector.https_only(true);
			let client = ClientEnum::Https(Client::builder().build(connector));
			return Ok(client)
		}
	}

	// Download process:
	// 0. setup tor, client, http, etc
	// 1. fill vector with all enums
	// 2. loop over vec, download metadata
	// 3. if current == version, remove from vec
	// 4. loop over vec, download links
	// 5. extract, upgrade

	#[tokio::main]
	pub async fn start(update: Arc<Mutex<Self>>, version: Version) -> Result<(), anyhow::Error> {
		// Start
		*update.lock().unwrap().updating.lock().unwrap() = true;

		// Set progress bar
		*update.lock().unwrap().msg.lock().unwrap() = MSG_START.to_string();
		*update.lock().unwrap().prog.lock().unwrap() = 0;
		info!("Update | Init | {}...", *update.lock().unwrap().msg.lock().unwrap());

		// Get temporary directory
		*update.lock().unwrap().msg.lock().unwrap() = MSG_TMP.to_string();
		// Cannot lock Arc<Mutex> twice in same line
		// so there will be some intermediate variables.
		info!("Update | Init | {} ... {}%", MSG_TMP.to_string(), *update.lock().unwrap().prog.lock().unwrap());
		let tmp_dir = Self::get_tmp_dir();
		*update.lock().unwrap().prog.lock().unwrap() += 10;

		// Make Pkg vector
		let prog = update.lock().unwrap().prog.clone();
		let msg = update.lock().unwrap().msg.clone();
		let vec = vec![
			Pkg::new(Gupax, &tmp_dir, prog.clone(), msg.clone()),
			Pkg::new(P2pool, &tmp_dir, prog.clone(), msg.clone()),
			Pkg::new(Xmrig, &tmp_dir, prog.clone(), msg.clone()),
		];
		let mut handles: Vec<JoinHandle<()>> = vec![];

		// Create Tor/HTTPS client
		if update.lock().unwrap().tor {
			*update.lock().unwrap().msg.lock().unwrap() = MSG_TOR.to_string()
		} else {
			*update.lock().unwrap().msg.lock().unwrap() = MSG_HTTPS.to_string()
		}
		let prog = *update.lock().unwrap().prog.lock().unwrap();
		info!("Update | Init | {} ... {}%", *update.lock().unwrap().msg.lock().unwrap(), prog);
		let client = Self::get_client(update.lock().unwrap().tor).await?;
		*update.lock().unwrap().prog.lock().unwrap() += 15;

		// Loop for metadata
		*update.lock().unwrap().msg.lock().unwrap() = MSG_METADATA.to_string();
		info!("Update | Metadata | Starting metadata fetch...");
		for pkg in vec.iter() {
			// Clone data before sending to async
			let name = pkg.name.clone();
			let new_ver = Arc::clone(&pkg.new_ver);
			let prog = Arc::clone(&pkg.prog);
			let client = client.clone();
			let link = pkg.link_metadata.to_string();
			// Send to async
			let handle: JoinHandle<()> = tokio::spawn(async move {
				match client {
					ClientEnum::Tor(t) => Pkg::get_metadata(name, new_ver, prog, t, link).await,
					ClientEnum::Https(h) => Pkg::get_metadata(name, new_ver, prog, h, link).await,
				};
			});
			handles.push(handle);
		}
		// Unwrap async
		for handle in handles {
			handle.await?;
		}
		info!("Update | Metadata ... OK");

		// Loop for version comparison
		info!("Update | Compare | Starting version comparison...");
		let prog = update.lock().unwrap().prog.clone();
		let msg = update.lock().unwrap().msg.clone();
		let mut vec2 = vec![];
		for pkg in vec.iter() {
			let new_ver = pkg.new_ver.lock().unwrap().to_owned();
			match pkg.name {
				Gupax  => {
					if new_ver == GUPAX_VERSION {
						info!("Update | Compare | {} {} == {} ... SKIPPING", pkg.name, pkg.new_ver.lock().unwrap(), GUPAX_VERSION);
					} else {
						info!("Update | Compare | {} {} != {} ... ADDING", pkg.name, pkg.new_ver.lock().unwrap(), GUPAX_VERSION);
						vec2.push(pkg);
					}
				}
				P2pool => {
					if new_ver == version.p2pool {
						info!("Update | Compare | {} {} == {} ... SKIPPING", pkg.name, pkg.new_ver.lock().unwrap(), version.p2pool);
					} else {
						info!("Update | Compare | {} {} != {} ... ADDING", pkg.name, pkg.new_ver.lock().unwrap(), version.p2pool);
						vec2.push(pkg);
					}
				}
				Xmrig  => {
					if new_ver == GUPAX_VERSION {
						info!("Update | Compare | {} {} == {} ... SKIPPING", pkg.name, pkg.new_ver.lock().unwrap(), version.xmrig);
					} else {
						info!("Update | Compare | {} {} != {} ... ADDING", pkg.name, pkg.new_ver.lock().unwrap(), version.xmrig);
						vec2.push(pkg);
					}
				}
			}
		}

		// Loop for download
		let mut handles: Vec<JoinHandle<()>> = vec![];
		info!("Update | Download | Starting download...");
		for pkg in vec2.iter() {
			// Clone data before async
			let name = pkg.name.clone();
			let bytes = Arc::clone(&pkg.bytes);
			let prog = Arc::clone(&pkg.prog);
			let client = client.clone();
			let version = pkg.new_ver.lock().unwrap();
			let link;
			// Download link = PREFIX + Version (found at runtime) + SUFFIX + Version + EXT
			// Example: https://github.com/hinto-janaiyo/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-standalone-x64
			// XMRig doesn't have a [v], so slice it out
			if pkg.name == Name::Xmrig {
				link = pkg.link_prefix.to_string() + &version + &pkg.link_suffix + &version[1..] + &pkg.link_extension;
			// TODO FIX ME
			// This is temp link for v0.0.1 of [Gupax]
			} else if pkg.name == Name::Gupax {
				link = "https://github.com/hinto-janaiyo/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-x64".to_string()
			} else {
				link = pkg.link_prefix.to_string() + &version + &pkg.link_suffix + &version + &pkg.link_extension;
			}
			info!("Update | Download | {} ... {}", pkg.name, link);
			let request = Pkg::get_request(link)?;
			let handle: JoinHandle<()> = tokio::spawn(async move {
				match client {
					ClientEnum::Tor(t) => Pkg::get_bytes(name, bytes, prog, t, request).await,
					ClientEnum::Https(h) => Pkg::get_bytes(name, bytes, prog, h, request).await,
				};
			});
			handles.push(handle);
		}
		for handle in handles {
			handle.await?;
		}
		info!("Update | Download ... OK");

		// Write to disk, extract
		let tmp = Self::get_tmp_dir();
//		std::fs::OpenOptions::new().mode(0o700).create(true).write(true).open(&tmp);
		std::fs::create_dir(&tmp)?;
		info!("Update | Extract | Starting extraction...");
		for pkg in vec2.iter() {
			let tmp = tmp.to_string() + &pkg.name.to_string();
			if pkg.name == Name::Gupax {
				std::fs::write(tmp, pkg.bytes.lock().unwrap().as_ref())?;
			} else {
				tar::Archive::new(flate2::read::GzDecoder::new(pkg.bytes.lock().unwrap().as_ref())).unpack(tmp)?;
			}
			*pkg.prog.lock().unwrap() += 5;
			info!("Update | Extract | {} ... {}%", pkg.name, pkg.prog.lock().unwrap());
		}
		info!("Update | Extract ... OK");
		*update.lock().unwrap().updating.lock().unwrap() = false;
		std::process::exit(0);
		Ok(())
	}
}

#[derive(Debug,Clone)]
enum ClientEnum {
    Tor(Client<ArtiHttpConnector<tor_rtcompat::PreferredRuntime, tls_api_native_tls::TlsConnector>>),
    Https(Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>),
}

//---------------------------------------------------------------------------------------------------- Pkg struct/impl
#[derive(Debug)]
pub struct Pkg {
	name: Name,
	link_metadata: &'static str,
	link_prefix: &'static str,
	link_suffix: &'static str,
	link_extension: &'static str,
	tmp_dir: String,
	prog: Arc<Mutex<u8>>,
	msg: Arc<Mutex<String>>,
	bytes: Arc<Mutex<hyper::body::Bytes>>,
	old_ver: String,
	new_ver: Arc<Mutex<String>>,
}

impl Pkg {
	pub fn new(name: Name, tmp_dir: &String, prog: Arc<Mutex<u8>>, msg: Arc<Mutex<String>>) -> Self {
		let link_metadata = match name {
			Gupax => GUPAX_METADATA,
			P2pool => P2POOL_METADATA,
			Xmrig => XMRIG_METADATA,
		};
		let link_prefix = match name {
			Gupax => GUPAX_PREFIX,
			P2pool => P2POOL_PREFIX,
			Xmrig => XMRIG_PREFIX,
		};
		let link_suffix = match name {
			Gupax => GUPAX_SUFFIX,
			P2pool => P2POOL_SUFFIX,
			Xmrig => XMRIG_SUFFIX,
		};
		let link_extension = match name {
			Gupax => GUPAX_EXTENSION,
			P2pool => P2POOL_EXTENSION,
			Xmrig => XMRIG_EXTENSION,
		};
		Self {
			name,
			link_metadata,
			link_prefix,
			link_suffix,
			link_extension,
			tmp_dir: tmp_dir.to_string(),
			prog,
			msg,
			bytes: Arc::new(Mutex::new(bytes::Bytes::new())),
			old_ver: String::new(),
			new_ver: Arc::new(Mutex::new(String::new())),
		}
	}

	// Generate GET request based off input URI + fake user agent
	pub fn get_request(link: String) -> Result<Request<Body>, anyhow::Error> {
		let user_agent = FAKE_USER_AGENT[thread_rng().gen_range(0..50)];
		let request = Request::builder()
			.method("GET")
			.uri(link)
			.header(hyper::header::USER_AGENT, HeaderValue::from_static(user_agent))
			.body(Body::empty())?;
		Ok(request)
	}

	// Get metadata using [Generic hyper::client<C>] & [Request]
	// and change [version, prog] under an Arc<Mutex>
	pub async fn get_metadata<C>(name: Name, new_ver: Arc<Mutex<String>>, prog: Arc<Mutex<u8>>, client: Client<C>, link: String) -> Result<(), Error>
		where C: hyper::client::connect::Connect + Clone + Send + Sync + 'static, {
		// Retry [3] times if version is not [v*]
		let mut n = 0;
		while n < 3 {
			let request = Pkg::get_request(link.clone())?;
			let mut response = client.request(request).await?;
			let body = hyper::body::to_bytes(response.body_mut()).await?;
			let body: TagName = serde_json::from_slice(&body)?;
			if body.tag_name.starts_with('v') {
				*new_ver.lock().unwrap() = body.tag_name.clone();
				*prog.lock().unwrap() += 5;
				info!("Update | Metadata | {} {} ... {}%", name, body.tag_name, *prog.lock().unwrap());
				return Ok(())
			}
			warn!("Update | Metadata | {} metadata fetch failed, retry [{}/3]...", name, n);
			n += 1;
		}
		error!("Update | Metadata | {} metadata fetch failed", name);
		Err(anyhow!("Metadata fetch failed"))
	}

	// Takes a [Request], fills the appropriate [Pkg]
	// [bytes] field with the [Archive/Standalone]
	pub async fn get_bytes<C>(name: Name, bytes: Arc<Mutex<bytes::Bytes>>, prog: Arc<Mutex<u8>>, client: Client<C>, request: Request<Body>) -> Result<(), anyhow::Error>
		where C: hyper::client::connect::Connect + Clone + Send + Sync + 'static, {
		// GitHub sends a 302 redirect, so we must follow
		// the [Location] header... only if Reqwest had custom
		// connectors so I didn't have to manually do this...
		let response = client.request(request).await?;
		let request = Self::get_request(response.headers().get(hyper::header::LOCATION).unwrap().to_str()?.to_string())?;
		let response = client.request(request).await?;
		let body = hyper::body::to_bytes(response.into_body()).await?;
		*bytes.lock().unwrap() = body;
		*prog.lock().unwrap() += 10;
		info!("Update | Download | {} ... {}%", name, *prog.lock().unwrap());
		Ok(())
	}
}

// This inherits the value of [tag_name] from GitHub's JSON API
#[derive(Debug, Serialize, Deserialize)]
struct TagName {
	tag_name: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Name {
	Gupax,
	P2pool,
	Xmrig,
}

impl std::fmt::Display for Name {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}
