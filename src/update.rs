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
use anyhow::{anyhow,Error};
use arti_client::{TorClient,TorClientConfig};
use arti_hyper::*;
use arti_hyper::*;
use crate::constants::GUPAX_VERSION;
//use crate::{Name::*,State};
use crate::disk::*;
use crate::update::Name::*;
use hyper::{Client,Body,Request};
use hyper::header::HeaderValue;
use hyper_tls::HttpsConnector;
use hyper::header::LOCATION;
use log::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Serialize,Deserialize};
use std::io::{Read,Write};
use std::path::PathBuf;
use std::sync::{Arc,Mutex};
use std::time::Duration;
use tls_api::{TlsConnector, TlsConnectorBuilder};
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::task::JoinHandle;
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
use zip::ZipArchive;
#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

//---------------------------------------------------------------------------------------------------- Constants
// Package naming schemes:
// gupax  | gupax-vX.X.X-(windows|macos|linux)-x64(standalone|bundle).(zip|tar.gz)
// p2pool | p2pool-vX.X.X-(windows|macos|linux)-x64.(zip|tar.gz)
// xmrig  | xmrig-X.X.X-(msvc-win64|macos-x64|linux-static-x64).(zip|tar.gz)
//
// Download link = PREFIX + Version (found at runtime) + SUFFIX + Version + EXT
// Example: https://github.com/hinto-janaiyo/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-standalone-x64.tar.gz
//
// Exceptions (there are always exceptions...):
//   - XMRig doesn't have a [v], so it is [xmrig-6.18.0-...]
//   - XMRig separates the hash and signature
//   - P2Pool hashes are in UPPERCASE

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
const GUPAX_EXTENSION: &'static str = "-windows-x64-standalone.zip";
#[cfg(target_os = "windows")]
const P2POOL_EXTENSION: &'static str = "-windows-x64.zip";
#[cfg(target_os = "windows")]
const XMRIG_EXTENSION: &'static str = "-msvc-win64.zip";

#[cfg(target_os = "macos")]
const GUPAX_EXTENSION: &'static str = "-macos-x64-standalone.tar.gz";
#[cfg(target_os = "macos")]
const P2POOL_EXTENSION: &'static str = "-macos-x64.tar.gz";
#[cfg(target_os = "macos")]
const XMRIG_EXTENSION: &'static str = "-macos-x64.tar.gz";

#[cfg(target_os = "linux")]
const GUPAX_EXTENSION: &'static str = "-linux-x64-standalone.tar.gz";
#[cfg(target_os = "linux")]
const P2POOL_EXTENSION: &'static str = "-linux-x64.tar.gz";
#[cfg(target_os = "linux")]
const XMRIG_EXTENSION: &'static str = "-linux-static-x64.tar.gz";

#[cfg(target_os = "windows")]
const GUPAX_BINARY: &'static str = "gupax.exe";
#[cfg(target_family = "unix")]
const GUPAX_BINARY: &'static str = "gupax";

#[cfg(target_os = "windows")]
const P2POOL_BINARY: &'static str = "p2pool.exe";
#[cfg(target_family = "unix")]
const P2POOL_BINARY: &'static str = "p2pool";

#[cfg(target_os = "windows")]
const XMRIG_BINARY: &'static str = "xmrig.exe";
#[cfg(target_family = "unix")]
const XMRIG_BINARY: &'static str = "xmrig";

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
const MSG_METADATA_RETRY: &'static str = "Fetching package metadata failed, attempt";
const MSG_COMPARE: &'static str = "Compare package versions";
const MSG_UP_TO_DATE: &'static str = "All packages already up-to-date";
const MSG_DOWNLOAD: &'static str = "Downloading packages";
const MSG_DOWNLOAD_RETRY: &'static str = "Downloading packages failed, attempt";
const MSG_EXTRACT: &'static str = "Extracting packages";
const MSG_UPGRADE: &'static str = "Upgrading packages";
pub const MSG_SUCCESS: &'static str = "Update successful";
pub const MSG_FAILED: &'static str = "Update failed";

const INIT: &'static str = "------------------- Init -------------------";
const METADATA: &'static str = "----------------- Metadata -----------------";
const COMPARE: &'static str = "----------------- Compare ------------------";
const DOWNLOAD: &'static str = "----------------- Download -----------------";
const EXTRACT: &'static str = "----------------- Extract ------------------";
const UPGRADE: &'static str = "----------------- Upgrade ------------------";

// These two are sequential and not async so no need for a constant message.
// The package in question will be known at runtime, so that will be printed.
//const MSG_EXTRACT: &'static str = "Extracting packages";
//const MSG_UPGRADE: &'static str = "Upgrading packages";

//---------------------------------------------------------------------------------------------------- Update struct/impl
// Contains values needed during update
// Progress bar structure:
// 0%  | Start
// 5%  | Create tmp directory, pkg list, fake user-agent
// 5%  | Create Tor/HTTPS client
// 30% | Download Metadata (x3)
// 5%  | Compare Versions (x3)
// 30% | Download Archive (x3)
// 5%  | Extract (x3)
// 5%  | Upgrade (x3)

#[derive(Clone)]
pub struct Update {
	pub path_gupax: String, // Full path to current gupax
	pub path_p2pool: String, // Full path to current p2pool
	pub path_xmrig: String, // Full path to current xmrig
	pub tmp_dir: String, // Full path to temporary directory
	pub updating: Arc<Mutex<bool>>, // Is an update in progress?
	pub prog: Arc<Mutex<f32>>, // Holds the 0-100% progress bar number
	pub msg: Arc<Mutex<String>>, // Message to display on [Gupax] tab while updating
	pub tor: bool, // Is Tor enabled or not?
}

impl Update {
	// Takes in current paths from [State]
	pub fn new(path_gupax: String, path_p2pool: PathBuf, path_xmrig: PathBuf, tor: bool) -> Self {
		Self {
			path_gupax,
			path_p2pool: path_p2pool.display().to_string(),
			path_xmrig: path_xmrig.display().to_string(),
			tmp_dir: "".to_string(),
			updating: Arc::new(Mutex::new(false)),
			prog: Arc::new(Mutex::new(0.0)),
			msg: Arc::new(Mutex::new(MSG_NONE.to_string())),
			tor,
		}
	}

	// Get a temporary random folder for package download contents
	// This used to use [std::env::temp_dir()] but there were issues
	// using [std::fs::rename()] on tmpfs -> disk (Invalid cross-device link (os error 18)).
	// So, uses the [Gupax] binary directory as a base, something like [/home/hinto/gupax/gupax_tmp_SG4xsDdVmr]
	pub fn get_tmp_dir() -> Result<String, anyhow::Error> {
		let rand_string: String = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(10)
			.map(char::from)
			.collect();
		let base = crate::get_exe_dir()?;
		#[cfg(target_os = "windows")]
		let tmp_dir = format!("{}{}{}{}", base, r"\gupax_update_", rand_string, r"\");
		#[cfg(target_family = "unix")]
		let tmp_dir = format!("{}{}{}{}", base, "/gupax_update_", rand_string, "/");
		info!("Update | Temporary directory ... {}", tmp_dir);
		Ok(tmp_dir)
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
			// This one below is non-async, but it doesn't bootstrap immediately.
//			let tor = TorClient::builder().bootstrap_behavior(arti_client::BootstrapBehavior::OnDemand).create_unbootstrapped()?;
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
	pub async fn start(update: Arc<Mutex<Self>>, og_ver: Arc<Mutex<Version>>, state_ver: Arc<Mutex<Version>>) -> Result<(), anyhow::Error> {
		//---------------------------------------------------------------------------------------------------- Init
		*update.lock().unwrap().updating.lock().unwrap() = true;
		// Set timer
		let now = std::time::Instant::now();

		// Set progress bar
		*update.lock().unwrap().msg.lock().unwrap() = MSG_START.to_string();
		*update.lock().unwrap().prog.lock().unwrap() = 0.0;
		info!("Update | {}", INIT);

		// Get temporary directory
		*update.lock().unwrap().msg.lock().unwrap() = MSG_TMP.to_string();
		// Cannot lock Arc<Mutex> twice in same line
		// so there will be some intermediate variables.
		info!("Update | {}", MSG_TMP.to_string());
		let tmp_dir = Self::get_tmp_dir()?;
		std::fs::create_dir(&tmp_dir)?;

		// Make Pkg vector
		let prog = update.lock().unwrap().prog.clone();
		let msg = update.lock().unwrap().msg.clone();
		let mut vec = vec![
			Pkg::new(Gupax, &tmp_dir, prog.clone(), msg.clone()),
			Pkg::new(P2pool, &tmp_dir, prog.clone(), msg.clone()),
			Pkg::new(Xmrig, &tmp_dir, prog.clone(), msg.clone()),
		];

		// Generate fake user-agent
		let user_agent = Pkg::get_user_agent();
		*update.lock().unwrap().prog.lock().unwrap() = 5.0;

		// Create Tor/HTTPS client
		if update.lock().unwrap().tor {
			*update.lock().unwrap().msg.lock().unwrap() = MSG_TOR.to_string()
		} else {
			*update.lock().unwrap().msg.lock().unwrap() = MSG_HTTPS.to_string()
		}
		let prog = *update.lock().unwrap().prog.lock().unwrap();
		info!("Update | {}", update.lock().unwrap().msg.lock().unwrap());
		let tor = update.lock().unwrap().tor;
		let client = Self::get_client(tor).await?;
		*update.lock().unwrap().prog.lock().unwrap() += 5.0;
		info!("Update | Init ... OK ... {}%", prog);

		//---------------------------------------------------------------------------------------------------- Metadata
		*update.lock().unwrap().msg.lock().unwrap() = MSG_METADATA.to_string();
		info!("Update | {}", METADATA);
		let mut vec2 = vec![];
		// Loop process:
		// 1. Start all async metadata fetches
		// 2. Wait for all to finish
		// 3. Iterate over all [pkg.new_ver], check if empty
		// 4. If not empty, move [pkg] to different vec
		// 5. At end, if original vec isn't empty, that means something failed
		// 6. Redo loop [3] times, with the original vec (that now only has the failed pkgs)
		//
		// This logic was originally in the [Pkg::get_metadata()]
		// function itself but for some reason, it was getting skipped over,
		// so the [new_ver] check is now here, in the outer scope.
		for i in 1..=3 {
			if i > 1 { *update.lock().unwrap().msg.lock().unwrap() = format!("{} [{}/3]", MSG_METADATA_RETRY.to_string(), i); }
			let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
			for pkg in vec.iter() {
				// Clone data before sending to async
				let name = pkg.name.clone();
				let new_ver = Arc::clone(&pkg.new_ver);
				let client = client.clone();
				let link = pkg.link_metadata.to_string();
				// Send to async
				let handle: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
					match client {
						ClientEnum::Tor(t) => Pkg::get_metadata(name, new_ver, t, link, user_agent).await,
						ClientEnum::Https(h) => Pkg::get_metadata(name, new_ver, h, link, user_agent).await,
					}
				});
				handles.push(handle);
			}
			// Handle await
			for handle in handles {
				// Two [??] will send the error.
				// We don't actually want to return the error here since we
				// prefer looping and retrying over immediately erroring out.
				match handle.await? {
					Err(e) => warn!("Update | {}", e),
					_ => (),
				}
			}
			// Check for empty version
			let mut indexes = vec![];
			for (index, pkg) in vec.iter().enumerate() {
				if pkg.new_ver.lock().unwrap().is_empty() {
					warn!("Update | {} failed, attempt [{}/3]...", pkg.name, i);
				} else {
					indexes.push(index);
					vec2.push(pkg.clone());
					*update.lock().unwrap().prog.lock().unwrap() += 10.0;
					info!("Update | {} {} ... OK", pkg.name, pkg.new_ver.lock().unwrap());
				}
			}
			// Order indexes from biggest to smallest
			// This prevents shifting the whole vector and causing panics.
			indexes.sort();
			indexes.reverse();
			for index in indexes {
				vec.remove(index);
			}
			if vec.is_empty() { break }
		}
		if vec.is_empty() {
			info!("Update | Metadata ... OK ... {}%", update.lock().unwrap().prog.lock().unwrap());
		} else {
			error!("Update | Metadata ... FAIL");
			return Err(anyhow!("Metadata fetch failed", ))
		}

		//---------------------------------------------------------------------------------------------------- Compare
		*update.lock().unwrap().msg.lock().unwrap() = MSG_COMPARE.to_string();
		info!("Update | {}", COMPARE);
		let prog = update.lock().unwrap().prog.clone();
		let msg = update.lock().unwrap().msg.clone();
		let mut vec3 = vec![];
		let mut new_pkgs = vec![];
		for pkg in vec2.iter() {
			let new_ver = pkg.new_ver.lock().unwrap().to_owned();
			match pkg.name {
				Gupax  => {
					if new_ver == GUPAX_VERSION {
						info!("Update | {} {} == {} ... SKIPPING", pkg.name, GUPAX_VERSION, new_ver);
					} else {
						info!("Update | {} {} != {} ... ADDING", pkg.name, GUPAX_VERSION, new_ver);
						new_pkgs.push(format!("\nGupax {}  ➡  {}", GUPAX_VERSION, new_ver));
						vec3.push(pkg);
					}
				}
				P2pool => {
					let old_ver = og_ver.lock().unwrap().p2pool.lock().unwrap().to_owned();
					if old_ver == new_ver {
						info!("Update | {} {} == {} ... SKIPPING", pkg.name, old_ver, new_ver);
					} else {
						info!("Update | {} {} != {} ... ADDING", pkg.name, old_ver, new_ver);
						new_pkgs.push(format!("\nP2Pool {}  ➡  {}", old_ver, new_ver));
						vec3.push(pkg);
					}
				}
				Xmrig  => {
					let old_ver = og_ver.lock().unwrap().xmrig.lock().unwrap().to_owned();
					if old_ver == new_ver {
						info!("Update | {} {} == {} ... SKIPPING", pkg.name, old_ver, new_ver);
					} else {
						info!("Update | {} {} != {} ... ADDING", pkg.name, old_ver, new_ver);
						new_pkgs.push(format!("\nXMRig {}  ➡  {}", old_ver, new_ver));
						vec3.push(pkg);
					}
				}
			}
		}
		*update.lock().unwrap().prog.lock().unwrap() += 5.0;
		info!("Update | Compare ... OK ... {}%", update.lock().unwrap().prog.lock().unwrap());
		// Return if 0 (all packages up-to-date)
		// Get amount of packages to divide up the percentage increases
		let pkg_amount = vec3.len() as f32;
		if pkg_amount == 0.0 {
			info!("Update | All packages up-to-date ... RETURNING");
			*update.lock().unwrap().prog.lock().unwrap() = 100.0;
			*update.lock().unwrap().msg.lock().unwrap() = MSG_UP_TO_DATE.to_string();
			return Ok(())
		}
		let new_pkgs: String = new_pkgs.concat();

		//---------------------------------------------------------------------------------------------------- Download
		*update.lock().unwrap().msg.lock().unwrap() = format!("{}{}", MSG_DOWNLOAD, new_pkgs);
		info!("Update | {}", DOWNLOAD);
		let mut vec4 = vec![];
		for i in 1..=3 {
			if i > 1 { *update.lock().unwrap().msg.lock().unwrap() = format!("{} [{}/3]{}", MSG_DOWNLOAD_RETRY.to_string(), i, new_pkgs); }
			let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
			for pkg in vec3.iter() {
				// Clone data before async
				let name = pkg.name.clone();
				let bytes = Arc::clone(&pkg.bytes);
				let client = client.clone();
				let version = pkg.new_ver.lock().unwrap();
				let link;
				// Download link = PREFIX + Version (found at runtime) + SUFFIX + Version + EXT
				// Example: https://github.com/hinto-janaiyo/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-x64-standalone
				// XMRig doesn't have a [v], so slice it out
				if pkg.name == Name::Xmrig {
					link = pkg.link_prefix.to_string() + &version + &pkg.link_suffix + &version[1..] + &pkg.link_extension;
				} else {
					link = pkg.link_prefix.to_string() + &version + &pkg.link_suffix + &version + &pkg.link_extension;
				}
				info!("Update | {} ... {}", pkg.name, link);
				let handle: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
					match client {
						ClientEnum::Tor(t) => Pkg::get_bytes(name, bytes, t, link, user_agent).await,
						ClientEnum::Https(h) => Pkg::get_bytes(name, bytes, h, link, user_agent).await,
					}
				});
				handles.push(handle);
			}
			// Handle await
			for handle in handles {
				match handle.await? {
					Err(e) => warn!("Update | {}", e),
					_ => (),
				}
			}
			// Check for empty bytes
			let mut indexes = vec![];
			for (index, pkg) in vec3.iter().enumerate() {
				if pkg.bytes.lock().unwrap().is_empty() {
					warn!("Update | {} failed, attempt [{}/3]...", pkg.name, i);
				} else {
					indexes.push(index);
					vec4.push(pkg.clone());
					*update.lock().unwrap().prog.lock().unwrap() += (30.0 / pkg_amount).round();
					info!("Update | {} ... OK", pkg.name);
				}
			}
			// Order indexes from biggest to smallest
			// This prevents shifting the whole vector and causing panics.
			indexes.sort();
			indexes.reverse();
			for index in indexes {
				vec3.remove(index);
			}
			if vec3.is_empty() { break }
		}
		if vec3.is_empty() {
			info!("Update | Download ... OK ... {}%", *update.lock().unwrap().prog.lock().unwrap());
		} else {
			error!("Update | Download ... FAIL");
			return Err(anyhow!("Download failed", ))
		}

		//---------------------------------------------------------------------------------------------------- Extract
		*update.lock().unwrap().msg.lock().unwrap() = format!("{}{}", MSG_EXTRACT, new_pkgs);
		info!("Update | {}", EXTRACT);
		for pkg in vec4.iter() {
			let tmp;
			match pkg.name {
				Name::Gupax => tmp = tmp_dir.to_owned() + GUPAX_BINARY,
				_ => tmp = tmp_dir.to_owned() + &pkg.name.to_string(),
			}
			#[cfg(target_os = "windows")]
			ZipArchive::extract(&mut ZipArchive::new(std::io::Cursor::new(pkg.bytes.lock().unwrap().as_ref()))?, tmp)?;
			#[cfg(target_family = "unix")]
			tar::Archive::new(flate2::read::GzDecoder::new(pkg.bytes.lock().unwrap().as_ref())).unpack(tmp)?;
			*update.lock().unwrap().prog.lock().unwrap() += (5.0 / pkg_amount).round();
			info!("Update | {} ... OK", pkg.name);
		}
		info!("Update | Extract ... OK ... {}%", *update.lock().unwrap().prog.lock().unwrap());

		//---------------------------------------------------------------------------------------------------- Upgrade
		// 0. Walk directories
		// 1. If basename matches known binary name, start
		// 2. Rename tmp path into current path
		// 3a. Update [State/Version]
		// 3b. Gupax version is builtin to binary, so no state change needed
		*update.lock().unwrap().msg.lock().unwrap() = format!("{}{}", MSG_UPGRADE, new_pkgs);
		info!("Update | {}", UPGRADE);
		// Just in case, create all folders
		for entry in WalkDir::new(tmp_dir.clone()) {
			let entry = entry?.clone();
			// If not a file, continue
			if ! entry.file_type().is_file() { continue }
			let basename = entry.file_name().to_str().ok_or(anyhow::Error::msg("WalkDir basename failed"))?;
			match basename {
				GUPAX_BINARY => {
					// Unix can replace running binaries no problem (they're loading into memory)
					// Windows locks binaries in place, so we must move (rename) current binary
					// into the temp folder, then move the new binary into the old ones spot.
					// Clearing the temp folder is now moved at startup instead at the end
					// of this function due to this behavior, thanks Windows.
					let path = update.lock().unwrap().path_gupax.clone();
					#[cfg(target_os = "windows")]
					let tmp_windows = tmp_dir.clone() + "gupax_old.exe";
					#[cfg(target_os = "windows")]
					info!("Update | WINDOWS ONLY ... Moving [{}] -> [{}]", &path, tmp_windows);
					#[cfg(target_os = "windows")]
					std::fs::rename(&path, tmp_windows)?;
					info!("Update | Moving [{}] -> [{}]", entry.path().display(), path);
					std::fs::rename(entry.path(), path)?;
					*update.lock().unwrap().prog.lock().unwrap() += (5.0 / pkg_amount).round();
				},
				P2POOL_BINARY => {
					let path = update.lock().unwrap().path_p2pool.clone();
					let path = std::path::Path::new(&path);
					info!("Update | Moving [{}] -> [{}]", entry.path().display(), path.display());
					std::fs::create_dir_all(path.parent().ok_or(anyhow::Error::msg("P2Pool path failed"))?)?;
					std::fs::rename(entry.path(), path)?;
					*og_ver.lock().unwrap().p2pool.lock().unwrap() = Pkg::get_new_pkg_version(P2pool, &vec4)?;
					*update.lock().unwrap().prog.lock().unwrap() += (5.0 / pkg_amount).round();
				},
				XMRIG_BINARY => {
					let path = update.lock().unwrap().path_xmrig.clone();
					let path = std::path::Path::new(&path);
					info!("Update | Moving [{}] -> [{}]", entry.path().display(), path.display());
					std::fs::create_dir_all(path.parent().ok_or(anyhow::Error::msg("XMRig path failed"))?)?;
					std::fs::rename(entry.path(), path)?;
					*og_ver.lock().unwrap().xmrig.lock().unwrap() = Pkg::get_new_pkg_version(Xmrig, &vec4)?;
					*update.lock().unwrap().prog.lock().unwrap() += (5.0 / pkg_amount).round();
				},
				_ => (),
			}
		}

		// Remove tmp dir (on Unix)
		#[cfg(target_family = "unix")]
		info!("Update | Removing temporary directory ... {}", tmp_dir);
		#[cfg(target_family = "unix")]
		std::fs::remove_dir_all(&tmp_dir)?;

		let seconds = now.elapsed().as_secs();
		info!("Update | Seconds elapsed ... [{}s]", seconds);
		match seconds {
			0 => *update.lock().unwrap().msg.lock().unwrap() = format!("{}! Took 0 seconds... Do you have 10Gbit internet or something...?!{}", MSG_SUCCESS, new_pkgs),
			1 => *update.lock().unwrap().msg.lock().unwrap() = format!("{}! Took 1 second... Wow!{}", MSG_SUCCESS, new_pkgs),
			_ => *update.lock().unwrap().msg.lock().unwrap() = format!("{}! Took {} seconds.{}", MSG_SUCCESS, seconds, new_pkgs),
		}
		*update.lock().unwrap().prog.lock().unwrap() = 100.0;
		Ok(())
	}
}

#[derive(Debug,Clone)]
pub enum ClientEnum {
    Tor(Client<ArtiHttpConnector<tor_rtcompat::PreferredRuntime, tls_api_native_tls::TlsConnector>>),
    Https(Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>),
}

//---------------------------------------------------------------------------------------------------- Pkg struct/impl
#[derive(Debug,Clone)]
pub struct Pkg {
	name: Name,
	link_metadata: &'static str,
	link_prefix: &'static str,
	link_suffix: &'static str,
	link_extension: &'static str,
	tmp_dir: String,
	prog: Arc<Mutex<f32>>,
	msg: Arc<Mutex<String>>,
	bytes: Arc<Mutex<hyper::body::Bytes>>,
	old_ver: String,
	new_ver: Arc<Mutex<String>>,
}

impl Pkg {
	pub fn new(name: Name, tmp_dir: &String, prog: Arc<Mutex<f32>>, msg: Arc<Mutex<String>>) -> Self {
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

	//---------------------------------------------------------------------------------------------------- Pkg functions
	// Generate fake [User-Agent] HTTP header
	fn get_user_agent() -> &'static str {
		let rand = thread_rng().gen_range(0..50);
		let user_agent = FAKE_USER_AGENT[rand];
		info!("Update | Randomly selecting User-Agent ({}/50) ... {}", rand, user_agent);
		user_agent
	}

	// Generate GET request based off input URI + fake user agent
	fn get_request(link: String, user_agent: &'static str) -> Result<Request<Body>, anyhow::Error> {
		let request = Request::builder()
			.method("GET")
			.uri(link)
			.header(hyper::header::USER_AGENT, HeaderValue::from_static(user_agent))
			.body(Body::empty())?;
		Ok(request)
	}

	// Get metadata using [Generic hyper::client<C>] & [Request]
	// and change [version, prog] under an Arc<Mutex>
	async fn get_metadata<C>(name: Name, new_ver: Arc<Mutex<String>>, client: Client<C>, link: String, user_agent: &'static str) -> Result<(), Error>
	where C: hyper::client::connect::Connect + Clone + Send + Sync + 'static, {
		let request = Pkg::get_request(link.clone(), user_agent)?;
		let mut response = client.request(request).await?;
		let body = hyper::body::to_bytes(response.body_mut()).await?;
		let body: TagName = serde_json::from_slice(&body)?;
		*new_ver.lock().unwrap() = body.tag_name.clone();
		Ok(())
	}

	// Takes a [Request], fills the appropriate [Pkg]
	// [bytes] field with the [Archive/Standalone]
	async fn get_bytes<C>(name: Name, bytes: Arc<Mutex<bytes::Bytes>>, client: Client<C>, link: String, user_agent: &'static str) -> Result<(), anyhow::Error>
	where C: hyper::client::connect::Connect + Clone + Send + Sync + 'static, {
		let request = Self::get_request(link.clone(), user_agent)?;
		let mut response = client.request(request).await?;
		// GitHub sends a 302 redirect, so we must follow
		// the [Location] header... only if Reqwest had custom
		// connectors so I didn't have to manually do this...
		if response.headers().contains_key(LOCATION) {
			let request = Self::get_request(response.headers().get(LOCATION).unwrap().to_str()?.to_string(), user_agent)?;
			response = client.request(request).await?;
		}
		let body = hyper::body::to_bytes(response.into_body()).await?;
		*bytes.lock().unwrap() = body;
		Ok(())
	}

	// Take in a [Name] and [Vec] of [Pkg]s, find
	// that [Name]'s corresponding new version.
	fn get_new_pkg_version(name: Name, vec: &Vec<&Pkg>) -> Result<String, Error> {
		for pkg in vec.iter() {
			if pkg.name == name {
				return Ok(pkg.new_ver.lock().unwrap().to_string())
			}
		}
		Err(anyhow::Error::msg("Couldn't find new_pkg_version"))
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
