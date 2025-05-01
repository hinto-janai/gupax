// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022-2023 hinto-janai
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
// You should have received a copy of the GNU General Public License
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
use crate::{
    constants::GUPAX_VERSION, disk::*, macros::*, update::Name::*, ErrorButtons, ErrorFerris,
    ErrorState, Restart,
};
use anyhow::{anyhow, Error};
use arti_client::TorClient;
use arti_hyper::*;
use hyper::{
    body::Body,
    client::Client,
    header::{HeaderValue, LOCATION},
    Request,
};
use log::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use walkdir::WalkDir;

// On apple-darwin targets there is an issue with the native and rustls
// tls implementation so this makes it fall back to the openssl variant.
//
// https://gitlab.torproject.org/tpo/core/arti/-/issues/715
#[cfg(not(target_os = "macos"))]
use tls_api_native_tls::TlsConnector;
#[cfg(target_os = "macos")]
use tls_api_openssl::TlsConnector;

use tls_api::{TlsConnector as TlsConnectorTrait, TlsConnectorBuilder};

#[cfg(target_os = "windows")]
use zip::ZipArchive;
//#[cfg(target_family = "unix")]
//use std::os::unix::fs::OpenOptionsExt;

//---------------------------------------------------------------------------------------------------- Constants
// Package naming schemes:
// gupax  | gupax-vX.X.X-(windows|macos|linux)-(x64|arm64)-(standalone|bundle).(zip|tar.gz)
// p2pool | p2pool-vX.X.X-(windows|macos|linux)-(x64|aarch64).(zip|tar.gz)
// xmrig  | xmrig-X.X.X-(msvc-win64|macos-x64|macos-arm64|linux-static-x64).(zip|tar.gz)
//
// Download link = PREFIX + Version (found at runtime) + SUFFIX + Version + EXT
// Example: https://github.com/hinto-janai/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-standalone-x64.tar.gz
//
// Exceptions (there are always exceptions...):
//   - XMRig doesn't have a [v], so it is [xmrig-6.18.0-...]
//   - XMRig separates the hash and signature
//   - P2Pool hashes are in UPPERCASE

const GUPAX_METADATA: &str = "https://api.github.com/repos/hinto-janai/gupax/releases/latest";
const P2POOL_METADATA: &str = "https://api.github.com/repos/SChernykh/p2pool/releases/latest";
const XMRIG_METADATA: &str = "https://api.github.com/repos/xmrig/xmrig/releases/latest";

const GUPAX_PREFIX: &str = "https://github.com/hinto-janai/gupax/releases/download/";
const P2POOL_PREFIX: &str = "https://github.com/SChernykh/p2pool/releases/download/";
const XMRIG_PREFIX: &str = "https://github.com/xmrig/xmrig/releases/download/";

const GUPAX_SUFFIX: &str = "/gupax-";
const P2POOL_SUFFIX: &str = "/p2pool-";
const XMRIG_SUFFIX: &str = "/xmrig-";

const GUPAX_HASH: &str = "SHA256SUMS";
const P2POOL_HASH: &str = "sha256sums.txt.asc";
const XMRIG_HASH: &str = "SHA256SUMS";

#[cfg(target_os = "windows")]
mod impl_platform {
    pub(super) const GUPAX_EXTENSION: &str = "-windows-x64-standalone.zip";
    pub(super) const P2POOL_EXTENSION: &str = "-windows-x64.zip";
    pub(super) const XMRIG_EXTENSION: &str = "-msvc-win64.zip";
    pub(super) const GUPAX_BINARY: &str = "Gupax.exe";
    pub(super) const P2POOL_BINARY: &str = "p2pool.exe";
    pub(super) const XMRIG_BINARY: &str = "xmrig.exe";
    pub(super) const VALID_GUPAX_1: &str = "GUPAX.exe";
    pub(super) const VALID_GUPAX_2: &str = "Gupax.exe";
    pub(super) const VALID_GUPAX_3: &str = "gupax.exe";
    pub(super) const VALID_XMRIG_1: &str = "XMRIG.exe";
    pub(super) const VALID_XMRIG_2: &str = "XMRig.exe";
    pub(super) const VALID_XMRIG_3: &str = "Xmrig.exe";
    pub(super) const VALID_XMRIG_4: &str = "xmrig.exe";
    pub(super) const VALID_P2POOL_1: &str = "P2POOL.exe";
    pub(super) const VALID_P2POOL_2: &str = "P2Pool.exe";
    pub(super) const VALID_P2POOL_3: &str = "P2pool.exe";
    pub(super) const VALID_P2POOL_4: &str = "p2pool.exe";
}

#[cfg(target_family = "unix")]
mod impl_unix {
    pub(super) const GUPAX_BINARY: &str = "gupax";
    pub(super) const P2POOL_BINARY: &str = "p2pool";
    pub(super) const XMRIG_BINARY: &str = "xmrig";
    pub(super) const VALID_GUPAX_1: &str = "GUPAX";
    pub(super) const VALID_GUPAX_2: &str = "Gupax";
    pub(super) const VALID_GUPAX_3: &str = "gupax";
    pub(super) const VALID_XMRIG_1: &str = "XMRIG";
    pub(super) const VALID_XMRIG_2: &str = "XMRig";
    pub(super) const VALID_XMRIG_3: &str = "Xmrig";
    pub(super) const VALID_XMRIG_4: &str = "xmrig";
    pub(super) const VALID_P2POOL_1: &str = "P2POOL";
    pub(super) const VALID_P2POOL_2: &str = "P2Pool";
    pub(super) const VALID_P2POOL_3: &str = "P2pool";
    pub(super) const VALID_P2POOL_4: &str = "p2pool";
}

#[cfg(target_os = "macos")]
#[cfg(target_arch = "x86_64")]
mod impl_platform {
    pub(super) use super::impl_unix::*;

    pub(super) const GUPAX_EXTENSION: &str = "-macos-x64-standalone.tar.gz";
    pub(super) const P2POOL_EXTENSION: &str = "-macos-x64.tar.gz";
    pub(super) const XMRIG_EXTENSION: &str = "-macos-x64.tar.gz";
}

#[cfg(target_os = "macos")]
#[cfg(target_arch = "aarch64")]
mod impl_platform {
    pub(super) use super::impl_unix::*;

    pub(super) const GUPAX_EXTENSION: &str = "-macos-arm64-standalone.tar.gz";
    pub(super) const P2POOL_EXTENSION: &str = "-macos-aarch64.tar.gz";
    pub(super) const XMRIG_EXTENSION: &str = "-macos-arm64.tar.gz";
}

#[cfg(target_os = "linux")]
mod impl_platform {
    pub(super) use super::impl_unix::*;

    pub(super) const GUPAX_EXTENSION: &str = "-linux-x64-standalone.tar.gz";
    pub(super) const P2POOL_EXTENSION: &str = "-linux-x64.tar.gz";
    pub(super) const XMRIG_EXTENSION: &str = "-linux-static-x64.tar.gz";
}

use impl_platform::*;

const VALID_GUPAX: [&str; 3] = [VALID_GUPAX_1, VALID_GUPAX_2, VALID_GUPAX_3];
const VALID_XMRIG: [&str; 4] = [VALID_XMRIG_1, VALID_XMRIG_2, VALID_XMRIG_3, VALID_XMRIG_4];
const VALID_P2POOL: [&str; 4] = [
    VALID_P2POOL_1,
    VALID_P2POOL_2,
    VALID_P2POOL_3,
    VALID_P2POOL_4,
];

// Some fake Curl/Wget user-agents because GitHub API requires one and a Tor browser
// user-agent might be fingerprintable without all the associated headers.
const FAKE_USER_AGENT: [&str; 25] = [
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
    "Wget/1.21.4",
    "Wget/1.24.5",
    "curl/8.4.0",
    "curl/8.5.0",
    "curl/8.6.0",
    "curl/8.7.0",
    "curl/8.7.1",
];

const MSG_NONE: &str = "No update in progress";
const MSG_START: &str = "Starting update";
const MSG_TMP: &str = "Creating temporary directory";
const MSG_TOR: &str = "Creating Tor+HTTPS client";
const MSG_HTTPS: &str = "Creating HTTPS client";
const MSG_METADATA: &str = "Fetching package metadata";
const MSG_METADATA_RETRY: &str = "Fetching package metadata failed, attempt";
const MSG_COMPARE: &str = "Compare package versions";
const MSG_UP_TO_DATE: &str = "All packages already up-to-date";
const MSG_DOWNLOAD: &str = "Downloading packages";
const MSG_DOWNLOAD_RETRY: &str = "Downloading packages failed, attempt";
const MSG_EXTRACT: &str = "Extracting packages";
const MSG_UPGRADE: &str = "Upgrading packages";
pub const MSG_SUCCESS: &str = "Update successful";
pub const MSG_FAILED: &str = "Update failed";
pub const MSG_FAILED_HELP: &str =
    "Consider manually replacing your executable from: https://gupax.io/downloads";

const INIT: &str = "------------------- Init -------------------";
const METADATA: &str = "----------------- Metadata -----------------";
const COMPARE: &str = "----------------- Compare ------------------";
const DOWNLOAD: &str = "----------------- Download -----------------";
const EXTRACT: &str = "----------------- Extract ------------------";
const UPGRADE: &str = "----------------- Upgrade ------------------";

//---------------------------------------------------------------------------------------------------- General functions
pub fn check_p2pool_path(path: &str) -> bool {
    let path = match crate::disk::into_absolute_path(path.to_string()) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let path = match path.file_name() {
        Some(p) => p,
        None => {
            error!("Couldn't get P2Pool file name");
            return false;
        }
    };
    path == VALID_P2POOL[0]
        || path == VALID_P2POOL[1]
        || path == VALID_P2POOL[2]
        || path == VALID_P2POOL[3]
}

pub fn check_xmrig_path(path: &str) -> bool {
    let path = match crate::disk::into_absolute_path(path.to_string()) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let path = match path.file_name() {
        Some(p) => p,
        None => {
            error!("Couldn't get XMRig file name");
            return false;
        }
    };
    path == VALID_XMRIG[0]
        || path == VALID_XMRIG[1]
        || path == VALID_XMRIG[2]
        || path == VALID_XMRIG[3]
}

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
    pub path_gupax: String,         // Full path to current gupax
    pub path_p2pool: String,        // Full path to current p2pool
    pub path_xmrig: String,         // Full path to current xmrig
    pub tmp_dir: String,            // Full path to temporary directory
    pub updating: Arc<Mutex<bool>>, // Is an update in progress?
    pub prog: Arc<Mutex<f32>>,      // Holds the 0-100% progress bar number
    pub msg: Arc<Mutex<String>>,    // Message to display on [Gupax] tab while updating
    pub tor: bool,                  // Is Tor enabled or not?
}

impl Update {
    // Takes in current paths from [State]
    pub fn new(path_gupax: String, path_p2pool: PathBuf, path_xmrig: PathBuf, tor: bool) -> Self {
        Self {
            path_gupax,
            path_p2pool: path_p2pool.display().to_string(),
            path_xmrig: path_xmrig.display().to_string(),
            tmp_dir: "".to_string(),
            updating: arc_mut!(false),
            prog: arc_mut!(0.0),
            msg: arc_mut!(MSG_NONE.to_string()),
            tor,
        }
    }

    // Get a temporary random folder for package download contents
    // This used to use [std::env::temp_dir()] but there were issues
    // using [std::fs::rename()] on tmpfs -> disk (Invalid cross-device link (os error 18)).
    // So, uses the [Gupax] binary directory as a base, something like [/home/hinto/gupax/gupax_update_SG4xsDdVmr]
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

    #[cold]
    #[inline(never)]
    // Get an HTTPS client. Uses [Arti] if Tor is enabled.
    // The base type looks something like [hyper::Client<...>].
    // This is then wrapped with the custom [ClientEnum] type to implement
    // dynamically returning either a [Tor+TLS|TLS-only] client at based on user settings.
    //     tor == true?  => return Tor client
    //     tor == false? => return normal TLS client
    //
    // Since functions that take generic INPUT are much easier to implement,
    // [get_response()] just takes a [hyper::Client<C>], which is passed to
    // it via deconstructing this [ClientEnum] with a match, like so:
    //     ClientEnum::Tor(T)   => get_response(... T ...)
    //     ClientEnum::Https(H) => get_response(... H ...)
    //
    pub fn get_client(tor: bool) -> Result<ClientEnum, anyhow::Error> {
        if tor {
            // Below is async, bootstraps immediately but has issues when recreating the circuit
            // let tor = TorClient::create_bootstrapped(TorClientConfig::default()).await?;
            // This one below is non-async, and doesn't bootstrap immediately.
            let tor = TorClient::builder()
                .bootstrap_behavior(arti_client::BootstrapBehavior::OnDemand)
                .create_unbootstrapped()?;
            // This makes sure the Tor circuit is different each time
            let tor = TorClient::isolated_client(&tor);
            let tls = TlsConnector::builder()?.build()?;
            let connector = ArtiHttpConnector::new(tor, tls);
            let client = ClientEnum::Tor(Client::builder().build(connector));
            Ok(client)
        } else {
            let mut connector = hyper_tls::HttpsConnector::new();
            connector.https_only(true);
            let client = ClientEnum::Https(Client::builder().build(connector));
            Ok(client)
        }
    }

    #[cold]
    #[inline(never)]
    // Intermediate function that spawns a new thread
    // which starts the async [start()] function that
    // actually contains the code. This is so that everytime
    // an update needs to happen (Gupax tab, auto-update), the
    // code only needs to be edited once, here.
    pub fn spawn_thread(
        og: &Arc<Mutex<State>>,
        gupax: &crate::disk::Gupax,
        state_path: &Path,
        update: &Arc<Mutex<Update>>,
        error_state: &mut ErrorState,
        restart: &Arc<Mutex<Restart>>,
    ) {
        // Check P2Pool path for safety
        // Attempt relative to absolute path
        let p2pool_path = match into_absolute_path(gupax.p2pool_path.clone()) {
            Ok(p) => p,
            Err(e) => {
                error_state.set(
                    format!(
                        "Provided P2Pool path could not be turned into an absolute path: {}",
                        e
                    ),
                    ErrorFerris::Error,
                    ErrorButtons::Okay,
                );
                return;
            }
        };
        // Attempt to get basename
        let file = match p2pool_path.file_name() {
            Some(p) => {
                // Attempt to turn into str
                match p.to_str() {
                    Some(p) => p,
                    None => {
                        error_state.set("Provided P2Pool path could not be turned into a UTF-8 string (are you using non-English characters?)", ErrorFerris::Error, ErrorButtons::Okay);
                        return;
                    }
                }
            }
            None => {
                error_state.set(
                    "Provided P2Pool path could not be found",
                    ErrorFerris::Error,
                    ErrorButtons::Okay,
                );
                return;
            }
        };
        // If it doesn't look like [P2Pool], its probably a bad move
        // to overwrite it with an update, so set an error.
        // Doesnt seem like you can [match] on array indexes
        // so that explains the ridiculous if/else.
        if check_p2pool_path(file) {
            info!("Update | Using P2Pool path: [{}]", p2pool_path.display());
        } else {
            warn!(
                "Update | Aborting update, incorrect P2Pool path: [{}]",
                file
            );
            let text = format!("Provided P2Pool path seems incorrect. Not starting update for safety.\nTry one of these: {:?}", VALID_P2POOL);
            error_state.set(text, ErrorFerris::Error, ErrorButtons::Okay);
            return;
        }

        // Check XMRig path for safety
        let xmrig_path = match into_absolute_path(gupax.xmrig_path.clone()) {
            Ok(p) => p,
            Err(e) => {
                error_state.set(
                    format!(
                        "Provided XMRig path could not be turned into an absolute path: {}",
                        e
                    ),
                    ErrorFerris::Error,
                    ErrorButtons::Okay,
                );
                return;
            }
        };
        let file = match xmrig_path.file_name() {
            Some(p) => {
                // Attempt to turn into str
                match p.to_str() {
                    Some(p) => p,
                    None => {
                        error_state.set("Provided XMRig path could not be turned into a UTF-8 string (are you using non-English characters?)", ErrorFerris::Error, ErrorButtons::Okay);
                        return;
                    }
                }
            }
            None => {
                error_state.set(
                    "Provided XMRig path could not be found",
                    ErrorFerris::Error,
                    ErrorButtons::Okay,
                );
                return;
            }
        };
        if check_xmrig_path(file) {
            info!("Update | Using XMRig path: [{}]", xmrig_path.display());
        } else {
            warn!("Update | Aborting update, incorrect XMRig path: [{}]", file);
            let text = format!("Provided XMRig path seems incorrect. Not starting update for safety.\nTry one of these: {:?}", VALID_XMRIG);
            error_state.set(text, ErrorFerris::Error, ErrorButtons::Okay);
            return;
        }

        lock!(update).path_p2pool = p2pool_path.display().to_string();
        lock!(update).path_xmrig = xmrig_path.display().to_string();
        lock!(update).tor = gupax.update_via_tor;

        // Clone before thread spawn
        let og = Arc::clone(og);
        let state_ver = Arc::clone(&lock!(og).version);
        let state_path = state_path.to_path_buf();
        let update = Arc::clone(update);
        let restart = Arc::clone(restart);
        info!("Spawning update thread...");
        std::thread::spawn(move || {
            match Update::start(update.clone(), og.clone(), state_ver.clone(), restart) {
                Ok(_) => {
                    info!("Update | Saving state...");
                    let original_version = lock!(og).version.clone();
                    lock!(og).version = state_ver;
                    match State::save(&mut lock!(og), &state_path) {
                        Ok(_) => info!("Update ... OK"),
                        Err(e) => {
                            warn!("Update | Saving state ... FAIL: {}", e);
                            lock!(og).version = original_version;
                            *lock2!(update, msg) =
                                "Saving new versions into state failed".to_string();
                        }
                    };
                }
                Err(e) => {
                    info!("Update ... FAIL: {}", e);
                    *lock2!(update, msg) = format!("{} | {}\n{}", MSG_FAILED, e, MSG_FAILED_HELP);
                }
            };
            *lock2!(update, updating) = false;
        });
    }

    #[cold]
    #[inline(never)]
    // Download process:
    // 0. setup tor, client, http, etc
    // 1. fill vector with all enums
    // 2. loop over vec, download metadata
    // 3. if current == version, remove from vec
    // 4. loop over vec, download links
    // 5. extract, upgrade
    #[tokio::main]
    pub async fn start(
        update: Arc<Mutex<Self>>,
        _og: Arc<Mutex<State>>,
        state_ver: Arc<Mutex<Version>>,
        restart: Arc<Mutex<Restart>>,
    ) -> Result<(), anyhow::Error> {
        //---------------------------------------------------------------------------------------------------- Init
        *lock2!(update, updating) = true;
        // Set timer
        let now = std::time::Instant::now();

        // Set progress bar
        *lock2!(update, msg) = MSG_START.to_string();
        *lock2!(update, prog) = 0.0;
        info!("Update | {}", INIT);

        // Get temporary directory
        let msg = MSG_TMP.to_string();
        info!("Update | {}", msg);
        *lock2!(update, msg) = msg;
        let tmp_dir = Self::get_tmp_dir()?;
        std::fs::create_dir(&tmp_dir)?;

        // Make Pkg vector
        let mut vec = vec![Pkg::new(Gupax), Pkg::new(P2pool), Pkg::new(Xmrig)];

        // Generate fake user-agent
        let user_agent = Pkg::get_user_agent();
        *lock2!(update, prog) = 5.0;

        // Create Tor/HTTPS client
        let lock = lock!(update);
        let tor = lock.tor;
        if tor {
            let msg = MSG_TOR.to_string();
            info!("Update | {}", msg);
            *lock!(lock.msg) = msg;
        } else {
            let msg = MSG_HTTPS.to_string();
            info!("Update | {}", msg);
            *lock!(lock.msg) = msg;
        }
        drop(lock);
        let mut client = Self::get_client(tor)?;
        *lock2!(update, prog) += 5.0;
        info!("Update | Init ... OK ... {}%", lock2!(update, prog));

        //---------------------------------------------------------------------------------------------------- Metadata
        *lock2!(update, msg) = MSG_METADATA.to_string();
        info!("Update | {}", METADATA);
        let mut vec2 = vec![];
        // Loop process:
        // 1. Start all async metadata fetches
        // 2. Wait for all to finish
        // 3. Iterate over all [pkg.new_ver], check if empty
        // 4. If not empty, move [pkg] to different vec
        // 5. At end, if original vec isn't empty, that means something failed
        // 6. Redo loop [3] times (rebuild circuit if using Tor), with the original vec (that now only has the failed pkgs)
        //
        // This logic was originally in the [Pkg::get_metadata()]
        // function itself but for some reason, it was getting skipped over,
        // so the [new_ver] check is now here, in the outer scope.
        for i in 1..=3 {
            if i > 1 {
                *lock2!(update, msg) = format!("{} [{}/3]", MSG_METADATA_RETRY, i);
            }
            let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
            for pkg in vec.iter() {
                // Clone data before sending to async
                let new_ver = Arc::clone(&pkg.new_ver);
                let client = client.clone();
                let link = pkg.link_metadata.to_string();
                // Send to async
                let handle: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
                    match client {
                        ClientEnum::Tor(t) => Pkg::get_metadata(new_ver, t, link, user_agent).await,
                        ClientEnum::Https(h) => {
                            Pkg::get_metadata(new_ver, h, link, user_agent).await
                        }
                    }
                });
                handles.push(handle);
            }
            // Handle await
            for handle in handles {
                // Two [??] will send the error.
                // We don't actually want to return the error here since we
                // prefer looping and retrying over immediately erroring out.
                if let Err(e) = handle.await? {
                    warn!("Update | {}", e)
                }
            }
            // Check for empty version
            let mut indexes = vec![];
            for (index, pkg) in vec.iter().enumerate() {
                if lock!(pkg.new_ver).is_empty() {
                    warn!("Update | {} failed, attempt [{}/3]...", pkg.name, i + 1);
                } else {
                    indexes.push(index);
                    vec2.push(pkg.clone());
                    *lock2!(update, prog) += 10.0;
                    info!("Update | {} {} ... OK", pkg.name, lock!(pkg.new_ver));
                }
            }
            // Order indexes from biggest to smallest
            // This prevents shifting the whole vector and causing panics.
            indexes.sort();
            indexes.reverse();
            for index in indexes {
                vec.remove(index);
            }
            if vec.is_empty() {
                break;
            }
            // Some Tor exit nodes seem to be blocked by GitHub's API,
            // so recreate the circuit every loop.
            if tor {
                info!("Update | Recreating Tor client...");
                client = Self::get_client(tor)?;
            }
        }
        if vec.is_empty() {
            info!("Update | Metadata ... OK ... {}%", lock2!(update, prog));
        } else {
            error!("Update | Metadata ... FAIL");
            return Err(anyhow!("Metadata fetch failed"));
        }

        //---------------------------------------------------------------------------------------------------- Compare
        *lock2!(update, msg) = MSG_COMPARE.to_string();
        info!("Update | {}", COMPARE);
        let mut vec3 = vec![];
        let mut new_pkgs = vec![];
        for pkg in vec2.iter() {
            let new_ver = lock!(pkg.new_ver).clone();
            let diff;
            let old_ver;
            let name;
            match pkg.name {
                Gupax => {
                    // Compare against the built-in compiled version as well as an in-memory version
                    // that gets updated during an update. This prevents the updater always thinking
                    // there's a new Gupax update since the user didnt restart and is still technically
                    // using the old version (even though the underlying binary was updated).
                    old_ver = lock!(state_ver).gupax.clone();
                    diff = old_ver != new_ver && GUPAX_VERSION != new_ver;
                    name = "Gupax";
                }
                P2pool => {
                    old_ver = lock!(state_ver).p2pool.clone();
                    diff = old_ver != new_ver;
                    name = "P2Pool";
                }
                Xmrig => {
                    old_ver = lock!(state_ver).xmrig.clone();
                    diff = old_ver != new_ver;
                    name = "XMRig";
                }
            }
            if diff {
                info!(
                    "Update | {} {} != {} ... ADDING",
                    pkg.name, old_ver, new_ver
                );
                new_pkgs.push(format!("\n{} {}  ->  {}", name, old_ver, new_ver));
                vec3.push(pkg);
            } else {
                info!(
                    "Update | {} {} == {} ... SKIPPING",
                    pkg.name, old_ver, new_ver
                );
            }
        }
        *lock2!(update, prog) += 5.0;
        info!("Update | Compare ... OK ... {}%", lock2!(update, prog));

        // Return if 0 (all packages up-to-date)
        // Get amount of packages to divide up the percentage increases
        let pkg_amount = vec3.len() as f32;
        if pkg_amount == 0.0 {
            info!("Update | All packages up-to-date ... RETURNING");
            *lock2!(update, prog) = 100.0;
            *lock2!(update, msg) = MSG_UP_TO_DATE.to_string();
            return Ok(());
        }
        let new_pkgs: String = new_pkgs.concat();

        //---------------------------------------------------------------------------------------------------- Download
        *lock2!(update, msg) = format!("{}{}", MSG_DOWNLOAD, new_pkgs);
        info!("Update | {}", DOWNLOAD);
        let mut vec4 = vec![];
        for i in 1..=3 {
            if i > 1 {
                *lock2!(update, msg) = format!("{} [{}/3]{}", MSG_DOWNLOAD_RETRY, i, new_pkgs);
            }
            let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
            for pkg in vec3.iter() {
                // Clone data before async
                let bytes = Arc::clone(&pkg.bytes);
                let client = client.clone();
                let version = lock!(pkg.new_ver);
                // Download link = PREFIX + Version (found at runtime) + SUFFIX + Version + EXT
                // Example: https://github.com/hinto-janai/gupax/releases/download/v0.0.1/gupax-v0.0.1-linux-x64-standalone
                // XMRig doesn't have a [v], so slice it out
                let link = match pkg.name {
                    Name::Xmrig => {
                        pkg.link_prefix.to_string()
                            + &version
                            + pkg.link_suffix
                            + &version[1..]
                            + pkg.link_extension
                    }
                    _ => {
                        pkg.link_prefix.to_string()
                            + &version
                            + pkg.link_suffix
                            + &version
                            + pkg.link_extension
                    }
                };
                info!("Update | {} ... {}", pkg.name, link);
                let handle: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
                    match client {
                        ClientEnum::Tor(t) => Pkg::get_bytes(bytes, t, link, user_agent).await,
                        ClientEnum::Https(h) => Pkg::get_bytes(bytes, h, link, user_agent).await,
                    }
                });
                handles.push(handle);
            }
            // Handle await
            for handle in handles {
                if let Err(e) = handle.await? {
                    warn!("Update | {}", e)
                }
            }
            // Check for empty bytes
            let mut indexes = vec![];
            for (index, pkg) in vec3.iter().enumerate() {
                if lock!(pkg.bytes).is_empty() {
                    warn!("Update | {} failed, attempt [{}/3]...", pkg.name, i);
                } else {
                    indexes.push(index);
                    vec4.push(pkg.clone());
                    *lock2!(update, prog) += (30.0 / pkg_amount).round();
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
            if vec3.is_empty() {
                break;
            }
        }
        if vec3.is_empty() {
            info!("Update | Download ... OK ... {}%", *lock2!(update, prog));
        } else {
            error!("Update | Download ... FAIL");
            return Err(anyhow!("Download failed"));
        }

        //---------------------------------------------------------------------------------------------------- Extract
        *lock2!(update, msg) = format!("{}{}", MSG_EXTRACT, new_pkgs);
        info!("Update | {}", EXTRACT);
        for pkg in vec4.iter() {
            let tmp = match pkg.name {
                Name::Gupax => tmp_dir.to_owned() + GUPAX_BINARY,
                _ => tmp_dir.to_owned() + &pkg.name.to_string(),
            };
            #[cfg(target_os = "windows")]
            ZipArchive::extract(
                &mut ZipArchive::new(std::io::Cursor::new(lock!(pkg.bytes).as_ref()))?,
                tmp,
            )?;
            #[cfg(target_family = "unix")]
            tar::Archive::new(flate2::read::GzDecoder::new(lock!(pkg.bytes).as_ref()))
                .unpack(tmp)?;
            *lock2!(update, prog) += (5.0 / pkg_amount).round();
            info!("Update | {} ... OK", pkg.name);
        }
        info!("Update | Extract ... OK ... {}%", *lock2!(update, prog));

        //---------------------------------------------------------------------------------------------------- Upgrade
        // 1. Walk directories
        // 2. If basename matches known binary name, start
        // 3. Rename tmp path into current path
        // 4. Update [State/Version]
        *lock2!(update, msg) = format!("{}{}", MSG_UPGRADE, new_pkgs);
        info!("Update | {}", UPGRADE);
        // If this bool doesn't get set, something has gone wrong because
        // we _didn't_ find a binary even though we downloaded it.
        let mut found = false;
        for entry in WalkDir::new(tmp_dir.clone()) {
            let entry = entry?.clone();
            // If not a file, continue
            if !entry.file_type().is_file() {
                continue;
            }
            let basename = entry
                .file_name()
                .to_str()
                .ok_or_else(|| anyhow!("WalkDir basename failed"))?;
            match basename {
                VALID_GUPAX_1 | VALID_GUPAX_2 | VALID_GUPAX_3 | VALID_P2POOL_1 | VALID_P2POOL_2
                | VALID_P2POOL_3 | VALID_P2POOL_4 | VALID_XMRIG_1 | VALID_XMRIG_2
                | VALID_XMRIG_3 | VALID_XMRIG_4 => {
                    found = true;
                    let name = match basename {
                        VALID_GUPAX_1 | VALID_GUPAX_2 | VALID_GUPAX_3 => Gupax,
                        VALID_P2POOL_1 | VALID_P2POOL_2 | VALID_P2POOL_3 | VALID_P2POOL_4 => P2pool,
                        _ => Xmrig,
                    };
                    let path = match name {
                        Gupax => lock!(update).path_gupax.clone(),
                        P2pool => lock!(update).path_p2pool.clone(),
                        Xmrig => lock!(update).path_xmrig.clone(),
                    };
                    let path = Path::new(&path);
                    // Unix can replace running binaries no problem (they're loaded into memory)
                    // Windows locks binaries in place, so we must move (rename) current binary
                    // into the temp folder, then move the new binary into the old ones spot.
                    // Clearing the temp folder is now moved at startup instead at the end
                    // of this function due to this behavior, thanks Windows.
                    #[cfg(target_os = "windows")]
                    if path.exists() {
                        let tmp_windows = match name {
                            Gupax => tmp_dir.clone() + "gupax_old.exe",
                            P2pool => tmp_dir.clone() + "p2pool_old.exe",
                            Xmrig => tmp_dir.clone() + "xmrig_old.exe",
                        };
                        info!(
                            "Update | WINDOWS ONLY ... Moving old [{}] -> [{}]",
                            path.display(),
                            tmp_windows
                        );
                        std::fs::rename(&path, tmp_windows)?;
                    }
                    info!(
                        "Update | Moving new [{}] -> [{}]",
                        entry.path().display(),
                        path.display()
                    );
                    // Create folder for [P2Pool/XMRig]
                    if name == P2pool || name == Xmrig {
                        std::fs::create_dir_all(
                            path.parent()
                                .ok_or_else(|| anyhow!(format!("{} path failed", name)))?,
                        )?;
                    }
                    // Move downloaded path into old path
                    std::fs::rename(entry.path(), path)?;
                    // Update [State] version
                    match name {
                        Gupax => {
                            lock!(state_ver).gupax = Pkg::get_new_pkg_version(Gupax, &vec4)?;
                            // If we're updating Gupax, set the [Restart] state so that the user knows to restart
                            *lock!(restart) = Restart::Yes;
                        }
                        P2pool => {
                            lock!(state_ver).p2pool = Pkg::get_new_pkg_version(P2pool, &vec4)?
                        }
                        Xmrig => lock!(state_ver).xmrig = Pkg::get_new_pkg_version(Xmrig, &vec4)?,
                    };
                    *lock2!(update, prog) += (5.0 / pkg_amount).round();
                }
                _ => (),
            }
        }
        if !found {
            return Err(anyhow!("Fatal error: Package binary could not be found"));
        }

        // Remove tmp dir (on Unix)
        #[cfg(target_family = "unix")]
        info!("Update | Removing temporary directory ... {}", tmp_dir);
        #[cfg(target_family = "unix")]
        std::fs::remove_dir_all(&tmp_dir)?;

        let seconds = now.elapsed().as_secs();
        info!("Update | Seconds elapsed ... [{}s]", seconds);
        match seconds {
            0 => {
                *lock2!(update, msg) =
                    format!("{}! Took 0 seconds... What...?!{}", MSG_SUCCESS, new_pkgs)
            }
            1 => {
                *lock2!(update, msg) = format!("{}! Took 1 second... Wow!{}", MSG_SUCCESS, new_pkgs)
            }
            _ => {
                *lock2!(update, msg) =
                    format!("{}! Took {} seconds.{}", MSG_SUCCESS, seconds, new_pkgs)
            }
        }
        *lock2!(update, prog) = 100.0;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ClientEnum {
    Tor(hyper::Client<ArtiHttpConnector<tor_rtcompat::PreferredRuntime, TlsConnector>>),
    Https(hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>),
}

//---------------------------------------------------------------------------------------------------- Pkg struct/impl
#[derive(Debug, Clone)]
pub struct Pkg {
    name: Name,
    link_metadata: &'static str,
    link_prefix: &'static str,
    link_suffix: &'static str,
    link_extension: &'static str,
    bytes: Arc<Mutex<hyper::body::Bytes>>,
    new_ver: Arc<Mutex<String>>,
}

impl Pkg {
    #[cold]
    #[inline(never)]
    pub fn new(name: Name) -> Self {
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
            bytes: arc_mut!(bytes::Bytes::new()),
            new_ver: arc_mut!(String::new()),
        }
    }

    //---------------------------------------------------------------------------------------------------- Pkg functions
    #[cold]
    #[inline(never)]
    // Generate fake [User-Agent] HTTP header
    pub fn get_user_agent() -> &'static str {
        let index = FAKE_USER_AGENT.len() - 1;

        let rand = thread_rng().gen_range(0..index);
        let user_agent = FAKE_USER_AGENT[rand];
        info!("Randomly selected User-Agent ({rand}/{index}) ... {user_agent}");
        user_agent
    }

    #[cold]
    #[inline(never)]
    // Generate GET request based off input URI + fake user agent
    fn get_request(link: String, user_agent: &'static str) -> Result<Request<Body>, anyhow::Error> {
        let request = Request::builder()
            .method("GET")
            .uri(link)
            .header(
                hyper::header::USER_AGENT,
                HeaderValue::from_static(user_agent),
            )
            .body(Body::empty())?;
        Ok(request)
    }

    #[cold]
    #[inline(never)]
    // Get metadata using [Generic hyper::client<C>] & [Request]
    // and change [version, prog] under an Arc<Mutex>
    async fn get_metadata<C>(
        new_ver: Arc<Mutex<String>>,
        client: Client<C>,
        link: String,
        user_agent: &'static str,
    ) -> Result<(), Error>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let request = Pkg::get_request(link, user_agent)?;
        let mut response = client.request(request).await?;
        let body = hyper::body::to_bytes(response.body_mut()).await?;
        let body: TagName = serde_json::from_slice(&body)?;
        *lock!(new_ver) = body.tag_name;
        Ok(())
    }

    #[cold]
    #[inline(never)]
    // Takes a [Request], fills the appropriate [Pkg]
    // [bytes] field with the [Archive/Standalone]
    async fn get_bytes<C>(
        bytes: Arc<Mutex<bytes::Bytes>>,
        client: Client<C>,
        link: String,
        user_agent: &'static str,
    ) -> Result<(), anyhow::Error>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let request = Self::get_request(link, user_agent)?;
        let mut response = client.request(request).await?;
        // GitHub sends a 302 redirect, so we must follow
        // the [Location] header... only if Reqwest had custom
        // connectors so I didn't have to manually do this...
        if response.headers().contains_key(LOCATION) {
            let request = Self::get_request(
                response
                    .headers()
                    .get(LOCATION)
                    .ok_or_else(|| anyhow!("HTTP Location header GET failed"))?
                    .to_str()?
                    .to_string(),
                user_agent,
            )?;
            response = client.request(request).await?;
        }
        let body = hyper::body::to_bytes(response.into_body()).await?;
        *lock!(bytes) = body;
        Ok(())
    }

    #[cold]
    #[inline(never)]
    // Take in a [Name] and [Vec] of [Pkg]s, find
    // that [Name]'s corresponding new version.
    fn get_new_pkg_version(name: Name, vec: &[&Pkg]) -> Result<String, Error> {
        for pkg in vec.iter() {
            if pkg.name == name {
                return Ok(lock!(pkg.new_ver).to_string());
            }
        }
        Err(anyhow!("Couldn't find new_pkg_version"))
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
