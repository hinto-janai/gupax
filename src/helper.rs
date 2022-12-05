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
	pub pub_api_p2pool: Arc<Mutex<PubP2poolApi>>, // P2Pool API state (for GUI/Helper thread)
	pub pub_api_xmrig: Arc<Mutex<PubXmrigApi>>,   // XMRig API state (for GUI/Helper thread)
	priv_api_p2pool: Arc<Mutex<PrivP2poolApi>>,   // For "watchdog" thread
	priv_api_xmrig: Arc<Mutex<PrivXmrigApi>>,     // For "watchdog" thread
}

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The main GUI thread will use this to display console text, online state, etc.
pub struct Process {
	pub name: ProcessName,     // P2Pool or XMRig?
	pub state: ProcessState,   // The state of the process (alive, dead, etc)
	pub signal: ProcessSignal, // Did the user click [Start/Stop/Restart]?
	pub start: Instant,        // Start time of process
	pub uptime: HumanTime,     // Human readable process uptime
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
	// required for P2Pool/XMRig to open their STDIN pipe, so whether [child]
	// or [child_pty] actually has a [Some] depends on the users setting.
	// [Simple]
	child: Option<Arc<Mutex<tokio::process::Child>>>,
	stdout: Option<tokio::process::ChildStdout>, // Handle to STDOUT pipe
	stderr: Option<tokio::process::ChildStderr>, // Handle to STDERR pipe

	// [Advanced] (PTY)
	child_pty: Option<Arc<Mutex<Box<dyn portable_pty::Child + Send + std::marker::Sync>>>>, // STDOUT/STDERR is combined automatically thanks to this PTY, nice
	stdin: Option<Box<dyn portable_pty::MasterPty + Send>>, // A handle to the process's MasterPTY/STDIN

	// This is the process's private output [String], used by both [Simple] and [Advanced].
	// The "watchdog" threads mutate this, the "helper" thread synchronizes the [Pub*Api] structs
	// so that the data in here is cloned there roughly once a second. GUI thread never touches this.
	output: String,
}

//---------------------------------------------------------------------------------------------------- [Process] Impl
impl Process {
	pub fn new(name: ProcessName, args: String, path: PathBuf) -> Self {
		let now = Instant::now();
		Self {
			name,
			state: ProcessState::Dead,
			signal: ProcessSignal::None,
			start: now,
			uptime: HumanTime::into_human(now.elapsed()),
			stdout: Option::None,
			stderr: Option::None,
			stdin: Option::None,
			child: Option::None,
			child_pty: Option::None,
			// P2Pool log level 1 produces a bit less than 100,000 lines a day.
			// Assuming each line averages 80 UTF-8 scalars (80 bytes), then this
			// initial buffer should last around a week (56MB) before resetting.
			output: String::with_capacity(56_000_000),
			input: vec![String::new()],
		}
	}

	// Borrow a [&str], return an owned split collection
	pub fn parse_args(args: &str) -> Vec<String> {
		args.split_whitespace().map(|s| s.to_owned()).collect()
	}
}

//---------------------------------------------------------------------------------------------------- [Process*] Enum
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessState {
	Alive,  // Process is online, GREEN!
	Dead,   // Process is dead, BLACK!
	Failed, // Process is dead AND exited with a bad code, RED!
	Middle, // Process is in the middle of something (starting, stopping, etc), YELLOW!
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
	pub fn new(instant: std::time::Instant, pub_api_p2pool: Arc<Mutex<PubP2poolApi>>, pub_api_xmrig: Arc<Mutex<PubXmrigApi>>) -> Self {
		Self {
			instant,
			human_time: HumanTime::into_human(instant.elapsed()),
			p2pool: Arc::new(Mutex::new(Process::new(ProcessName::P2pool, String::new(), PathBuf::new()))),
			xmrig: Arc::new(Mutex::new(Process::new(ProcessName::Xmrig, String::new(), PathBuf::new()))),
			priv_api_p2pool: Arc::new(Mutex::new(PrivP2poolApi::new())),
			priv_api_xmrig: Arc::new(Mutex::new(PrivXmrigApi::new())),
			// These are created when initializing [App], since it needs a handle to it as well
			pub_api_p2pool,
			pub_api_xmrig,
		}
	}

	// The tokio runtime that blocks while async reading both STDOUT/STDERR
	// Cheaper than spawning 2 OS threads just to read 2 pipes (...right? :D)
	#[tokio::main]
	async fn async_read_stdout_stderr(process: Arc<Mutex<Process>>) {
		let process_stdout = Arc::clone(&process);
		let process_stderr = Arc::clone(&process);
		let stdout = process.lock().unwrap().child.as_ref().unwrap().lock().unwrap().stdout.take().unwrap();
		let stderr = process.lock().unwrap().child.as_ref().unwrap().lock().unwrap().stderr.take().unwrap();

		// Create STDOUT pipe job
		let stdout_job = tokio::spawn(async move {
			let mut reader = BufReader::new(stdout).lines();
			while let Ok(Some(line)) = reader.next_line().await {
				println!("{}", line); // For debugging.
				writeln!(process_stdout.lock().unwrap().output, "{}", line);
			}
		});
		// Create STDERR pipe job
		let stderr_job = tokio::spawn(async move {
			let mut reader = BufReader::new(stderr).lines();
			while let Ok(Some(line)) = reader.next_line().await {
				println!("{}", line); // For debugging.
				writeln!(process_stderr.lock().unwrap().output, "{}", line);
			}
		});
		// Block and read both until they are closed (automatic when process dies)
		// The ordering of STDOUT/STDERR should be automatic thanks to the locks.
		tokio::join![stdout_job, stderr_job];
	}

	// Reads a PTY which combines STDOUT/STDERR for me, yay
	fn read_pty(process: Arc<Mutex<Process>>, reader: Box<dyn std::io::Read + Send>) {
		use std::io::BufRead;
		let mut stdout = std::io::BufReader::new(reader).lines();
		while let Some(Ok(line)) = stdout.next() {
			println!("{}", line); // For debugging.
			writeln!(process.lock().unwrap().output, "{}", line);
		}
	}

	//---------------------------------------------------------------------------------------------------- P2Pool specific
	// Intermediate function that parses the arguments, and spawns the P2Pool watchdog thread.
	pub fn spawn_p2pool(helper: &Arc<Mutex<Self>>, state: &crate::disk::P2pool, path: std::path::PathBuf) {
		let mut args = Vec::with_capacity(500);
		let path = path.clone();
		let mut api_path = path.clone();
		api_path.pop();

		// [Simple]
		if state.simple {
			// Build the p2pool argument
			let (ip, rpc, zmq) = crate::node::enum_to_ip_rpc_zmq_tuple(state.node);         // Get: (IP, RPC, ZMQ)
			args.push("--wallet".to_string()); args.push(state.address.clone());            // Wallet address
			args.push("--host".to_string()); args.push(ip.to_string());                     // IP Address
			args.push("--rpc-port".to_string()); args.push(rpc.to_string());                // RPC Port
			args.push("--zmq-port".to_string()); args.push(zmq.to_string());                // ZMQ Port
			args.push("--data-api".to_string()); args.push(api_path.display().to_string()); // API Path
			args.push("--local-api".to_string()); // Enable API
			args.push("--no-color".to_string());  // Remove color escape sequences, Gupax terminal can't parse it :(
			args.push("--mini".to_string());      // P2Pool Mini

		// [Advanced]
		} else {
			// Overriding command arguments
			if !state.arguments.is_empty() {
				for arg in state.arguments.split_whitespace() {
					args.push(arg.to_string());
				}
			// Else, build the argument
			} else {
				args.push(state.address.clone());      // Wallet
				args.push(state.selected_ip.clone());  // IP
				args.push(state.selected_rpc.clone()); // RPC
				args.push(state.selected_zmq.clone()); // ZMQ
				args.push("--local-api".to_string());  // Enable API
				args.push("--no-color".to_string());   // Remove color escape sequences
				if state.mini { args.push("--mini".to_string()); };      // Mini
				args.push(format!("--loglevel {}", state.log_level));    // Log Level
				args.push(format!("--out-peers {}", state.out_peers));   // Out Peers
				args.push(format!("--in-peers {}", state.in_peers));     // In Peers
				args.push(format!("--data-api {}", api_path.display())); // API Path
			}
		}

		// Print arguments to console
		crate::disk::print_dash(&format!("P2Pool | Launch arguments ... {:#?}", args));

		// Spawn watchdog thread
		let simple = !state.simple; // Will this process need a PTY (STDIN)?
		let process = Arc::clone(&helper.lock().unwrap().p2pool);
		let pub_api = Arc::clone(&helper.lock().unwrap().pub_api_p2pool);
		let priv_api = Arc::clone(&helper.lock().unwrap().priv_api_p2pool);
		thread::spawn(move || {
			if simple {
				Self::spawn_simple_p2pool_watchdog(process, pub_api, priv_api, args, path);
			} else {
				Self::spawn_pty_p2pool_watchdog(process, pub_api, priv_api, args, path);
			}
		});
	}

	// The [Simple] P2Pool watchdog tokio runtime, using async features with no PTY (STDIN).
	#[tokio::main]
	async fn spawn_simple_p2pool_watchdog(process: Arc<Mutex<Process>>, pub_api: Arc<Mutex<PubP2poolApi>>, priv_api: Arc<Mutex<PrivP2poolApi>>, args: Vec<String>, path: std::path::PathBuf) {
		// 1a. Create command
		let child = Arc::new(Mutex::new(tokio::process::Command::new(path)
			.args(args)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.stdin(Stdio::piped())
			.spawn().unwrap()));

        // 2. Set process state
        let mut lock = process.lock().unwrap();
        lock.state = ProcessState::Alive;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
		lock.child = Some(Arc::clone(&child));
		drop(lock);

		// 3. Spawn STDOUT/STDERR thread
		let process_clone = Arc::clone(&process);
		thread::spawn(move || {
			Self::async_read_stdout_stderr(process_clone);
		});

		// 4. Loop forever as watchdog until process dies
		loop {
			// a. Watch user SIGNAL
			if process.lock().unwrap().signal == ProcessSignal::Stop {
				process.lock().unwrap().child.as_mut().unwrap().lock().unwrap().kill().await;
				process.lock().unwrap().signal = ProcessSignal::None;
			}
//			let signal = match process.lock().unwrap().signal {
//				ProcessSignal::Stop    => { crate::disk::print_dash("KILLING P2POOL"); process.lock().unwrap().child.as_mut().unwrap().lock().unwrap().kill().await.unwrap() },
//				ProcessSignal::Restart => process.lock().unwrap().child.as_mut().unwrap().lock().unwrap().kill().await,
//				_ => Ok(()),
//			};
			// b. Create STDIN task
			if !process.lock().unwrap().input.is_empty() { /* process it */ }
			// c. Create API task
			let async_file_read = { /* tokio async file read job */ };
			// d. Execute async tasks
//			tokio::join![signal];
			// f. Sleep (900ms)
			std::thread::sleep(MILLI_900);
		}
	}

	// The [Advanced] P2Pool watchdog. Spawns 1 OS thread for reading a PTY (STDOUT+STDERR), and combines the [Child] with a PTY so STDIN actually works.
	#[tokio::main]
	async fn spawn_pty_p2pool_watchdog(process: Arc<Mutex<Process>>, pub_api: Arc<Mutex<PubP2poolApi>>, priv_api: Arc<Mutex<PrivP2poolApi>>, args: Vec<String>, path: std::path::PathBuf) {
		// 1a. Create PTY
		let pty = portable_pty::native_pty_system();
		let mut pair = pty.openpty(portable_pty::PtySize {
			rows: 24,
			cols: 80,
			pixel_width: 0,
			pixel_height: 0,
		}).unwrap();
		// 1b. Create command
		let mut cmd = portable_pty::CommandBuilder::new(path);
		cmd.args(args);
		// 1c. Create child
		let child_pty = Arc::new(Mutex::new(pair.slave.spawn_command(cmd).unwrap()));

        // 2. Set process state
        let mut lock = process.lock().unwrap();
        lock.state = ProcessState::Alive;
        lock.signal = ProcessSignal::None;
        lock.start = Instant::now();
		lock.child_pty = Some(Arc::clone(&child_pty));
		let reader = pair.master.try_clone_reader().unwrap(); // Get STDOUT/STDERR before moving the PTY
		lock.stdin = Some(pair.master);
		drop(lock);

		// 3. Spawn PTY read thread
		let process_clone = Arc::clone(&process);
		thread::spawn(move || {
			Self::read_pty(process_clone, reader);
		});

		// 4. Loop as watchdog
		loop {
			// Set timer
			let now = Instant::now();

			// Check SIGNAL
			if process.lock().unwrap().signal == ProcessSignal::Stop {
				child_pty.lock().unwrap().kill(); // This actually sends a SIGHUP to p2pool (closes the PTY, hangs up on p2pool)
				// Wait to get the exit status
				let exit_status = match child_pty.lock().unwrap().wait() {
					Ok(e) => if e.success() { "Successful" } else { "Failed" },
					_ => "Unknown Error",
				};
				let mut lock = process.lock().unwrap();
				let uptime = lock.uptime.clone();
				info!("P2Pool | Stopped ... Uptime was: [{}], Exit status: [{}]", uptime, exit_status);
				writeln!(lock.output, "{}\nP2Pool stopped | Uptime: [{}] | Exit status: [{}]\n{}\n\n", HORI_DOUBLE, uptime, exit_status, HORI_DOUBLE);
				lock.signal = ProcessSignal::None;
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
	pub fn spawn_xmrig(state: &crate::disk::Xmrig, api_path: &std::path::Path) {
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
	// Intermediate function that spawns the helper thread.
	pub fn spawn_helper(helper: &Arc<Mutex<Self>>) {
		let helper = Arc::clone(helper);
		thread::spawn(move || { Self::helper(helper); });
	}

	// [helper] = Actual Arc
	// [h]      = Temporary lock that gets dropped
	// [jobs]   = Vector of async jobs ready to go
//	#[tokio::main]
	pub fn helper(helper: Arc<Mutex<Self>>) {
		// Begin loop
		loop {

		// 1. Create "jobs" vector holding async tasks
//		let jobs: Vec<tokio::task::JoinHandle<Result<(), anyhow::Error>>> = vec![];

		// 2. Loop init timestamp
		let start = Instant::now();

		// 7. Set Gupax/P2Pool/XMRig uptime
		let mut h = helper.lock().unwrap();
		h.human_time = HumanTime::into_human(h.instant.elapsed());
		drop(h);

		// 8. Calculate if we should sleep or not.
		// If we should sleep, how long?
		let elapsed = start.elapsed().as_millis();
		if elapsed < 1000 {
			// Casting from u128 to u64 should be safe here, because [elapsed]
			// is less than 1000, meaning it can fit into a u64 easy.
			std::thread::sleep(std::time::Duration::from_millis((1000-elapsed) as u64));
		}

		// 9. End loop
		}
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
	pub fn into_human(d: Duration) -> HumanTime {
		HumanTime(d)
	}

	fn plural(f: &mut std::fmt::Formatter, started: &mut bool, name: &str, value: u64) -> std::fmt::Result {
		if value > 0 {
			if *started { f.write_str(" ")?; }
		}
		write!(f, "{}{}", value, name)?;
		if value > 1 {
			f.write_str("s")?;
		}
		*started = true;
		Ok(())
	}
}

impl std::fmt::Display for HumanTime {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let secs = self.0.as_secs();
		if secs == 0 {
			f.write_str("0s")?;
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
		Self::plural(f, started, " year", years)?;
		Self::plural(f, started, " month", months)?;
		Self::plural(f, started, " day", days)?;
		Self::plural(f, started, " hour", hours)?;
		Self::plural(f, started, " minute", minutes)?;
		Self::plural(f, started, " second", seconds)?;
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
//         if line.contains("[0-9].[0-9]+ XMR") { n += 1; }
//     }
//
// This regex function takes [0.0003~] seconds (10x faster):
//     let regex = Regex::new("[0-9].[0-9]+ XMR").unwrap();
//     let n = regex.find_iter(P2POOL_OUTPUT).count();
//
// Both are nominally fast enough where it doesn't matter too much but meh, why not use regex.
struct P2poolRegex {
	xmr: regex::Regex,
}

impl P2poolRegex {
	fn new() -> Self {
		Self { xmr: regex::Regex::new("[0-9].[0-9]+ XMR").unwrap(), }
	}
}

//---------------------------------------------------------------------------------------------------- Public P2Pool API
// GUI thread interfaces with this.
pub struct PubP2poolApi {
	// One off
	pub mini: bool,
	// Output
	pub output: String,
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

impl PubP2poolApi {
	pub fn new() -> Self {
		Self {
			mini: true,
			output: String::with_capacity(56_000_000),
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

	// Mutate [PubP2poolApi] with data from a [PrivP2poolApi].
	fn update_from_priv(self, output: String, regex: P2poolRegex, private: PrivP2poolApi, uptime: f64) -> Self {
		// 1. Parse STDOUT
		let (payouts, xmr) = Self::calc_payouts_and_xmr(&output, &regex);
		let stdout_parse = Self {
			output: output.clone(),
			payouts,
			xmr,
			..self // <- So useful
		};
		// 2. Time calculations
		let hour_day_month = Self::update_hour_day_month(stdout_parse, uptime);
		// 3. Final priv -> pub conversion
		Self {
			hashrate_15m: HumanNumber::from_u128(private.hashrate_15m),
			hashrate_1h: HumanNumber::from_u128(private.hashrate_1h),
			hashrate_24h: HumanNumber::from_u128(private.hashrate_24h),
			shares_found: HumanNumber::from_u128(private.shares_found),
			average_effort: HumanNumber::to_percent(private.average_effort),
			current_effort: HumanNumber::to_percent(private.current_effort),
			connections: HumanNumber::from_u16(private.connections),
			..hour_day_month // <- Holy cow this is so good
		}
	}

	// Essentially greps the output for [x.xxxxxxxxxxxx XMR] where x = a number.
	// It sums each match and counts along the way, handling an error by not adding and printing to console.
	fn calc_payouts_and_xmr(output: &str, regex: &P2poolRegex) -> (u128 /* payout count */, f64 /* total xmr */) {
		let mut iter = regex.xmr.find_iter(output);
		let mut result: f64 = 0.0;
		let mut count: u128 = 0;
		for i in iter {
			if let Some(text) = i.as_str().split_whitespace().next() {
				match text.parse::<f64>() {
					Ok(num) => result += num,
					Err(e)  => error!("P2Pool | Total XMR sum calculation error: [{}]", e),
				}
				count += 1;
			}
		}
		(count, result)
	}

	// Updates the struct with hour/day/month calculations given an uptime in f64 seconds.
	fn update_hour_day_month(self, elapsed: f64) -> Self {
		// Payouts
		let per_sec = (self.payouts as f64) / elapsed;
		let payouts_hour = (per_sec * 60.0) * 60.0;
		let payouts_day = payouts_hour * 24.0;
		let payouts_month = payouts_day * 30.0;
		// Total XMR
		let per_sec = self.xmr / elapsed;
		let xmr_hour = (per_sec * 60.0) * 60.0;
		let xmr_day = payouts_hour * 24.0;
		let xmr_month = payouts_day * 30.0;
		Self {
			payouts_hour,
			payouts_day,
			payouts_month,
			xmr_hour,
			xmr_day,
			xmr_month,
			..self
		}
	}
}

//---------------------------------------------------------------------------------------------------- Private P2Pool API
// This is the data the "watchdog" threads mutate.
// It matches directly to P2Pool's [local/stats] JSON API file (excluding a few stats).
// P2Pool seems to initialize all stats at 0 (or 0.0), so no [Option] wrapper seems needed.
#[derive(Debug, Serialize, Deserialize)]
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

//---------------------------------------------------------------------------------------------------- Public XMRig API
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

impl PubXmrigApi {
	pub fn new() -> Self {
		Self {
			output: String::with_capacity(56_000_000),
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
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
