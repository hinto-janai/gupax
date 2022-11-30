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
	process::Command,
	time::*,
	thread,
};
use crate::constants::*;
use log::*;

//---------------------------------------------------------------------------------------------------- [Helper] Struct
// A meta struct holding all the data that gets processed in this thread
pub struct Helper {
	uptime: HumanTime,     // Gupax uptime formatting for humans
	p2pool: Process,       // P2Pool process state
	xmrig: Process,        // XMRig process state
	p2pool_api: P2poolApi, // P2Pool API state
	xmrig_api: XmrigApi,   // XMRig API state
}

impl Helper {
	pub fn new(instant: std::time::Instant) -> Self {
		Self {
			uptime: HumanTime::into_human(instant.elapsed()),
			p2pool: Process::new(ProcessName::P2pool, String::new(), PathBuf::new()),
			xmrig: Process::new(ProcessName::Xmrig, String::new(), PathBuf::new()),
			p2pool_api: P2poolApi::new(),
			xmrig_api: XmrigApi::new(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The main GUI thread will use this to display console text, online state, etc.
pub struct Process {
	name: ProcessName,     // P2Pool or XMRig?
	state: ProcessState,   // The state of the process (alive, dead, etc)
	signal: ProcessSignal, // Did the user click [Start/Stop/Restart]?
	start: Instant,        // Starttime of process
	uptime: HumanTime,     // Uptime of process
	output: String,        // This is the process's stdout + stderr
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
	input: Vec<String>,
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
	Alive,    // Process is online, GREEN!
	Dead,     // Process is dead, BLACK!
	Failed,   // Process is dead AND exited with a bad code, RED!
	// Process is starting up, YELLOW!
	// Really, processes start instantly, this just accounts for the delay
	// between the main thread and this threads 1 second event loop.
	Starting,
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

//---------------------------------------------------------------------------------------------------- [P2poolApi]
pub struct P2poolApi {

}

impl P2poolApi {
	pub fn new() -> Self {
		Self {
		}
	}
}

//---------------------------------------------------------------------------------------------------- [XmrigApi]
pub struct XmrigApi {

}

impl XmrigApi {
	pub fn new() -> Self {
		Self {
		}
	}
}

//---------------------------------------------------------------------------------------------------- The "helper" loop
#[tokio::main]
pub async fn helper() {
	thread::spawn(|| { loop {
	// 1. Spawn child processes (if signal found)
	// 2. Create stdout pipe thread (if new child process)
	// 3. Send stdin (if signal found)
	// 4. Kill child process (if signal found)
	// 4. Collect P2Pool API task (if alive)
	// 5. Collect XMRig HTTP API task (if alive)
	// 6. Execute all async tasks
	// 7. Set Gupax/P2Pool/XMRig uptime
	}});
}
