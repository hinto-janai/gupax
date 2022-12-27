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
//     if p2pool.lock().unwrap().signal == ProcessSignal::Stop {
//         stop_p2pool(),
//     }
//
// This also includes all things related to handling the child processes (P2Pool/XMRig):
// piping their stdout/stderr/stdin, accessing their APIs (HTTP + disk files), etc.

//---------------------------------------------------------------------------------------------------- Import
use std::{
	sync::{Arc,Mutex},
	path::PathBuf,
	process::Stdio,
	fmt::Write,
	time::*,
	thread,
};
use crate::{
	constants::*,
	SudoState,
	human::*,
};
use sysinfo::SystemExt;
use serde::{Serialize,Deserialize};
use sysinfo::{CpuExt,ProcessExt};
use log::*;

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
	pub instant: Instant,                         // Gupax start as an [Instant]
	pub uptime: HumanTime,                        // Gupax uptime formatting for humans
	pub pub_sys: Arc<Mutex<Sys>>,                 // The public API for [sysinfo] that the [Status] tab reads from
	pub p2pool: Arc<Mutex<Process>>,              // P2Pool process state
	pub xmrig: Arc<Mutex<Process>>,               // XMRig process state
	pub gui_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for GUI thread)
	pub gui_api_xmrig: Arc<Mutex<PubXmrigApi>>,   // XMRig API state (for GUI thread)
	pub img_p2pool: Arc<Mutex<ImgP2pool>>,        // A static "image" of the data P2Pool started with
	pub img_xmrig: Arc<Mutex<ImgXmrig>>,          // A static "image" of the data XMRig started with
	pub_api_p2pool: Arc<Mutex<PubP2poolApi>>,     // P2Pool API state (for Helper/P2Pool thread)
	pub_api_xmrig: Arc<Mutex<PubXmrigApi>>,       // XMRig API state (for Helper/XMRig thread)
	priv_api_p2pool_local: Arc<Mutex<PrivP2poolLocalApi>>,      // Serde struct(s) for P2Pool's API files
	priv_api_p2pool_network: Arc<Mutex<PrivP2poolNetworkApi>>,
	priv_api_p2pool_pool: Arc<Mutex<PrivP2poolPoolApi>>,
	priv_api_xmrig: Arc<Mutex<PrivXmrigApi>>, // Serde struct for XMRig's HTTP API
}

// The communication between the data here and the GUI thread goes as follows:
// [GUI] <---> [Helper] <---> [Watchdog] <---> [Private Data only available here]
//
// Both [GUI] and [Helper] own their separate [Pub*Api] structs.
// Since P2Pool & XMRig will be updating their information out of sync,
// it's the helpers job to lock everything, and move the watchdog [Pub*Api]s
// on a 1-second interval into the [GUI]'s [Pub*Api] struct, atomically.

//----------------------------------------------------------------------------------------------------
#[derive(Debug,Clone)]
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
impl Default for Sys { fn default() -> Self { Self::new() } }

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The main GUI thread will use this to display console text, online state, etc.
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
	child: Option<Arc<Mutex<Box<dyn portable_pty::Child + Send + std::marker::Sync>>>>, // STDOUT/STDERR is combined automatically thanks to this PTY, nice
	stdin: Option<Box<dyn portable_pty::MasterPty + Send>>, // A handle to the process's MasterPTY/STDIN

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
			stdin: Option::None,
			child: Option::None,
			output_parse: Arc::new(Mutex::new(String::with_capacity(500))),
			output_pub: Arc::new(Mutex::new(String::with_capacity(500))),
			input: vec![String::new()],
		}
	}

	// Borrow a [&str], return an owned split collection
	pub fn parse_args(args: &str) -> Vec<String> {
		args.split_whitespace().map(|s| s.to_owned()).collect()
	}

	// Convenience functions
	pub fn is_alive(&self) -> bool {
		self.state == ProcessState::Alive || self.state == ProcessState::Middle
	}

	pub fn is_waiting(&self) -> bool {
		self.state == ProcessState::Middle || self.state == ProcessState::Waiting
	}
}

//---------------------------------------------------------------------------------------------------- [Process*] Enum
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessState {
	Alive,  // Process is online, GREEN!
	Dead,   // Process is dead, BLACK!
	Failed, // Process is dead AND exited with a bad code, RED!
	Middle, // Process is in the middle of something ([re]starting/stopping), YELLOW!
	Waiting, // Process was successfully killed by a restart, and is ready to be started again, YELLOW!
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessSignal {
	None,
	Start,
	Stop,
	Restart,
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessName {
	P2pool,
	Xmrig,
}

impl std::fmt::Display for ProcessState  { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:#?}", self) } }
impl std::fmt::Display for ProcessSignal { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:#?}", self) } }
impl std::fmt::Display for ProcessName   {
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
	pub fn new(instant: std::time::Instant, pub_sys: Arc<Mutex<Sys>>, p2pool: Arc<Mutex<Process>>, xmrig: Arc<Mutex<Process>>, gui_api_p2pool: Arc<Mutex<PubP2poolApi>>, gui_api_xmrig: Arc<Mutex<PubXmrigApi>>, img_p2pool: Arc<Mutex<ImgP2pool>>, img_xmrig: Arc<Mutex<ImgXmrig>>) -> Self {
		Self {
			instant,
			pub_sys,
			uptime: HumanTime::into_human(instant.elapsed()),
			priv_api_p2pool_local: Arc::new(Mutex::new(PrivP2poolLocalApi::new())),
			priv_api_p2pool_network: Arc::new(Mutex::new(PrivP2poolNetworkApi::new())),
			priv_api_p2pool_pool: Arc::new(Mutex::new(PrivP2poolPoolApi::new())),
			priv_api_xmrig: Arc::new(Mutex::new(PrivXmrigApi::new())),
			pub_api_p2pool: Arc::new(Mutex::new(PubP2poolApi::new())),
			pub_api_xmrig: Arc::new(Mutex::new(PubXmrigApi::new())),
			// These are created when initializing [App], since it needs a handle to it as well
			p2pool,
			xmrig,
			gui_api_p2pool,
			gui_api_xmrig,
			img_p2pool,
			img_xmrig,
		}
	}

	// Reads a PTY which combines STDOUT/STDERR for me, yay
	fn read_pty(output_parse: Arc<Mutex<String>>, output_pub: Arc<Mutex<String>>, reader: Box<dyn std::io::Read + Send>, name: ProcessName) {
		use std::io::BufRead;
		let mut stdout = std::io::BufReader::new(reader).lines();
		// We don't need to write twice for XMRig, since we dont parse it... yet.
		if name == ProcessName::Xmrig {
			while let Some(Ok(line)) = stdout.next() {
//				println!("{}", line); // For debugging.
//				if let Err(e) = writeln!(output_parse.lock().unwrap(), "{}", line) { error!("PTY | Output error: {}", e); }
				if let Err(e) = writeln!(output_pub.lock().unwrap(), "{}", line) { error!("PTY | Output error: {}", e); }
			}
		} else {
			while let Some(Ok(line)) = stdout.next() {
//				println!("{}", line); // For debugging.
				if let Err(e) = writeln!(output_parse.lock().unwrap(), "{}", line) { error!("PTY | Output error: {}", e); }
				if let Err(e) = writeln!(output_pub.lock().unwrap(), "{}", line) { error!("PTY | Output error: {}", e); }
			}
		}
	}

	// Reset output if larger than max bytes.
	// This will also append a message showing it was reset.
	fn check_reset_gui_output(output: &mut String, name: ProcessName) {
		let len = output.len();
		if len > GUI_OUTPUT_LEEWAY {
			info!("{} Watchdog | Output is nearing {} bytes, resetting!", name, MAX_GUI_OUTPUT_BYTES);
			let text = format!("{}\n{} GUI log is exceeding the maximum: {} bytes!\nI've reset the logs for you!\n{}\n\n\n\n", HORI_CONSOLE, name, MAX_GUI_OUTPUT_BYTES, HORI_CONSOLE);
			output.clear();
			output.push_str(&text);
			debug!("{} Watchdog | Resetting GUI output ... OK", name);
		} else {
			debug!("{} Watchdog | GUI output reset not needed! Current byte length ... {}", name, len);
		}
	}

	// Read P2Pool/XMRig's API file to a [String].
	fn path_to_string(path: &std::path::PathBuf, name: ProcessName) -> std::result::Result<String, std::io::Error> {
		match std::fs::read_to_string(path) {
			Ok(s) => Ok(s),
			Err(e) => { warn!("{} API | [{}] read error: {}", name, path.display(), e); Err(e) },
		}
	}

	//---------------------------------------------------------------------------------------------------- P2Pool specific
	// Just sets some signals for the watchdog thread to pick up on.
	pub fn stop_p2pool(helper: &Arc<Mutex<Self>>) {
		info!("P2Pool | Attempting to stop...");
		helper.lock().unwrap().p2pool.lock().unwrap().signal = ProcessSignal::Stop;
		helper.lock().unwrap().p2pool.lock().unwrap().state = ProcessState::Middle;
	}

	// The "restart frontend" to a "frontend" function.
	// Basically calls to kill the current p2pool, waits a little, then starts the below function in a a new thread, then exit.
	pub fn restart_p2pool(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: &std::path::PathBuf) {
		info!("P2Pool | Attempting to restart...");
		helper.lock().unwrap().p2pool.lock().unwrap().signal = ProcessSignal::Restart;
		helper.lock().unwrap().p2pool.lock().unwrap().state = ProcessState::Middle;

		let helper = Arc::clone(helper);
		let state = state.clone();
		let path = path.clone();
		// This thread lives to wait, start p2pool then die.
		thread::spawn(move || {
			while helper.lock().unwrap().p2pool.lock().unwrap().is_alive() {
				warn!("P2Pool | Want to restart but process is still alive, waiting...");
				thread::sleep(SECOND);
			}
			// Ok, process is not alive, start the new one!
			info!("P2Pool | Old process seems dead, starting new one!");
			Self::start_p2pool(&helper, &state, &path);
		});
		info!("P2Pool | Restart ... OK");
	}

	// The "frontend" function that parses the arguments, and spawns either the [Simple] or [Advanced] P2Pool watchdog thread.
	pub fn start_p2pool(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: &std::path::PathBuf) {
		helper.lock().unwrap().p2pool.lock().unwrap().state = ProcessState::Middle;

		let (args, api_path_local, api_path_network, api_path_pool) = Self::build_p2pool_args_and_mutate_img(helper, state, path);

		// Print arguments & user settings to console
		crate::disk::print_dash(&format!(
			"P2Pool | Launch arguments: {:#?} | Local API Path: {:#?} | Network API Path: {:#?} | Pool API Path: {:#?}",
			 args,
			 api_path_local,
			 api_path_network,
			 api_path_pool,
		));

		// Spawn watchdog thread
		let process = Arc::clone(&helper.lock().unwrap().p2pool);
		let gui_api = Arc::clone(&helper.lock().unwrap().gui_api_p2pool);
		let pub_api = Arc::clone(&helper.lock().unwrap().pub_api_p2pool);
		let path = path.clone();
		thread::spawn(move || {
			Self::spawn_p2pool_watchdog(process, gui_api, pub_api, args, path, api_path_local, api_path_network, api_path_pool);
		});
	}

	// Takes in a 95-char Monero address, returns the first and last
	// 6 characters separated with dots like so: [4abcde...abcdef]
	fn head_tail_of_monero_address(address: &str) -> String {
		if address.len() < 95 { return "???".to_string() }
		let head = &address[0..5];
		let tail = &address[89..95];
		head.to_owned() + "..." + tail
	}

	// Takes in some [State/P2pool] and parses it to build the actual command arguments.
	// Returns the [Vec] of actual arguments, and mutates the [ImgP2pool] for the main GUI thread
	// It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
	pub fn build_p2pool_args_and_mutate_img(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: &std::path::PathBuf) -> (Vec<String>, PathBuf, PathBuf, PathBuf) {
		let mut args = Vec::with_capacity(500);
		let path = path.clone();
		let mut api_path = path;
		api_path.pop();

		// [Simple]
		if state.simple {
			// Build the p2pool argument
			let (ip, rpc, zmq) = crate::node::enum_to_ip_rpc_zmq_tuple(state.node);         // Get: (IP, RPC, ZMQ)
			args.push("--wallet".to_string());   args.push(state.address.clone());          // Wallet address
			args.push("--host".to_string());     args.push(ip.to_string());                 // IP Address
			args.push("--rpc-port".to_string()); args.push(rpc.to_string());                // RPC Port
			args.push("--zmq-port".to_string()); args.push(zmq.to_string());                // ZMQ Port
			args.push("--data-api".to_string()); args.push(api_path.display().to_string()); // API Path
			args.push("--local-api".to_string()); // Enable API
			args.push("--no-color".to_string());  // Remove color escape sequences, Gupax terminal can't parse it :(
			args.push("--mini".to_string());      // P2Pool Mini
			*helper.lock().unwrap().img_p2pool.lock().unwrap() = ImgP2pool {
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
				// This parses the input and attemps to fill out
				// the [ImgP2pool]... This is pretty bad code...
				let mut last = "";
				let lock = helper.lock().unwrap();
				let mut p2pool_image = lock.img_p2pool.lock().unwrap();
				let mut mini = false;
				for arg in state.arguments.split_whitespace() {
					match last {
						"--mini"      => { mini = true; p2pool_image.mini = "P2Pool Mini".to_string(); },
						"--wallet"    => p2pool_image.address = Self::head_tail_of_monero_address(arg),
						"--host"      => p2pool_image.host = arg.to_string(),
						"--rpc-port"  => p2pool_image.rpc = arg.to_string(),
						"--zmq-port"  => p2pool_image.zmq = arg.to_string(),
						"--out-peers" => p2pool_image.out_peers = arg.to_string(),
						"--in-peers"  => p2pool_image.in_peers = arg.to_string(),
						"--data-api"  => api_path = PathBuf::from(arg),
						_ => (),
					}
					if !mini { p2pool_image.mini = "P2Pool Main".to_string(); }
					args.push(arg.to_string());
					last = arg;
				}
			// Else, build the argument
			} else {
				args.push("--wallet".to_string());    args.push(state.address.clone());          // Wallet
				args.push("--host".to_string());      args.push(state.selected_ip.to_string());  // IP
				args.push("--rpc-port".to_string());  args.push(state.selected_rpc.to_string()); // RPC
				args.push("--zmq-port".to_string());  args.push(state.selected_zmq.to_string()); // ZMQ
				args.push("--loglevel".to_string());  args.push(state.log_level.to_string());    // Log Level
				args.push("--out-peers".to_string()); args.push(state.out_peers.to_string());    // Out Peers
				args.push("--in-peers".to_string());  args.push(state.in_peers.to_string());     // In Peers
				args.push("--data-api".to_string());  args.push(api_path.display().to_string()); // API Path
				args.push("--local-api".to_string());               // Enable API
				args.push("--no-color".to_string());                // Remove color escape sequences
				if state.mini { args.push("--mini".to_string()); }; // Mini
				*helper.lock().unwrap().img_p2pool.lock().unwrap() = ImgP2pool {
					mini: if state.mini { "P2Pool Mini".to_string() } else { "P2Pool Main".to_string() },
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

	// The P2Pool watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
	fn spawn_p2pool_watchdog(process: Arc<Mutex<Process>>, gui_api: Arc<Mutex<PubP2poolApi>>, pub_api: Arc<Mutex<PubP2poolApi>>, args: Vec<String>, path: std::path::PathBuf, api_path_local: std::path::PathBuf, api_path_network: std::path::PathBuf, api_path_pool: std::path::PathBuf) {
		// 1a. Create PTY
		debug!("P2Pool | Creating PTY...");
		let pty = portable_pty::native_pty_system();
		let pair = pty.openpty(portable_pty::PtySize {
			rows: 100,
			cols: 1000,
			pixel_width: 0,
			pixel_height: 0,
		}).unwrap();
		// 1b. Create command
		debug!("P2Pool | Creating command...");
		let mut cmd = portable_pty::CommandBuilder::new(path.as_path());
		cmd.args(args);
		cmd.cwd(path.as_path().parent().unwrap());
		// 1c. Create child
		debug!("P2Pool | Creating child...");
		let child_pty = Arc::new(Mutex::new(pair.slave.spawn_command(cmd).unwrap()));

        // 2. Set process state
		debug!("P2Pool | Setting process state...");
        let mut lock = process.lock().unwrap();
        lock.state = ProcessState::Alive;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
		lock.child = Some(Arc::clone(&child_pty));
		let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
		lock.stdin = Some(pair.master);
		drop(lock);

		// 3. Spawn PTY read thread
		debug!("P2Pool | Spawning PTY read thread...");
		let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
		let output_pub = Arc::clone(&process.lock().unwrap().output_pub);
		thread::spawn(move || {
			Self::read_pty(output_parse, output_pub, reader, ProcessName::P2pool);
		});
		let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
		let output_pub = Arc::clone(&process.lock().unwrap().output_pub);

		debug!("P2Pool | Cleaning old [local] API files...");
		// Attempt to remove stale API file
		match std::fs::remove_file(&api_path_local) {
			Ok(_) => info!("P2Pool | Attempting to remove stale API file ... OK"),
			Err(e) => warn!("P2Pool | Attempting to remove stale API file ... FAIL ... {}", e),
		}
		// Attempt to create a default empty one.
		use std::io::Write;
		if std::fs::File::create(&api_path_local).is_ok() {
			let text = r#"{"hashrate_15m":0,"hashrate_1h":0,"hashrate_24h":0,"shares_found":0,"average_effort":0.0,"current_effort":0.0,"connections":0}"#;
			match std::fs::write(&api_path_local, text) {
				Ok(_) => info!("P2Pool | Creating default empty API file ... OK"),
				Err(e) => warn!("P2Pool | Creating default empty API file ... FAIL ... {}", e),
			}
		}
		let regex = P2poolRegex::new();
		let start = process.lock().unwrap().start;

		// Reset stats before loop
		*pub_api.lock().unwrap() = PubP2poolApi::new();
		*gui_api.lock().unwrap() = PubP2poolApi::new();

		// 4. Loop as watchdog
		info!("P2Pool | Entering watchdog mode... woof!");
		let mut tick = 0;
		loop {
			// Set timer
			let now = Instant::now();
			debug!("P2Pool Watchdog | ----------- Start of loop -----------");
			tick += 1;

			// Check if the process is secretly died without us knowing :)
			if let Ok(Some(code)) = child_pty.lock().unwrap().try_wait() {
				debug!("P2Pool Watchdog | Process secretly died! Getting exit status");
				let exit_status = match code.success() {
					true  => { process.lock().unwrap().state = ProcessState::Dead; "Successful" },
					false => { process.lock().unwrap().state = ProcessState::Failed; "Failed" },
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("P2Pool Watchdog | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				// This is written directly into the GUI, because sometimes the 900ms event loop can't catch it.
				if let Err(e) = writeln!(
					gui_api.lock().unwrap().output,
					"{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
					HORI_CONSOLE,
					uptime,
					exit_status,
					HORI_CONSOLE
				) {
					error!("P2Pool Watchdog | GUI Uptime/Exit status write failed: {}", e);
				}
				process.lock().unwrap().signal = ProcessSignal::None;
				debug!("P2Pool Watchdog | Secret dead process reap OK, breaking");
				break
			}

			// Check SIGNAL
			if process.lock().unwrap().signal == ProcessSignal::Stop {
				debug!("P2Pool Watchdog | Stop SIGNAL caught");
				// This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
				if let Err(e) = child_pty.lock().unwrap().kill() { error!("P2Pool Watchdog | Kill error: {}", e); }
				// Wait to get the exit status
				let exit_status = match child_pty.lock().unwrap().wait() {
					Ok(e) => {
						if e.success() {
							process.lock().unwrap().state = ProcessState::Dead; "Successful"
						} else {
							process.lock().unwrap().state = ProcessState::Failed; "Failed"
						}
					},
					_ => { process.lock().unwrap().state = ProcessState::Failed; "Unknown Error" },
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("P2Pool Watchdog | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				// This is written directly into the GUI API, because sometimes the 900ms event loop can't catch it.
				if let Err(e) = writeln!(
					gui_api.lock().unwrap().output,
					"{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
					HORI_CONSOLE,
					uptime,
					exit_status,
					HORI_CONSOLE
				) {
					error!("P2Pool Watchdog | GUI Uptime/Exit status write failed: {}", e);
				}
				process.lock().unwrap().signal = ProcessSignal::None;
				debug!("P2Pool Watchdog | Stop SIGNAL done, breaking");
				break
			// Check RESTART
			} else if process.lock().unwrap().signal == ProcessSignal::Restart {
				debug!("P2Pool Watchdog | Restart SIGNAL caught");
				// This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
				if let Err(e) = child_pty.lock().unwrap().kill() { error!("P2Pool Watchdog | Kill error: {}", e); }
				// Wait to get the exit status
				let exit_status = match child_pty.lock().unwrap().wait() {
					Ok(e) => if e.success() { "Successful" } else { "Failed" },
					_ => "Unknown Error",
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("P2Pool Watchdog | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				// This is written directly into the GUI API, because sometimes the 900ms event loop can't catch it.
				if let Err(e) = writeln!(
					gui_api.lock().unwrap().output,
					"{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
					HORI_CONSOLE,
					uptime,
					exit_status,
					HORI_CONSOLE
				) {
					error!("P2Pool Watchdog | GUI Uptime/Exit status write failed: {}", e);
				}
				process.lock().unwrap().state = ProcessState::Waiting;
				debug!("P2Pool Watchdog | Restart SIGNAL done, breaking");
				break
			}

			// Check vector of user input
			let mut lock = process.lock().unwrap();
			if !lock.input.is_empty() {
				let input = std::mem::take(&mut lock.input);
				for line in input {
					debug!("P2Pool Watchdog | User input not empty, writing to STDIN: [{}]", line);
					if let Err(e) = writeln!(lock.stdin.as_mut().unwrap(), "{}", line) { error!("P2Pool Watchdog | STDIN error: {}", e); }
				}
			}
			drop(lock);


			// Check if logs need resetting
			debug!("P2Pool Watchdog | Attempting GUI log reset check");
			let mut lock = gui_api.lock().unwrap();
			Self::check_reset_gui_output(&mut lock.output, ProcessName::P2pool);
			drop(lock);

			// Always update from output
			debug!("P2Pool Watchdog | Starting [update_from_output()]");
			PubP2poolApi::update_from_output(&pub_api, &output_parse, &output_pub, start.elapsed(), &regex);

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
			if tick >= 60 {
				debug!("P2Pool Watchdog | Attempting [network] & [pool] API file read");
				if let (Ok(network_api), Ok(pool_api)) = (Self::path_to_string(&api_path_network, ProcessName::P2pool), Self::path_to_string(&api_path_pool, ProcessName::P2pool)) {
					if let (Ok(network_api), Ok(pool_api)) = (PrivP2poolNetworkApi::from_str(&network_api), PrivP2poolPoolApi::from_str(&pool_api)) {
						PubP2poolApi::update_from_network_pool(&pub_api, network_api, pool_api);
						tick = 0;
					}
				}
			}

			// Sleep (only if 900ms hasn't passed)
			let elapsed = now.elapsed().as_millis();
			// Since logic goes off if less than 1000, casting should be safe
			if elapsed < 900 {
				let sleep = (900-elapsed) as u64;
				debug!("P2Pool Watchdog | END OF LOOP -  Tick: [{}/60] - Sleeping for [{}]ms...", tick, sleep);
				std::thread::sleep(std::time::Duration::from_millis(sleep));
			} else {
				debug!("P2Pool Watchdog | END OF LOOP - Tick: [{}/60] Not sleeping!", tick);
			}
		}

		// 5. If loop broke, we must be done here.
		info!("P2Pool Watchdog | Watchdog thread exiting... Goodbye!");
	}

	//---------------------------------------------------------------------------------------------------- XMRig specific, most functions are very similar to P2Pool's
	// If processes are started with [sudo] on macOS, they must also
	// be killed with [sudo] (even if I have a direct handle to it as the
	// parent process...!). This is only needed on macOS, not Linux.
	fn sudo_kill(pid: u32, sudo: &Arc<Mutex<SudoState>>) -> bool {
		// Spawn [sudo] to execute [kill] on the given [pid]
		let mut child = std::process::Command::new("sudo")
			.args(["--stdin", "kill", "-9", &pid.to_string()])
			.stdin(Stdio::piped())
			.spawn().unwrap();

		// Write the [sudo] password to STDIN.
		let mut stdin = child.stdin.take().unwrap();
		use std::io::Write;
		if let Err(e) = writeln!(stdin, "{}\n", sudo.lock().unwrap().pass) { error!("Sudo Kill | STDIN error: {}", e); }

		// Return exit code of [sudo/kill].
		child.wait().unwrap().success()
	}

	// Just sets some signals for the watchdog thread to pick up on.
	pub fn stop_xmrig(helper: &Arc<Mutex<Self>>) {
		info!("XMRig | Attempting to stop...");
		helper.lock().unwrap().xmrig.lock().unwrap().signal = ProcessSignal::Stop;
		helper.lock().unwrap().xmrig.lock().unwrap().state = ProcessState::Middle;
	}

	// The "restart frontend" to a "frontend" function.
	// Basically calls to kill the current xmrig, waits a little, then starts the below function in a a new thread, then exit.
	pub fn restart_xmrig(helper: &Arc<Mutex<Self>>, state: &crate::disk::Xmrig, path: &std::path::PathBuf, sudo: Arc<Mutex<SudoState>>) {
		info!("XMRig | Attempting to restart...");
		helper.lock().unwrap().xmrig.lock().unwrap().signal = ProcessSignal::Restart;
		helper.lock().unwrap().xmrig.lock().unwrap().state = ProcessState::Middle;

		let helper = Arc::clone(helper);
		let state = state.clone();
		let path = path.clone();
		// This thread lives to wait, start xmrig then die.
		thread::spawn(move || {
			while helper.lock().unwrap().xmrig.lock().unwrap().state != ProcessState::Waiting {
				warn!("XMRig | Want to restart but process is still alive, waiting...");
				thread::sleep(SECOND);
			}
			// Ok, process is not alive, start the new one!
			info!("XMRig | Old process seems dead, starting new one!");
			Self::start_xmrig(&helper, &state, &path, sudo);
		});
		info!("XMRig | Restart ... OK");
	}

	pub fn start_xmrig(helper: &Arc<Mutex<Self>>, state: &crate::disk::Xmrig, path: &std::path::PathBuf, sudo: Arc<Mutex<SudoState>>) {
		helper.lock().unwrap().xmrig.lock().unwrap().state = ProcessState::Middle;

		let (args, api_ip_port) = Self::build_xmrig_args_and_mutate_img(helper, state, path);

		// Print arguments & user settings to console
		crate::disk::print_dash(&format!("XMRig | Launch arguments: {:#?}", args));
		info!("XMRig | Using path: [{}]", path.display());

		// Spawn watchdog thread
		let process = Arc::clone(&helper.lock().unwrap().xmrig);
		let gui_api = Arc::clone(&helper.lock().unwrap().gui_api_xmrig);
		let pub_api = Arc::clone(&helper.lock().unwrap().pub_api_xmrig);
		let priv_api = Arc::clone(&helper.lock().unwrap().priv_api_xmrig);
		let path = path.clone();
		thread::spawn(move || {
			Self::spawn_xmrig_watchdog(process, gui_api, pub_api, priv_api, args, path, sudo, api_ip_port);
		});
	}

	// Takes in some [State/Xmrig] and parses it to build the actual command arguments.
	// Returns the [Vec] of actual arguments, and mutates the [ImgXmrig] for the main GUI thread
	// It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
	pub fn build_xmrig_args_and_mutate_img(helper: &Arc<Mutex<Self>>, state: &crate::disk::Xmrig, path: &std::path::PathBuf) -> (Vec<String>, String) {
		let mut args = Vec::with_capacity(500);
		let mut api_ip = String::with_capacity(15);
		let mut api_port = String::with_capacity(5);
		let path = path.clone();
		// The actual binary we're executing is [sudo], technically
		// the XMRig path is just an argument to sudo, so add it.
		// Before that though, add the ["--prompt"] flag and set it
		// to emptyness so that it doesn't show up in the output.
		if cfg!(unix) {
			args.push(r#"--prompt="#.to_string());
			args.push("--".to_string());
			args.push(path.display().to_string());
		}

		// [Simple]
		if state.simple {
			// Build the xmrig argument
			let rig = if state.simple_rig.is_empty() { GUPAX_VERSION_UNDERSCORE.to_string() } else { state.simple_rig.clone() }; // Rig name
			args.push("--url".to_string()); args.push("127.0.0.1:3333".to_string());          // Local P2Pool (the default)
			args.push("--threads".to_string()); args.push(state.current_threads.to_string()); // Threads
			args.push("--user".to_string()); args.push(rig);                                  // Rig name
			args.push("--no-color".to_string());                                              // No color
			args.push("--http-host".to_string()); args.push("127.0.0.1".to_string());         // HTTP API IP
			args.push("--http-port".to_string()); args.push("18088".to_string());             // HTTP API Port
			if state.pause != 0 { args.push("--pause-on-active".to_string()); args.push(state.pause.to_string()); } // Pause on active
			*helper.lock().unwrap().img_xmrig.lock().unwrap() = ImgXmrig {
				threads: state.current_threads.to_string(),
				url: "127.0.0.1:3333 (Local P2Pool)".to_string(),
			};
			api_ip = "127.0.0.1".to_string();
			api_port = "18088".to_string();

		// [Advanced]
		} else {
			// Overriding command arguments
			if !state.arguments.is_empty() {
				// This parses the input and attemps to fill out
				// the [ImgXmrig]... This is pretty bad code...
				let mut last = "";
				let lock = helper.lock().unwrap();
				let mut xmrig_image = lock.img_xmrig.lock().unwrap();
				for arg in state.arguments.split_whitespace() {
					match last {
						"--threads"   => xmrig_image.threads = arg.to_string(),
						"--url"       => xmrig_image.url = arg.to_string(),
						"--http-host" => api_ip = if arg == "localhost" { "127.0.0.1".to_string() } else { arg.to_string() },
						"--http-port" => api_port = arg.to_string(),
						_ => (),
					}
					args.push(if arg == "localhost" { "127.0.0.1".to_string() } else { arg.to_string() });
					last = arg;
				}
			// Else, build the argument
			} else {
				// XMRig doesn't understand [localhost]
				api_ip = if state.api_ip == "localhost" || state.api_ip.is_empty() { "127.0.0.1".to_string() } else { state.api_ip.to_string() };
				api_port = if state.api_port.is_empty() { "18088".to_string() } else { state.api_port.to_string() };
				let url = format!("{}:{}", state.selected_ip, state.selected_port); // Combine IP:Port into one string
				args.push("--user".to_string()); args.push(state.address.clone());                // Wallet
				args.push("--threads".to_string()); args.push(state.current_threads.to_string()); // Threads
				args.push("--rig-id".to_string()); args.push(state.selected_rig.to_string());     // Rig ID
				args.push("--url".to_string()); args.push(url.clone());                           // IP/Port
				args.push("--http-host".to_string()); args.push(api_ip.to_string());              // HTTP API IP
				args.push("--http-port".to_string()); args.push(api_port.to_string());            // HTTP API Port
				args.push("--no-color".to_string());                         // No color escape codes
				if state.tls { args.push("--tls".to_string()); }             // TLS
				if state.keepalive { args.push("--keepalive".to_string()); } // Keepalive
				if state.pause != 0 { args.push("--pause-on-active".to_string()); args.push(state.pause.to_string()); } // Pause on active
				*helper.lock().unwrap().img_xmrig.lock().unwrap() = ImgXmrig {
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

	// The XMRig watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
	// This isn't actually async, a tokio runtime is unfortunately needed because [Hyper] is an async library (HTTP API calls)
	#[tokio::main]
	async fn spawn_xmrig_watchdog(process: Arc<Mutex<Process>>, gui_api: Arc<Mutex<PubXmrigApi>>, pub_api: Arc<Mutex<PubXmrigApi>>, _priv_api: Arc<Mutex<PrivXmrigApi>>, args: Vec<String>, path: std::path::PathBuf, sudo: Arc<Mutex<SudoState>>, mut api_ip_port: String) {
		// 1a. Create PTY
		debug!("XMRig | Creating PTY...");
		let pty = portable_pty::native_pty_system();
		let mut pair = pty.openpty(portable_pty::PtySize {
			rows: 100,
			cols: 1000,
			pixel_width: 0,
			pixel_height: 0,
		}).unwrap();
		// 1b. Create command
		debug!("XMRig | Creating command...");
		#[cfg(target_os = "windows")]
		let cmd = Self::create_xmrig_cmd_windows(args, path);
		#[cfg(target_family = "unix")]
		let cmd = Self::create_xmrig_cmd_unix(args, path);
		// 1c. Create child
		debug!("XMRig | Creating child...");
		let child_pty = Arc::new(Mutex::new(pair.slave.spawn_command(cmd).unwrap()));

		// 2. Input [sudo] pass, wipe, then drop.
		if cfg!(unix) {
			debug!("XMRig | Inputting [sudo] and wiping...");
			// 1d. Sleep to wait for [sudo]'s non-echo prompt (on Unix).
			// this prevents users pass from showing up in the STDOUT.
			std::thread::sleep(std::time::Duration::from_secs(3));
			if let Err(e) = writeln!(pair.master, "{}", sudo.lock().unwrap().pass) { error!("XMRig | Sudo STDIN error: {}", e); };
			SudoState::wipe(&sudo);
		}

        // 3. Set process state
		debug!("XMRig | Setting process state...");
        let mut lock = process.lock().unwrap();
        lock.state = ProcessState::Alive;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
		lock.child = Some(Arc::clone(&child_pty));
		let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
		lock.stdin = Some(pair.master);
		drop(lock);

		// 4. Spawn PTY read thread
		debug!("XMRig | Spawning PTY read thread...");
		let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
		let output_pub = Arc::clone(&process.lock().unwrap().output_pub);
		thread::spawn(move || {
			Self::read_pty(output_parse, output_pub, reader, ProcessName::Xmrig);
		});
		// We don't parse anything in XMRigs output... yet.
//		let output_parse = Arc::clone(&process.lock().unwrap().output_parse);
		let output_pub = Arc::clone(&process.lock().unwrap().output_pub);

		let client: hyper::Client<hyper::client::HttpConnector> = hyper::Client::builder().build(hyper::client::HttpConnector::new());
		let start = process.lock().unwrap().start;
		let api_uri = {
			if !api_ip_port.ends_with('/') { api_ip_port.push('/'); }
			"http://".to_owned() + &api_ip_port + XMRIG_API_URI
		};
		info!("XMRig | Final API URI: {}", api_uri);

		// Reset stats before loop
		*pub_api.lock().unwrap() = PubXmrigApi::new();
		*gui_api.lock().unwrap() = PubXmrigApi::new();

		// 5. Loop as watchdog
		info!("XMRig | Entering watchdog mode... woof!");
		loop {
			// Set timer
			let now = Instant::now();
			debug!("XMRig Watchdog | ----------- Start of loop -----------");

			// Check if the process secretly died without us knowing :)
			if let Ok(Some(code)) = child_pty.lock().unwrap().try_wait() {
				debug!("XMRig Watchdog | Process secretly died on us! Getting exit status...");
				let exit_status = match code.success() {
					true  => { process.lock().unwrap().state = ProcessState::Dead; "Successful" },
					false => { process.lock().unwrap().state = ProcessState::Failed; "Failed" },
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("XMRig | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				if let Err(e) = writeln!(
					gui_api.lock().unwrap().output,
					"{}\nXMRig stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
					HORI_CONSOLE,
					uptime,
					exit_status,
					HORI_CONSOLE
				) {
					error!("XMRig Watchdog | GUI Uptime/Exit status write failed: {}", e);
				}
				process.lock().unwrap().signal = ProcessSignal::None;
				debug!("XMRig Watchdog | Secret dead process reap OK, breaking");
				break
			}

			// Stop on [Stop/Restart] SIGNAL
			let signal = process.lock().unwrap().signal;
			if signal == ProcessSignal::Stop || signal == ProcessSignal::Restart  {
				debug!("XMRig Watchdog | Stop/Restart SIGNAL caught");
				// macOS requires [sudo] again to kill [XMRig]
				if cfg!(target_os = "macos") {
					// If we're at this point, that means the user has
					// entered their [sudo] pass again, after we wiped it.
					// So, we should be able to find it in our [Arc<Mutex<SudoState>>].
					Self::sudo_kill(child_pty.lock().unwrap().process_id().unwrap(), &sudo);
					// And... wipe it again (only if we're stopping full).
					// If we're restarting, the next start will wipe it for us.
					if signal != ProcessSignal::Restart { SudoState::wipe(&sudo); }
				} else if let Err(e) = child_pty.lock().unwrap().kill() {
					error!("XMRig Watchdog | Kill error: {}", e);
				}
				let exit_status = match child_pty.lock().unwrap().wait() {
					Ok(e) => {
						let mut process = process.lock().unwrap();
						if e.success() {
							if process.signal == ProcessSignal::Stop { process.state = ProcessState::Dead; }
							"Successful"
						} else {
							if process.signal == ProcessSignal::Stop { process.state = ProcessState::Failed; }
							"Failed"
						}
					},
					_ => {
						let mut process = process.lock().unwrap();
						if process.signal == ProcessSignal::Stop { process.state = ProcessState::Failed; }
						"Unknown Error"
					},
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("XMRig | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				if let Err(e) = writeln!(
					gui_api.lock().unwrap().output,
					"{}\nXMRig stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n\n\n",
					HORI_CONSOLE,
					uptime,
					exit_status,
					HORI_CONSOLE
				) {
					error!("XMRig Watchdog | GUI Uptime/Exit status write failed: {}", e);
				}
				let mut process = process.lock().unwrap();
				match process.signal {
					ProcessSignal::Stop    => process.signal = ProcessSignal::None,
					ProcessSignal::Restart => process.state = ProcessState::Waiting,
					_ => (),
				}
				debug!("XMRig Watchdog | Stop/Restart SIGNAL done, breaking");
				break
			}

			// Check vector of user input
			let mut lock = process.lock().unwrap();
			if !lock.input.is_empty() {
				let input = std::mem::take(&mut lock.input);
				for line in input {
					debug!("XMRig Watchdog | User input not empty, writing to STDIN: [{}]", line);
					if let Err(e) = writeln!(lock.stdin.as_mut().unwrap(), "{}", line) { error!("XMRig Watchdog | STDIN error: {}", e); };
				}
			}
			drop(lock);

			// Check if logs need resetting
			debug!("XMRig Watchdog | Attempting GUI log reset check");
			let mut lock = gui_api.lock().unwrap();
			Self::check_reset_gui_output(&mut lock.output, ProcessName::Xmrig);
			drop(lock);

			// Always update from output
			debug!("XMRig Watchdog | Starting [update_from_output()]");
			PubXmrigApi::update_from_output(&pub_api, &output_pub, start.elapsed());

			// Send an HTTP API request
			debug!("XMRig Watchdog | Attempting HTTP API request...");
			if let Ok(priv_api) = PrivXmrigApi::request_xmrig_api(client.clone(), &api_uri).await {
				debug!("XMRig Watchdog | HTTP API request OK, attempting [update_from_priv()]");
				PubXmrigApi::update_from_priv(&pub_api, priv_api);
			} else {
				warn!("XMRig Watchdog | Could not send HTTP API request to: {}", api_uri);
			}

			// Sleep (only if 900ms hasn't passed)
			let elapsed = now.elapsed().as_millis();
			// Since logic goes off if less than 1000, casting should be safe
			if elapsed < 900 {
				let sleep = (900-elapsed) as u64;
				debug!("XMRig Watchdog | END OF LOOP - Sleeping for [{}]ms...", sleep);
				std::thread::sleep(std::time::Duration::from_millis(sleep));
			} else {
				debug!("XMRig Watchdog | END OF LOOP - Not sleeping!");
			}
		}

		// 5. If loop broke, we must be done here.
		info!("XMRig Watchdog | Watchdog thread exiting... Goodbye!");
	}

	//---------------------------------------------------------------------------------------------------- The "helper"
	fn update_pub_sys_from_sysinfo(sysinfo: &sysinfo::System, pub_sys: &mut Sys, pid: &sysinfo::Pid, helper: &Helper, max_threads: usize) {
		let gupax_uptime = helper.uptime.to_string();
		let cpu = &sysinfo.cpus()[0];
		let gupax_cpu_usage = format!("{:.2}%", sysinfo.process(*pid).unwrap().cpu_usage()/(max_threads as f32));
		let gupax_memory_used_mb = HumanNumber::from_u64(sysinfo.process(*pid).unwrap().memory()/1_000_000);
		let gupax_memory_used_mb = format!("{} megabytes", gupax_memory_used_mb);
		let system_cpu_model = format!("{} ({}MHz)", cpu.brand(), cpu.frequency());
		let system_memory = {
			let used = (sysinfo.used_memory() as f64)/1_000_000_000.0;
			let total = (sysinfo.total_memory() as f64)/1_000_000_000.0;
			format!("{:.3} GB / {:.3} GB", used, total)
		};
		let system_cpu_usage = {
			let mut total: f32 = 0.0;
			for cpu in sysinfo.cpus() {
				total += cpu.cpu_usage();
			}
			format!("{:.2}%", total/(max_threads as f32))
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

	// The "helper" thread. Syncs data between threads here and the GUI.
	pub fn spawn_helper(helper: &Arc<Mutex<Self>>, mut sysinfo: sysinfo::System, pid: sysinfo::Pid, max_threads: usize) {
		// The ordering of these locks is _very_ important. They MUST be in sync with how the main GUI thread locks stuff
		// or a deadlock will occur given enough time. They will eventually both want to lock the [Arc<Mutex>] the other
		// thread is already locking. Yes, I figured this out the hard way, hence the vast amount of debug!() messages.
		// Example of different order (BAD!):
		//
		// GUI Main       -> locks [p2pool] first
		// Helper         -> locks [gui_api_p2pool] first
		// GUI Status Tab -> trys to lock [gui_api_p2pool] -> CAN'T
		// Helper         -> trys to lock [p2pool] -> CAN'T
		//
		// These two threads are now in a deadlock because both
		// are trying to access locks the other one already has.
		//
		// The locking order here must be in the same chronological
		// order as the main GUI thread (top to bottom).

		let helper = Arc::clone(helper);
		let lock = helper.lock().unwrap();
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
		let mut lock = helper.lock().unwrap();                                debug!("Helper | Locking (1/8) ... [helper]");
		let p2pool = p2pool.lock().unwrap();                                  debug!("Helper | Locking (2/8) ... [p2pool]");
		let xmrig = xmrig.lock().unwrap();                                    debug!("Helper | Locking (3/8) ... [xmrig]");
		let mut lock_pub_sys = pub_sys.lock().unwrap();                       debug!("Helper | Locking (4/8) ... [pub_sys]");
		let mut gui_api_p2pool = gui_api_p2pool.lock().unwrap();              debug!("Helper | Locking (5/8) ... [gui_api_p2pool]");
		let mut gui_api_xmrig = gui_api_xmrig.lock().unwrap();                debug!("Helper | Locking (6/8) ... [gui_api_xmrig]");
		let mut pub_api_p2pool = pub_api_p2pool.lock().unwrap();              debug!("Helper | Locking (7/8) ... [pub_api_p2pool]");
		let mut pub_api_xmrig = pub_api_xmrig.lock().unwrap();                debug!("Helper | Locking (8/8) ... [pub_api_xmrig]");
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
		sysinfo.refresh_cpu_specifics(sysinfo_cpu);                debug!("Helper | Sysinfo refresh (1/3) ... [cpu]");
		sysinfo.refresh_processes_specifics(sysinfo_processes);    debug!("Helper | Sysinfo refresh (2/3) ... [processes]");
		sysinfo.refresh_memory();                                  debug!("Helper | Sysinfo refresh (3/3) ... [memory]");
		debug!("Helper | Sysinfo OK, running [update_pub_sys_from_sysinfo()]");
		Self::update_pub_sys_from_sysinfo(&sysinfo, &mut lock_pub_sys, &pid, &lock, max_threads);

		// 3. Drop... (almost) EVERYTHING... IN REVERSE!
		drop(lock_pub_sys);     debug!("Helper | Unlocking (1/8) ... [pub_sys]");
		drop(xmrig);            debug!("Helper | Unlocking (2/8) ... [xmrig]");
		drop(p2pool);           debug!("Helper | Unlocking (3/8) ... [p2pool]");
		drop(pub_api_xmrig);    debug!("Helper | Unlocking (4/8) ... [pub_api_xmrig]");
		drop(pub_api_p2pool);   debug!("Helper | Unlocking (5/8) ... [pub_api_p2pool]");
		drop(gui_api_xmrig);    debug!("Helper | Unlocking (6/8) ... [gui_api_xmrig]");
		drop(gui_api_p2pool);   debug!("Helper | Unlocking (7/8) ... [gui_api_p2pool]");
		drop(lock);             debug!("Helper | Unlocking (8/8) ... [helper]");

		// 4. Calculate if we should sleep or not.
		// If we should sleep, how long?
		let elapsed = start.elapsed().as_millis();
		if elapsed < 1000 {
			// Casting from u128 to u64 should be safe here, because [elapsed]
			// is less than 1000, meaning it can fit into a u64 easy.
			let sleep = (1000-elapsed) as u64;
			debug!("Helper | END OF LOOP - Sleeping for [{}]ms...", sleep);
			std::thread::sleep(std::time::Duration::from_millis(sleep));
		} else {
			debug!("Helper | END OF LOOP - Not sleeping!");
		}

		// 5. End loop
		}
		});
	}
}

//---------------------------------------------------------------------------------------------------- Regexes
// Not to be confused with the [Regexes] struct in [main.rs], this one is meant
// for parsing the output of P2Pool and finding payouts and total XMR found.
// Why Regex instead of the standard library?
//    1. I'm already using Regex
//    2. It's insanely faster
//
// The following STDLIB implementation takes [0.003~] seconds to find all matches given a [String] with 30k lines:
//     let mut n = 0;
//     for line in P2POOL_OUTPUT.lines() {
//         if line.contains("payout of [0-9].[0-9]+ XMR") { n += 1; }
//     }
//
// This regex function takes [0.0003~] seconds (10x faster):
//     let regex = Regex::new("payout of [0-9].[0-9]+ XMR").unwrap();
//     let n = regex.find_iter(P2POOL_OUTPUT).count();
//
// Both are nominally fast enough where it doesn't matter too much but meh, why not use regex.
struct P2poolRegex {
	payout: regex::Regex,
	float: regex::Regex,
}

impl P2poolRegex {
	fn new() -> Self {
		Self {
			payout: regex::Regex::new("payout of [0-9].[0-9]+ XMR").unwrap(),
			float: regex::Regex::new("[0-9].[0-9]+").unwrap(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- XMR AtomicUnit
#[derive(Debug, Clone)]
struct AtomicUnit(u128);

impl AtomicUnit {
	fn new() -> Self {
		Self(0)
	}

	fn sum_vec(vec: &Vec<Self>) -> Self {
		let mut sum = 0;
		for int in vec {
			sum += int.0;
		}
		Self(sum)
	}

	fn to_f64(&self) -> f64 {
		self.0 as f64 / 1_000_000_000_000.0
	}

	fn to_human_number_12_point(&self) -> HumanNumber {
		let f = self.0 as f64 / 1_000_000_000_000.0;
		HumanNumber::from_f64_12_point(f)
	}

	fn to_human_number_no_fmt(&self) -> HumanNumber {
		let f = self.0 as f64 / 1_000_000_000_000.0;
		HumanNumber::from_f64_no_fmt(f)
	}
}

// Displays AtomicUnit as a real XMR floating point.
impl std::fmt::Display for AtomicUnit {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", Self::to_human_number_12_point(self))
	}
}

//---------------------------------------------------------------------------------------------------- [PayoutOrd]
// This is the struct for ordering P2Pool payout lines into a structured and ordered vector of elements.
// The structure goes as follows:
//
// Vec<(String, AtomicUnit, u64)>
// "2022-08-17 12:16:11.8662" | 0.002382256231 XMR | Block 2573821
// [0] = DATE
// [1] = XMR IN ATOMIC-UNITS
// [2] = MONERO BLOCK
#[derive(Debug, Clone)]
pub struct PayoutOrd(Vec<(String, AtomicUnit, HumanNumber)>);

impl PayoutOrd {
	fn new() -> Self {
		Self(vec![(String::from("????-??-?? ??:??:??.????"), AtomicUnit::new(), HumanNumber::unknown())])
	}

	// Takes the raw components (no wrapper types), convert them and pushes to existing [Self]
	fn push(&mut self, date: String, atomic_unit: u128, block: u64) {
		let atomic_unit = AtomicUnit(atomic_unit);
		let block = HumanNumber::from_u64(block);
		self.0.push((date, atomic_unit, block));
	}

	// Sort [Self] from highest payout to lowest
	fn sort_payout_high_to_low(&mut self) {
		// This is a little confusing because wrapper types are basically 1 element tuples so:
		// self.0 = The [Vec] within [PayoutOrd]
		// b.1.0  = [b] is [(String, AtomicUnit, HumanNumber)], [.1] is the [AtomicUnit] inside it, [.0] is the [u128] inside that
		// a.1.0  = Same deal, but we compare it with the previous value (b)
		self.0.sort_by(|a, b| b.1.0.cmp(&a.1.0));
	}

	fn sort_payout_low_to_high(&mut self) {
		self.0.sort_by(|a, b| a.1.0.cmp(&b.1.0));
	}

	// Recent <-> Oldest relies on the line order.
	// The raw log lines will be shown instead of this struct.
}

impl Default for PayoutOrd { fn default() -> Self { Self::new() } }

impl std::fmt::Display for PayoutOrd {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		for i in &self.0 {
			writeln!(f, "{} | {} XMR | Block {}", i.0, i.1, i.2)?;
		}
		Ok(())
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
	pub address: String,   // What address is the current p2pool paying out to? (This gets shortened to [4xxxxx...xxxxxx])
	pub host: String,      // What monerod are we using?
	pub rpc: String,       // What is the RPC port?
	pub zmq: String,       // What is the ZMQ port?
	pub out_peers: String, // How many out-peers?
	pub in_peers: String,  // How many in-peers?
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
#[derive(Debug,Clone,PartialEq)]
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
	// The API below needs a raw int [hashrate] to go off of and
	// there's not a good way to access it without doing weird
	// [Arc<Mutex>] shenanigans, so the raw [hashrate_1h] is
	// copied here instead.
	pub hashrate: u64,
	// Network API
	pub monero_difficulty: HumanNumber, // e.g: [15,000,000]
	pub monero_hashrate: HumanNumber,   // e.g: [1.000 GH/s]
	pub hash: String,
	pub height: HumanNumber,
	pub reward: u64, // Atomic units
	// Pool API
	pub p2pool_difficulty: HumanNumber,
	pub p2pool_hashrate: HumanNumber,
	pub miners: HumanNumber, // Current amount of miners on P2Pool sidechain
	// Mean (calcualted in functions, not serialized)
	pub solo_block_mean: HumanTime,   // Time it would take the user to find a solo block
	pub p2pool_block_mean: HumanTime, // Time it takes the P2Pool sidechain to find a block
	pub p2pool_share_mean: HumanTime, // Time it would take the user to find a P2Pool share
	// Percent
	pub p2pool_percent: HumanNumber,      // Percentage of P2Pool hashrate capture of overall Monero hashrate.
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
			hashrate: 0,
			monero_difficulty: HumanNumber::unknown(),
			monero_hashrate: HumanNumber::unknown(),
			hash: String::from("???"),
			height: HumanNumber::unknown(),
			reward: 0,
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

	// The issue with just doing [gui_api = pub_api] is that values get overwritten.
	// This doesn't matter for any of the values EXCEPT for the output, so we must
	// manually append it instead of overwriting.
	// This is used in the "helper" thread.
	fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
		let mut output = std::mem::take(&mut gui_api.output);
		let buf = std::mem::take(&mut pub_api.output);
		if !buf.is_empty() { output.push_str(&buf); }
		*gui_api = Self {
			output,
			..pub_api.clone()
		};
	}

	// Essentially greps the output for [x.xxxxxxxxxxxx XMR] where x = a number.
	// It sums each match and counts along the way, handling an error by not adding and printing to console.
	fn calc_payouts_and_xmr(output: &str, regex: &P2poolRegex) -> (u128 /* payout count */, f64 /* total xmr */) {
		let iter = regex.payout.find_iter(output);
		let mut sum: f64 = 0.0;
		let mut count: u128 = 0;
		for i in iter {
			match regex.float.find(i.as_str()).unwrap().as_str().parse::<f64>() {
				Ok(num) => { sum += num; count += 1; },
				Err(e)  => error!("P2Pool | Total XMR sum calculation error: [{}]", e),
			}
		}
		(count, sum)
	}

	// Mutate "watchdog"'s [PubP2poolApi] with data the process output.
	fn update_from_output(public: &Arc<Mutex<Self>>, output_parse: &Arc<Mutex<String>>, output_pub: &Arc<Mutex<String>>, elapsed: std::time::Duration, regex: &P2poolRegex) {
		// 1. Take the process's current output buffer and combine it with Pub (if not empty)
		let mut output_pub = output_pub.lock().unwrap();
		if !output_pub.is_empty() {
			public.lock().unwrap().output.push_str(&std::mem::take(&mut *output_pub));
		}

		// 2. Parse the full STDOUT
		let mut output_parse = output_parse.lock().unwrap();
		let (payouts_new, xmr_new) = Self::calc_payouts_and_xmr(&output_parse, regex);
		// 3. Throw away [output_parse]
		output_parse.clear();
		drop(output_parse);
		// 4. Add to current values
		let mut public = public.lock().unwrap();
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
			debug!("P2Pool Watchdog | New [Payout] found in output ... {}", payouts_new);
			debug!("P2Pool Watchdog | Total [Payout] should be ... {}", payouts);
			debug!("P2Pool Watchdog | Correct [Payout per] should be ... [{}/hour, {}/day, {}/month]", payouts_hour, payouts_day, payouts_month);
		}
		if xmr_new != 0.0 {
			debug!("P2Pool Watchdog | New [XMR mined] found in output ... {}", xmr_new);
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
		let mut public = public.lock().unwrap();
		*public = Self {
			hashrate_15m: HumanNumber::from_u64(local.hashrate_15m),
			hashrate_1h: HumanNumber::from_u64(local.hashrate_1h),
			hashrate_24h: HumanNumber::from_u64(local.hashrate_24h),
			shares_found: HumanNumber::from_u64(local.shares_found),
			average_effort: HumanNumber::to_percent(local.average_effort),
			current_effort: HumanNumber::to_percent(local.current_effort),
			connections: HumanNumber::from_u16(local.connections),
			hashrate: local.hashrate_1h,
			..std::mem::take(&mut *public)
		};
	}

	// Mutate [PubP2poolApi] with data from a [PrivP2pool(Network|Pool)Api].
	fn update_from_network_pool(public: &Arc<Mutex<Self>>, net: PrivP2poolNetworkApi, pool: PrivP2poolPoolApi) {
		let user_hashrate = public.lock().unwrap().hashrate; // The user's total P2Pool hashrate
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
			p2pool_block_mean = HumanTime::into_human(std::time::Duration::from_secs(monero_difficulty / p2pool_hashrate));
			let f = (user_hashrate as f32 / p2pool_hashrate as f32) * 100.0;
			user_p2pool_percent = HumanNumber::to_percent_no_fmt(f);
		};
		let p2pool_percent;
		let user_monero_percent;
		if monero_hashrate == 0 {
			p2pool_percent = HumanNumber::unknown();
			user_monero_percent = HumanNumber::unknown();
		} else {
			let f = (p2pool_hashrate as f32 / monero_hashrate as f32) * 100.0;
			p2pool_percent = HumanNumber::to_percent_no_fmt(f);
			let f = (user_hashrate as f32 / monero_hashrate as f32) * 100.0;
			user_monero_percent = HumanNumber::to_percent_no_fmt(f);
		};
		let solo_block_mean;
		let p2pool_share_mean;
		if user_hashrate == 0 {
			solo_block_mean = HumanTime::new();
			p2pool_share_mean = HumanTime::new();
		} else {
			solo_block_mean = HumanTime::into_human(std::time::Duration::from_secs(monero_difficulty / user_hashrate));
			p2pool_share_mean = HumanTime::into_human(std::time::Duration::from_secs(p2pool_difficulty / user_hashrate));
		}
		let mut public = public.lock().unwrap();
		*public = Self {
			monero_difficulty: HumanNumber::from_u64(monero_difficulty),
			monero_hashrate: HumanNumber::from_u64_to_gigahash_3_point(monero_hashrate),
			hash: net.hash,
			height: HumanNumber::from_u32(net.height),
			reward: net.reward,
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
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Local" Api
// This matches directly to P2Pool's [local/stats] JSON API file (excluding a few stats).
// P2Pool seems to initialize all stats at 0 (or 0.0), so no [Option] wrapper seems needed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PrivP2poolLocalApi {
	hashrate_15m: u64,
	hashrate_1h: u64,
	hashrate_24h: u64,
	shares_found: u64,
	average_effort: f32,
	current_effort: f32,
	connections: u16, // No one will have more than 65535 connections... right?
}

impl Default for PrivP2poolLocalApi { fn default() -> Self { Self::new() } }

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
			Err(e) => { warn!("P2Pool Local API | Could not deserialize API data: {}", e); Err(e) },
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

impl Default for PrivP2poolNetworkApi { fn default() -> Self { Self::new() } }

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
			Err(e) => { warn!("P2Pool Network API | Could not deserialize API data: {}", e); Err(e) },
		}
	}
}

//---------------------------------------------------------------------------------------------------- Private P2Pool "Pool" API
// This matches P2Pool's [pool/stats] JSON API file.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PrivP2poolPoolApi {
	pool_statistics: PoolStatistics,
}

impl Default for PrivP2poolPoolApi { fn default() -> Self { Self::new() } }

impl PrivP2poolPoolApi {
	fn new() -> Self {
		Self {
			pool_statistics: PoolStatistics::new(),
		}
	}

	fn from_str(string: &str) -> std::result::Result<Self, serde_json::Error> {
		match serde_json::from_str::<Self>(string) {
			Ok(a) => Ok(a),
			Err(e) => { warn!("P2Pool Pool API | Could not deserialize API data: {}", e); Err(e) },
		}
	}
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PoolStatistics {
	hashRate: u64,
	miners: u32,
}
impl Default for PoolStatistics { fn default() -> Self { Self::new() } }
impl PoolStatistics { fn new() -> Self { Self { hashRate: 0, miners: 0 } } }

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
		}
	}

	fn combine_gui_pub_api(gui_api: &mut Self, pub_api: &mut Self) {
		let output = std::mem::take(&mut gui_api.output);
		let buf = std::mem::take(&mut pub_api.output);
		*gui_api = Self {
			output,
			..std::mem::take(pub_api)
		};
		if !buf.is_empty() { gui_api.output.push_str(&buf); }
	}

	// This combines the buffer from the PTY thread [output_pub]
	// with the actual [PubApiXmrig] output field.
	fn update_from_output(public: &Arc<Mutex<Self>>, output_pub: &Arc<Mutex<String>>, elapsed: std::time::Duration) {
		// 1. Take process output buffer if not empty
		let mut output_pub = output_pub.lock().unwrap();
		let mut public = public.lock().unwrap();
		// 2. Append
		if !output_pub.is_empty() {
			public.output.push_str(&std::mem::take(&mut *output_pub));
		}
		// 3. Update uptime
		public.uptime = HumanTime::into_human(elapsed);
	}

	// Formats raw private data into ready-to-print human readable version.
	fn update_from_priv(public: &Arc<Mutex<Self>>, private: PrivXmrigApi) {
		let mut public = public.lock().unwrap();
		*public = Self {
			worker_id: private.worker_id,
			resources: HumanNumber::from_load(private.resources.load_average),
			hashrate: HumanNumber::from_hashrate(private.hashrate.total),
			diff: HumanNumber::from_u128(private.connection.diff),
			accepted: HumanNumber::from_u128(private.connection.accepted),
			rejected: HumanNumber::from_u128(private.connection.rejected),
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
	// Send an HTTP request to XMRig's API, serialize it into [Self] and return it
	async fn request_xmrig_api(client: hyper::Client<hyper::client::HttpConnector>, api_uri: &str) -> std::result::Result<Self, anyhow::Error> {
		let request = hyper::Request::builder()
			.method("GET")
			.uri(api_uri)
			.body(hyper::Body::empty())?;
		let response = tokio::time::timeout(std::time::Duration::from_millis(500), client.request(request)).await?;
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

//---------------------------------------------------------------------------------------------------- PubMoneroApi
#[derive(Debug,Clone)]
struct PubMoneroApi {
	size: HumanNumber, // Blockchain size in GB
	diff: HumanNumber, // Current difficulty
	height: HumanNumber, // Current height
	incoming: HumanNumber, // In-peers
	outgoing: HumanNumber, // Out-peers
	restricted: bool, // Is RPC in restricted mode?
	synchronized: bool, // Are we synced?
	tx_pool_size: HumanNumber, // Current amout of TX in TX pool
}

//---------------------------------------------------------------------------------------------------- PrivMoneroApi
// This matches some stats from monerod's JSON-RPC HTTP call [get_info]
// It _seems_ monerod initializes stats with [0], so no [Option], hopefully nothing panics :D
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PrivMoneroApi {
	result: Result,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Result {
	database_size: u128, // bytes
	difficulty: u128,
	height: u64,
	incoming_connections_count: u32,
	nettype: String, // mainnet, stagenet, testnet
	outgoing_connections_count: u32,
	restricted: bool,
	status: String, // OK
	synchronized: bool,
	tx_pool_size: u32, // tx pool
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
	#[test]
	fn sort_payout_ord() {
		use crate::helper::PayoutOrd;
		use crate::helper::AtomicUnit;
		use crate::helper::HumanNumber;
		let mut payout_ord = PayoutOrd(vec![
			("2022-09-08 18:42:55.4636".to_string(), AtomicUnit(1000000000), HumanNumber::from_u64(2654321)),
			("2022-09-09 16:18:26.7582".to_string(), AtomicUnit(2000000000), HumanNumber::from_u64(2654322)),
			("2022-09-10 11:15:21.1272".to_string(), AtomicUnit(3000000000), HumanNumber::from_u64(2654323)),
		]);
		println!("OG: {:#?}", payout_ord);

		// High to Low
		PayoutOrd::sort_payout_high_to_low(&mut payout_ord);
		println!("AFTER PAYOUT HIGH TO LOW: {:#?}", payout_ord);
		let should_be =
r#"2022-09-10 11:15:21.1272 | 0.003000000000 XMR | Block 2,654,323
2022-09-09 16:18:26.7582 | 0.002000000000 XMR | Block 2,654,322
2022-09-08 18:42:55.4636 | 0.001000000000 XMR | Block 2,654,321
"#;
		println!("SHOULD_BE:\n{}", should_be);
		println!("IS:\n{}", payout_ord);
		assert_eq!(payout_ord.to_string(), should_be);

		// Low to High
		PayoutOrd::sort_payout_low_to_high(&mut payout_ord);
		println!("AFTER PAYOUT LOW TO HIGH: {:#?}", payout_ord);
		let should_be =
r#"2022-09-08 18:42:55.4636 | 0.001000000000 XMR | Block 2,654,321
2022-09-09 16:18:26.7582 | 0.002000000000 XMR | Block 2,654,322
2022-09-10 11:15:21.1272 | 0.003000000000 XMR | Block 2,654,323
"#;
		println!("SHOULD_BE:\n{}", should_be);
		println!("IS:\n{}", payout_ord);
		assert_eq!(payout_ord.to_string(), should_be);
	}

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
	fn build_p2pool_regex() {
		crate::helper::P2poolRegex::new();
	}

	#[test]
	fn calc_payouts_and_xmr_from_output_p2pool() {
		use crate::helper::{PubP2poolApi,P2poolRegex};
		use std::sync::{Arc,Mutex};
		let public = Arc::new(Mutex::new(PubP2poolApi::new()));
		let output_parse = Arc::new(Mutex::new(String::from(
			r#"payout of 5.000000000001 XMR in block 1111
			payout of 5.000000000001 XMR in block 1112
			payout of 5.000000000001 XMR in block 1113"#
		)));
		let output_pub = Arc::new(Mutex::new(String::new()));
		let elapsed = std::time::Duration::from_secs(60);
		let regex = P2poolRegex::new();
		PubP2poolApi::update_from_output(&public, &output_parse, &output_pub, elapsed, &regex);
		let public = public.lock().unwrap();
		println!("{:#?}", public);
		assert_eq!(public.payouts,       3);
		assert_eq!(public.payouts_hour,  180.0);
		assert_eq!(public.payouts_day,   4320.0);
		assert_eq!(public.payouts_month, 129600.0);
		assert_eq!(public.xmr,           15.000000000003);
		assert_eq!(public.xmr_hour,      900.00000000018);
		assert_eq!(public.xmr_day,       21600.00000000432);
		assert_eq!(public.xmr_month,     648000.0000001296);
	}

	#[test]
	fn update_pub_p2pool_from_local_network_pool() {
		use std::sync::{Arc,Mutex};
		use crate::helper::PubP2poolApi;
		use crate::helper::PrivP2poolLocalApi;
		use crate::helper::PrivP2poolNetworkApi;
		use crate::helper::PrivP2poolPoolApi;
		use crate::helper::PoolStatistics;
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
			}
		};
		// Update Local
		PubP2poolApi::update_from_local(&public, local);
		let p = public.lock().unwrap();
		println!("AFTER LOCAL: {:#?}", p);
		assert_eq!(p.hashrate_15m.to_string(),   "10,000");
		assert_eq!(p.hashrate_1h.to_string(),    "20,000");
		assert_eq!(p.hashrate_24h.to_string(),   "30,000");
		assert_eq!(p.shares_found.to_string(),   "1,000");
		assert_eq!(p.average_effort.to_string(), "100.00%");
		assert_eq!(p.current_effort.to_string(), "200.00%");
		assert_eq!(p.connections.to_string(),    "1,234");
		assert_eq!(p.hashrate,                   20000);
		drop(p);
		// Update Network + Pool
		PubP2poolApi::update_from_network_pool(&public, network, pool);
		let p = public.lock().unwrap();
		println!("AFTER NETWORK+POOL: {:#?}", p);
		assert_eq!(p.monero_difficulty.to_string(),   "300,000,000,000");
		assert_eq!(p.monero_hashrate.to_string(),     "2.500 GH/s");
		assert_eq!(p.hash.to_string(),                "asdf");
		assert_eq!(p.height.to_string(),              "1,234");
		assert_eq!(p.reward,                          2345);
		assert_eq!(p.p2pool_difficulty.to_string(),   "10,000,000");
		assert_eq!(p.p2pool_hashrate.to_string(),     "1.000 MH/s");
		assert_eq!(p.miners.to_string(),              "1,000");
		assert_eq!(p.solo_block_mean.to_string(),     "5 months, 21 days, 9 hours, 52 minutes");
		assert_eq!(p.p2pool_block_mean.to_string(),   "3 days, 11 hours, 20 minutes");
		assert_eq!(p.p2pool_share_mean.to_string(),   "8 minutes, 20 seconds");
		assert_eq!(p.p2pool_percent.to_string(),      "0.04%");
		assert_eq!(p.user_p2pool_percent.to_string(), "2%");
		assert_eq!(p.user_monero_percent.to_string(), "0.0008%");
		drop(p);
	}

	#[test]
	fn serde_priv_p2pool_local_api() {
		let data =
			r#"{
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
		let data_after_ser =
r#"{
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
		let data =
			r#"{
				"difficulty": 319028180924,
				"hash": "22ae1b83d727bb2ff4efc17b485bc47bc8bf5e29a7b3af65baf42213ac70a39b",
				"height": 2776576,
				"reward": 600499860000,
				"timestamp": 1670953659
			}"#;
		let priv_api = crate::helper::PrivP2poolNetworkApi::from_str(data).unwrap();
		let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
		println!("{}", json);
		let data_after_ser =
r#"{
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
		let data =
			r#"{
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
		let data_after_ser =
r#"{
  "pool_statistics": {
    "hashRate": 10225772,
    "miners": 713
  }
}"#;
		assert_eq!(data_after_ser, json)
	}

	#[test]
	fn serde_priv_xmrig_api() {
		let data =
		r#"{
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
		let priv_api = serde_json::from_str::<PrivXmrigApi>(&data).unwrap();
		let json = serde_json::ser::to_string_pretty(&priv_api).unwrap();
		println!("{}", json);
		let data_after_ser =
r#"{
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
