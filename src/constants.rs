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

pub const GUPAX: &str = concat!("Gupax ", env!("CARGO_PKG_VERSION"));
pub const GUPAX_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
pub const P2POOL_VERSION: &str = "v2.4";
pub const XMRIG_VERSION: &str = "v6.18.0";
pub const COMMIT: &str = include_str!("../.git/refs/heads/main");

// App frame resolution, [4:3] aspect ratio, [1.33:1]
pub const APP_MIN_WIDTH: f32 = 640.0;
pub const APP_MIN_HEIGHT: f32 = 480.0;
pub const APP_MAX_WIDTH: f32 = 2560.0;
pub const APP_MAX_HEIGHT: f32 = 1920.0;
// Default, 1280x960
pub const APP_DEFAULT_WIDTH: f32 = 1280.0;
pub const APP_DEFAULT_HEIGHT: f32 = 960.0;

// Use macOS shaped icon for macOS
#[cfg(target_os = "macos")]
pub const BYTES_ICON: &[u8] = include_bytes!("../images/icons/icon@2x.png");
#[cfg(not(target_os = "macos"))]
pub const BYTES_ICON: &[u8] = include_bytes!("../images/icons/icon.png");
pub const BYTES_BANNER: &[u8] = include_bytes!("../images/banner.png");
pub const HORIZONTAL: &str = "--------------------------------------------";
// The text to separate my "process stopped, here's stats" text from the process output in the console.
pub const HORI_CONSOLE: &str = "---------------------------------------------------------------------------------------------------------------------------";

// P2Pool & XMRig default API stuff
#[cfg(target_os = "windows")]
pub const P2POOL_API_PATH: &str = r"local\stats"; // The default relative FS path of P2Pool's local API
#[cfg(target_family = "unix")]
pub const P2POOL_API_PATH: &str = "local/stats";
pub const XMRIG_API_URI: &str = "1/summary"; // The default relative URI of XMRig's API

// Process state tooltips (online, offline, etc)
pub const P2POOL_ALIVE:  &str = "P2Pool is online";
pub const P2POOL_DEAD:   &str = "P2Pool is offline";
pub const P2POOL_FAILED: &str = "P2Pool is offline, and failed when exiting";
pub const P2POOL_MIDDLE: &str = "P2Pool is in the middle of (re)starting/stopping";

pub const XMRIG_ALIVE:  &str = "XMRig is online";
pub const XMRIG_DEAD:   &str = "XMRig is offline";
pub const XMRIG_FAILED: &str = "XMRig is offline, and failed when exiting";
pub const XMRIG_MIDDLE: &str = "XMRig is in the middle of (re)starting/stopping";

// This is the typical space added when using
// [ui.separator()] or [ui.group()]
// Used for subtracting the width/height so
// things actually line up.
pub const SPACE: f32 = 10.0;

// Some colors
pub const RED: egui::Color32 = egui::Color32::from_rgb(230, 50, 50);
pub const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 230, 100);
pub const YELLOW: egui::Color32 = egui::Color32::from_rgb(230, 230, 100);
pub const BRIGHT_YELLOW: egui::Color32 = egui::Color32::from_rgb(250, 250, 100);
pub const GRAY: egui::Color32 = egui::Color32::GRAY;
pub const LIGHT_GRAY: egui::Color32 = egui::Color32::LIGHT_GRAY;
pub const BLACK: egui::Color32 = egui::Color32::BLACK;
pub const DARK_GRAY: egui::Color32 = egui::Color32::from_rgb(18, 18, 18);

// [Duration] constants
pub const SECOND: std::time::Duration = std::time::Duration::from_secs(1);
pub const ZERO_SECONDS: std::time::Duration = std::time::Duration::from_secs(0);
pub const MILLI_900: std::time::Duration = std::time::Duration::from_millis(900);
pub const TOKIO_SECOND: tokio::time::Duration = std::time::Duration::from_secs(1);

// The explaination given to the user on why XMRig needs sudo.
pub const XMRIG_ADMIN_REASON: &str =
r#"The large hashrate difference between XMRig and other miners like Monero and P2Pool's built-in miners is mostly due to XMRig configuring CPU MSRs and setting up hugepages. Other miners like Monero or P2Pool's built-in miner do not do this. It can be done manually but it isn't recommended since XMRig does this for you automatically, but only if it has the proper admin priviledges."#;
// Password buttons
pub const PASSWORD_TEXT: &str = "Enter sudo/admin password...";
pub const PASSWORD_LEAVE: &str = "Return to the previous screen";
pub const PASSWORD_ENTER: &str = "Attempt with the current password";
pub const PASSWORD_HIDE: &str = "Toggle hiding/showing the password";


// OS specific
#[cfg(target_os = "windows")]
pub const OS: &'static str = " Windows";
#[cfg(target_os = "windows")]
pub const OS_NAME: &'static str = "Windows";

#[cfg(target_os = "macos")]
pub const OS: &'static str = " macOS";
#[cfg(target_os = "macos")]
pub const OS_NAME: &'static str = "macOS";

#[cfg(target_os = "linux")]
pub const OS: &str = "🐧 Linux";
#[cfg(target_os = "linux")]
pub const OS_NAME: &str = "Linux";

// Tooltips
// Gupax
pub const GUPAX_UPDATE: &str = "Check for updates on Gupax, P2Pool, and XMRig via GitHub's API and upgrade automatically";
pub const GUPAX_AUTO_UPDATE: &str = "Automatically check for updates at startup";
pub const GUPAX_SHOULD_RESTART: &str = "Gupax was updated. A restart is recommended but not required";
pub const GUPAX_UP_TO_DATE: &str = "Gupax is up-to-date";
#[cfg(not(target_os = "macos"))]
pub const GUPAX_UPDATE_VIA_TOR: &str = "Update through the Tor network. Tor is embedded within Gupax; a Tor system proxy is not required";
#[cfg(target_os = "macos")] // Arti library has issues on macOS
pub const GUPAX_UPDATE_VIA_TOR: &'static str = "WARNING: This option is unstable on macOS. Update through the Tor network. Tor is embedded within Gupax; a Tor system proxy is not required";
pub const GUPAX_ASK_BEFORE_QUIT: &str = "Ask before quitting Gupax";
pub const GUPAX_SAVE_BEFORE_QUIT: &str = "Automatically save any changed settings before quitting";
pub const GUPAX_WIDTH: &str = "Set the width of the Gupax window";
pub const GUPAX_HEIGHT: &str = "Set the height of the Gupax window";
pub const GUPAX_LOCK_WIDTH: &str = "Automatically match the HEIGHT against the WIDTH in a 4:3 ratio";
pub const GUPAX_LOCK_HEIGHT: &str = "Automatically match the WIDTH against the HEIGHT in a 4:3 ratio";
pub const GUPAX_NO_LOCK: &str = "Allow individual selection of width and height";
pub const GUPAX_SET: &str = "Set the width/height of the Gupax window to the current values";
pub const GUPAX_TAB_ABOUT: &str = "Set the tab Gupax starts on to: About";
pub const GUPAX_TAB_STATUS: &str = "Set the tab Gupax starts on to: Status";
pub const GUPAX_TAB_GUPAX: &str = "Set the tab Gupax starts on to: Gupax";
pub const GUPAX_TAB_P2POOL: &str = "Set the tab Gupax starts on to: P2Pool";
pub const GUPAX_TAB_XMRIG: &str = "Set the tab Gupax starts on to: XMRig";

pub const GUPAX_SIMPLE: &str =
r#"Use simple Gupax settings:
    - Update button
    - Basic toggles"#;
pub const GUPAX_ADVANCED: &str =
r#"Use advanced Gupax settings:
    - Update button
    - Basic toggles
    - P2Pool/XMRig binary path selector
    - Gupax resolution sliders"#;
pub const GUPAX_SELECT: &str = "Open a file explorer to select a file";
pub const GUPAX_PATH_P2POOL: &str = "The location of the P2Pool binary: Both absolute and relative paths are accepted; A red [X] will appear if there is no file found at the given path";
pub const GUPAX_PATH_XMRIG: &str = "The location of the XMRig binary: Both absolute and relative paths are accepted; A red [X] will appear if there is no file found at the given path";

// P2Pool
pub const P2POOL_MAIN: &str = "Use the P2Pool main-chain. This P2Pool finds shares faster, but has a higher difficulty. Suitable for miners with more than 50kH/s";
pub const P2POOL_MINI: &str = "Use the P2Pool mini-chain. This P2Pool finds shares slower, but has a lower difficulty. Suitable for miners with less than 50kH/s";
pub const P2POOL_OUT: &str = "How many out-bound peers to connect to? (you connecting to others)";
pub const P2POOL_IN: &str = "How many in-bound peers to allow? (others connecting to you)";
pub const P2POOL_LOG: &str = "Verbosity of the console log";
pub const P2POOL_AUTO_NODE: &str = "Automatically ping the community Monero nodes at Gupax startup";
pub const P2POOL_AUTO_SELECT: &str = "Automatically select the fastest community Monero node after pinging";
pub const P2POOL_SELECT_FASTEST: &str = "Select the fastest community Monero node";
pub const P2POOL_PING: &str = "Ping the built-in community Monero nodes";
pub const P2POOL_ADDRESS: &str = "You must use a primary Monero address to mine on P2Pool (starts with a 4). It is highly recommended to create a new wallet for P2Pool mining; wallet addresses are public on P2Pool!";
pub const P2POOL_ARGUMENTS: &str = "Start P2Pool with these arguments and override all below settings; If the [--data-api] & [--local-api] flag is not given, Gupax will append it to the arguments automatically so that the [Status] tab can work";
pub const P2POOL_SIMPLE: &str =
r#"Use simple P2Pool settings:
    - Remote community Monero node
    - Default P2Pool settings + Mini"#;
pub const P2POOL_ADVANCED: &str =
r#"Use advanced P2Pool settings:
    - Overriding command arguments
    - Manual node list
    - P2Pool Main/Mini selection
    - Out/In peer setting
    - Log level setting"#;
pub const P2POOL_NAME: &str = "Add a unique name to identify this node; Only [A-Za-z0-9-_] and spaces allowed; Max length = 30 characters";
pub const P2POOL_NODE_IP: &str = "Specify the Monero Node IP to connect to with P2Pool; It must be a valid IPv4 address or a valid domain name; Max length = 255 characters";
pub const P2POOL_RPC_PORT: &str = "Specify the RPC port of the Monero node; [1-65535]";
pub const P2POOL_ZMQ_PORT: &str = "Specify the ZMQ port of the Monero node; [1-65535]";

// Node/Pool list
pub const LIST_ADD: &str = "Add the current values to the list";
pub const LIST_SAVE: &str = "Save the current values to the already existing entry";
pub const LIST_DELETE: &str = "Delete the currently selected entry";
pub const LIST_CLEAR: &str = "Clear all current values";

// XMRig
pub const XMRIG_SIMPLE: &str =
r#"Use simple XMRig settings:
	- Mine to local P2Pool (localhost:3333)
	- CPU thread slider
	- HTTP API @ localhost:18088"#;
pub const XMRIG_ADVANCED: &str =
r#"Use advanced XMRig settings:
	- Overriding config file
	- Custom payout address
	- CPU thread slider
	- Manual pool list
	- TLS setting
	- Keepalive setting
	- Custom HTTP API IP/Port"#;
pub const XMRIG_ARGUMENTS: &str = "Start XMRig with these arguments and override all below settings; If the [http-api] options are not set, Gupax will append it to the arguments automatically so that the [Status] tab can work";
pub const XMRIG_ADDRESS: &str = "Specify which Monero address to send payouts to; Must be a valid primary address (starts with 4)";
pub const XMRIG_NAME: &str = "Add a unique name to identify this pool; Only [A-Za-z0-9-_] and spaces allowed; Max length = 30 characters";
pub const XMRIG_IP: &str = "Specify the pool IP to connect to with XMRig; It must be a valid IPv4 address or a valid domain name; Max length = 255 characters";
pub const XMRIG_PORT: &str = "Specify the port of the pool; [1-65535]";
pub const XMRIG_RIG: &str = "Add an optional rig ID. This will be the name shown on the pool; Only [A-Za-z0-9-_] and spaces allowed; Max length = 30 characters";
pub const XMRIG_PAUSE: &str = "THIS SETTING IS DISABLED IF SET TO [0]. Pause mining if user is active, resume after";
pub const XMRIG_API_IP: &str = "Specify which IP to bind to for XMRig's HTTP API";
pub const XMRIG_API_PORT: &str = "Specify which port to bind to for XMRig's HTTP API";
pub const XMRIG_TLS: &str = "Enable SSL/TLS connections (needs pool support)";
pub const XMRIG_KEEPALIVE: &str = "Send keepalived packet to prevent timeout (needs pool support)";
pub const XMRIG_THREADS: &str = "Number of CPU threads to use for mining";

// CLI argument messages
pub const ARG_HELP: &str =
r#"USAGE: ./gupax [--flag]

    --help         Print this help message
    --version      Print version and build info
    --state        Print Gupax state
    --nodes        Print the manual node list
    --no-startup   Disable all auto-startup settings for this instance
    --reset-state  Reset all Gupax state (your settings)
    --reset-nodes  Reset the manual node list in the [P2Pool] tab
    --reset-pools  Reset the manual pool list in the [XMRig] tab
    --reset-all    Reset the state, the manual node list, and the manual pool list
    --ferris       Print an extremely cute crab

To view more detailed console debug information, start Gupax with
the environment variable [RUST_LOG] set to a log level like so:
    RUST_LOG=(trace|debug|info|warn|error) ./gupax"#;
pub const ARG_COPYRIGHT: &str =
r#"Gupax is licensed under GPLv3.
For more information, see link below:
<https://github.com/hinto-janaiyo/gupax>"#;
