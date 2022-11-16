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

pub const GUPAX_VERSION: &'static str = concat!("v", env!("CARGO_PKG_VERSION"));
pub const P2POOL_VERSION: &'static str = "v2.4";
pub const XMRIG_VERSION: &'static str = "v6.18.0";
pub const COMMIT: &'static str = include_str!("../.git/refs/heads/main");

// Use macOS shaped icon for macOS
#[cfg(target_os = "macos")]
pub const BYTES_ICON: &[u8] = include_bytes!("../images/icons/icon@2x.png");
#[cfg(not(target_os = "macos"))]
pub const BYTES_ICON: &[u8] = include_bytes!("../images/icons/icon.png");
pub const BYTES_BANNER: &[u8] = include_bytes!("../images/banner.png");
pub const P2POOL_BASE_ARGS: &'static str = "";
pub const XMRIG_BASE_ARGS: &'static str = "--http-host=127.0.0.1 --http-port=18088 --algo=rx/0 --coin=Monero --randomx-cache-qos";
pub const HORIZONTAL: &'static str = "--------------------------------------------";

// This is the typical space added when using
// [ui.separator()] or [ui.group()]
// Used for subtracting the width/height so
// things actually line up.
pub const SPACE: f32 = 10.0;

// OS specific
#[cfg(target_os = "windows")]
pub const OS: &'static str = " Windows";
#[cfg(target_os = "windows")]
pub const OS_NAME: &'static str = "Windows";
#[cfg(target_os = "windows")]
pub const HUGEPAGES_1GB: bool = false;

#[cfg(target_os = "macos")]
pub const OS: &'static str = " macOS";
#[cfg(target_os = "macos")]
pub const OS_NAME: &'static str = "macOS";
#[cfg(target_os = "macos")]
pub const HUGEPAGES_1GB: bool = false;

#[cfg(target_os = "linux")]
pub const OS: &'static str = "🐧 Linux";
#[cfg(target_os = "linux")]
pub const OS_NAME: &'static str = "Linux";
#[cfg(target_os = "linux")]
pub const HUGEPAGES_1GB: bool = true;

// Tooltips
// Gupax
pub const GUPAX_UPDATE: &'static str = "Check for updates on Gupax, P2Pool, and XMRig via GitHub's API and upgrade automatically";
pub const GUPAX_AUTO_UPDATE: &'static str = "Automatically check for updates at startup";
pub const GUPAX_UPDATE_VIA_TOR: &'static str = "Update through the Tor network. Tor is embedded within Gupax; a Tor system proxy is not required";
pub const GUPAX_AUTO_NODE: &'static str = "Automatically ping the community Monero nodes and select the fastest at startup for P2Pool";
pub const GUPAX_ASK_BEFORE_QUIT: &'static str = "Ask before quitting Gupax";
pub const GUPAX_SAVE_BEFORE_QUIT: &'static str = "Automatically save any changed settings before quitting";
pub const GUPAX_PATH_P2POOL: &'static str = "The location of the P2Pool binary: Both absolute and relative paths are accepted; A red [X] will appear if there is no file found at the given path";
pub const GUPAX_PATH_XMRIG: &'static str = "The location of the XMRig binary: Both absolute and relative paths are accepted; A red [X] will appear if there is no file found at the given path";
// P2Pool
pub const P2POOL_MAIN: &'static str = "Use the P2Pool main-chain. This P2Pool finds shares faster, but has a higher difficulty. Suitable for miners with more than 50kH/s";
pub const P2POOL_MINI: &'static str = "Use the P2Pool mini-chain. This P2Pool finds shares slower, but has a lower difficulty. Suitable for miners with less than 50kH/s";
pub const P2POOL_OUT: &'static str = "How many out-bound peers to connect to? (you connecting to others)";
pub const P2POOL_IN: &'static str = "How many in-bound peers to allow? (others connecting to you)";
pub const P2POOL_LOG: &'static str = "Verbosity of the console log";
pub const P2POOL_COMMUNITY: &'static str = "Connect to a community trusted Monero node: This is convenient because you don't have to download the Monero blockchain but it comes at the cost of privacy";
pub const P2POOL_MANUAL: &'static str = "Manually specify your own Monero node settings";
pub const P2POOL_AUTO_NODE: &'static str = "Automatically ping the community Monero nodes at Gupax startup";
pub const P2POOL_AUTO_SELECT: &'static str = "Automatically select the fastest community Monero node after pinging";
pub const P2POOL_SELECT_FASTEST: &'static str = "Select the fastest community Monero node";
pub const P2POOL_PING: &'static str = "Ping the built-in community Monero nodes";
pub const P2POOL_ADDRESS: &'static str = "You must use a primary Monero address to mine on P2Pool (starts with a 4). It is highly recommended to create a new wallet for P2Pool mining; wallet addresses are public on P2Pool!";
pub const P2POOL_COMMAND: &'static str = "Start P2Pool with these arguments and override all below settings; If the [--data-api] flag is not given, Gupax will append it to the arguments automatically so that the [Status] tab can work";
pub const P2POOL_SIMPLE: &'static str =
r#"Use simple settings:
    - Remote community Monero node
    - Default P2Pool settings + Mini"#;
pub const P2POOL_ADVANCED: &'static str =
r#"Use advanced settings:
    - Overriding command arguments
    - Manual node selection
    - P2Pool Main/Mini selection
    - Out/In peer setting
    - Log level setting"#;
pub const P2POOL_NAME: &'static str = "Add a unique name to identify this node; Only [A-Za-z0-9-_] and spaces allowed; Max length = 30 characters";
pub const P2POOL_NODE_IP: &'static str = "Specify the Monero Node IP to connect to with P2Pool; It must be a valid IPv4 address or a valid domain name; Max length = 255 characters";
pub const P2POOL_RPC_PORT: &'static str = "Specify the RPC port of the Monero node; [1-65535]";
pub const P2POOL_ZMQ_PORT: &'static str = "Specify the ZMQ port of the Monero node; [1-65535]";
pub const P2POOL_ADD: &'static str = "Add the current values to the list";
pub const P2POOL_DELETE: &'static str = "Delete the currently selected node";
pub const P2POOL_CLEAR: &'static str = "Clear all current values";

// XMRig
pub const XMRIG_P2POOL: &'static str = "Mine to your own P2Pool instance (localhost:3333)";
pub const XMRIG_MANUAL: &'static str = "Manually specify where to mine to";
pub const XMRIG_TLS: &'static str = "Enable SSL/TLS connections (needs pool support)";
pub const XMRIG_HUGEPAGES_JIT: &'static str = "Enable hugepages for RandomX JIT code. Note: 1GB hugepages is automatically enabled (only available on Linux)";
pub const XMRIG_NICEHASH: &'static str = "Enable nicehash.com support";
pub const XMRIG_KEEPALIVE: &'static str = "Send keepalived packet to prevent timeout (needs pool support)";
pub const XMRIG_THREADS: &'static str = "Number of CPU threads to use for mining";
pub const XMRIG_PRIORITY: &'static str = "Set process priority (0 idle, 2 normal to 5 highest)";

// CLI argument messages
pub const ARG_HELP: &'static str =
r#"USAGE: ./gupax [--flags]

    -h | --help         Print this help message
    -v | --version      Print version and build info
    -l | --node-list    Print the manual node list
    -s | --state        Print Gupax state
    -n | --no-startup   Disable all auto-startup settings for this instance
    -r | --reset        Reset all Gupax state and the manual node list
    -f | --ferris       Print an extremely cute crab

To view more detailed console debug information, start Gupax with
the environment variable [RUST_LOG] set to a log level like so:
    RUST_LOG=(trace|debug|info|warn|error) ./gupax"#;
pub const ARG_COPYRIGHT: &'static str =
r#"Gupax is licensed under GPLv3.
For more information, see link below:
<https://github.com/hinto-janaiyo/gupax>"#;
