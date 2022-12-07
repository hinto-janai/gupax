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
// found here, e.g: User clicks [Start P2Pool] -> Arc<Mutex<ProcessSignal> is set
// indicating to this thread during its loop: "I should start P2Pool!", e.g:
//
//     match p2pool.lock().unwrap().signal {
//         ProcessSignal::Start => start_p2pool(),
//         ...
//     }
//
// This also includes all things related to handling the child processes (P2Pool/XMRig):
// piping their stdout/stderr/stdin, accessing their APIs (HTTP + disk files), etc.

//---------------------------------------------------------------------------------------------------- Import
use std::{
	sync::{Arc,Mutex},
	path::PathBuf,
	process::{Command,Stdio},
	fmt::Write,
	time::*,
	thread,
};
use serde::{Serialize,Deserialize};
use crate::constants::*;
use num_format::{Buffer,Locale};
use log::*;

//---------------------------------------------------------------------------------------------------- Constants
const LOCALE: num_format::Locale = num_format::Locale::en;

//---------------------------------------------------------------------------------------------------- [Helper] Struct
// A meta struct holding all the data that gets processed in this thread
pub struct Helper {
	pub instant: Instant,                         // Gupax start as an [Instant]
	pub human_time: HumanTime,                    // Gupax uptime formatting for humans
	pub p2pool: Arc<Mutex<Process>>,              // P2Pool process state
	pub xmrig: Arc<Mutex<Process>>,               // XMRig process state
	pub gui_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for GUI thread)
	pub gui_api_xmrig: Arc<Mutex<PubXmrigApi>>,   // XMRig API state (for GUI thread)
	pub img_p2pool: Arc<Mutex<ImgP2pool>>,        // A static "image" of the data P2Pool started with
	pub img_xmrig: Arc<Mutex<ImgXmrig>>,          // A static "image" of the data XMRig started with
	pub_api_p2pool: Arc<Mutex<PubP2poolApi>>,     // P2Pool API state (for Helper/P2Pool thread)
	pub_api_xmrig: Arc<Mutex<PubXmrigApi>>,       // XMRig API state (for Helper/XMRig thread)
	priv_api_p2pool: Arc<Mutex<PrivP2poolApi>>,   // For "watchdog" thread
	priv_api_xmrig: Arc<Mutex<PrivXmrigApi>>,     // For "watchdog" thread
}

// The communication between the data here and the GUI thread goes as follows:
// [GUI] <---> [Helper] <---> [Watchdog] <---> [Private Data only available here]
//
// Both [GUI] and [Helper] own their separate [Pub*Api] structs.
// Since P2Pool & XMRig will be updating their information out of sync,
// it's the helpers job to lock everything, and move the watchdog [Pub*Api]s
// on a 1-second interval into the [GUI]'s [Pub*Api] struct, atomically.

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
	// The "watchdog" threads mutate this, the "helper" thread synchronizes the [Pub*Api] structs
	// so that the data in here is cloned there roughly once a second. GUI thread never touches this.
	output: Arc<Mutex<String>>,

	// Start time of process.
	start: std::time::Instant,
}

//---------------------------------------------------------------------------------------------------- [Process] Impl
impl Process {
	pub fn new(name: ProcessName, args: String, path: PathBuf) -> Self {
		Self {
			name,
			state: ProcessState::Dead,
			signal: ProcessSignal::None,
			start: Instant::now(),
			stdin: Option::None,
			child: Option::None,
			// P2Pool log level 1 produces a bit less than 100,000 lines a day.
			// Assuming each line averages 80 UTF-8 scalars (80 bytes), then this
			// initial buffer should last around a week (56MB) before resetting.
			output: Arc::new(Mutex::new(String::with_capacity(56_000_000))),
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
impl std::fmt::Display for ProcessName   { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:#?}", self) } }

//---------------------------------------------------------------------------------------------------- [Helper]
use tokio::io::{BufReader,AsyncBufReadExt};

impl Helper {
	//---------------------------------------------------------------------------------------------------- General Functions
	pub fn new(instant: std::time::Instant, p2pool: Arc<Mutex<Process>>, xmrig: Arc<Mutex<Process>>, gui_api_p2pool: Arc<Mutex<PubP2poolApi>>, gui_api_xmrig: Arc<Mutex<PubXmrigApi>>, img_p2pool: Arc<Mutex<ImgP2pool>>, img_xmrig: Arc<Mutex<ImgXmrig>>) -> Self {
		Self {
			instant,
			human_time: HumanTime::into_human(instant.elapsed()),
			priv_api_p2pool: Arc::new(Mutex::new(PrivP2poolApi::new())),
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
	fn read_pty(output: Arc<Mutex<String>>, reader: Box<dyn std::io::Read + Send>) {
		use std::io::BufRead;
		let mut stdout = std::io::BufReader::new(reader).lines();
		while let Some(Ok(line)) = stdout.next() {
//			println!("{}", line); // For debugging.
			writeln!(output.lock().unwrap(), "{}", line);
		}
	}

	// Reset output if larger than 55_999_000 bytes (around 1 week of logs).
	// The actual [String] holds 56_000_000, but this allows for some leeway so it doesn't allocate more memory.
	// This will also append a message showing it was reset.
	fn check_reset_output(output: &Arc<Mutex<String>>, name: ProcessName) {
		let mut output = output.lock().unwrap();
		if output.len() > 55_999_000 {
			let name = match name {
				ProcessName::P2pool => "P2Pool",
				ProcessName::Xmrig  => "XMRig",
			};
			info!("{} | Output is nearing 56,000,000 bytes, resetting!", name);
			let text = format!("{}\n{} logs are exceeding the maximum: 56,000,000 bytes!\nI've reset the logs for you, your stats may now be inaccurate since they depend on these logs!\nI think you rather have that than have it hogging your memory, though!\n{}", HORI_CONSOLE, name, HORI_CONSOLE);
			output.clear();
			output.push_str(&text);
		}
	}

	//---------------------------------------------------------------------------------------------------- P2Pool specific
	// Read P2Pool's API file.
	fn read_p2pool_api(path: &std::path::PathBuf) -> Result<String, std::io::Error> {
		match std::fs::read_to_string(path) {
			Ok(s) => Ok(s),
			Err(e) => { warn!("P2Pool API | [{}] read error: {}", path.display(), e); Err(e) },
		}
	}

	// Deserialize the above [String] into a [PrivP2poolApi]
	fn str_to_priv_p2pool_api(string: &str) -> Result<PrivP2poolApi, serde_json::Error> {
		match serde_json::from_str::<PrivP2poolApi>(string) {
			Ok(a) => Ok(a),
			Err(e) => { warn!("P2Pool API | Could not deserialize API data: {}", e); Err(e) },
		}
	}

	// Just sets some signals for the watchdog thread to pick up on.
	pub fn stop_p2pool(helper: &Arc<Mutex<Self>>) {
		info!("P2Pool | Attempting stop...");
		helper.lock().unwrap().p2pool.lock().unwrap().signal = ProcessSignal::Stop;
		helper.lock().unwrap().p2pool.lock().unwrap().state = ProcessState::Middle;
	}

	// The "restart frontend" to a "frontend" function.
	// Basically calls to kill the current p2pool, waits a little, then starts the below function in a a new thread, then exit.
	pub fn restart_p2pool(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: &std::path::PathBuf) {
		info!("P2Pool | Attempting restart...");
		helper.lock().unwrap().p2pool.lock().unwrap().signal = ProcessSignal::Restart;
		helper.lock().unwrap().p2pool.lock().unwrap().state = ProcessState::Middle;

		let helper = Arc::clone(&helper);
		let state = state.clone();
		let path = path.clone();
		// This thread lives to wait, start p2pool then die.
		thread::spawn(move || {
			while helper.lock().unwrap().p2pool.lock().unwrap().is_alive() {
				warn!("P2Pool Restart | Process still alive, waiting...");
				thread::sleep(SECOND);
			}
			// Ok, process is not alive, start the new one!
			Self::start_p2pool(&helper, &state, &path);
		});
		info!("P2Pool | Restart ... OK");
	}

	// The "frontend" function that parses the arguments, and spawns either the [Simple] or [Advanced] P2Pool watchdog thread.
	pub fn start_p2pool(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: &std::path::PathBuf) {
		helper.lock().unwrap().p2pool.lock().unwrap().state = ProcessState::Middle;

		let args = Self::build_p2pool_args_and_mutate_img(helper, state, path);

		// Print arguments & user settings to console
		crate::disk::print_dash(&format!("P2Pool | Launch arguments ... {:#?}", args));

		// Spawn watchdog thread
		let process = Arc::clone(&helper.lock().unwrap().p2pool);
		let pub_api = Arc::clone(&helper.lock().unwrap().pub_api_p2pool);
		let priv_api = Arc::clone(&helper.lock().unwrap().priv_api_p2pool);
		let path = path.clone();
		thread::spawn(move || {
			Self::spawn_p2pool_watchdog(process, pub_api, priv_api, args, path);
		});
	}

	// Takes in some [State/P2pool] and parses it to build the actual command arguments.
	// Returns the [Vec] of actual arguments, and mutates the [ImgP2pool] for the main GUI thread
	// It returns a value... and mutates a deeply nested passed argument... this is some pretty bad code...
	pub fn build_p2pool_args_and_mutate_img(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: &std::path::PathBuf) -> Vec<String> {
		let mut args = Vec::with_capacity(500);
		let path = path.clone();
		let mut api_path = path.clone();
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
				mini: true,
				address: state.address.clone(),
				host: ip.to_string(),
				rpc: rpc.to_string(),
				zmq: zmq.to_string(),
				log_level: "3".to_string(),
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
				for arg in state.arguments.split_whitespace() {
					match last {
						"--mini"      => p2pool_image.mini = true,
						"--wallet"    => p2pool_image.address = arg.to_string(),
						"--host"      => p2pool_image.host = arg.to_string(),
						"--rpc-port"  => p2pool_image.rpc = arg.to_string(),
						"--zmq-port"  => p2pool_image.zmq = arg.to_string(),
						"--loglevel"  => p2pool_image.log_level = arg.to_string(),
						"--out-peers" => p2pool_image.out_peers = arg.to_string(),
						"--in-peers"  => p2pool_image.in_peers = arg.to_string(),
						_ => (),
					}
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
					mini: state.mini,
					address: state.address.clone(),
					host: state.selected_ip.to_string(),
					rpc: state.selected_rpc.to_string(),
					zmq: state.selected_zmq.to_string(),
					log_level: state.log_level.to_string(),
					out_peers: state.out_peers.to_string(),
					in_peers: state.in_peers.to_string(),
				}
			}
		}
		args
	}

	// The P2Pool watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
	#[tokio::main]
	async fn spawn_p2pool_watchdog(process: Arc<Mutex<Process>>, pub_api: Arc<Mutex<PubP2poolApi>>, priv_api: Arc<Mutex<PrivP2poolApi>>, args: Vec<String>, mut path: std::path::PathBuf) {
		// 1a. Create PTY
		let pty = portable_pty::native_pty_system();
		let pair = pty.openpty(portable_pty::PtySize {
			rows: 24,
			cols: 80,
			pixel_width: 0,
			pixel_height: 0,
		}).unwrap();
		// 1b. Create command
		let mut cmd = portable_pty::CommandBuilder::new(path.as_path());
		cmd.args(args);
		cmd.cwd(path.as_path().parent().unwrap());
		// 1c. Create child
		let child_pty = Arc::new(Mutex::new(pair.slave.spawn_command(cmd).unwrap()));

        // 2. Set process state
        let mut lock = process.lock().unwrap();
        lock.state = ProcessState::Alive;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
		lock.child = Some(Arc::clone(&child_pty));
		let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
		lock.stdin = Some(pair.master);
		drop(lock);

		// 3. Spawn PTY read thread
		let output_clone = Arc::clone(&process.lock().unwrap().output);
		thread::spawn(move || {
			Self::read_pty(output_clone, reader);
		});

		path.pop();
		path.push(P2POOL_API_PATH);
		let regex = P2poolRegex::new();
		let output = Arc::clone(&process.lock().unwrap().output);
		let start = process.lock().unwrap().start;

		// 4. Loop as watchdog
		loop {
			// Set timer
			let now = Instant::now();

			// Check SIGNAL
			if process.lock().unwrap().signal == ProcessSignal::Stop {
				child_pty.lock().unwrap().kill(); // This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
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
				info!("P2Pool | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				// This is written directly into the public API, because sometimes the 900ms event loop can't catch it.
				writeln!(pub_api.lock().unwrap().output, "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n", HORI_CONSOLE, uptime, exit_status, HORI_CONSOLE);
				process.lock().unwrap().signal = ProcessSignal::None;
				break
			// Check RESTART
			} else if process.lock().unwrap().signal == ProcessSignal::Restart {
				child_pty.lock().unwrap().kill(); // This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
				// Wait to get the exit status
				let exit_status = match child_pty.lock().unwrap().wait() {
					Ok(e) => if e.success() { "Successful" } else { "Failed" },
					_ => "Unknown Error",
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("P2Pool | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				// This is written directly into the public API, because sometimes the 900ms event loop can't catch it.
				writeln!(pub_api.lock().unwrap().output, "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n", HORI_CONSOLE, uptime, exit_status, HORI_CONSOLE);
				process.lock().unwrap().state = ProcessState::Waiting;
				break
			// Check if the process is secretly died without us knowing :)
			} else if let Ok(Some(code)) = child_pty.lock().unwrap().try_wait() {
				let exit_status = match code.success() {
					true  => { process.lock().unwrap().state = ProcessState::Dead; "Successful" },
					false => { process.lock().unwrap().state = ProcessState::Failed; "Failed" },
				};
				let uptime = HumanTime::into_human(start.elapsed());
				info!("P2Pool | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				// This is written directly into the public API, because sometimes the 900ms event loop can't catch it.
				writeln!(pub_api.lock().unwrap().output, "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n", HORI_CONSOLE, uptime, exit_status, HORI_CONSOLE);
				process.lock().unwrap().signal = ProcessSignal::None;
				break
			}

			// Check vector of user input
			let mut lock = process.lock().unwrap();
			if !lock.input.is_empty() {
				let input = std::mem::take(&mut lock.input);
				for line in input {
					writeln!(lock.stdin.as_mut().unwrap(), "{}", line);
				}
			}
			drop(lock);

			// Always update from output
			PubP2poolApi::update_from_output(&pub_api, &output, start.elapsed(), &regex);

			// Read API file into string
			if let Ok(string) = Self::read_p2pool_api(&path) {
				// Deserialize
				if let Ok(s) = Self::str_to_priv_p2pool_api(&string) {
					// Update the structs.
					PubP2poolApi::update_from_priv(&pub_api, &priv_api);
				}
			}

			// Check if logs need resetting
			Self::check_reset_output(&output, ProcessName::P2pool);

			// Sleep (only if 900ms hasn't passed)
			let elapsed = now.elapsed().as_millis();
			// Since logic goes off if less than 1000, casting should be safe
			if elapsed < 900 { std::thread::sleep(std::time::Duration::from_millis((900-elapsed) as u64)); }
		}

		// 5. If loop broke, we must be done here.
		info!("P2Pool | Watchdog thread exiting... Goodbye!");
	}

	//---------------------------------------------------------------------------------------------------- XMRig specific
	// Intermediate function that parses the arguments, and spawns the XMRig watchdog thread.
	pub fn spawn_xmrig(helper: &Arc<Mutex<Self>>, state: &crate::disk::Xmrig, path: std::path::PathBuf) {
		let mut args = Vec::with_capacity(500);
		if state.simple {
			let rig_name = if state.simple_rig.is_empty() { GUPAX_VERSION.to_string() } else { state.simple_rig.clone() }; // Rig name
			args.push(format!("--threads {}", state.current_threads)); // Threads
			args.push(format!("--user {}", state.simple_rig));         // Rig name
			args.push(format!("--url 127.0.0.1:3333"));                // Local P2Pool (the default)
			args.push("--no-color".to_string());                       // No color escape codes
			if state.pause != 0 { args.push(format!("--pause-on-active {}", state.pause)); } // Pause on active
		} else {
			if !state.arguments.is_empty() {
				for arg in state.arguments.split_whitespace() {
					args.push(arg.to_string());
				}
			} else {
				args.push(format!("--user {}", state.address.clone()));    // Wallet
				args.push(format!("--threads {}", state.current_threads)); // Threads
				args.push(format!("--rig-id {}", state.selected_rig));     // Rig ID
				args.push(format!("--url {}:{}", state.selected_ip.clone(), state.selected_port.clone())); // IP/Port
				args.push(format!("--http-host {}", state.api_ip).to_string());   // HTTP API IP
				args.push(format!("--http-port {}", state.api_port).to_string()); // HTTP API Port
				args.push("--no-color".to_string());                         // No color escape codes
				if state.tls { args.push("--tls".to_string()); }             // TLS
				if state.keepalive { args.push("--keepalive".to_string()); } // Keepalive
				if state.pause != 0 { args.push(format!("--pause-on-active {}", state.pause)); } // Pause on active
			}
		}
		// Print arguments to console
		crate::disk::print_dash(&format!("XMRig | Launch arguments ... {:#?}", args));

		// Spawn watchdog thread
		thread::spawn(move || {
			Self::spawn_xmrig_watchdog(args);
		});
	}

	// The actual XMRig watchdog tokio runtime.
	#[tokio::main]
	pub async fn spawn_xmrig_watchdog(args: Vec<String>) {
	}

	//---------------------------------------------------------------------------------------------------- The "helper"
	// The "helper" thread. Syncs data between threads here and the GUI.
	pub fn spawn_helper(helper: &Arc<Mutex<Self>>) {
		let mut helper = Arc::clone(helper);
		thread::spawn(move || {
		info!("Helper | Hello from helper thread! Entering loop where I will spend the rest of my days...");
		// Begin loop
		loop {
		// 1. Loop init timestamp
		let start = Instant::now();

		// 2. Lock... EVERYTHING!
		let mut lock = helper.lock().unwrap();
		let mut gui_api_p2pool = lock.gui_api_p2pool.lock().unwrap();
		let mut gui_api_xmrig = lock.gui_api_xmrig.lock().unwrap();
		let mut pub_api_p2pool = lock.pub_api_p2pool.lock().unwrap();
		let mut pub_api_xmrig = lock.pub_api_xmrig.lock().unwrap();
		let p2pool = lock.p2pool.lock().unwrap();
		let xmrig = lock.xmrig.lock().unwrap();
		// Calculate Gupax's uptime always.
		let human_time = HumanTime::into_human(lock.instant.elapsed());
		// If both [P2Pool/XMRig] are alive...
		if p2pool.is_alive() && xmrig.is_alive() {
			*gui_api_p2pool = std::mem::take(&mut pub_api_p2pool);
			*gui_api_xmrig = std::mem::take(&mut pub_api_xmrig);
		// If only [P2Pool] is alive...
		} else if p2pool.is_alive() {
			*gui_api_p2pool = std::mem::take(&mut pub_api_p2pool);
		// If only [XMRig] is alive...
		} else if xmrig.is_alive() {
			*gui_api_xmrig = std::mem::take(&mut pub_api_xmrig);
		}

		// 2. Drop... (almost) EVERYTHING... IN REVERSE!
		drop(xmrig);
		drop(p2pool);
		drop(pub_api_xmrig);
		drop(pub_api_p2pool);
		drop(gui_api_xmrig);
		drop(gui_api_p2pool);
		// Update the time... then drop :)
		lock.human_time = human_time;
		drop(lock);

		// 3. Calculate if we should sleep or not.
		// If we should sleep, how long?
		let elapsed = start.elapsed().as_millis();
		if elapsed < 1000 {
			// Casting from u128 to u64 should be safe here, because [elapsed]
			// is less than 1000, meaning it can fit into a u64 easy.
			std::thread::sleep(std::time::Duration::from_millis((1000-elapsed) as u64));
		}

		// 4. End loop
		}

		// 5. Something has gone terribly wrong if the helper exited this loop.
		let text = "HELPER THREAD HAS ESCAPED THE LOOP...!";
		error!("{}", text);error!("{}", text);error!("{}", text);panic!("{}", text);

		});
	}
}

//---------------------------------------------------------------------------------------------------- [HumanTime]
// This converts a [std::time::Duration] into something more readable.
// Used for uptime display purposes: [7 years, 8 months, 15 days, 23 hours, 35 minutes, 1 second]
// Code taken from [https://docs.rs/humantime/] and edited to remove sub-second time, change spacing and some words.
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HumanTime(Duration);

impl HumanTime {
	pub fn new() -> HumanTime {
		HumanTime(ZERO_SECONDS)
	}

	pub fn into_human(d: Duration) -> HumanTime {
		HumanTime(d)
	}

	fn plural(f: &mut std::fmt::Formatter, started: &mut bool, name: &str, value: u64) -> std::fmt::Result {
		if value > 0 {
			if *started {
				f.write_str(", ")?;
			}
			write!(f, "{} {}", value, name)?;
			if value > 1 {
				f.write_str("s")?;
			}
			*started = true;
		}
		Ok(())
	}
}

impl std::fmt::Display for HumanTime {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let secs = self.0.as_secs();
		if secs == 0 {
			f.write_str("0 seconds")?;
			return Ok(());
		}

		let years = secs / 31_557_600;  // 365.25d
		let ydays = secs % 31_557_600;
		let months = ydays / 2_630_016;  // 30.44d
		let mdays = ydays % 2_630_016;
		let days = mdays / 86400;
		let day_secs = mdays % 86400;
		let hours = day_secs / 3600;
		let minutes = day_secs % 3600 / 60;
		let seconds = day_secs % 60;

		let ref mut started = false;
		Self::plural(f, started, "year", years)?;
		Self::plural(f, started, "month", months)?;
		Self::plural(f, started, "day", days)?;
		Self::plural(f, started, "hour", hours)?;
		Self::plural(f, started, "minute", minutes)?;
		Self::plural(f, started, "second", seconds)?;
		Ok(())
	}
}

//---------------------------------------------------------------------------------------------------- [HumanNumber]
// Human readable numbers.
// Float    | [1234.57] -> [1,234]                    | Casts as u64/u128, adds comma
// Unsigned | [1234567] -> [1,234,567]                | Adds comma
// Percent  | [99.123] -> [99.12%]                    | Truncates to 2 after dot, adds percent
// Percent  | [0.001]  -> [0%]                        | Rounds down, removes redundant zeros
// Hashrate | [123.0, 311.2, null] -> [123, 311, ???] | Casts, replaces null with [???]
// CPU Load | [12.0, 11.4, null] -> [12.0, 11.4, ???] | No change, just into [String] form
#[derive(Debug, Clone)]
pub struct HumanNumber(String);

impl std::fmt::Display for HumanNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", &self.0)
	}
}

impl HumanNumber {
	fn unknown() -> Self {
		Self("???".to_string())
	}
	fn to_percent(f: f32) -> Self {
		if f < 0.01 {
			Self("0%".to_string())
		} else {
			Self(format!("{:.2}%", f))
		}
	}
	fn from_f32(f: f32) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&(f as u64), &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_f64(f: f64) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&(f as u128), &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_u8(u: u8) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&u, &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_u16(u: u16) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&u, &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_u32(u: u32) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&u, &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_u64(u: u64) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&u, &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_u128(u: u128) -> Self {
		let mut buf = num_format::Buffer::new();
		buf.write_formatted(&u, &LOCALE);
		Self(buf.as_str().to_string())
	}
	fn from_hashrate(array: [Option<f32>; 3]) -> Self {
		let mut string = "[".to_string();
		let mut buf = num_format::Buffer::new();

		let mut n = 0;
		for i in array {
			match i {
				Some(f) => {
					let f = f as u128;
					buf.write_formatted(&f, &LOCALE);
					string.push_str(buf.as_str());
					string.push_str(" H/s");
				},
				None => string.push_str("??? H/s"),
			}
			if n != 2 {
				string.push_str(", ");
				n += 1;
			} else {
				string.push(']');
				break
			}
		}

		Self(string)
	}
	fn from_load(array: [Option<f32>; 3]) -> Self {
		let mut string = "[".to_string();
		let mut n = 0;
		for i in array {
			match i {
				Some(f) => string.push_str(format!("{}", f).as_str()),
				None => string.push_str("???"),
			}
			if n != 2 {
				string.push_str(", ");
				n += 1;
			} else {
				string.push(']');
				break
			}
		}
		Self(string)
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
//         if line.contains("You received a payout of [0-9].[0-9]+ XMR") { n += 1; }
//     }
//
// This regex function takes [0.0003~] seconds (10x faster):
//     let regex = Regex::new("You received a payout of [0-9].[0-9]+ XMR").unwrap();
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
			payout: regex::Regex::new("You received a payout of [0-9].[0-9]+ XMR").unwrap(),
			float: regex::Regex::new("[0-9].[0-9]+").unwrap(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- [ImgP2pool]
// A static "image" of data that P2Pool started with.
// This is just a snapshot of the user data when they initially started P2Pool.
// Created by [start_p2pool()] and return to the main GUI thread where it will store it.
// No need for an [Arc<Mutex>] since the Helper thread doesn't need this information.
#[derive(Debug, Clone)]
pub struct ImgP2pool {
	pub mini: bool,        // Did the user start on the mini-chain?
	pub address: String,   // What address is the current p2pool paying out to? (This gets shortened to [4xxxxx...xxxxxx])
	pub host: String,      // What monerod are we using?
	pub rpc: String,       // What is the RPC port?
	pub zmq: String,       // What is the ZMQ port?
	pub out_peers: String, // How many out-peers?
	pub in_peers: String,  // How many in-peers?
	pub log_level: String, // What log level?
}

impl ImgP2pool {
	pub fn new() -> Self {
		Self {
			mini: true,
			address: String::new(),
			host: String::new(),
			rpc: String::new(),
			zmq: String::new(),
			out_peers: String::new(),
			in_peers: String::new(),
			log_level: String::new(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- Public P2Pool API
// Helper/GUI threads both have a copy of this, Helper updates
// the GUI's version on a 1-second interval from the private data.
#[derive(Debug, Clone)]
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
	// The rest are serialized from the API, then turned into [HumanNumber]s
	pub hashrate_15m: HumanNumber,
	pub hashrate_1h: HumanNumber,
	pub hashrate_24h: HumanNumber,
	pub shares_found: HumanNumber,
	pub average_effort: HumanNumber,
	pub current_effort: HumanNumber,
	pub connections: HumanNumber,
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
		}
	}

	// Mutate [PubP2poolApi] with data the process output.
	fn update_from_output(public: &Arc<Mutex<Self>>, output: &Arc<Mutex<String>>, elapsed: std::time::Duration, regex: &P2poolRegex) {
		// 1. Clone output
		let output = output.lock().unwrap().clone();

		// 2. Parse STDOUT
		let (payouts, xmr) = Self::calc_payouts_and_xmr(&output, &regex);

		// 3. Calculate hour/day/month given elapsed time
		let elapsed_as_secs_f64 = elapsed.as_secs_f64();
		// Payouts
		let per_sec = (payouts as f64) / elapsed_as_secs_f64;
		let payouts_hour = (per_sec * 60.0) * 60.0;
		let payouts_day = payouts_hour * 24.0;
		let payouts_month = payouts_day * 30.0;
		// Total XMR
		let per_sec = xmr / elapsed_as_secs_f64;
		let xmr_hour = (per_sec * 60.0) * 60.0;
		let xmr_day = payouts_hour * 24.0;
		let xmr_month = payouts_day * 30.0;

		// 4. Mutate the struct with the new info
		let mut public = public.lock().unwrap();
		*public = Self {
			uptime: HumanTime::into_human(elapsed),
			output,
			payouts,
			xmr,
			payouts_hour,
			payouts_day,
			payouts_month,
			xmr_hour,
			xmr_day,
			xmr_month,
			..public.clone()
		};
	}

	// Mutate [PubP2poolApi] with data from a [PrivP2poolApi] and the process output.
	fn update_from_priv(public: &Arc<Mutex<Self>>, private: &Arc<Mutex<PrivP2poolApi>>) {
		// priv -> pub conversion
		let private = private.lock().unwrap();
		let mut public = public.lock().unwrap();
		*public = Self {
			hashrate_15m: HumanNumber::from_u128(private.hashrate_15m),
			hashrate_1h: HumanNumber::from_u128(private.hashrate_1h),
			hashrate_24h: HumanNumber::from_u128(private.hashrate_24h),
			shares_found: HumanNumber::from_u128(private.shares_found),
			average_effort: HumanNumber::to_percent(private.average_effort),
			current_effort: HumanNumber::to_percent(private.current_effort),
			connections: HumanNumber::from_u16(private.connections),
			..public.clone()
		}
	}

	// Essentially greps the output for [x.xxxxxxxxxxxx XMR] where x = a number.
	// It sums each match and counts along the way, handling an error by not adding and printing to console.
	fn calc_payouts_and_xmr(output: &str, regex: &P2poolRegex) -> (u128 /* payout count */, f64 /* total xmr */) {
		let iter = regex.payout.find_iter(output);
		let mut result: f64 = 0.0;
		let mut count: u128 = 0;
		for i in iter {
			match regex.float.find(i.as_str()).unwrap().as_str().parse::<f64>() {
				Ok(num) => { result += num; count += 1; },
				Err(e)  => error!("P2Pool | Total XMR sum calculation error: [{}]", e),
			}
		}
		(count, result)
	}
}

//---------------------------------------------------------------------------------------------------- Private P2Pool API
// This is the data the "watchdog" threads mutate.
// It matches directly to P2Pool's [local/stats] JSON API file (excluding a few stats).
// P2Pool seems to initialize all stats at 0 (or 0.0), so no [Option] wrapper seems needed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct PrivP2poolApi {
	hashrate_15m: u128,
	hashrate_1h: u128,
	hashrate_24h: u128,
	shares_found: u128,
	average_effort: f32,
	current_effort: f32,
	connections: u16, // No one will have more than 65535 connections... right?
}

impl PrivP2poolApi {
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
}

//---------------------------------------------------------------------------------------------------- [ImgXmrig]
#[derive(Debug, Clone)]
pub struct ImgXmrig {}
impl ImgXmrig {
	pub fn new() -> Self {
		Self {}
	}
}

//---------------------------------------------------------------------------------------------------- Public XMRig API
#[derive(Debug, Clone)]
pub struct PubXmrigApi {
	output: String,
	worker_id: String,
	resources: HumanNumber,
	hashrate: HumanNumber,
	pool: String,
	diff: HumanNumber,
	accepted: HumanNumber,
	rejected: HumanNumber,
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
			worker_id: "???".to_string(),
			resources: HumanNumber::unknown(),
			hashrate: HumanNumber::unknown(),
			pool: "???".to_string(),
			diff: HumanNumber::unknown(),
			accepted: HumanNumber::unknown(),
			rejected: HumanNumber::unknown(),
		}
	}

	// Formats raw private data into ready-to-print human readable version.
	fn from_priv(private: PrivXmrigApi, output: String) -> Self {
		Self {
			output: output.clone(),
			worker_id: private.worker_id,
			resources: HumanNumber::from_load(private.resources.load_average),
			hashrate: HumanNumber::from_hashrate(private.hashrate.total),
			pool: private.connection.pool,
			diff: HumanNumber::from_u128(private.connection.diff),
			accepted: HumanNumber::from_u128(private.connection.accepted),
			rejected: HumanNumber::from_u128(private.connection.rejected),
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
	pool: String,
	diff: u128,
	accepted: u128,
	rejected: u128,
}
impl Connection {
	fn new() -> Self {
		Self {
			pool: String::new(),
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
