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

// This file represents the "helper" thread, which is the full separate thread
// that runs alongside the main [App] GUI thread. It exists for the entire duration
// of Gupax so that things can be handled without locking up the GUI thread.
//
// This thread is a continual 1 second loop, collecting available jobs on the
// way down and (if possible) asynchronously executing them at the very end.
//
// The main GUI thread will interface with this thread by mutating the Arc<Mutex>'s
// found here, e.g: User clicks [Stop P2Pool] -> Arc<Mutex<ProcessSignal> is set
// indicating to this thread during its loop: "I should stop P2Pool!", e.g:
//
//     if lock!(p2pool).signal == ProcessSignal::Stop {
//         stop_p2pool(),
//     }
//
// This also includes all things related to handling the child processes (P2Pool/XMRig):
// piping their stdout/stderr/stdin, accessing their APIs (HTTP + disk files), etc.

//---------------------------------------------------------------------------------------------------- Import
use crate::regex::{P2POOL_REGEX, XMRIG_REGEX};
use crate::{constants::*, human::*, macros::*, xmr::*, GupaxP2poolApi, RemoteNode, SudoState};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Write,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    thread,
    time::*,
};
use sysinfo::SystemExt;
use sysinfo::{CpuExt, ProcessExt};

//---------------------------------------------------------------------------------------------------- Constants
// The max amount of bytes of process output we are willing to
// hold in memory before it's too much and we need to reset.
const MAX_GUI_OUTPUT_BYTES: usize = 500_000;
// Just a little leeway so a reset will go off before the [String] allocates more memory.
const GUI_OUTPUT_LEEWAY: usize = MAX_GUI_OUTPUT_BYTES - 1000;

// Some constants for generating hashrate/difficulty.
const MONERO_BLOCK_TIME_IN_SECONDS: u64 = 120;
const P2POOL_BLOCK_TIME_IN_SECONDS: u64 = 10;

//---------------------------------------------------------------------------------------------------- [Helper] Struct
// A meta struct holding all the data that gets processed in this thread
pub struct Helper {
    pub instant: Instant,                             // Gupax start as an [Instant]
    pub uptime: HumanTime,                            // Gupax uptime formatting for humans
    pub pub_sys: Arc<Mutex<Sys>>, // The public API for [sysinfo] that the [Status] tab reads from
    pub p2pool: Arc<Mutex<Process>>, // P2Pool process state
    pub xmrig: Arc<Mutex<Process>>, // XMRig process state
    pub gui_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for GUI thread)
    pub gui_api_xmrig: Arc<Mutex<PubXmrigApi>>, // XMRig API state (for GUI thread)
    pub img_p2pool: Arc<Mutex<ImgP2pool>>, // A static "image" of the data P2Pool started with
    pub img_xmrig: Arc<Mutex<ImgXmrig>>, // A static "image" of the data XMRig started with
    pub_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for Helper/P2Pool thread)
    pub_api_xmrig: Arc<Mutex<PubXmrigApi>>, // XMRig API state (for Helper/XMRig thread)
    pub gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>, //
}

// The communication between the data here and the GUI thread goes as follows:
// [GUI] <---> [Helper] <---> [Watchdog] <---> [Private Data only available here]
//
// Both [GUI] and [Helper] own their separate [Pub*Api] structs.
// Since P2Pool & XMRig will be updating their information out of sync,
// it's the helpers job to lock everything, and move the watchdog [Pub*Api]s
// on a 1-second interval into the [GUI]'s [Pub*Api] struct, atomically.

//----------------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Sys {
    pub gupax_uptime: String,
    pub gupax_cpu_usage: String,
    pub gupax_memory_used_mb: String,
    pub system_cpu_model: String,
    pub system_memory: String,
    pub system_cpu_usage: String,
}

impl Sys {
    pub fn new() -> Self {
        Self {
            gupax_uptime: "0 seconds".to_string(),
            gupax_cpu_usage: "???%".to_string(),
            gupax_memory_used_mb: "??? megabytes".to_string(),
            system_cpu_usage: "???%".to_string(),
            system_memory: "???GB / ???GB".to_string(),
            system_cpu_model: "???".to_string(),
        }
    }
}
impl Default for Sys {
    fn default() -> Self {
        Self::new()
    }
}

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The main GUI thread will use this to display console text, online state, etc.
#[derive(Debug)]
pub struct Process {
    pub name: ProcessName,     // P2Pool or XMRig?
    pub state: ProcessState,   // The state of the process (alive, dead, etc)
    pub signal: ProcessSignal, // Did the user click [Start/Stop/Restart]?
    // STDIN Problem:
    //     - User can input many many commands in 1 second
    //     - The process loop only processes every 1 second
    //     - If there is only 1 [String] holding the user input,
    //       the user could overwrite their last input before
    //       the loop even has a chance to process their last command
    // STDIN Solution:
    //     - When the user inputs something, push it to a [Vec]
    //     - In the process loop, loop over every [Vec] element and
    //       send each one individually to the process stdin
    //
    pub input: Vec<String>,

    // The below are the handles to the actual child process.
    // [Simple] has no STDIN, but [Advanced] does. A PTY (pseudo-terminal) is
    // required for P2Pool/XMRig to open their STDIN pipe.
    //	child: Option<Arc<Mutex<Box<dyn portable_pty::Child + Send + std::marker::Sync>>>>, // STDOUT/STDERR is combined automatically thanks to this PTY, nice
    //	stdin: Option<Box<dyn portable_pty::MasterPty + Send>>, // A handle to the process's MasterPTY/STDIN

    // This is the process's private output [String], used by both [Simple] and [Advanced].
    // "parse" contains the output that will be parsed, then tossed out. "pub" will be written to
    // the same as parse, but it will be [swap()]'d by the "helper" thread into the GUIs [String].
    // The "helper" thread synchronizes this swap so that the data in here is moved there
    // roughly once a second. GUI thread never touches this.
    output_parse: Arc<Mutex<String>>,
    output_pub: Arc<Mutex<String>>,

    // Start time of process.
    start: std::time::Instant,
}

//---------------------------------------------------------------------------------------------------- [Process] Impl
impl Process {
    pub fn new(name: ProcessName, _args: String, _path: PathBuf) -> Self {
        Self {
            name,
            state: ProcessState::Dead,
            signal: ProcessSignal::None,
            start: Instant::now(),
            //			stdin: Option::None,
            //			child: Option::None,
            output_parse: arc_mut!(String::with_capacity(500)),
            output_pub: arc_mut!(String::with_capacity(500)),
            input: vec![String::new()],
        }
    }

    // Borrow a [&str], return an owned split collection
    #[inline]
    pub fn parse_args(args: &str) -> Vec<String> {
        args.split_whitespace().map(|s| s.to_owned()).collect()
    }

    #[inline]
    // Convenience functions
    pub fn is_alive(&self) -> bool {
        self.state == ProcessState::Alive
            || self.state == ProcessState::Middle
            || self.state == ProcessState::Syncing
            || self.state == ProcessState::NotMining
    }

    #[inline]
    pub fn is_waiting(&self) -> bool {
        self.state == ProcessState::Middle || self.state == ProcessState::Waiting
    }

    #[inline]
    pub fn is_syncing(&self) -> bool {
        self.state == ProcessState::Syncing
    }

    #[inline]
    pub fn is_not_mining(&self) -> bool {
        self.state == ProcessState::NotMining
    }
}

//---------------------------------------------------------------------------------------------------- [Process*] Enum
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProcessState {
    Alive,   // Process is online, GREEN!
    Dead,    // Process is dead, BLACK!
    Failed,  // Process is dead AND exited with a bad code, RED!
    Middle,  // Process is in the middle of something ([re]starting/stopping), YELLOW!
    Waiting, // Process was successfully killed by a restart, and is ready to be started again, YELLOW!

    // Only for P2Pool, ORANGE.
    Syncing,

    // Only for XMRig, ORANGE.
    NotMining,
}

impl Default for ProcessState {
    fn default() -> Self {
        Self::Dead
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProcessSignal {
    None,
    Start,
    Stop,
    Restart,
}

impl Default for ProcessSignal {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProcessName {
    P2pool,
    Xmrig,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl std::fmt::Display for ProcessSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl std::fmt::Display for ProcessName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProcessName::P2pool => write!(f, "P2Pool"),
            ProcessName::Xmrig => write!(f, "XMRig"),
        }
    }
}

//---------------------------------------------------------------------------------------------------- [Helper]
impl Helper {
    //---------------------------------------------------------------------------------------------------- General Functions
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        instant: std::time::Instant,
        pub_sys: Arc<Mutex<Sys>>,
        p2pool: Arc<Mutex<Process>>,
        xmrig: Arc<Mutex<Process>>,
        gui_api_p2pool: Arc<Mutex<PubP2poolApi>>,
        gui_api_xmrig: Arc<Mutex<PubXmrigApi>>,
        img_p2pool: Arc<Mutex<ImgP2pool>>,
        img_xmrig: Arc<Mutex<ImgXmrig>>,
        gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>,
    ) -> Self {
        Self {
            instant,
            pub_sys,
            uptime: HumanTime::into_human(instant.elapsed()),
            pub_api_p2pool: arc_mut!(PubP2poolApi::new()),
            pub_api_xmrig: arc_mut!(PubXmrigApi::new()),
            // These are created when initializing [App], since it needs a handle to it as well
            p2pool,
            xmrig,
            gui_api_p2pool,
            gui_api_xmrig,
            img_p2pool,
            img_xmrig,
            gupax_p2pool_api,
        }
    }

    #[cold]
    #[inline(never)]
    fn read_pty_xmrig(
        output_parse: Arc<Mutex<String>>,
        output_pub: Arc<Mutex<String>>,
        reader: Box<dyn std::io::Read + Send>,
    ) {
        use std::io::BufRead;
        let mut stdout = std::io::BufReader::new(reader).lines();

        // Run a ANSI escape sequence filter for the first few lines.
        let mut i = 0;
        while let Some(Ok(line)) = stdout.next() {
            let line = strip_ansi_escapes::strip_str(line);
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("XMRig PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("XMRig PTY Pub | Output error: {}", e);
            }
            if i > 20 {
                break;
            } else {
                i += 1;
            }
        }

        while let Some(Ok(line)) = stdout.next() {
            //			println!("{}", line); // For debugging.
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("XMRig PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("XMRig PTY Pub | Output error: {}", e);
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn read_pty_p2pool(
        output_parse: Arc<Mutex<String>>,
        output_pub: Arc<Mutex<String>>,
        reader: Box<dyn std::io::Read + Send>,
        gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>,
    ) {
        use std::io::BufRead;
        let mut stdout = std::io::BufReader::new(reader).lines();

        // Run a ANSI escape sequence filter for the first few lines.
        let mut i = 0;
        while let Some(Ok(line)) = stdout.next() {
            let line = strip_ansi_escapes::strip_str(line);
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("P2Pool PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("P2Pool PTY Pub | Output error: {}", e);
            }
            if i > 20 {
                break;
            } else {
                i += 1;
            }
        }

        while let Some(Ok(line)) = stdout.next() {
            //			println!("{}", line); // For debugging.
            if P2POOL_REGEX.payout.is_match(&line) {
                debug!("P2Pool PTY | Found payout, attempting write: {}", line);
                let (date, atomic_unit, block) = PayoutOrd::parse_raw_payout_line(&line);
                let formatted_log_line = GupaxP2poolApi::format_payout(&date, &atomic_unit, &block);
                GupaxP2poolApi::add_payout(
                    &mut lock!(gupax_p2pool_api),
                    &formatted_log_line,
                    date,
                    atomic_unit,
                    block,
                );
                if let Err(e) = GupaxP2poolApi::write_to_all_files(
                    &lock!(gupax_p2pool_api),
                    &formatted_log_line,
                ) {
                    error!("P2Pool PTY GupaxP2poolApi | Write error: {}", e);
                }
            }
            if let Err(e) = writeln!(lock!(output_parse), "{}", line) {
                error!("P2Pool PTY Parse | Output error: {}", e);
            }
            if let Err(e) = writeln!(lock!(output_pub), "{}", line) {
                error!("P2Pool PTY Pub | Output error: {}", e);
            }
        }
    }

    // Reset output if larger than max bytes.
    // This will also append a message showing it was reset.
    fn check_reset_gui_output(output: &mut String, name: ProcessName) {
        let len = output.len();
        if len > GUI_OUTPUT_LEEWAY {
            info!(
                "{} Watchdog | Output is nearing {} bytes, resetting!",
                name, MAX_GUI_OUTPUT_BYTES
            );
            let text = format!("{}\n{} GUI log is exceeding the maximum: {} bytes!\nResetting the logs...\n{}\n\n\n\n", HORI_CONSOLE, name, MAX_GUI_OUTPUT_BYTES, HORI_CONSOLE);
            output.clear();
            output.push_str(&text);
            debug!("{} Watchdog | Resetting GUI output ... OK", name);
        } else {
            debug!(
                "{} Watchdog | GUI output reset not needed! Current byte length ... {}",
                name, len
            );
        }
    }

    // Read P2Pool/XMRig's API file to a [String].
    fn path_to_string(
        path: &std::path::PathBuf,
        name: ProcessName,
    ) -> std::result::Result<String, std::io::Error> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(e) => {
                warn!("{} API | [{}] read error: {}", name, path.display(), e);
                Err(e)
            }
        }
    }

    //---------------------------------------------------------------------------------------------------- P2Pool specific
    #[cold]
    #[inline(never)]
    // Just sets some signals for the watchdog thread to pick up on.
    pub fn stop_p2pool(helper: &Arc<Mutex<Self>>) {
        info!("P2Pool | Attempting to stop...");
        lock2!(helper, p2pool).signal = ProcessSignal::Stop;
        lock2!(helper, p2pool).state = ProcessState::Middle;
    }

    #[cold]
    #[inline(never)]
    // The "restart frontend" to a "frontend" function.
    // Basically calls to kill the current p2pool, waits a little, then starts the below function in a a new thread, then exit.
    pub fn restart_p2pool(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::P2pool,
        path: &std::path::PathBuf,
        backup_hosts: Option<Vec<crate::Node>>,
    ) {
        info!("P2Pool | Attempting to restart...");
        lock2!(helper, p2pool).signal = ProcessSignal::Restart;
        lock2!(helper, p2pool).state = ProcessState::Middle;

        let helper = Arc::clone(helper);
        let state = state.clone();
        let path = path.clone();
        // This thread lives to wait, start p2pool then die.
        thread::spawn(move || {
            while lock2!(helper, p2pool).is_alive() {
                warn!("P2Pool | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("P2Pool | Old process seems dead, starting new one!");
            Self::start_p2pool(&helper, &state, &path, backup_hosts);
        });
        info!("P2Pool | Restart ... OK");
    }

    #[cold]
    #[inline(never)]
    // The "frontend" function that parses the arguments, and spawns either the [Simple] or [Advanced] P2Pool watchdog thread.
    pub fn start_p2pool(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::P2pool,
        path: &std::path::PathBuf,
        backup_hosts: Option<Vec<crate::Node>>,
    ) {
        lock2!(helper, p2pool).state = ProcessState::Middle;

        let (args, api_path_local, api_path_network, api_path_pool) =
            Self::build_p2pool_args_and_mutate_img(helper, state, path, backup_hosts);

        // Print arguments & user settings to console
        crate::disk::print_dash(&format!(
			"P2Pool | Launch arguments: {:#?} | Local API Path: {:#?} | Network API Path: {:#?} | Pool API Path: {:#?}",
			 args,
			 api_path_local,
			 api_path_network,
			 api_path_pool,
		));

        // Spawn watchdog thread
        let process = Arc::clone(&lock!(helper).p2pool);
        let gui_api = Arc::clone(&lock!(helper).gui_api_p2pool);
        let pub_api = Arc::clone(&lock!(helper).pub_api_p2pool);
        let gupax_p2pool_api = Arc::clone(&lock!(helper).gupax_p2pool_api);
        let path = path.clone();
        thread::spawn(move || {
            Self::spawn_p2pool_watchdog(
                process,
                gui_api,
                pub_api,
                args,
                path,
                api_path_local,
                api_path_network,
                api_path_pool,
                gupax_p2pool_api,
            );
        });
    }

    // Takes in a 95-char Monero address, returns the first and last
    // 6 characters separated with dots like so: [4abcde...abcdef]
    fn head_tail_of_monero_address(address: &str) -> String {
        if address.len() < 95 {
            return "???".to_string();
        }
        let head = &address[0..6];
        let tail = &address[89..95];
        head.to_owned() + "..." + tail
    }

    #[cold]
    #[inline(never)]
    // Takes in some [State/P2pool] and parses it to build the actual command arguments.
    // Returns the [Vec] of actual arguments, and mutates the [ImgP2pool] for the main GUI thread
    // It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
    pub fn build_p2pool_args_and_mutate_img(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::P2pool,
        path: &std::path::PathBuf,
        backup_hosts: Option<Vec<crate::Node>>,
    ) -> (Vec<String>, PathBuf, PathBuf, PathBuf) {
        let mut args = Vec::with_capacity(500);
        let path = path.clone();
        let mut api_path = path;
        api_path.pop();

        // [Simple]
        if state.simple {
            // Build the p2pool argument
            let (ip, rpc, zmq) = RemoteNode::get_ip_rpc_zmq(&state.node); // Get: (IP, RPC, ZMQ)
            args.push("--wallet".to_string());
            args.push(state.address.clone()); // Wallet address
            args.push("--host".to_string());
            args.push(ip.to_string()); // IP Address
            args.push("--rpc-port".to_string());
            args.push(rpc.to_string()); // RPC Port
            args.push("--zmq-port".to_string());
            args.push(zmq.to_string()); // ZMQ Port
            args.push("--data-api".to_string());
            args.push(api_path.display().to_string()); // API Path
            args.push("--local-api".to_string()); // Enable API
            args.push("--no-color".to_string()); // Remove color escape sequences, Gupax terminal can't parse it :(
            args.push("--mini".to_string()); // P2Pool Mini
            args.push("--light-mode".to_string()); // Assume user is not using P2Pool to mine.

            // Push other nodes if `backup_host`.
            if let Some(nodes) = backup_hosts {
                for node in nodes {
                    if (node.ip.as_str(), node.rpc.as_str(), node.zmq.as_str()) != (ip, rpc, zmq) {
                        args.push("--host".to_string());
                        args.push(node.ip.to_string());
                        args.push("--rpc-port".to_string());
                        args.push(node.rpc.to_string());
                        args.push("--zmq-port".to_string());
                        args.push(node.zmq.to_string());
                    }
                }
            }

            *lock2!(helper, img_p2pool) = ImgP2pool {
                mini: "P2Pool Mini".to_string(),
                address: Self::head_tail_of_monero_address(&state.address),
                host: ip.to_string(),
                rpc: rpc.to_string(),
                zmq: zmq.to_string(),
                out_peers: "10".to_string(),
                in_peers: "10".to_string(),
            };

        // [Advanced]
        } else {
            // Overriding command arguments
            if !state.arguments.is_empty() {
                // This parses the input and attempts to fill out
                // the [ImgP2pool]... This is pretty bad code...
                let mut last = "";
                let lock = lock!(helper);
                let mut p2pool_image = lock!(lock.img_p2pool);
                let mut mini = false;
                for arg in state.arguments.split_whitespace() {
                    match last {
                        "--mini" => {
                            mini = true;
                            p2pool_image.mini = "P2Pool Mini".to_string();
                        }
                        "--wallet" => p2pool_image.address = Self::head_tail_of_monero_address(arg),
                        "--host" => p2pool_image.host = arg.to_string(),
                        "--rpc-port" => p2pool_image.rpc = arg.to_string(),
                        "--zmq-port" => p2pool_image.zmq = arg.to_string(),
                        "--out-peers" => p2pool_image.out_peers = arg.to_string(),
                        "--in-peers" => p2pool_image.in_peers = arg.to_string(),
                        "--data-api" => api_path = PathBuf::from(arg),
                        _ => (),
                    }
                    if !mini {
                        p2pool_image.mini = "P2Pool Main".to_string();
                    }
                    let arg = if arg == "localhost" { "127.0.0.1" } else { arg };
                    args.push(arg.to_string());
                    last = arg;
                }
            // Else, build the argument
            } else {
                let ip = if state.ip == "localhost" {
                    "127.0.0.1"
                } else {
                    &state.ip
                };
                args.push("--wallet".to_string());
                args.push(state.address.clone()); // Wallet
                args.push("--host".to_string());
                args.push(ip.to_string()); // IP
                args.push("--rpc-port".to_string());
                args.push(state.rpc.to_string()); // RPC
                args.push("--zmq-port".to_string());
                args.push(state.zmq.to_string()); // ZMQ
                args.push("--loglevel".to_string());
                args.push(state.log_level.to_string()); // Log Level
                args.push("--out-peers".to_string());
                args.push(state.out_peers.to_string()); // Out Peers
                args.push("--in-peers".to_string());
                args.push(state.in_peers.to_string()); // In Peers
                args.push("--data-api".to_string());
                args.push(api_path.display().to_string()); // API Path
                args.push("--local-api".to_string()); // Enable API
                args.push("--no-color".to_string()); // Remove color escape sequences
                args.push("--light-mode".to_string()); // Assume user is not using P2Pool to mine.
                if state.mini {
                    args.push("--mini".to_string());
                }; // Mini

                // Push other nodes if `backup_host`.
                if let Some(nodes) = backup_hosts {
                    for node in nodes {
                        let ip = if node.ip == "localhost" {
                            "127.0.0.1"
                        } else {
                            &node.ip
                        };
                        if (node.ip.as_str(), node.rpc.as_str(), node.zmq.as_str())
                            != (ip, &state.rpc, &state.zmq)
                        {
                            args.push("--host".to_string());
                            args.push(node.ip.to_string());
                            args.push("--rpc-port".to_string());
                            args.push(node.rpc.to_string());
                            args.push("--zmq-port".to_string());
                            args.push(node.zmq.to_string());
                        }
                    }
                }

                *lock2!(helper, img_p2pool) = ImgP2pool {
                    mini: if state.mini {
                        "P2Pool Mini".to_string()
                    } else {
                        "P2Pool Main".to_string()
                    },
                    address: Self::head_tail_of_monero_address(&state.address),
                    host: state.selected_ip.to_string(),
                    rpc: state.selected_rpc.to_string(),
                    zmq: state.selected_zmq.to_string(),
                    out_peers: state.out_peers.to_string(),
                    in_peers: state.in_peers.to_string(),
                };
            }
        }
        let mut api_path_local = api_path.clone();
        let mut api_path_network = api_path.clone();
        let mut api_path_pool = api_path.clone();
        api_path_local.push(P2POOL_API_PATH_LOCAL);
        api_path_network.push(P2POOL_API_PATH_NETWORK);
        api_path_pool.push(P2POOL_API_PATH_POOL);
        (args, api_path_local, api_path_network, api_path_pool)
    }

    #[cold]
    #[inline(never)]
    #[expect(clippy::too_many_arguments)]
    // The P2Pool watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
    fn spawn_p2pool_watchdog(
        process: Arc<Mutex<Process>>,
        gui_api: Arc<Mutex<PubP2poolApi>>,
        pub_api: Arc<Mutex<PubP2poolApi>>,
        args: Vec<String>,
        path: std::path::PathBuf,
        api_path_local: std::path::PathBuf,
        api_path_network: std::path::PathBuf,
        api_path_pool: std::path::PathBuf,
        gupax_p2pool_api: Arc<Mutex<GupaxP2poolApi>>,
    ) {
        // 1a. Create PTY
        debug!("P2Pool | Creating PTY...");
        let pty = portable_pty::native_pty_system();
        let pair = pty
            .openpty(portable_pty::PtySize {
                rows: 100,
                cols: 1000,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        // 1b. Create command
        debug!("P2Pool | Creating command...");
        let mut cmd = portable_pty::CommandBuilder::new(path.as_path());
        cmd.args(args);
        cmd.env("NO_COLOR", "true");
        cmd.cwd(path.as_path().parent().unwrap());
        // 1c. Create child
        debug!("P2Pool | Creating child...");
        let child_pty = arc_mut!(pair.slave.spawn_command(cmd).unwrap());
        drop(pair.slave);

        // 2. Set process state
        debug!("P2Pool | Setting process state...");
        let mut lock = lock!(process);
        lock.state = ProcessState::Syncing;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
        let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
        let mut stdin = pair.master.take_writer().unwrap();
        drop(lock);

        // 3. Spawn PTY read thread
        debug!("P2Pool | Spawning PTY read thread...");
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);
        let gupax_p2pool_api = Arc::clone(&gupax_p2pool_api);
        thread::spawn(move || {
            Self::read_pty_p2pool(output_parse, output_pub, reader, gupax_p2pool_api);
        });
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);

        debug!("P2Pool | Cleaning old [local] API files...");
        // Attempt to remove stale API file
        match std::fs::remove_file(&api_path_local) {
            Ok(_) => info!("P2Pool | Attempting to remove stale API file ... OK"),
            Err(e) => warn!(
                "P2Pool | Attempting to remove stale API file ... FAIL ... {}",
                e
            ),
        }
        // Attempt to create a default empty one.
        use std::io::Write;
        if std::fs::File::create(&api_path_local).is_ok() {
            let text = r#"{"hashrate_15m":0,"hashrate_1h":0,"hashrate_24h":0,"shares_found":0,"average_effort":0.0,"current_effort":0.0,"connections":0}"#;
            match std::fs::write(&api_path_local, text) {
                Ok(_) => info!("P2Pool | Creating default empty API file ... OK"),
                Err(e) => warn!(
                    "P2Pool | Creating default empty API file ... FAIL ... {}",
                    e
                ),
            }
        }
        let start = lock!(process).start;

        // Reset stats before loop
        *lock!(pub_api) = PubP2poolApi::new();
        *lock!(gui_api) = PubP2poolApi::new();

        // 4. Loop as watchdog
        info!("P2Pool | Entering watchdog mode... woof!");
        loop {
            // Set timer
            let now = Instant::now();
            debug!("P2Pool Watchdog | ----------- Start of loop -----------");
            lock!(gui_api).tick += 1;

            // Check if the process is secretly died without us knowing :)
            if let Ok(Some(code)) = lock!(child_pty).try_wait() {
                debug!("P2Pool Watchdog | Process secretly died! Getting exit status");
                let exit_status = match code.success() {
                    true => {
                        lock!(process).state = ProcessState::Dead;
                        "Successful"
                    }
                    false => {
                        lock!(process).state = ProcessState::Failed;
                        "Failed"
                    }
                };
                let uptime = HumanTime::into_human(start.elapsed());
                info!(
                    "P2Pool Watchdog | Stopped ... Uptime was: [{}], Exit status: [{}]",
                    uptime, exit_status
                );
                // This is written directly into the GUI, because sometimes the 900ms event loop can't catch it.
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
                    HORI_CONSOLE,
                    uptime,
                    exit_status,
                    HORI_CONSOLE
                ) {
                    error!(
                        "P2Pool Watchdog | GUI Uptime/Exit status write failed: {}",
                        e
                    );
                }
                lock!(process).signal = ProcessSignal::None;
                debug!("P2Pool Watchdog | Secret dead process reap OK, breaking");
                break;
            }

            // Check SIGNAL
            if lock!(process).signal == ProcessSignal::Stop {
                debug!("P2Pool Watchdog | Stop SIGNAL caught");
                // This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
                if let Err(e) = lock!(child_pty).kill() {
                    error!("P2Pool Watchdog | Kill error: {}", e);
                }
                // Wait to get the exit status
                let exit_status = match lock!(child_pty).wait() {
                    Ok(e) => {
                        if e.success() {
                            lock!(process).state = ProcessState::Dead;
                            "Successful"
                        } else {
                            lock!(process).state = ProcessState::Failed;
                            "Failed"
                        }
                    }
                    _ => {
                        lock!(process).state = ProcessState::Failed;
                        "Unknown Error"
                    }
                };
                let uptime = HumanTime::into_human(start.elapsed());
                info!(
                    "P2Pool Watchdog | Stopped ... Uptime was: [{}], Exit status: [{}]",
                    uptime, exit_status
                );
                // This is written directly into the GUI API, because sometimes the 900ms event loop can't catch it.
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
                    HORI_CONSOLE,
                    uptime,
                    exit_status,
                    HORI_CONSOLE
                ) {
                    error!(
                        "P2Pool Watchdog | GUI Uptime/Exit status write failed: {}",
                        e
                    );
                }
                lock!(process).signal = ProcessSignal::None;
                debug!("P2Pool Watchdog | Stop SIGNAL done, breaking");
                break;
            // Check RESTART
            } else if lock!(process).signal == ProcessSignal::Restart {
                debug!("P2Pool Watchdog | Restart SIGNAL caught");
                // This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
                if let Err(e) = lock!(child_pty).kill() {
                    error!("P2Pool Watchdog | Kill error: {}", e);
                }
                // Wait to get the exit status
                let exit_status = match lock!(child_pty).wait() {
                    Ok(e) => {
                        if e.success() {
                            "Successful"
                        } else {
                            "Failed"
                        }
                    }
                    _ => "Unknown Error",
                };
                let uptime = HumanTime::into_human(start.elapsed());
                info!(
                    "P2Pool Watchdog | Stopped ... Uptime was: [{}], Exit status: [{}]",
                    uptime, exit_status
                );
                // This is written directly into the GUI API, because sometimes the 900ms event loop can't catch it.
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
                    HORI_CONSOLE,
                    uptime,
                    exit_status,
                    HORI_CONSOLE
                ) {
                    error!(
                        "P2Pool Watchdog | GUI Uptime/Exit status write failed: {}",
                        e
                    );
                }
                lock!(process).state = ProcessState::Waiting;
                debug!("P2Pool Watchdog | Restart SIGNAL done, breaking");
                break;
            }

            // Check vector of user input
            let mut lock = lock!(process);
            if !lock.input.is_empty() {
                let input = std::mem::take(&mut lock.input);
                for line in input {
                    if line.is_empty() {
                        continue;
                    }
                    debug!(
                        "P2Pool Watchdog | User input not empty, writing to STDIN: [{}]",
                        line
                    );
                    // Windows terminals (or at least the PTY abstraction I'm using, portable_pty)
                    // requires a [\r\n] to end a line, whereas Unix is okay with just a [\n].
                    //
                    // I have literally read all of [portable_pty]'s source code, dug into Win32 APIs,
                    // even rewrote some of the actual PTY code in order to understand why STDIN doesn't work
                    // on Windows. It's because of a fucking missing [\r]. Another reason to hate Windows :D
                    //
                    // XMRig did actually work before though, since it reads STDIN directly without needing a newline.
                    #[cfg(target_os = "windows")]
                    if let Err(e) = write!(stdin, "{}\r\n", line) {
                        error!("P2Pool Watchdog | STDIN error: {}", e);
                    }
                    #[cfg(target_family = "unix")]
                    if let Err(e) = writeln!(stdin, "{}", line) {
                        error!("P2Pool Watchdog | STDIN error: {}", e);
                    }
                    // Flush.
                    if let Err(e) = stdin.flush() {
                        error!("P2Pool Watchdog | STDIN flush error: {}", e);
                    }
                }
            }
            drop(lock);

            // Check if logs need resetting
            debug!("P2Pool Watchdog | Attempting GUI log reset check");
            let mut lock = lock!(gui_api);
            Self::check_reset_gui_output(&mut lock.output, ProcessName::P2pool);
            drop(lock);

            // Always update from output
            debug!("P2Pool Watchdog | Starting [update_from_output()]");
            PubP2poolApi::update_from_output(
                &pub_api,
                &output_parse,
                &output_pub,
                start.elapsed(),
                &process,
            );

            // Read [local] API
            debug!("P2Pool Watchdog | Attempting [local] API file read");
            if let Ok(string) = Self::path_to_string(&api_path_local, ProcessName::P2pool) {
                // Deserialize
                if let Ok(local_api) = PrivP2poolLocalApi::from_str(&string) {
                    // Update the structs.
                    PubP2poolApi::update_from_local(&pub_api, local_api);
                }
            }
            // If more than 1 minute has passed, read the other API files.
            if lock!(gui_api).tick >= 60 {
                debug!("P2Pool Watchdog | Attempting [network] & [pool] API file read");
                if let (Ok(network_api), Ok(pool_api)) = (
                    Self::path_to_string(&api_path_network, ProcessName::P2pool),
                    Self::path_to_string(&api_path_pool, ProcessName::P2pool),
                ) {
                    if let (Ok(network_api), Ok(pool_api)) = (
                        PrivP2poolNetworkApi::from_str(&network_api),
                        PrivP2poolPoolApi::from_str(&pool_api),
                    ) {
                        PubP2poolApi::update_from_network_pool(&pub_api, network_api, pool_api);
                        lock!(gui_api).tick = 0;
                    }
                }
            }

            // Sleep (only if 900ms hasn't passed)
            let elapsed = now.elapsed().as_millis();
            // Since logic goes off if less than 1000, casting should be safe
            if elapsed < 900 {
                let sleep = (900 - elapsed) as u64;
                debug!(
                    "P2Pool Watchdog | END OF LOOP -  Tick: [{}/60] - Sleeping for [{}]ms...",
                    lock!(gui_api).tick,
                    sleep
                );
                sleep!(sleep);
            } else {
                debug!(
                    "P2Pool Watchdog | END OF LOOP - Tick: [{}/60] Not sleeping!",
                    lock!(gui_api).tick
                );
            }
        }

        // 5. If loop broke, we must be done here.
        info!("P2Pool Watchdog | Watchdog thread exiting... Goodbye!");
    }

    //---------------------------------------------------------------------------------------------------- XMRig specific, most functions are very similar to P2Pool's
    #[cold]
    #[inline(never)]
    // If processes are started with [sudo] on macOS, they must also
    // be killed with [sudo] (even if I have a direct handle to it as the
    // parent process...!). This is only needed on macOS, not Linux.
    fn sudo_kill(pid: u32, sudo: &Arc<Mutex<SudoState>>) -> bool {
        // Spawn [sudo] to execute [kill] on the given [pid]
        let mut child = std::process::Command::new("sudo")
            .args(["--stdin", "kill", "-9", &pid.to_string()])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        // Write the [sudo] password to STDIN.
        let mut stdin = child.stdin.take().unwrap();
        use std::io::Write;
        if let Err(e) = writeln!(stdin, "{}\n", lock!(sudo).pass) {
            error!("Sudo Kill | STDIN error: {}", e);
        }

        // Return exit code of [sudo/kill].
        child.wait().unwrap().success()
    }

    #[cold]
    #[inline(never)]
    // Just sets some signals for the watchdog thread to pick up on.
    pub fn stop_xmrig(helper: &Arc<Mutex<Self>>) {
        info!("XMRig | Attempting to stop...");
        lock2!(helper, xmrig).signal = ProcessSignal::Stop;
        lock2!(helper, xmrig).state = ProcessState::Middle;
    }

    #[cold]
    #[inline(never)]
    // The "restart frontend" to a "frontend" function.
    // Basically calls to kill the current xmrig, waits a little, then starts the below function in a a new thread, then exit.
    pub fn restart_xmrig(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::Xmrig,
        path: &std::path::PathBuf,
        sudo: Arc<Mutex<SudoState>>,
    ) {
        info!("XMRig | Attempting to restart...");
        lock2!(helper, xmrig).signal = ProcessSignal::Restart;
        lock2!(helper, xmrig).state = ProcessState::Middle;

        let helper = Arc::clone(helper);
        let state = state.clone();
        let path = path.clone();
        // This thread lives to wait, start xmrig then die.
        thread::spawn(move || {
            while lock2!(helper, xmrig).state != ProcessState::Waiting {
                warn!("XMRig | Want to restart but process is still alive, waiting...");
                sleep!(1000);
            }
            // Ok, process is not alive, start the new one!
            info!("XMRig | Old process seems dead, starting new one!");
            Self::start_xmrig(&helper, &state, &path, sudo);
        });
        info!("XMRig | Restart ... OK");
    }

    #[cold]
    #[inline(never)]
    pub fn start_xmrig(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::Xmrig,
        path: &std::path::PathBuf,
        sudo: Arc<Mutex<SudoState>>,
    ) {
        lock2!(helper, xmrig).state = ProcessState::Middle;

        let (args, api_ip_port) = Self::build_xmrig_args_and_mutate_img(helper, state, path);

        // Print arguments & user settings to console
        crate::disk::print_dash(&format!("XMRig | Launch arguments: {:#?}", args));
        info!("XMRig | Using path: [{}]", path.display());

        // Spawn watchdog thread
        let process = Arc::clone(&lock!(helper).xmrig);
        let gui_api = Arc::clone(&lock!(helper).gui_api_xmrig);
        let pub_api = Arc::clone(&lock!(helper).pub_api_xmrig);
        let path = path.clone();
        thread::spawn(move || {
            Self::spawn_xmrig_watchdog(process, gui_api, pub_api, args, path, sudo, api_ip_port);
        });
    }

    #[cold]
    #[inline(never)]
    // Takes in some [State/Xmrig] and parses it to build the actual command arguments.
    // Returns the [Vec] of actual arguments, and mutates the [ImgXmrig] for the main GUI thread
    // It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
    pub fn build_xmrig_args_and_mutate_img(
        helper: &Arc<Mutex<Self>>,
        state: &crate::disk::Xmrig,
        path: &std::path::PathBuf,
    ) -> (Vec<String>, String) {
        let mut args = Vec::with_capacity(500);
        let mut api_ip = String::with_capacity(15);
        let mut api_port = String::with_capacity(5);
        let path = path.clone();
        // The actual binary we're executing is [sudo], technically
        // the XMRig path is just an argument to sudo, so add it.
        // Before that though, add the ["--prompt"] flag and set it
        // to emptiness so that it doesn't show up in the output.
        if cfg!(unix) {
            args.push(r#"--prompt="#.to_string());
            args.push("--".to_string());
            args.push(path.display().to_string());
        }

        // [Simple]
        if state.simple {
            // Build the xmrig argument
            let rig = if state.simple_rig.is_empty() {
                GUPAX_VERSION_UNDERSCORE.to_string()
            } else {
                state.simple_rig.clone()
            }; // Rig name
            args.push("--url".to_string());
            args.push("127.0.0.1:3333".to_string()); // Local P2Pool (the default)
            args.push("--threads".to_string());
            args.push(state.current_threads.to_string()); // Threads
            args.push("--user".to_string());
            args.push(rig); // Rig name
            args.push("--no-color".to_string()); // No color
            args.push("--http-host".to_string());
            args.push("127.0.0.1".to_string()); // HTTP API IP
            args.push("--http-port".to_string());
            args.push("18088".to_string()); // HTTP API Port
            if state.pause != 0 {
                args.push("--pause-on-active".to_string());
                args.push(state.pause.to_string());
            } // Pause on active
            *lock2!(helper, img_xmrig) = ImgXmrig {
                threads: state.current_threads.to_string(),
                url: "127.0.0.1:3333 (Local P2Pool)".to_string(),
            };
            api_ip = "127.0.0.1".to_string();
            api_port = "18088".to_string();

        // [Advanced]
        } else {
            // Overriding command arguments
            if !state.arguments.is_empty() {
                // This parses the input and attempts to fill out
                // the [ImgXmrig]... This is pretty bad code...
                let mut last = "";
                let lock = lock!(helper);
                let mut xmrig_image = lock!(lock.img_xmrig);
                for arg in state.arguments.split_whitespace() {
                    match last {
                        "--threads" => xmrig_image.threads = arg.to_string(),
                        "--url" => xmrig_image.url = arg.to_string(),
                        "--http-host" => {
                            api_ip = if arg == "localhost" {
                                "127.0.0.1".to_string()
                            } else {
                                arg.to_string()
                            }
                        }
                        "--http-port" => api_port = arg.to_string(),
                        _ => (),
                    }
                    args.push(if arg == "localhost" {
                        "127.0.0.1".to_string()
                    } else {
                        arg.to_string()
                    });
                    last = arg;
                }
            // Else, build the argument
            } else {
                // XMRig doesn't understand [localhost]
                let ip = if state.ip == "localhost" || state.ip.is_empty() {
                    "127.0.0.1"
                } else {
                    &state.ip
                };
                api_ip = if state.api_ip == "localhost" || state.api_ip.is_empty() {
                    "127.0.0.1".to_string()
                } else {
                    state.api_ip.to_string()
                };
                api_port = if state.api_port.is_empty() {
                    "18088".to_string()
                } else {
                    state.api_port.to_string()
                };
                let url = format!("{}:{}", ip, state.port); // Combine IP:Port into one string
                args.push("--user".to_string());
                args.push(state.address.clone()); // Wallet
                args.push("--threads".to_string());
                args.push(state.current_threads.to_string()); // Threads
                args.push("--rig-id".to_string());
                args.push(state.rig.to_string()); // Rig ID
                args.push("--url".to_string());
                args.push(url.clone()); // IP/Port
                args.push("--http-host".to_string());
                args.push(api_ip.to_string()); // HTTP API IP
                args.push("--http-port".to_string());
                args.push(api_port.to_string()); // HTTP API Port
                args.push("--no-color".to_string()); // No color escape codes
                if state.tls {
                    args.push("--tls".to_string());
                } // TLS
                if state.keepalive {
                    args.push("--keepalive".to_string());
                } // Keepalive
                if state.pause != 0 {
                    args.push("--pause-on-active".to_string());
                    args.push(state.pause.to_string());
                } // Pause on active
                *lock2!(helper, img_xmrig) = ImgXmrig {
                    url,
                    threads: state.current_threads.to_string(),
                };
            }
        }
        (args, format!("{}:{}", api_ip, api_port))
    }

    // We actually spawn [sudo] on Unix, with XMRig being the argument.
    #[cfg(target_family = "unix")]
    fn create_xmrig_cmd_unix(args: Vec<String>, path: PathBuf) -> portable_pty::CommandBuilder {
        let mut cmd = portable_pty::cmdbuilder::CommandBuilder::new("sudo");
        cmd.args(args);
        cmd.cwd(path.as_path().parent().unwrap());
        cmd
    }

    // Gupax should be admin on Windows, so just spawn XMRig normally.
    #[cfg(target_os = "windows")]
    fn create_xmrig_cmd_windows(args: Vec<String>, path: PathBuf) -> portable_pty::CommandBuilder {
        let mut cmd = portable_pty::cmdbuilder::CommandBuilder::new(path.clone());
        cmd.args(args);
        cmd.cwd(path.as_path().parent().unwrap());
        cmd
    }

    #[cold]
    #[inline(never)]
    // The XMRig watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
    // This isn't actually async, a tokio runtime is unfortunately needed because [Hyper] is an async library (HTTP API calls)
    #[tokio::main]
    async fn spawn_xmrig_watchdog(
        process: Arc<Mutex<Process>>,
        gui_api: Arc<Mutex<PubXmrigApi>>,
        pub_api: Arc<Mutex<PubXmrigApi>>,
        args: Vec<String>,
        path: std::path::PathBuf,
        sudo: Arc<Mutex<SudoState>>,
        mut api_ip_port: String,
    ) {
        // 1a. Create PTY
        debug!("XMRig | Creating PTY...");
        let pty = portable_pty::native_pty_system();
        let pair = pty
            .openpty(portable_pty::PtySize {
                rows: 100,
                cols: 1000,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        // 1b. Create command
        debug!("XMRig | Creating command...");
        #[cfg(target_os = "windows")]
        let cmd = Self::create_xmrig_cmd_windows(args, path);
        #[cfg(target_family = "unix")]
        let cmd = Self::create_xmrig_cmd_unix(args, path);
        // 1c. Create child
        debug!("XMRig | Creating child...");
        let child_pty = arc_mut!(pair.slave.spawn_command(cmd).unwrap());
        drop(pair.slave);

        let mut stdin = pair.master.take_writer().unwrap();

        // 2. Input [sudo] pass, wipe, then drop.
        if cfg!(unix) {
            debug!("XMRig | Inputting [sudo] and wiping...");
            // a) Sleep to wait for [sudo]'s non-echo prompt (on Unix).
            // this prevents users pass from showing up in the STDOUT.
            sleep!(3000);
            if let Err(e) = writeln!(stdin, "{}", lock!(sudo).pass) {
                error!("XMRig | Sudo STDIN error: {}", e);
            };
            SudoState::wipe(&sudo);

            // b) Reset GUI STDOUT just in case.
            debug!("XMRig | Clearing GUI output...");
            lock!(gui_api).output.clear();
        }

        // 3. Set process state
        debug!("XMRig | Setting process state...");
        let mut lock = lock!(process);
        lock.state = ProcessState::NotMining;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
        let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
        drop(lock);

        // 4. Spawn PTY read thread
        debug!("XMRig | Spawning PTY read thread...");
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);
        thread::spawn(move || {
            Self::read_pty_xmrig(output_parse, output_pub, reader);
        });
        let output_parse = Arc::clone(&lock!(process).output_parse);
        let output_pub = Arc::clone(&lock!(process).output_pub);

        let client: hyper::Client<hyper::client::HttpConnector> =
            hyper::Client::builder().build(hyper::client::HttpConnector::new());
        let start = lock!(process).start;
        let api_uri = {
            if !api_ip_port.ends_with('/') {
                api_ip_port.push('/');
            }
            "http://".to_owned() + &api_ip_port + XMRIG_API_URI
        };
        info!("XMRig | Final API URI: {}", api_uri);

        // Reset stats before loop
        *lock!(pub_api) = PubXmrigApi::new();
        *lock!(gui_api) = PubXmrigApi::new();

        // 5. Loop as watchdog
        info!("XMRig | Entering watchdog mode... woof!");
        loop {
            // Set timer
            let now = Instant::now();
            debug!("XMRig Watchdog | ----------- Start of loop -----------");

            // Check if the process secretly died without us knowing :)
            if let Ok(Some(code)) = lock!(child_pty).try_wait() {
                debug!("XMRig Watchdog | Process secretly died on us! Getting exit status...");
                let exit_status = match code.success() {
                    true => {
                        lock!(process).state = ProcessState::Dead;
                        "Successful"
                    }
                    false => {
                        lock!(process).state = ProcessState::Failed;
                        "Failed"
                    }
                };
                let uptime = HumanTime::into_human(start.elapsed());
                info!(
                    "XMRig | Stopped ... Uptime was: [{}], Exit status: [{}]",
                    uptime, exit_status
                );
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "{}\nXMRig stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
                    HORI_CONSOLE,
                    uptime,
                    exit_status,
                    HORI_CONSOLE
                ) {
                    error!(
                        "XMRig Watchdog | GUI Uptime/Exit status write failed: {}",
                        e
                    );
                }
                lock!(process).signal = ProcessSignal::None;
                debug!("XMRig Watchdog | Secret dead process reap OK, breaking");
                break;
            }

            // Stop on [Stop/Restart] SIGNAL
            let signal = lock!(process).signal;
            if signal == ProcessSignal::Stop || signal == ProcessSignal::Restart {
                debug!("XMRig Watchdog | Stop/Restart SIGNAL caught");
                // macOS requires [sudo] again to kill [XMRig]
                if cfg!(target_os = "macos") {
                    // If we're at this point, that means the user has
                    // entered their [sudo] pass again, after we wiped it.
                    // So, we should be able to find it in our [Arc<Mutex<SudoState>>].
                    Self::sudo_kill(lock!(child_pty).process_id().unwrap(), &sudo);
                    // And... wipe it again (only if we're stopping full).
                    // If we're restarting, the next start will wipe it for us.
                    if signal != ProcessSignal::Restart {
                        SudoState::wipe(&sudo);
                    }
                } else if let Err(e) = lock!(child_pty).kill() {
                    error!("XMRig Watchdog | Kill error: {}", e);
                }
                let exit_status = match lock!(child_pty).wait() {
                    Ok(e) => {
                        let mut process = lock!(process);
                        if e.success() {
                            if process.signal == ProcessSignal::Stop {
                                process.state = ProcessState::Dead;
                            }
                            "Successful"
                        } else {
                            if process.signal == ProcessSignal::Stop {
                                process.state = ProcessState::Failed;
                            }
                            "Failed"
                        }
                    }
                    _ => {
                        let mut process = lock!(process);
                        if process.signal == ProcessSignal::Stop {
                            process.state = ProcessState::Failed;
                        }
                        "Unknown Error"
                    }
                };
                let uptime = HumanTime::into_human(start.elapsed());
                info!(
                    "XMRig | Stopped ... Uptime was: [{}], Exit status: [{}]",
                    uptime, exit_status
                );
                if let Err(e) = writeln!(
                    lock!(gui_api).output,
                    "{}\nXMRig stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
                    HORI_CONSOLE,
                    uptime,
                    exit_status,
                    HORI_CONSOLE
                ) {
                    error!(
                        "XMRig Watchdog | GUI Uptime/Exit status write failed: {}",
                        e
                    );
                }
                let mut process = lock!(process);
                match process.signal {
                    ProcessSignal::Stop => process.signal = ProcessSignal::None,
                    ProcessSignal::Restart => process.state = ProcessState::Waiting,
                    _ => (),
                }
                debug!("XMRig Watchdog | Stop/Restart SIGNAL done, breaking");
                break;
            }

            // Check vector of user input
            let mut lock = lock!(process);
            if !lock.input.is_empty() {
                let input = std::mem::take(&mut lock.input);
                for line in input {
                    if line.is_empty() {
                        continue;
                    }
                    debug!(
                        "XMRig Watchdog | User input not empty, writing to STDIN: [{}]",
                        line
                    );
                    #[cfg(target_os = "windows")]
                    if let Err(e) = write!(stdin, "{}\r\n", line) {
                        error!("XMRig Watchdog | STDIN error: {}", e);
                    }
                    #[cfg(target_family = "unix")]
                    if let Err(e) = writeln!(stdin, "{}", line) {
                        error!("XMRig Watchdog | STDIN error: {}", e);
                    }
                    // Flush.
                    if let Err(e) = stdin.flush() {
                        error!("XMRig Watchdog | STDIN flush error: {}", e);
                    }
                }
            }
            drop(lock);

            // Check if logs need resetting
            debug!("XMRig Watchdog | Attempting GUI log reset check");
            let mut lock = lock!(gui_api);
            Self::check_reset_gui_output(&mut lock.output, ProcessName::Xmrig);
            drop(lock);

            // Always update from output
            debug!("XMRig Watchdog | Starting [update_from_output()]");
            PubXmrigApi::update_from_output(
                &pub_api,
                &output_pub,
                &output_parse,
                start.elapsed(),
                &process,
            );

            // Send an HTTP API request
            debug!("XMRig Watchdog | Attempting HTTP API request...");
            if let Ok(priv_api) = PrivXmrigApi::request_xmrig_api(client.clone(), &api_uri).await {
                debug!("XMRig Watchdog | HTTP API request OK, attempting [update_from_priv()]");
                PubXmrigApi::update_from_priv(&pub_api, priv_api);
            } else {
                warn!(
                    "XMRig Watchdog | Could not send HTTP API request to: {}",
                    api_uri
                );
            }

            // Sleep (only if 900ms hasn't passed)
            let elapsed = now.elapsed().as_millis();
            // Since logic goes off if less than 1000, casting should be safe
            if elapsed < 900 {
                let sleep = (900 - elapsed) as u64;
                debug!(
                    "XMRig Watchdog | END OF LOOP - Sleeping for [{}]ms...",
                    sleep
                );
                sleep!(sleep);
            } else {
                debug!("XMRig Watchdog | END OF LOOP - Not sleeping!");
            }
        }

        // 5. If loop broke, we must be done here.
        info!("XMRig Watchdog | Watchdog thread exiting... Goodbye!");
    }

    //---------------------------------------------------------------------------------------------------- The "helper"
    #[inline(always)] // called once
    fn update_pub_sys_from_sysinfo(
        sysinfo: &sysinfo::System,
        pub_sys: &mut Sys,
        pid: &sysinfo::Pid,
        helper: &Helper,
        max_threads: usize,
    ) {
        let gupax_uptime = helper.uptime.to_string();
        let cpu = &sysinfo.cpus()[0];
        let gupax_cpu_usage = format!(
            "{:.2}%",
            sysinfo.process(*pid).unwrap().cpu_usage() / (max_threads as f32)
        );
        let gupax_memory_used_mb =
            HumanNumber::from_u64(sysinfo.process(*pid).unwrap().memory() / 1_000_000);
        let gupax_memory_used_mb = format!("{} megabytes", gupax_memory_used_mb);
        let system_cpu_model = format!("{} ({}MHz)", cpu.brand(), cpu.frequency());
        let system_memory = {
            let used = (sysinfo.used_memory() as f64) / 1_000_000_000.0;
            let total = (sysinfo.total_memory() as f64) / 1_000_000_000.0;
            format!("{:.3} GB / {:.3} GB", used, total)
        };
        let system_cpu_usage = {
            let mut total: f32 = 0.0;
            for cpu in sysinfo.cpus() {
                total += cpu.cpu_usage();
            }
            format!("{:.2}%", total / (max_threads as f32))
        };
        *pub_sys = Sys {
            gupax_uptime,
            gupax_cpu_usage,
            gupax_memory_used_mb,
            system_cpu_usage,
            system_memory,
            system_cpu_model,
        };
    }

    #[cold]
    #[inline(never)]
    // The "helper" thread. Syncs data between threads here and the GUI.
    pub fn spawn_helper(
        helper: &Arc<Mutex<Self>>,
        mut sysinfo: sysinfo::System,
        pid: sysinfo::Pid,
        max_threads: usize,
    ) {
        // The ordering of these locks is _very_ important. They MUST be in sync with how the main GUI thread locks stuff
        // or a deadlock will occur given enough time. They will eventually both want to lock the [Arc<Mutex>] the other
        // thread is already locking. Yes, I figured this out the hard way, hence the vast amount of debug!() messages.
        // Example of different order (BAD!):
        //
        // GUI Main       -> locks [p2pool] first
        // Helper         -> locks [gui_api_p2pool] first
        // GUI Status Tab -> tries to lock [gui_api_p2pool] -> CAN'T
        // Helper         -> tries to lock [p2pool] -> CAN'T
        //
        // These two threads are now in a deadlock because both
        // are trying to access locks the other one already has.
        //
        // The locking order here must be in the same chronological
        // order as the main GUI thread (top to bottom).

        let helper = Arc::clone(helper);
        let lock = lock!(helper);
        let p2pool = Arc::clone(&lock.p2pool);
        let xmrig = Arc::clone(&lock.xmrig);
        let pub_sys = Arc::clone(&lock.pub_sys);
        let gui_api_p2pool = Arc::clone(&lock.gui_api_p2pool);
        let gui_api_xmrig = Arc::clone(&lock.gui_api_xmrig);
        let pub_api_p2pool = Arc::clone(&lock.pub_api_p2pool);
        let pub_api_xmrig = Arc::clone(&lock.pub_api_xmrig);
        drop(lock);

        let sysinfo_cpu = sysinfo::CpuRefreshKind::everything();
        let sysinfo_processes = sysinfo::ProcessRefreshKind::new().with_cpu();

        thread::spawn(move || {
            info!("Helper | Hello from helper thread! Entering loop where I will spend the rest of my days...");
            // Begin loop
            loop {
                // 1. Loop init timestamp
                let start = Instant::now();
                debug!("Helper | ----------- Start of loop -----------");

                // Ignore the invasive [debug!()] messages on the right side of the code.
                // The reason why they are there are so that it's extremely easy to track
                // down the culprit of an [Arc<Mutex>] deadlock. I know, they're ugly.

                // 2. Lock... EVERYTHING!
                let mut lock = lock!(helper);
                debug!("Helper | Locking (1/8) ... [helper]");
                let p2pool = lock!(p2pool);
                debug!("Helper | Locking (2/8) ... [p2pool]");
                let xmrig = lock!(xmrig);
                debug!("Helper | Locking (3/8) ... [xmrig]");
                let mut lock_pub_sys = lock!(pub_sys);
                debug!("Helper | Locking (4/8) ... [pub_sys]");
                let mut gui_api_p2pool = lock!(gui_api_p2pool);
                debug!("Helper | Locking (5/8) ... [gui_api_p2pool]");
                let mut gui_api_xmrig = lock!(gui_api_xmrig);
                debug!("Helper | Locking (6/8) ... [gui_api_xmrig]");
                let mut pub_api_p2pool = lock!(pub_api_p2pool);
                debug!("Helper | Locking (7/8) ... [pub_api_p2pool]");
                let mut pub_api_xmrig = lock!(pub_api_xmrig);
                debug!("Helper | Locking (8/8) ... [pub_api_xmrig]");
                // Calculate Gupax's uptime always.
                lock.uptime = HumanTime::into_human(lock.instant.elapsed());
                // If [P2Pool] is alive...
                if p2pool.is_alive() {
                    debug!("Helper | P2Pool is alive! Running [combine_gui_pub_api()]");
                    PubP2poolApi::combine_gui_pub_api(&mut gui_api_p2pool, &mut pub_api_p2pool);
                } else {
                    debug!("Helper | P2Pool is dead! Skipping...");
                }
                // If [XMRig] is alive...
                if xmrig.is_alive() {
                    debug!("Helper | XMRig is alive! Running [combine_gui_pub_api()]");
                    PubXmrigApi::combine_gui_pub_api(&mut gui_api_xmrig, &mut pub_api_xmrig);
                } else {
                    debug!("Helper | XMRig is dead! Skipping...");
                }

                // 2. Selectively refresh [sysinfo] for only what we need (better performance).
                sysinfo.refresh_cpu_specifics(sysinfo_cpu);
                debug!("Helper | Sysinfo refresh (1/3) ... [cpu]");
                sysinfo.refresh_processes_specifics(sysinfo_processes);
                debug!("Helper | Sysinfo refresh (2/3) ... [processes]");
                sysinfo.refresh_memory();
                debug!("Helper | Sysinfo refresh (3/3) ... [memory]");
                debug!("Helper | Sysinfo OK, running [update_pub_sys_from_sysinfo()]");
                Self::update_pub_sys_from_sysinfo(
                    &sysinfo,
                    &mut lock_pub_sys,
                    &pid,
                    &lock,
                    max_threads,
                );

                // 3. Drop... (almost) EVERYTHING... IN REVERSE!
                drop(lock_pub_sys);
                debug!("Helper | Unlocking (1/8) ... [pub_sys]");
                drop(xmrig);
                debug!("Helper | Unlocking (2/8) ... [xmrig]");
                drop(p2pool);
                debug!("Helper | Unlocking (3/8) ... [p2pool]");
                drop(pub_api_xmrig);
                debug!("Helper | Unlocking (4/8) ... [pub_api_xmrig]");
                drop(pub_api_p2pool);
                debug!("Helper | Unlocking (5/8) ... [pub_api_p2pool]");
                drop(gui_api_xmrig);
                debug!("Helper | Unlocking (6/8) ... [gui_api_xmrig]");
                drop(gui_api_p2pool);
                debug!("Helper | Unlocking (7/8) ... [gui_api_p2pool]");
                drop(lock);
                debug!("Helper | Unlocking (8/8) ... [helper]");

                // 4. Calculate if we should sleep or not.
                // If we should sleep, how long?
                let elapsed = start.elapsed().as_millis();
                if elapsed < 1000 {
                    // Casting from u128 to u64 should be safe here, because [elapsed]
                    // is less than 1000, meaning it can fit into a u64 easy.
                    let sleep = (1000 - elapsed) as u64;
                    debug!("Helper | END OF LOOP - Sleeping for [{}]ms...", sleep);
                    sleep!(sleep);
                } else {
                    debug!("Helper | END OF LOOP - Not sleeping!");
                }

                // 5. End loop
            }
        });
    }
}

//---------------------------------------------------------------------------------------------------- [ImgP2pool]
// A static "image" of data that P2Pool started with.
// This is just a snapshot of the user data when they initially started P2Pool.
// Created by [start_p2pool()] and return to the main GUI thread where it will store it.
// No need for an [Arc<Mutex>] since the Helper thread doesn't need this information.
#[derive(Debug, Clone)]
pub struct ImgP2pool {
    pub mini: String,      // Did the user start on the mini-chain?
    pub address: String, // What address is the current p2pool paying out to? (This gets shortened to [4xxxxx...xxxxxx])
    pub host: String,    // What monerod are we using?
    pub rpc: String,     // What is the RPC port?
    pub zmq: String,     // What is the ZMQ port?
    pub out_peers: String, // How many out-peers?
    pub in_peers: String, // How many in-peers?
}

impl Default for ImgP2pool {
    fn default() -> Self {
        Self::new()
    }
}

impl ImgP2pool {
    pub fn new() -> Self {
        Self {
            mini: String::from("???"),
            address: String::from("???"),
            host: String::from("???"),
            rpc: String::from("???"),
            zmq: String::from("???"),
            out_peers: String::from("???"),
            in_peers: String::from("???"),
        }
    }
}

//---------------------------------------------------------------------------------------------------- Public P2Pool API
// Helper/GUI threads both have a copy of this, Helper updates
// the GUI's version on a 1-second interval from the private data.
#[derive(Debug, Clone, PartialEq)]
pub struct PubP2poolApi {
    // Output
    pub output: String,
    // Uptime
    pub uptime: HumanTime,
    // These are manually parsed from the STDOUT.
    pub payouts: u128,
    pub payouts_hour: f64,
    pub payouts_day: f64,
    pub payouts_month: f64,
    pub xmr: f64,
    pub xmr_hour: f64,
    pub xmr_day: f64,
    pub xmr_month: f64,
    // Local API
    pub hashrate_15m: HumanNumber,
    pub hashrate_1h: HumanNumber,
    pub hashrate_24h: HumanNumber,
    pub shares_found: HumanNumber,
    pub average_effort: HumanNumber,
    pub current_effort: HumanNumber,
    pub connections: HumanNumber,
    // The API needs a raw ints to go off of and
    // there's not a good way to access it without doing weird
    // [Arc<Mutex>] shenanigans, so some raw ints are stored here.
    pub user_p2pool_hashrate_u64: u64,
    pub p2pool_difficulty_u64: u64,
    pub monero_difficulty_u64: u64,
    pub p2pool_hashrate_u64: u64,
    pub monero_hashrate_u64: u64,
    // Tick. Every loop this gets incremented.
    // At 60, it indicated we should read the below API files.
    pub tick: u8,
    // Network API
    pub monero_difficulty: HumanNumber, // e.g: [15,000,000]
    pub monero_hashrate: HumanNumber,   // e.g: [1.000 GH/s]
    pub hash: String,                   // Current block hash
    pub height: HumanNumber,
    pub reward: AtomicUnit,
    // Pool API
    pub p2pool_difficulty: HumanNumber,
    pub p2pool_hashrate: HumanNumber,
    pub miners: HumanNumber, // Current amount of miners on P2Pool sidechain
    // Mean (calculated in functions, not serialized)
    pub solo_block_mean: HumanTime, // Time it would take the user to find a solo block
    pub p2pool_block_mean: HumanTime, // Time it takes the P2Pool sidechain to find a block
    pub p2pool_share_mean: HumanTime, // Time it would take the user to find a P2Pool share
    // Percent
    pub p2pool_percent: HumanNumber, // Percentage of P2Pool hashrate capture of overall Monero hashrate.
    pub user_p2pool_percent: HumanNumber, // How much percent the user's hashrate accounts for in P2Pool.
    pub user_monero_percent: HumanNumber, // How much percent the user's hashrate accounts for in all of Monero hashrate.
}

impl Default for PubP2poolApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PubP2poolApi {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            uptime: HumanTime::new(),
            payouts: 0,
            payouts_hour: 0.0,
            payouts_day: 0.0,
            payouts_month: 0.0,
            xmr: 0.0,
            xmr_hour: 0.0,
            xmr_day: 0.0,
            xmr_month: 0.0,
            hashrate_15m: HumanNumber::unknown(),
            hashrate_1h: HumanNumber::unknown(),
            hashrate_24h: HumanNumber::unknown(),
            shares_found: HumanNumber::unknown(),
            average_effort: HumanNumber::unknown(),
            current_effort: HumanNumber::unknown(),
            connections: HumanNumber::unknown(),
            tick: 0,
            user_p2pool_hashrate_u64: 0,
            p2pool_difficulty_u64: 0,
            monero_difficulty_u64: 0,
            p2pool_hashrate_u64: 0,
            monero_hashrate_u64: 0,
            monero_difficulty: HumanNumber::unknown(),
            monero_hashrate: HumanNumber::unknown(),
            hash: String::from("???"),
            height: HumanNumber::unknown(),
            reward: AtomicUnit::new(),
            p2pool_difficulty: HumanNumber::unknown(),
            p2pool_hashrate: HumanNumber::unknown(),
            miners: HumanNumber::unknown(),
            solo_block_mean: HumanTime::new(),
            p2pool_block_mean: HumanTime::new(),
            p2pool_share_mean: HumanTime::new(),
            p2pool_percent: HumanNumber::unknown(),
            user_p2pool_percent: HumanNumber::unknown(),
            user_monero_percent: HumanNumber::unknown(),
        }
    }

    #[inline]
    // The issue with just doing [gui_api = pub_api] is that values get overwritten.
    // This doesn't matter for any of the values EXCEPT for the output, so we must
    // manually append it instead of overwriting.
    // This is used in the "helper" thread.
    fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
        let mut output = std::mem::take(&mut gui_api.output);
        let buf = std::mem::take(&mut pub_api.output);
        if !buf.is_empty() {
            output.push_str(&buf);
        }
        *gui_api = Self {
            output,
            tick: std::mem::take(&mut gui_api.tick),
            ..pub_api.clone()
        };
    }

    #[inline]
    // Essentially greps the output for [x.xxxxxxxxxxxx XMR] where x = a number.
    // It sums each match and counts along the way, handling an error by not adding and printing to console.
    fn calc_payouts_and_xmr(output: &str) -> (u128 /* payout count */, f64 /* total xmr */) {
        let iter = P2POOL_REGEX.payout.find_iter(output);
        let mut sum: f64 = 0.0;
        let mut count: u128 = 0;
        for i in iter {
            if let Some(word) = P2POOL_REGEX.payout_float.find(i.as_str()) {
                match word.as_str().parse::<f64>() {
                    Ok(num) => {
                        sum += num;
                        count += 1;
                    }
                    Err(e) => error!("P2Pool | Total XMR sum calculation error: [{}]", e),
                }
            }
        }
        (count, sum)
    }

    // Mutate "watchdog"'s [PubP2poolApi] with data the process output.
    fn update_from_output(
        public: &Arc<Mutex<Self>>,
        output_parse: &Arc<Mutex<String>>,
        output_pub: &Arc<Mutex<String>>,
        elapsed: std::time::Duration,
        process: &Arc<Mutex<Process>>,
    ) {
        // 1. Take the process's current output buffer and combine it with Pub (if not empty)
        let mut output_pub = lock!(output_pub);
        if !output_pub.is_empty() {
            lock!(public)
                .output
                .push_str(&std::mem::take(&mut *output_pub));
        }

        // 2. Parse the full STDOUT
        let mut output_parse = lock!(output_parse);
        let (payouts_new, xmr_new) = Self::calc_payouts_and_xmr(&output_parse);
        // Check for "SYNCHRONIZED" only if we aren't already.
        if lock!(process).state == ProcessState::Syncing {
            // How many times the word was captured.
            let synchronized_captures = P2POOL_REGEX.synchronized.find_iter(&output_parse).count();

            // If P2Pool receives shares before syncing, it will start mining on its own sidechain.
            // In this instance, we technically are "synced" on block 1 and P2Pool will print "SYNCHRONIZED"
            // although, that doesn't necessarily mean we're synced on main/mini-chain.
            //
            // So, if we find a `next block = 1`, that means we
            // must look for at least 2 instances of "SYNCHRONIZED",
            // one for the sidechain, one for main/mini.
            if P2POOL_REGEX.next_height_1.is_match(&output_parse) {
                if synchronized_captures > 1 {
                    lock!(process).state = ProcessState::Alive;
                }
            } else if synchronized_captures > 0 {
                // if there is no `next block = 1`, fallback to
                // just finding 1 instance of "SYNCHRONIZED".
                lock!(process).state = ProcessState::Alive;
            }
        }
        // 3. Throw away [output_parse]
        output_parse.clear();
        drop(output_parse);
        // 4. Add to current values
        let mut public = lock!(public);
        let (payouts, xmr) = (public.payouts + payouts_new, public.xmr + xmr_new);

        // 5. Calculate hour/day/month given elapsed time
        let elapsed_as_secs_f64 = elapsed.as_secs_f64();
        // Payouts
        let per_sec = (payouts as f64) / elapsed_as_secs_f64;
        let payouts_hour = (per_sec * 60.0) * 60.0;
        let payouts_day = payouts_hour * 24.0;
        let payouts_month = payouts_day * 30.0;
        // Total XMR
        let per_sec = xmr / elapsed_as_secs_f64;
        let xmr_hour = (per_sec * 60.0) * 60.0;
        let xmr_day = xmr_hour * 24.0;
        let xmr_month = xmr_day * 30.0;

        if payouts_new != 0 {
            debug!(
                "P2Pool Watchdog | New [Payout] found in output ... {}",
                payouts_new
            );
            debug!("P2Pool Watchdog | Total [Payout] should be ... {}", payouts);
            debug!(
                "P2Pool Watchdog | Correct [Payout per] should be ... [{}/hour, {}/day, {}/month]",
                payouts_hour, payouts_day, payouts_month
            );
        }
        if xmr_new != 0.0 {
            debug!(
                "P2Pool Watchdog | New [XMR mined] found in output ... {}",
                xmr_new
            );
            debug!("P2Pool Watchdog | Total [XMR mined] should be ... {}", xmr);
            debug!("P2Pool Watchdog | Correct [XMR mined per] should be ... [{}/hour, {}/day, {}/month]", xmr_hour, xmr_day, xmr_month);
        }

        // 6. Mutate the struct with the new info
        *public = Self {
            uptime: HumanTime::into_human(elapsed),
            payouts,
            xmr,
            payouts_hour,
            payouts_day,
            payouts_month,
            xmr_hour,
            xmr_day,
            xmr_month,
            ..std::mem::take(&mut *public)
        };
    }

    // Mutate [PubP2poolApi] with data from a [PrivP2poolLocalApi] and the process output.
    fn update_from_local(public: &Arc<Mutex<Self>>, local: PrivP2poolLocalApi) {
        let mut public = lock!(public);
        *public = Self {
            hashrate_15m: HumanNumber::from_u64(local.hashrate_15m),
            hashrate_1h: HumanNumber::from_u64(local.hashrate_1h),
            hashrate_24h: HumanNumber::from_u64(local.hashrate_24h),
            shares_found: HumanNumber::from_u64(local.shares_found),
            average_effort: HumanNumber::to_percent(local.average_effort),
            current_effort: HumanNumber::to_percent(local.current_effort),
            connections: HumanNumber::from_u32(local.connections),
            user_p2pool_hashrate_u64: local.hashrate_1h,
            ..std::mem::take(&mut *public)
        };
    }

    // Mutate [PubP2poolApi] with data from a [PrivP2pool(Network|Pool)Api].
    fn update_from_network_pool(
        public: &Arc<Mutex<Self>>,
        net: PrivP2poolNetworkApi,
        pool: PrivP2poolPoolApi,
    ) {
        let user_hashrate = lock!(public).user_p2pool_hashrate_u64; // The user's total P2Pool hashrate
        let monero_difficulty = net.difficulty;
        let monero_hashrate = monero_difficulty / MONERO_BLOCK_TIME_IN_SECONDS;
        let p2pool_hashrate = pool.pool_statistics.hashRate;
        let p2pool_difficulty = p2pool_hashrate * P2POOL_BLOCK_TIME_IN_SECONDS;
        // These [0] checks prevent dividing by 0 (it [panic!()]s)
        let p2pool_block_mean;
        let user_p2pool_percent;
        if p2pool_hashrate == 0 {
            p2pool_block_mean = HumanTime::new();
            user_p2pool_percent = HumanNumber::unknown();
        } else {
            p2pool_block_mean = HumanTime::into_human(std::time::Duration::from_secs(
                monero_difficulty / p2pool_hashrate,
            ));
            let f = (user_hashrate as f64 / p2pool_hashrate as f64) * 100.0;
            user_p2pool_percent = HumanNumber::from_f64_to_percent_6_point(f);
        };
        let p2pool_percent;
        let user_monero_percent;
        if monero_hashrate == 0 {
            p2pool_percent = HumanNumber::unknown();
            user_monero_percent = HumanNumber::unknown();
        } else {
            let f = (p2pool_hashrate as f64 / monero_hashrate as f64) * 100.0;
            p2pool_percent = HumanNumber::from_f64_to_percent_6_point(f);
            let f = (user_hashrate as f64 / monero_hashrate as f64) * 100.0;
            user_monero_percent = HumanNumber::from_f64_to_percent_6_point(f);
        };
        let solo_block_mean;
        let p2pool_share_mean;
        if user_hashrate == 0 {
            solo_block_mean = HumanTime::new();
            p2pool_share_mean = HumanTime::new();
        } else {
            solo_block_mean = HumanTime::into_human(std::time::Duration::from_secs(
                monero_difficulty / user_hashrate,
            ));
            p2pool_share_mean = HumanTime::into_human(std::time::Duration::from_secs(
                p2pool_difficulty / user_hashrate,
            ));
        }
        let mut public = lock!(public);
        *public = Self {
            p2pool_difficulty_u64: p2pool_difficulty,
            monero_difficulty_u64: monero_difficulty,
            p2pool_hashrate_u64: p2pool_hashrate,
            monero_hashrate_u64: monero_hashrate,
            monero_difficulty: HumanNumber::from_u64(monero_difficulty),
            monero_hashrate: HumanNumber::from_u64_to_gigahash_3_point(monero_hashrate),
            hash: net.hash,
            height: HumanNumber::from_u32(net.height),
            reward: AtomicUnit::from_u64(net.reward),
            p2pool_difficulty: HumanNumber::from_u64(p2pool_difficulty),
            p2pool_hashrate: HumanNumber::from_u64_to_megahash_3_point(p2pool_hashrate),
            miners: HumanNumber::from_u32(pool.pool_statistics.miners),
            solo_block_mean,
            p2pool_block_mean,
            p2pool_share_mean,
            p2pool_percent,
            user_p2pool_percent,
            user_monero_percent,
            ..std::mem::take(&mut *public)
        };
    }

    #[inline]
    pub fn calculate_share_or_block_time(hashrate: u64, difficulty: u64) -> HumanTime {
        if hashrate == 0 {
            HumanTime::new()
        } else {
            HumanTime::from_u64(difficulty / hashrate)
        }
    }

    #[inline]
    pub fn calculate_dominance(my_hashrate: u64, global_hashrate: u64) -> HumanNumber {
        if global_hashrate == 0 {
            HumanNumber::unknown()
        } else {
            let f = (my_hashrate as f64 / global_hashrate as f64) * 100.0;
            HumanNumber::from_f64_to_percent_6_point(f)
        }
    }

    pub const fn calculate_tick_bar(&self) -> &'static str {
        // The stars are reduced by one because it takes a frame to render the stats.
        // We want 0 stars at the same time stats are rendered, so it looks a little off here.
        match self.tick {
            1 => "[                                                            ]",
            2 => "[*                                                           ]",
            3 => "[**                                                          ]",
            4 => "[***                                                         ]",
            5 => "[****                                                        ]",
            6 => "[*****                                                       ]",
            7 => "[******                                                      ]",
            8 => "[*******                                                     ]",
            9 => "[********                                                    ]",
            10 => "[*********                                                   ]",
            11 => "[**********                                                  ]",
            12 => "[***********                                                 ]",
            13 => "[************                                                ]",
            14 => "[*************                                               ]",
            15 => "[**************                                              ]",
            16 => "[***************                                             ]",
            17 => "[****************                                            ]",
            18 => "[*****************                                           ]",
            19 => "[******************                                          ]",
            20 => "[*******************                                         ]",
            21 => "[********************                                        ]",
            22 => "[*********************                                       ]",
            23 => "[**********************                                      ]",
            24 => "[***********************                                     ]",
            25 => "[************************                                    ]",
            26 => "[*************************                                   ]",
            27 => "[**************************                                  ]",
            28 => "[***************************                                 ]",
            29 => "[****************************                                ]",
            30 => "[*****************************                               ]",
            31 => "[******************************                              ]",
            32 => "[*******************************                             ]",
            33 => "[********************************                            ]",
            34 => "[*********************************                           ]",
            35 => "[**********************************                          ]",
            36 => "[***********************************                         ]",
            37 => "[************************************                        ]",
            38 => "[*************************************                       ]",
            39 => "[**************************************                      ]",
            40 => "[***************************************                     ]",
            41 => "[****************************************                    ]",
            42 => "[*****************************************                   ]",
            43 => "[******************************************                  ]",
            44 => "[*******************************************                 ]",
            45 => "[********************************************                ]",
            46 => "[*********************************************               ]",
            47 => "[**********************************************              ]",
            48 => "[***********************************************             ]",
            49 => "[************************************************            ]",
            50 => "[*************************************************           ]",
            51 => "[**************************************************          ]",
            52 => "[***************************************************         ]",
            53 => "[****************************************************        ]",
            54 => "[*****************************************************       ]",
            55 => "[******************************************************      ]",
            56 => "[*******************************************************     ]",
            57 => "[********************************************************    ]",
            58 => "[*********************************************************   ]",
            59 => "[**********************************************************  ]",
            60 => "[*********************************************************** ]",
            _ => "[************************************************************]",
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Local" Api
// This matches directly to P2Pool's [local/stratum] JSON API file (excluding a few stats).
// P2Pool seems to initialize all stats at 0 (or 0.0), so no [Option] wrapper seems needed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PrivP2poolLocalApi {
    hashrate_15m: u64,
    hashrate_1h: u64,
    hashrate_24h: u64,
    shares_found: u64,
    average_effort: f32,
    current_effort: f32,
    connections: u32, // This is a `uint32_t` in `p2pool`
}

impl Default for PrivP2poolLocalApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivP2poolLocalApi {
    fn new() -> Self {
        Self {
            hashrate_15m: 0,
            hashrate_1h: 0,
            hashrate_24h: 0,
            shares_found: 0,
            average_effort: 0.0,
            current_effort: 0.0,
            connections: 0,
        }
    }

    // Deserialize the above [String] into a [PrivP2poolApi]
    fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(string) {
            Ok(a) => Ok(a),
            Err(e) => {
                warn!("P2Pool Local API | Could not deserialize API data: {}", e);
                Err(e)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Network" API
// This matches P2Pool's [network/stats] JSON API file.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PrivP2poolNetworkApi {
    difficulty: u64,
    hash: String,
    height: u32,
    reward: u64,
    timestamp: u32,
}

impl Default for PrivP2poolNetworkApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivP2poolNetworkApi {
    fn new() -> Self {
        Self {
            difficulty: 0,
            hash: String::from("???"),
            height: 0,
            reward: 0,
            timestamp: 0,
        }
    }

    fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(string) {
            Ok(a) => Ok(a),
            Err(e) => {
                warn!("P2Pool Network API | Could not deserialize API data: {}", e);
                Err(e)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Pool" API
// This matches P2Pool's [pool/stats] JSON API file.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PrivP2poolPoolApi {
    pool_statistics: PoolStatistics,
}

impl Default for PrivP2poolPoolApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivP2poolPoolApi {
    fn new() -> Self {
        Self {
            pool_statistics: PoolStatistics::new(),
        }
    }

    fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
        match serde_json::from_str::<Self>(string) {
            Ok(a) => Ok(a),
            Err(e) => {
                warn!("P2Pool Pool API | Could not deserialize API data: {}", e);
                Err(e)
            }
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PoolStatistics {
    hashRate: u64,
    miners: u32,
}
impl Default for PoolStatistics {
    fn default() -> Self {
        Self::new()
    }
}
impl PoolStatistics {
    fn new() -> Self {
        Self {
            hashRate: 0,
            miners: 0,
        }
    }
}

//---------------------------------------------------------------------------------------------------- [ImgXmrig]
#[derive(Debug, Clone)]
pub struct ImgXmrig {
    pub threads: String,
    pub url: String,
}

impl Default for ImgXmrig {
    fn default() -> Self {
        Self::new()
    }
}

impl ImgXmrig {
    pub fn new() -> Self {
        Self {
            threads: "???".to_string(),
            url: "???".to_string(),
        }
    }
}

//---------------------------------------------------------------------------------------------------- Public XMRig API
#[derive(Debug, Clone)]
pub struct PubXmrigApi {
    pub output: String,
    pub uptime: HumanTime,
    pub worker_id: String,
    pub resources: HumanNumber,
    pub hashrate: HumanNumber,
    pub diff: HumanNumber,
    pub accepted: HumanNumber,
    pub rejected: HumanNumber,

    pub hashrate_raw: f32,
}

impl Default for PubXmrigApi {
    fn default() -> Self {
        Self::new()
    }
}

impl PubXmrigApi {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            uptime: HumanTime::new(),
            worker_id: "???".to_string(),
            resources: HumanNumber::unknown(),
            hashrate: HumanNumber::unknown(),
            diff: HumanNumber::unknown(),
            accepted: HumanNumber::unknown(),
            rejected: HumanNumber::unknown(),
            hashrate_raw: 0.0,
        }
    }

    #[inline]
    fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
        let output = std::mem::take(&mut gui_api.output);
        let buf = std::mem::take(&mut pub_api.output);
        *gui_api = Self {
            output,
            ..std::mem::take(pub_api)
        };
        if !buf.is_empty() {
            gui_api.output.push_str(&buf);
        }
    }

    // This combines the buffer from the PTY thread [output_pub]
    // with the actual [PubApiXmrig] output field.
    fn update_from_output(
        public: &Arc<Mutex<Self>>,
        output_parse: &Arc<Mutex<String>>,
        output_pub: &Arc<Mutex<String>>,
        elapsed: std::time::Duration,
        process: &Arc<Mutex<Process>>,
    ) {
        // 1. Take the process's current output buffer and combine it with Pub (if not empty)
        let mut output_pub = lock!(output_pub);

        {
            let mut public = lock!(public);
            if !output_pub.is_empty() {
                public.output.push_str(&std::mem::take(&mut *output_pub));
            }
            // Update uptime
            public.uptime = HumanTime::into_human(elapsed);
        }

        // 2. Check for "new job"/"no active...".
        let mut output_parse = lock!(output_parse);
        if XMRIG_REGEX.new_job.is_match(&output_parse) {
            lock!(process).state = ProcessState::Alive;
        } else if XMRIG_REGEX.not_mining.is_match(&output_parse) {
            lock!(process).state = ProcessState::NotMining;
        }

        // 3. Throw away [output_parse]
        output_parse.clear();
        drop(output_parse);
    }

    // Formats raw private data into ready-to-print human readable version.
    fn update_from_priv(public: &Arc<Mutex<Self>>, private: PrivXmrigApi) {
        let mut public = lock!(public);
        let hashrate_raw = match private.hashrate.total.first() {
            Some(Some(h)) => *h,
            _ => 0.0,
        };

        *public = Self {
            worker_id: private.worker_id,
            resources: HumanNumber::from_load(private.resources.load_average),
            hashrate: HumanNumber::from_hashrate(private.hashrate.total),
            diff: HumanNumber::from_u128(private.connection.diff),
            accepted: HumanNumber::from_u128(private.connection.accepted),
            rejected: HumanNumber::from_u128(private.connection.rejected),
            hashrate_raw,
            ..std::mem::take(&mut *public)
        }
    }
}

//---------------------------------------------------------------------------------------------------- Private XMRig API
// This matches to some JSON stats in the HTTP call [summary],
// e.g: [wget -qO- localhost:18085/1/summary].
// XMRig doesn't initialize stats at 0 (or 0.0) and instead opts for [null]
// which means some elements need to be wrapped in an [Option] or else serde will [panic!].
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PrivXmrigApi {
    worker_id: String,
    resources: Resources,
    connection: Connection,
    hashrate: Hashrate,
}

impl PrivXmrigApi {
    fn new() -> Self {
        Self {
            worker_id: String::new(),
            resources: Resources::new(),
            connection: Connection::new(),
            hashrate: Hashrate::new(),
        }
    }

    #[inline]
    // Send an HTTP request to XMRig's API, serialize it into [Self] and return it
    async fn request_xmrig_api(
        client: hyper::Client<hyper::client::HttpConnector>,
        api_uri: &str,
    ) -> std::result::Result<Self, anyhow::Error> {
        let request = hyper::Request::builder()
            .method("GET")
            .uri(api_uri)
            .body(hyper::Body::empty())?;
        let response = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            client.request(request),
        )
        .await?;
        let body = hyper::body::to_bytes(response?.body_mut()).await?;
        Ok(serde_json::from_slice::<Self>(&body)?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Resources {
    load_average: [Option<f32>; 3],
}
impl Resources {
    fn new() -> Self {
        Self {
            load_average: [Some(0.0), Some(0.0), Some(0.0)],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Connection {
    diff: u128,
    accepted: u128,
    rejected: u128,
}
impl Connection {
    fn new() -> Self {
        Self {
            diff: 0,
            accepted: 0,
            rejected: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Hashrate {
    total: [Option<f32>; 3],
}
impl Hashrate {
    fn new() -> Self {
        Self {
            total: [Some(0.0), Some(0.0), Some(0.0)],
        }
    }
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reset_gui_output() {
        let max = crate::helper::GUI_OUTPUT_LEEWAY;
        let mut string = String::with_capacity(max);
        for _ in 0..=max {
            string.push('0');
        }
        crate::Helper::check_reset_gui_output(&mut string, crate::ProcessName::P2pool);
        // Some text gets added, so just check for less than 500 bytes.
        assert!(string.len() < 500);
    }

    #[test]
    fn combine_gui_pub_p2pool_api() {
        use crate::helper::PubP2poolApi;
        let mut gui_api = PubP2poolApi::new();
        let mut pub_api = PubP2poolApi::new();
        pub_api.payouts = 1;
        pub_api.payouts_hour = 2.0;
        pub_api.payouts_day = 3.0;
        pub_api.payouts_month = 4.0;
        pub_api.xmr = 1.0;
        pub_api.xmr_hour = 2.0;
        pub_api.xmr_day = 3.0;
        pub_api.xmr_month = 4.0;
        println!("BEFORE - GUI_API: {:#?}\nPUB_API: {:#?}", gui_api, pub_api);
        assert_ne!(gui_api, pub_api);
        PubP2poolApi::combine_gui_pub_api(&mut gui_api, &mut pub_api);
        println!("AFTER - GUI_API: {:#?}\nPUB_API: {:#?}", gui_api, pub_api);
        assert_eq!(gui_api, pub_api);
        pub_api.xmr = 2.0;
        PubP2poolApi::combine_gui_pub_api(&mut gui_api, &mut pub_api);
        assert_eq!(gui_api, pub_api);
        assert_eq!(gui_api.xmr, 2.0);
        assert_eq!(pub_api.xmr, 2.0);
    }

    #[test]
    fn calc_payouts_and_xmr_from_output_p2pool() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			payout of 5.000000000001 XMR in block 1112
			payout of 5.000000000001 XMR in block 1113"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        let public = public.lock().unwrap();
        println!("{:#?}", public);
        assert_eq!(public.payouts, 3);
        assert_eq!(public.payouts_hour, 180.0);
        assert_eq!(public.payouts_day, 4320.0);
        assert_eq!(public.payouts_month, 129600.0);
        assert_eq!(public.xmr, 15.000000000003);
        assert_eq!(public.xmr_hour, 900.00000000018);
        assert_eq!(public.xmr_day, 21600.00000000432);
        assert_eq!(public.xmr_month, 648000.0000001296);
    }

    #[test]
    fn set_p2pool_synchronized() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			NOTICE  2021-12-27 21:42:17.2008 SideChain SYNCHRONIZED
			payout of 5.000000000001 XMR in block 1113"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));

        // It only gets checked if we're `Syncing`.
        process.lock().unwrap().state = ProcessState::Syncing;
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::Alive);
    }

    #[test]
    fn p2pool_synchronized_false_positive() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));

        // The SideChain that is "SYNCHRONIZED" in this output is
        // probably not main/mini, but the sidechain started on height 1,
        // so this should _not_ trigger alive state.
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			SideChain new chain tip: next height = 1,
			NOTICE  2021-12-27 21:42:17.2008 SideChain SYNCHRONIZED
			payout of 5.000000000001 XMR in block 1113"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));

        // It only gets checked if we're `Syncing`.
        process.lock().unwrap().state = ProcessState::Syncing;
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::Syncing); // still syncing
    }

    #[test]
    fn p2pool_synchronized_double_synchronized() {
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));

        // The 1st SideChain that is "SYNCHRONIZED" in this output is
        // the sidechain started on height 1, but there is another one
        // which means the real main/mini is probably synced,
        // so this _should_ trigger alive state.
        let output_parse = Arc::new(Mutex::new(String::from(
            r#"payout of 5.000000000001 XMR in block 1111
			SideChain new chain tip: next height = 1,
			NOTICE  2021-12-27 21:42:17.2008 SideChain SYNCHRONIZED
			payout of 5.000000000001 XMR in block 1113
			NOTICE  2021-12-27 21:42:17.2100 SideChain SYNCHRONIZED"#,
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::P2pool,
            "".to_string(),
            PathBuf::new(),
        )));

        // It only gets checked if we're `Syncing`.
        process.lock().unwrap().state = ProcessState::Syncing;
        PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::Alive);
    }

    #[test]
    fn update_pub_p2pool_from_local_network_pool() {
        use crate::helper::PoolStatistics;
        use crate::helper::PrivP2poolLocalApi;
        use crate::helper::PrivP2poolNetworkApi;
        use crate::helper::PrivP2poolPoolApi;
        use crate::helper::PubP2poolApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubP2poolApi::new()));
        let local = PrivP2poolLocalApi {
            hashrate_15m: 10_000,
            hashrate_1h: 20_000,
            hashrate_24h: 30_000,
            shares_found: 1000,
            average_effort: 100.000,
            current_effort: 200.000,
            connections: 1234,
        };
        let network = PrivP2poolNetworkApi {
            difficulty: 300_000_000_000,
            hash: "asdf".to_string(),
            height: 1234,
            reward: 2345,
            timestamp: 3456,
        };
        let pool = PrivP2poolPoolApi {
            pool_statistics: PoolStatistics {
                hashRate: 1_000_000, // 1 MH/s
                miners: 1_000,
            },
        };
        // Update Local
        PubP2poolApi::update_from_local(&public, local);
        let p = public.lock().unwrap();
        println!("AFTER LOCAL: {:#?}", p);
        assert_eq!(p.hashrate_15m.to_string(), "10,000");
        assert_eq!(p.hashrate_1h.to_string(), "20,000");
        assert_eq!(p.hashrate_24h.to_string(), "30,000");
        assert_eq!(p.shares_found.to_string(), "1,000");
        assert_eq!(p.average_effort.to_string(), "100.00%");
        assert_eq!(p.current_effort.to_string(), "200.00%");
        assert_eq!(p.connections.to_string(), "1,234");
        assert_eq!(p.user_p2pool_hashrate_u64, 20000);
        drop(p);
        // Update Network + Pool
        PubP2poolApi::update_from_network_pool(&public, network, pool);
        let p = public.lock().unwrap();
        println!("AFTER NETWORK+POOL: {:#?}", p);
        assert_eq!(p.monero_difficulty.to_string(), "300,000,000,000");
        assert_eq!(p.monero_hashrate.to_string(), "2.500 GH/s");
        assert_eq!(p.hash.to_string(), "asdf");
        assert_eq!(p.height.to_string(), "1,234");
        assert_eq!(p.reward.to_u64(), 2345);
        assert_eq!(p.p2pool_difficulty.to_string(), "10,000,000");
        assert_eq!(p.p2pool_hashrate.to_string(), "1.000 MH/s");
        assert_eq!(p.miners.to_string(), "1,000");
        assert_eq!(
            p.solo_block_mean.to_string(),
            "5 months, 21 days, 9 hours, 52 minutes"
        );
        assert_eq!(
            p.p2pool_block_mean.to_string(),
            "3 days, 11 hours, 20 minutes"
        );
        assert_eq!(p.p2pool_share_mean.to_string(), "8 minutes, 20 seconds");
        assert_eq!(p.p2pool_percent.to_string(), "0.040000%");
        assert_eq!(p.user_p2pool_percent.to_string(), "2.000000%");
        assert_eq!(p.user_monero_percent.to_string(), "0.000800%");
        drop(p);
    }

    #[test]
    fn set_xmrig_mining() {
        use crate::helper::PubXmrigApi;
        use std::sync::{Arc, Mutex};
        let public = Arc::new(Mutex::new(PubXmrigApi::new()));
        let output_parse = Arc::new(Mutex::new(String::from(
            "[2022-02-12 12:49:30.311]  net      no active pools, stop mining",
        )));
        let output_pub = Arc::new(Mutex::new(String::new()));
        let elapsed = std::time::Duration::from_secs(60);
        let process = Arc::new(Mutex::new(Process::new(
            ProcessName::Xmrig,
            "".to_string(),
            PathBuf::new(),
        )));

        process.lock().unwrap().state = ProcessState::Alive;
        PubXmrigApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        println!("{:#?}", process);
        assert!(process.lock().unwrap().state == ProcessState::NotMining);

        let output_parse = Arc::new(Mutex::new(String::from("[2022-02-12 12:49:30.311]  net      new job from 192.168.2.1:3333 diff 402K algo rx/0 height 2241142 (11 tx)")));
        PubXmrigApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &process);
        assert!(process.lock().unwrap().state == ProcessState::Alive);
    }

    #[test]
    fn serde_priv_p2pool_local_api() {
        let data = r#"{
				"hashrate_15m": 12,
				"hashrate_1h": 11111,
				"hashrate_24h": 468967,
				"total_hashes": 2019283840922394082390,
				"shares_found": 289037,
				"average_effort": 915.563,
				"current_effort": 129.297,
				"connections": 123,
				"incoming_connections": 96
			}"#;
        let priv_api = crate::helper::PrivP2poolLocalApi::from_str(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "hashrate_15m": 12,
  "hashrate_1h": 11111,
  "hashrate_24h": 468967,
  "shares_found": 289037,
  "average_effort": 915.563,
  "current_effort": 129.297,
  "connections": 123
}"#;
        assert_eq!(data_after_ser, json)
    }

    #[test]
    fn serde_priv_p2pool_network_api() {
        let data = r#"{
				"difficulty": 319028180924,
				"hash": "22ae1b83d727bb2ff4efc17b485bc47bc8bf5e29a7b3af65baf42213ac70a39b",
				"height": 2776576,
				"reward": 600499860000,
				"timestamp": 1670953659
			}"#;
        let priv_api = crate::helper::PrivP2poolNetworkApi::from_str(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "difficulty": 319028180924,
  "hash": "22ae1b83d727bb2ff4efc17b485bc47bc8bf5e29a7b3af65baf42213ac70a39b",
  "height": 2776576,
  "reward": 600499860000,
  "timestamp": 1670953659
}"#;
        assert_eq!(data_after_ser, json)
    }

    #[test]
    fn serde_priv_p2pool_pool_api() {
        let data = r#"{
				"pool_list": ["pplns"],
				"pool_statistics": {
					"hashRate": 10225772,
					"miners": 713,
					"totalHashes": 487463929193948,
					"lastBlockFoundTime": 1670453228,
					"lastBlockFound": 2756570,
					"totalBlocksFound": 4
				}
			}"#;
        let priv_api = crate::helper::PrivP2poolPoolApi::from_str(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "pool_statistics": {
    "hashRate": 10225772,
    "miners": 713
  }
}"#;
        assert_eq!(data_after_ser, json)
    }

    #[test]
    fn serde_priv_xmrig_api() {
        let data = r#"{
		    "id": "6226e3sd0cd1a6es",
		    "worker_id": "hinto",
		    "uptime": 123,
		    "restricted": true,
		    "resources": {
		        "memory": {
		            "free": 123,
		            "total": 123123,
		            "resident_set_memory": 123123123
		        },
		        "load_average": [10.97, 10.58, 10.47],
		        "hardware_concurrency": 12
		    },
		    "features": ["api", "asm", "http", "hwloc", "tls", "opencl", "cuda"],
		    "results": {
		        "diff_current": 123,
		        "shares_good": 123,
		        "shares_total": 123,
		        "avg_time": 123,
		        "avg_time_ms": 123,
		        "hashes_total": 123,
		        "best": [123, 123, 123, 13, 123, 123, 123, 123, 123, 123],
		        "error_log": []
		    },
		    "algo": "rx/0",
		    "connection": {
		        "pool": "localhost:3333",
		        "ip": "127.0.0.1",
		        "uptime": 123,
		        "uptime_ms": 123,
		        "ping": 0,
		        "failures": 0,
		        "tls": null,
		        "tls-fingerprint": null,
		        "algo": "rx/0",
		        "diff": 123,
		        "accepted": 123,
		        "rejected": 123,
		        "avg_time": 123,
		        "avg_time_ms": 123,
		        "hashes_total": 123,
		        "error_log": []
		    },
		    "version": "6.18.0",
		    "kind": "miner",
		    "ua": "XMRig/6.18.0 (Linux x86_64) libuv/2.0.0-dev gcc/10.2.1",
		    "cpu": {
		        "brand": "blah blah blah",
		        "family": 1,
		        "model": 2,
		        "stepping": 0,
		        "proc_info": 123,
		        "aes": true,
		        "avx2": true,
		        "x64": true,
		        "64_bit": true,
		        "l2": 123123,
		        "l3": 123123,
		        "cores": 12,
		        "threads": 24,
		        "packages": 1,
		        "nodes": 1,
		        "backend": "hwloc/2.8.0a1-git",
		        "msr": "ryzen_19h",
		        "assembly": "ryzen",
		        "arch": "x86_64",
		        "flags": ["aes", "vaes", "avx", "avx2", "bmi2", "osxsave", "pdpe1gb", "sse2", "ssse3", "sse4.1", "popcnt", "cat_l3"]
		    },
		    "donate_level": 0,
		    "paused": false,
		    "algorithms": ["cn/1", "cn/2", "cn/r", "cn/fast", "cn/half", "cn/xao", "cn/rto", "cn/rwz", "cn/zls", "cn/double", "cn/ccx", "cn-lite/1", "cn-heavy/0", "cn-heavy/tube", "cn-heavy/xhv", "cn-pico", "cn-pico/tlo", "cn/upx2", "rx/0", "rx/wow", "rx/arq", "rx/graft", "rx/sfx", "rx/keva", "argon2/chukwa", "argon2/chukwav2", "argon2/ninja", "astrobwt", "astrobwt/v2", "ghostrider"],
		    "hashrate": {
		        "total": [111.11, 111.11, 111.11],
		        "highest": 111.11,
		        "threads": [
		            [111.11, 111.11, 111.11]
		        ]
		    },
		    "hugepages": true
		}"#;
        use crate::helper::PrivXmrigApi;
        let priv_api = serde_json::from_str::<PrivXmrigApi>(data).unwrap();
        let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
        println!("{}", json);
        let data_after_ser = r#"{
  "worker_id": "hinto",
  "resources": {
    "load_average": [
      10.97,
      10.58,
      10.47
    ]
  },
  "connection": {
    "diff": 123,
    "accepted": 123,
    "rejected": 123
  },
  "hashrate": {
    "total": [
      111.11,
      111.11,
      111.11
    ]
  }
}"#;
        assert_eq!(data_after_ser, json)
    }
}
