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
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub const GUPAX_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION")); // e.g: v1.0.0
pub const P2POOL_VERSION: &str = "v3.10";
pub const XMRIG_VERSION: &str = "v6.21.1";
pub const COMMIT: &str = env!("COMMIT"); // set in build.rs
                                         // e.g: Gupax_v1_0_0
                                         // Would have been [Gupax_v1.0.0] but P2Pool truncates everything after [.]
pub const GUPAX_VERSION_UNDERSCORE: &str = concat!(
    "Gupax_v",
    env!("CARGO_PKG_VERSION_MAJOR"),
    "_",
    env!("CARGO_PKG_VERSION_MINOR"),
    "_",
    env!("CARGO_PKG_VERSION_PATCH"),
);

// App frame resolution, [4:3] aspect ratio, [1.33:1]
pub const APP_MIN_WIDTH: f32 = 640.0;
pub const APP_MIN_HEIGHT: f32 = 480.0;
pub const APP_MAX_WIDTH: f32 = 3840.0;
pub const APP_MAX_HEIGHT: f32 = 2160.0;
// Default, 1280x960
pub const APP_DEFAULT_WIDTH: f32 = 1280.0;
pub const APP_DEFAULT_HEIGHT: f32 = 960.0;
// App resolution scaling
pub const APP_MIN_SCALE: f32 = 0.1;
pub const APP_MAX_SCALE: f32 = 2.0;
pub const APP_DEFAULT_SCALE: f32 = 1.0;

// Constants specific for Linux distro packaging of Gupax
#[cfg(feature = "distro")]
pub const DISTRO_NO_UPDATE: &str = r#"This [Gupax] was compiled for use as a Linux distro package. Built-in updates are disabled. The below settings [Update-via-Tor] & [Auto-Update] will not do anything. Please use your package manager to update [Gupax/P2Pool/XMRig]."#;

// Use macOS shaped icon for macOS
#[cfg(target_os = "macos")]
pub const BYTES_ICON: &[u8] = include_bytes!("../images/icons/icon@2x.png");
#[cfg(not(target_os = "macos"))]
pub const BYTES_ICON: &[u8] = include_bytes!("../images/icons/icon.png");
pub const BYTES_BANNER: &[u8] = include_bytes!("../images/banner.png");
pub const HORIZONTAL: &str = "--------------------------------------------";
pub const HORI_CONSOLE: &str = "---------------------------------------------------------------------------------------------------------------------------";

// Keyboard shortcuts
pub const KEYBOARD_SHORTCUTS: &str = r#"*---------------------------------------*
|             Key shortcuts             |
|---------------------------------------|
|             F11 | Fullscreen          |
|          Escape | Quit screen         |
|              Up | Start/Restart       |
|            Down | Stop                |
|               Z | Left Tab            |
|               X | Right Tab           |
|               C | Left Submenu        |
|               V | Right Submenu       |
|               S | Save                |
|               R | Reset               |
*---------------------------------------*"#;
// P2Pool & XMRig default API stuff
#[cfg(target_os = "windows")]
pub const P2POOL_API_PATH_LOCAL: &str = r"local\stratum";
#[cfg(target_os = "windows")]
pub const P2POOL_API_PATH_NETWORK: &str = r"network\stats";
#[cfg(target_os = "windows")]
pub const P2POOL_API_PATH_POOL: &str = r"pool\stats";
#[cfg(target_family = "unix")]
pub const P2POOL_API_PATH_LOCAL: &str = "local/stratum";
#[cfg(target_family = "unix")]
pub const P2POOL_API_PATH_NETWORK: &str = "network/stats";
#[cfg(target_family = "unix")]
pub const P2POOL_API_PATH_POOL: &str = "pool/stats";
pub const XMRIG_API_URI: &str = "1/summary"; // The default relative URI of XMRig's API

// Process state tooltips (online, offline, etc)
pub const P2POOL_ALIVE: &str = "P2Pool is online and fully synchronized";
pub const P2POOL_DEAD: &str = "P2Pool is offline";
pub const P2POOL_FAILED: &str = "P2Pool is offline and failed when exiting";
pub const P2POOL_MIDDLE: &str = "P2Pool is in the middle of (re)starting/stopping";
pub const P2POOL_SYNCING: &str =
    "P2Pool is still syncing. This indicator will turn GREEN when P2Pool is ready";

pub const XMRIG_ALIVE: &str = "XMRig is online and mining";
pub const XMRIG_DEAD: &str = "XMRig is offline";
pub const XMRIG_FAILED: &str = "XMRig is offline and failed when exiting";
pub const XMRIG_MIDDLE: &str = "XMRig is in the middle of (re)starting/stopping";
pub const XMRIG_NOT_MINING: &str = "XMRig is online, but not mining to any pool";

// This is the typical space added when using
// [ui.separator()] or [ui.group()]
// Used for subtracting the width/height so
// things actually line up.
pub const SPACE: f32 = 10.0;

// Some colors
pub const RED: egui::Color32 = egui::Color32::from_rgb(230, 50, 50);
pub const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 230, 100);
pub const BLUE: egui::Color32 = egui::Color32::from_rgb(100, 175, 255);
pub const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 120, 40);
pub const YELLOW: egui::Color32 = egui::Color32::from_rgb(230, 230, 100);
pub const BRIGHT_YELLOW: egui::Color32 = egui::Color32::from_rgb(250, 250, 100);
pub const BONE: egui::Color32 = egui::Color32::from_rgb(190, 190, 190); // In between LIGHT_GRAY <-> GRAY
pub const WHITE: egui::Color32 = egui::Color32::WHITE;
pub const GRAY: egui::Color32 = egui::Color32::GRAY;
pub const LIGHT_GRAY: egui::Color32 = egui::Color32::LIGHT_GRAY;
pub const BLACK: egui::Color32 = egui::Color32::BLACK;
pub const DARK_GRAY: egui::Color32 = egui::Color32::from_gray(13);

// [Duration] constants
pub const SECOND: std::time::Duration = std::time::Duration::from_secs(1);

// The explanation given to the user on why XMRig needs sudo.
pub const XMRIG_ADMIN_REASON: &str = r#"The large hashrate difference between XMRig and other miners like Monero and P2Pool's built-in miners is mostly due to XMRig configuring CPU MSRs and setting up hugepages. Other miners like Monero or P2Pool's built-in miner do not do this. It can be done manually but it isn't recommended since XMRig does this for you automatically, but only if it has the proper admin privileges."#;
// Password buttons
pub const PASSWORD_TEXT: &str = "Enter sudo/admin password...";
pub const PASSWORD_LEAVE: &str = "Return to the previous screen";
pub const PASSWORD_ENTER: &str = "Attempt with the current password";
pub const PASSWORD_HIDE: &str = "Toggle hiding/showing the password";

// OS specific
#[cfg(target_os = "windows")]
pub const OS: &str = " Windows";
#[cfg(target_os = "windows")]
pub const OS_NAME: &str = "Windows";
#[cfg(target_os = "windows")]
pub const WINDOWS_NOT_ADMIN: &str = "XMRig will most likely mine slower than normal without Administrator permissions. Please consider restarting Gupax as an Administrator.";

#[cfg(target_os = "macos")]
pub const OS: &str = " macOS";
#[cfg(target_os = "macos")]
pub const OS_NAME: &str = "macOS";

#[cfg(target_os = "linux")]
pub const OS: &str = "🐧 Linux";
#[cfg(target_os = "linux")]
pub const OS_NAME: &str = "Linux";

// Tooltips
// Status
pub const STATUS_GUPAX_UPTIME: &str = "How long Gupax has been online";
pub const STATUS_GUPAX_CPU_USAGE: &str =
    "How much CPU Gupax is currently using. This accounts for all your threads (it is out of 100%)";
pub const STATUS_GUPAX_MEMORY_USAGE: &str = "How much memory Gupax is currently using in Megabytes";
pub const STATUS_GUPAX_SYSTEM_CPU_USAGE: &str = "How much CPU your entire system is currently using. This accounts for all your threads (it is out of 100%)";
pub const STATUS_GUPAX_SYSTEM_MEMORY: &str =
    "How much memory your entire system has (including swap) and is currently using in Gigabytes";
pub const STATUS_GUPAX_SYSTEM_CPU_MODEL: &str =
    "The detected model of your system's CPU and its current frequency";
//--
pub const STATUS_P2POOL_UPTIME: &str = "How long P2Pool has been online";
pub const STATUS_P2POOL_PAYOUTS:     &str = "The total amount of payouts received in this instance of P2Pool and an extrapolated estimate of how many you will receive. Warning: these stats will be quite inaccurate if your P2Pool hasn't been running for a long time!";
pub const STATUS_P2POOL_XMR:         &str = "The total amount of XMR mined in this instance of P2Pool and an extrapolated estimate of how many you will mine in the future. Warning: these stats will be quite inaccurate if your P2Pool hasn't been running for a long time!";
pub const STATUS_P2POOL_HASHRATE:    &str = "The total amount of hashrate your P2Pool has pointed at it in 15 minute, 1 hour, and 24 hour averages";
pub const STATUS_P2POOL_SHARES: &str = "The total amount of shares found on P2Pool";
pub const STATUS_P2POOL_EFFORT: &str =
    "The average amount of effort needed to find a share, and the current effort";
pub const STATUS_P2POOL_CONNECTIONS: &str = "The total amount of miner connections on this P2Pool";
pub const STATUS_P2POOL_MONERO_NODE: &str = "The Monero node being used by P2Pool";
pub const STATUS_P2POOL_POOL: &str = "The P2Pool sidechain you're currently connected to";
pub const STATUS_P2POOL_ADDRESS: &str = "The Monero address P2Pool will send payouts to";
//--
pub const STATUS_XMRIG_UPTIME: &str = "How long XMRig has been online";
pub const STATUS_XMRIG_CPU:         &str = "The average CPU load of XMRig. [1.0] represents 1 thread is maxed out, e.g: If you have 8 threads, [4.0] means half your threads are maxed out.";
pub const STATUS_XMRIG_HASHRATE: &str = "The average hashrate of XMRig";
pub const STATUS_XMRIG_DIFFICULTY: &str = "The current difficulty of the job XMRig is working on";
pub const STATUS_XMRIG_SHARES: &str = "The amount of accepted and rejected shares";
pub const STATUS_XMRIG_POOL: &str = "The pool XMRig is currently mining to";
pub const STATUS_XMRIG_THREADS: &str = "The amount of threads XMRig is currently using";
// Status Submenus
pub const STATUS_SUBMENU_PROCESSES: &str =
    "View the status of process related data for [Gupax|P2Pool|XMRig]";
pub const STATUS_SUBMENU_P2POOL: &str = "View P2Pool specific data";
pub const STATUS_SUBMENU_HASHRATE: &str = "Compare your CPU hashrate with others";
//-- P2Pool
pub const STATUS_SUBMENU_PAYOUT:    &str = "The total amount of payouts received via P2Pool across all time. This includes all payouts you have ever received using Gupax and P2Pool.";
pub const STATUS_SUBMENU_XMR:       &str = "The total of XMR mined via P2Pool across all time. This includes all the XMR you have ever mined using Gupax and P2Pool.";
pub const STATUS_SUBMENU_LATEST: &str = "Sort the payouts from latest to oldest";
pub const STATUS_SUBMENU_OLDEST: &str = "Sort the payouts from oldest to latest";
pub const STATUS_SUBMENU_BIGGEST: &str = "Sort the payouts from biggest to smallest";
pub const STATUS_SUBMENU_SMALLEST: &str = "Sort the payouts from smallest to biggest";
pub const STATUS_SUBMENU_AUTOMATIC: &str =
    "Automatically calculate share/block time with your current P2Pool 1 hour average hashrate";
pub const STATUS_SUBMENU_MANUAL:    &str = "Manually input a hashrate to calculate share/block time with current P2Pool/Monero network stats";
pub const STATUS_SUBMENU_HASH: &str = "Use [Hash] as the hashrate metric";
pub const STATUS_SUBMENU_KILO: &str = "Use [Kilo] as the hashrate metric (1,000x hash)";
pub const STATUS_SUBMENU_MEGA: &str = "Use [Mega] as the hashrate metric (1,000,000x hash)";
pub const STATUS_SUBMENU_GIGA: &str = "Use [Giga] as the hashrate metric (1,000,000,000x hash)";
pub const STATUS_SUBMENU_P2POOL_BLOCK_MEAN: &str =
    "The average time it takes for P2Pool to find a block";
pub const STATUS_SUBMENU_YOUR_P2POOL_HASHRATE: &str = "Your 1 hour average hashrate on P2Pool";
pub const STATUS_SUBMENU_P2POOL_SHARE_MEAN: &str =
    "The average time it takes for your hashrate to find a share on P2Pool";
pub const STATUS_SUBMENU_SOLO_BLOCK_MEAN: &str =
    "The average time it would take for your hashrate to find a block solo mining Monero";
pub const STATUS_SUBMENU_MONERO_DIFFICULTY:     &str = "The current Monero network's difficulty (how many hashes it will take on average to find a block)";
pub const STATUS_SUBMENU_MONERO_HASHRATE: &str = "The current Monero network's hashrate";
pub const STATUS_SUBMENU_P2POOL_DIFFICULTY:     &str = "The current P2Pool network's difficulty (how many hashes it will take on average to find a share)";
pub const STATUS_SUBMENU_P2POOL_HASHRATE: &str = "The current P2Pool network's hashrate";
pub const STATUS_SUBMENU_P2POOL_MINERS: &str = "The current amount of miners on P2Pool";
pub const STATUS_SUBMENU_P2POOL_DOMINANCE: &str =
    "The percent of hashrate P2Pool accounts for in the entire Monero network";
pub const STATUS_SUBMENU_YOUR_P2POOL_DOMINANCE: &str =
    "The percent of hashrate you account for in P2Pool";
pub const STATUS_SUBMENU_YOUR_MONERO_DOMINANCE: &str =
    "The percent of hashrate you account for in the entire Monero network";
pub const STATUS_SUBMENU_PROGRESS_BAR:          &str = "The next time Gupax will update P2Pool stats. Each [*] is 900ms (updates roughly every 54 seconds)";
//-- Benchmarks
pub const STATUS_SUBMENU_YOUR_CPU: &str = "The CPU detected by Gupax";
pub const STATUS_SUBMENU_YOUR_BENCHMARKS: &str =
    "How many benchmarks your CPU has had uploaded to [https://xmrig.com/benchmark] ";
pub const STATUS_SUBMENU_YOUR_RANK: &str =
    "Your CPU's rank out of all CPUs listed on [https://xmrig.com/benchmark] (higher is better)";
pub const STATUS_SUBMENU_YOUR_HIGH: &str =
    "The highest hashrate recorded for your CPU on [https://xmrig.com/benchmark]";
pub const STATUS_SUBMENU_YOUR_AVERAGE: &str =
    "The average hashrate of your CPU based off the data at [https://xmrig.com/benchmark]";
pub const STATUS_SUBMENU_YOUR_LOW: &str =
    "The lowest hashrate recorded for your CPU on [https://xmrig.com/benchmark]";
pub const STATUS_SUBMENU_OTHER_CPUS:       &str = "A list of ALL the recorded CPU benchmarks. The CPUs most similar to yours are listed first. All this data is taken from [https://xmrig.com/benchmark].";
pub const STATUS_SUBMENU_OTHER_CPU: &str = "The CPU name";
pub const STATUS_SUBMENU_OTHER_RELATIVE:   &str = "The relative hashrate power compared to the fastest recorded CPU, which is current: [AMD EPYC 7T83 64-Core Processor]";
pub const STATUS_SUBMENU_OTHER_HIGH: &str = "Highest hashrate record";
pub const STATUS_SUBMENU_OTHER_AVERAGE: &str = "Average hashrate";
pub const STATUS_SUBMENU_OTHER_LOW: &str = "Lowest hashrate record";
pub const STATUS_SUBMENU_OTHER_RANK: &str = "The rank of this CPU out of [1567] (lower is better)";
pub const STATUS_SUBMENU_OTHER_BENCHMARKS: &str =
    "How many benchmarks this CPU has had posted to [https://xmrig.com/benchmark]";

// Gupax
pub const GUPAX_UPDATE: &str =
    "Check for updates on Gupax, P2Pool, and XMRig via GitHub's API and upgrade automatically";
pub const GUPAX_AUTO_UPDATE: &str = "Automatically check for updates at startup";
pub const GUPAX_SHOULD_RESTART: &str =
    "Gupax was updated. A restart is recommended but not required";
pub const GUPAX_UP_TO_DATE: &str = "Gupax is up-to-date";
#[cfg(not(target_os = "macos"))]
pub const GUPAX_UPDATE_VIA_TOR:   &str = "Update through the Tor network. Tor is embedded within Gupax; a Tor system proxy is not required";
#[cfg(target_os = "macos")] // Arti library has issues on macOS
pub const GUPAX_UPDATE_VIA_TOR:   &str = "WARNING: This option is unstable on macOS. Update through the Tor network. Tor is embedded within Gupax; a Tor system proxy is not required";
pub const GUPAX_ASK_BEFORE_QUIT: &str = "Ask before quitting Gupax";
pub const GUPAX_SAVE_BEFORE_QUIT: &str = "Automatically save any changed settings before quitting";
pub const GUPAX_AUTO_P2POOL:      &str = "Automatically start P2Pool on Gupax startup. If you are using [P2Pool Simple], this will NOT wait for your [Auto-Ping] to finish, it will start P2Pool on the pool you already have selected. This option will fail if your P2Pool settings aren't valid!";
pub const GUPAX_AUTO_XMRIG:       &str = "Automatically start XMRig on Gupax startup. This option will fail if your XMRig settings aren't valid!";
pub const GUPAX_ADJUST: &str = "Adjust and set the width/height of the Gupax window";
pub const GUPAX_WIDTH: &str = "Set the width of the Gupax window";
pub const GUPAX_HEIGHT: &str = "Set the height of the Gupax window";
pub const GUPAX_SCALE: &str =
    "Set the resolution scaling of the Gupax window (resize window to re-apply scaling)";
pub const GUPAX_LOCK_WIDTH: &str =
    "Automatically match the HEIGHT against the WIDTH in a 4:3 ratio";
pub const GUPAX_LOCK_HEIGHT: &str =
    "Automatically match the WIDTH against the HEIGHT in a 4:3 ratio";
pub const GUPAX_NO_LOCK: &str = "Allow individual selection of width and height";
pub const GUPAX_SET: &str = "Set the width/height of the Gupax window to the current values";
pub const GUPAX_TAB: &str = "Set the default tab Gupax starts on";
pub const GUPAX_TAB_ABOUT: &str = "Set the tab Gupax starts on to: About";
pub const GUPAX_TAB_STATUS: &str = "Set the tab Gupax starts on to: Status";
pub const GUPAX_TAB_GUPAX: &str = "Set the tab Gupax starts on to: Gupax";
pub const GUPAX_TAB_P2POOL: &str = "Set the tab Gupax starts on to: P2Pool";
pub const GUPAX_TAB_XMRIG: &str = "Set the tab Gupax starts on to: XMRig";

pub const GUPAX_SIMPLE: &str = r#"Use simple Gupax settings:
  - Update button
  - Basic toggles"#;
pub const GUPAX_ADVANCED: &str = r#"Use advanced Gupax settings:
  - Update button
  - Basic toggles
  - P2Pool/XMRig binary path selector
  - Gupax resolution sliders
  - Gupax start-up tab selector"#;
pub const GUPAX_SELECT: &str = "Open a file explorer to select a file";
pub const GUPAX_PATH: &str = "Use custom PATHs when looking for P2Pool/XMRig";
pub const GUPAX_PATH_P2POOL: &str = "The location of the P2Pool binary: Both absolute and relative paths are accepted; A red [X] will appear if there is no file found at the given path";
pub const GUPAX_PATH_XMRIG: &str = "The location of the XMRig binary: Both absolute and relative paths are accepted; A red [X] will appear if there is no file found at the given path";

// P2Pool
pub const P2POOL_MAIN:                   &str = "Use the P2Pool main-chain. This P2Pool finds blocks faster, but has a higher difficulty. Suitable for miners with more than 50kH/s";
pub const P2POOL_MINI:                   &str = "Use the P2Pool mini-chain. This P2Pool finds blocks slower, but has a lower difficulty. Suitable for miners with less than 50kH/s";
pub const P2POOL_OUT: &str = "How many out-bound peers to connect to? (you connecting to others)";
pub const P2POOL_IN: &str = "How many in-bound peers to allow? (others connecting to you)";
pub const P2POOL_LOG: &str = "Verbosity of the console log";
pub const P2POOL_AUTO_NODE: &str = "Automatically ping the remote Monero nodes at Gupax startup";
pub const P2POOL_AUTO_SELECT: &str =
    "Automatically select the fastest remote Monero node after pinging";
pub const P2POOL_BACKUP_HOST_SIMPLE: &str = r#"Automatically switch to the other nodes listed if the current one is down.

Note: you must ping the remote nodes or this feature will default to only using the currently selected node."#;
pub const P2POOL_BACKUP_HOST_ADVANCED: &str =
    "Automatically switch to the other nodes in your list if the current one is down.";
pub const P2POOL_SELECT_FASTEST: &str = "Select the fastest remote Monero node";
pub const P2POOL_SELECT_RANDOM: &str = "Select a random remote Monero node";
pub const P2POOL_SELECT_LAST: &str = "Select the previous remote Monero node";
pub const P2POOL_SELECT_NEXT: &str = "Select the next remote Monero node";
pub const P2POOL_PING: &str = "Ping the built-in remote Monero nodes";
pub const P2POOL_ADDRESS:                &str = "You must use a primary Monero address to mine on P2Pool (starts with a 4). It is highly recommended to create a new wallet since addresses are public on P2Pool!";
pub const P2POOL_COMMUNITY_NODE_WARNING: &str = r#"TL;DR: Run & use your own Monero Node.

Using a Remote Monero Node is convenient but comes at the cost of privacy and reliability.

You may encounter connection issues with remote nodes which may cause mining performance loss! Late info from laggy nodes will cause your mining jobs to start later than they should.

Running and using your own local Monero node improves privacy and ensures your connection is as stable as your own internet connection. This comes at the cost of downloading and syncing Monero's blockchain yourself (currently 170GB). If you have the disk space, consider using the [Advanced] tab and connecting to your own Monero node.

For a simple guide, see the [Running a Local Monero Node] section on Gupax's GitHub by clicking this message."#;

pub const P2POOL_INPUT: &str = "Send a command to P2Pool";
pub const P2POOL_ARGUMENTS: &str = r#"WARNING: Use [--no-color] and make sure to set [--data-api <PATH>] & [--local-api] so that the [Status] tab can work!

Start P2Pool with these arguments and override all below settings"#;
pub const P2POOL_SIMPLE: &str = r#"Use simple P2Pool settings:
  - Remote remote Monero node
  - Default P2Pool settings + Mini
  - Backup host setting"#;
pub const P2POOL_ADVANCED: &str = r#"Use advanced P2Pool settings:
  - Terminal input
  - Overriding command arguments
  - Manual node list
  - P2Pool Main/Mini selection
  - Out/In peer setting
  - Log level setting
  - Backup host setting"#;
pub const P2POOL_NAME: &str = "Add a unique name to identify this node; Only [A-Za-z0-9-_.] and spaces allowed; Max length = 30 characters";
pub const P2POOL_NODE_IP: &str = "Specify the Monero Node IP to connect to with P2Pool; It must be a valid IPv4 address or a valid domain name; Max length = 255 characters";
pub const P2POOL_RPC_PORT: &str = "Specify the RPC port of the Monero node; [1-65535]";
pub const P2POOL_ZMQ_PORT: &str = "Specify the ZMQ port of the Monero node; [1-65535]";
pub const P2POOL_PATH_NOT_FILE: &str = "P2Pool binary not found at the given PATH in the Gupax tab! To fix: goto the [Gupax Advanced] tab, select [Open] and specify where P2Pool is located.";
pub const P2POOL_PATH_NOT_VALID: &str = "P2Pool binary at the given PATH in the Gupax tab doesn't look like P2Pool! To fix: goto the [Gupax Advanced] tab, select [Open] and specify where P2Pool is located.";
pub const P2POOL_PATH_OK: &str = "P2Pool was found at the given PATH";
pub const P2POOL_PATH_EMPTY: &str = "P2Pool PATH is empty! To fix: goto the [Gupax Advanced] tab, select [Open] and specify where P2Pool is located.";

// Node/Pool list
pub const LIST_ADD: &str = "Add the current values to the list";
pub const LIST_SAVE: &str = "Save the current values to the already existing entry";
pub const LIST_DELETE: &str = "Delete the currently selected entry";
pub const LIST_CLEAR: &str = "Clear all current values";

// XMRig
pub const XMRIG_SIMPLE: &str = r#"Use simple XMRig settings:
  - Mine to local P2Pool (localhost:3333)
  - CPU thread slider
  - HTTP API @ localhost:18088"#;
pub const XMRIG_ADVANCED: &str = r#"Use advanced XMRig settings:
  - Terminal input
  - Overriding command arguments
  - Custom payout address
  - CPU thread slider
  - Manual pool list
  - Custom HTTP API IP/Port
  - TLS setting
  - Keepalive setting"#;
pub const XMRIG_INPUT: &str = "Send a command to XMRig";
pub const XMRIG_ARGUMENTS: &str = r#"WARNING: Use [--no-color] and make sure to set [--http-host <IP>] & [--http-port <PORT>] so that the [Status] tab can work!

Start XMRig with these arguments and override all below settings"#;
pub const XMRIG_ADDRESS:        &str = "Specify which Monero address to payout to. This does nothing if mining to P2Pool since the address being paid out to will be the one P2Pool started with. This doubles as a rig identifier for P2Pool and some pools.";
pub const XMRIG_NAME:           &str = "Add a unique name to identify this pool; Only [A-Za-z0-9-_.] and spaces allowed; Max length = 30 characters";
pub const XMRIG_IP:             &str = "Specify the pool IP to connect to with XMRig; It must be a valid IPv4 address or a valid domain name; Max length = 255 characters";
pub const XMRIG_PORT: &str = "Specify the port of the pool; [1-65535]";
pub const XMRIG_RIG:            &str = "Add an optional rig ID. This will be the name shown on the pool; Only [A-Za-z0-9-_] and spaces allowed; Max length = 30 characters";
#[cfg(not(target_os = "linux"))]
pub const XMRIG_PAUSE: &str =
    "THIS SETTING IS DISABLED IF SET TO [0]. Pause mining if user is active, resume after";
pub const XMRIG_API_IP: &str =
    "Specify which IP to bind to for XMRig's HTTP API; If empty: [localhost/127.0.0.1]";
pub const XMRIG_API_PORT: &str =
    "Specify which port to bind to for XMRig's HTTP API; If empty: [18088]";
pub const XMRIG_TLS: &str = "Enable SSL/TLS connections (needs pool support)";
pub const XMRIG_KEEPALIVE: &str = "Send keepalive packets to prevent timeout (needs pool support)";
pub const XMRIG_THREADS: &str = "Number of CPU threads to use for mining";
pub const XMRIG_PATH_NOT_FILE:  &str = "XMRig binary not found at the given PATH in the Gupax tab! To fix: goto the [Gupax Advanced] tab, select [Open] and specify where XMRig is located.";
pub const XMRIG_PATH_NOT_VALID: &str = "XMRig binary at the given PATH in the Gupax tab doesn't look like XMRig! To fix: goto the [Gupax Advanced] tab, select [Open] and specify where XMRig is located.";
pub const XMRIG_PATH_OK: &str = "XMRig was found at the given PATH";
pub const XMRIG_PATH_EMPTY:     &str = "XMRig PATH is empty! To fix: goto the [Gupax Advanced] tab, select [Open] and specify where XMRig is located.";

// CLI argument messages
pub const ARG_HELP: &str = r#"USAGE: ./gupax [--flag]

    --help            Print this help message
    --version         Print version and build info
    --state           Print Gupax state
    --nodes           Print the manual node list
    --payouts         Print the P2Pool payout log, payout count, and total XMR mined
    --no-startup      Disable all auto-startup settings for this instance (auto-update, auto-ping, etc)
    --reset-state     Reset all Gupax state (your settings)
    --reset-nodes     Reset the manual node list in the [P2Pool] tab
    --reset-pools     Reset the manual pool list in the [XMRig] tab
    --reset-payouts   Reset the permanent P2Pool stats that appear in the [Status] tab
    --reset-all       Reset the state, manual node list, manual pool list, and P2Pool stats

To view more detailed console debug information, start Gupax with
the environment variable [RUST_LOG] set to a log level like so:
    RUST_LOG=(trace|debug|info|warn|error) ./gupax"#;
pub const ARG_COPYRIGHT: &str = r#"Gupax is licensed under GPLv3.
For more information, see link below:
<https://github.com/hinto-janai/gupax>"#;

//---------------------------------------------------------------------------------------------------- Visuals
use egui::epaint::{Rounding, Shadow, Stroke};

use egui::{style::Spacing, Color32, Visuals};

use egui::style::{Selection, WidgetVisuals, Widgets};
use once_cell::sync::Lazy;

pub const ACCENT_COLOR: Color32 = Color32::from_rgb(200, 100, 100);
pub const BG: Color32 = Color32::from_gray(20);

// This is based off [`Visuals::dark()`].
pub static VISUALS: Lazy<Visuals> = Lazy::new(|| {
    let selection = Selection {
        bg_fill: ACCENT_COLOR,
        stroke: Stroke::new(1.0, Color32::from_gray(255)),
    };

    // Based off default dark() mode.
    // https://docs.rs/egui/0.24.1/src/egui/style.rs.html#1210
    let widgets = Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: BG,
            bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines
            fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
            rounding: Rounding::same(10.0),
            expansion: 0.0,
            weak_bg_fill: BG,
        },
        inactive: WidgetVisuals {
            bg_fill: Color32::from_gray(50),
            bg_stroke: Default::default(),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
            rounding: Rounding::same(10.0),
            expansion: 0.0,
            weak_bg_fill: Color32::from_gray(50),
        },
        hovered: WidgetVisuals {
            bg_fill: Color32::from_gray(80),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
            fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
            rounding: Rounding::same(10.0),
            expansion: 1.0,
            weak_bg_fill: Color32::from_gray(80),
        },
        active: WidgetVisuals {
            bg_fill: Color32::from_gray(55),
            bg_stroke: Stroke::new(1.0, Color32::WHITE),
            fg_stroke: Stroke::new(2.0, Color32::WHITE),
            rounding: Rounding::same(10.0),
            expansion: 1.0,
            weak_bg_fill: Color32::from_gray(120),
        },
        open: WidgetVisuals {
            bg_fill: Color32::from_gray(27),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
            rounding: Rounding::same(10.0),
            expansion: 0.0,
            weak_bg_fill: Color32::from_gray(120),
        },
    };

    // https://docs.rs/egui/0.24.1/src/egui/style.rs.html#1113
    Visuals {
        widgets,
        selection,
        hyperlink_color: Color32::from_rgb(90, 170, 255),
        faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
        extreme_bg_color: Color32::from_gray(10),            // e.g. TextEdit background
        code_bg_color: Color32::from_gray(64),
        warn_fg_color: Color32::from_rgb(255, 143, 0), // orange
        error_fg_color: Color32::from_rgb(255, 0, 0),  // red
        window_rounding: Rounding::same(6.0),
        window_shadow: Shadow::big_dark(),
        popup_shadow: Shadow::small_dark(),
        ..Visuals::dark()
    }
});

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
    #[test]
    fn gupax_version_is_semver() {
        assert_eq!(crate::GUPAX_VERSION.len(), 6);
    }

    #[test]
    fn default_app_ratio_is_4_by_3() {
        assert_eq!(
            format!("{:.3}", crate::APP_MIN_WIDTH / crate::APP_MIN_HEIGHT),
            "1.333"
        );
        assert_eq!(
            format!(
                "{:.3}",
                crate::APP_DEFAULT_WIDTH / crate::APP_DEFAULT_HEIGHT
            ),
            "1.333"
        );
    }

    #[test]
    fn git_commit_eq_or_gt_40_chars() {
        assert!(crate::COMMIT.len() >= 40);
    }
}
