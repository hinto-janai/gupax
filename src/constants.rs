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

use std::net::{Ipv4Addr,SocketAddrV4};

// Compile-time constants
pub const BYTES_ICON: &[u8] = include_bytes!("../images/png/icon.png");
pub const BYTES_BANNER: &[u8] = include_bytes!("../images/png/banner.png");
pub const P2POOL_BASE_ARGS: &'static str = "--host 127.0.0.1 --rpc-port 18081 --zmq-port 18083 --loglevel 3 --out-peers 10 --in-peers 10";
pub const XMRIG_BASE_ARGS: &'static str = "--http-host=127.0.0.1 --http-port=18088 --algo=rx/0 --coin=Monero --randomx-cache-qos";

// OS specific
#[cfg(target_os = "windows")]
pub const OS: &'static str = " Windows";
#[cfg(target_os = "windows")]
pub const HUGEPAGES_1GB: bool = false;

#[cfg(target_os = "macos")]
pub const OS: &'static str = " macOS";
#[cfg(target_os = "macos")]
pub const HUGEPAGES_1GB: bool = false;

#[cfg(target_os = "linux")]
pub const OS: &'static str = "🐧 Linux";
#[cfg(target_os = "linux")]
pub const HUGEPAGES_1GB: bool = true;

// Community Monerod nodes
pub const IP_RINO: &'static str = "node.community.rino.io";
pub const RPC_RINO: u16 = 18081;
pub const ZMQ_RINO: u16 = 18083;

pub const IP_SETH: &'static str = "node.sethforprivacy.com";
pub const RPC_SETH: u16 = 18089;
pub const ZMQ_SETH: u16 = 18083;

pub const IP_SELSTA: &'static str = "selsta1.featherwallet.net";
pub const RPC_SELSTA: u16 = 18081;
pub const ZMQ_SELSTA: u16 = 18083;

// Tooltips
// Gupax
pub const GUPAX_CHECK_FOR_UPDATES: &'static str = "Check for Gupax, P2Pool, and XMRig updates via GitHub's API";
pub const GUPAX_UPGRADE: &'static str = "Upgrade anything that is out-of-date";
pub const GUPAX_AUTO_UPDATE: &'static str = "Automatically check for updates at startup";
pub const GUPAX_ASK_BEFORE_QUIT: &'static str = "Ask before quitting if processes are still alive";
pub const GUPAX_PATH_CONFIG: &'static str = "The location of the Gupax configuration file";
pub const GUPAX_PATH_P2POOL: &'static str = "The location of the P2Pool binary";
pub const GUPAX_PATH_XMRIG: &'static str = "The location of the XMRig binary";
// P2Pool
pub const P2POOL_MAIN: &'static str = "The P2Pool main-chain. This P2Pool finds shares faster, but has a higher difficulty. Suitable for miners with more than 50kH/s";
pub const P2POOL_MINI: &'static str = "The P2Pool mini-chain. This P2Pool finds shares slower, but has a lower difficulty. Suitable for miners with less than 50kH/s";
pub const P2POOL_OUT: &'static str = "How many out-bound peers (you connecting to others) to connect to?";
pub const P2POOL_IN: &'static str = "How many in-bound peers (others connecting to you) to connect to?";
pub const P2POOL_LOG: &'static str = "Verbosity of the console log";
pub const P2POOL_COMMUNITY: &'static str = "Connect to a community trusted Monero node: This is convenient because you don't have to download the Monero blockchain but it comes at the cost of privacy";
pub const P2POOL_MANUAL: &'static str = "Manually specify your own Monero node settings";
// XMRig
pub const XMRIG_P2POOL: &'static str = "Mine to your own P2Pool instance (localhost:3333)";
pub const XMRIG_MANUAL: &'static str = "Manually specify where to mine to";
pub const XMRIG_TLS: &'static str = "Enable SSL/TLS connections (needs pool support)";
pub const XMRIG_HUGEPAGES_JIT: &'static str = "Enable hugepages for RandomX JIT code. Note: 1GB hugepages is automatically enabled (only available on Linux)";
pub const XMRIG_NICEHASH: &'static str = "Enable nicehash.com support";
pub const XMRIG_KEEPALIVE: &'static str = "Send keepalived packet to prevent timeout (needs pool support)";
pub const XMRIG_THREADS: &'static str = "Number of CPU threads to use for mining";
pub const XMRIG_PRIORITY: &'static str = "Set process priority (0 idle, 2 normal to 5 highest)";
