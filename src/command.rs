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

// This file handles all things related to child processes (P2Pool/XMRig).
// The main GUI thread will interface with the [Arc<Mutex<...>>] data found
// here, e.g: User clicks [Start P2Pool] -> Init p2pool thread here.

//---------------------------------------------------------------------------------------------------- Import
use std::{
	sync::{Arc,Mutex},
	path::PathBuf,
	process::Command,
	thread,
};
use crate::constants::*;
use log::*;

//---------------------------------------------------------------------------------------------------- [Process] Struct
// This holds all the state of a (child) process.
// The actual process thread runs in a 1 second loop, reading/writing to this struct.
// The main GUI thread will use this to display console text, online state, etc.
pub struct Process {
	name: ProcessName,     // P2Pool or XMRig?
	online: bool,          // Is the process alive?
	args: String,          // A single [String] containing the arguments
	path: PathBuf,         // The absolute path to the process binary
	signal: ProcessSignal, // Did the user click [Stop/Restart]?
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
		Self {
			name,
			online: false,
			args,
			path,
			signal: ProcessSignal::None,
			output: String::new(),
			input: vec![String::new()],
		}
	}

	// Borrow a [&str], return an owned split collection
	pub fn parse_args(args: &str) -> Vec<String> {
		args.split_whitespace().map(|s| s.to_owned()).collect()
	}

	pub fn spawn(process: &Arc<Mutex<Self>>, name: ProcessName) {
		// Setup
		let process = Arc::clone(process);
		let args = Self::parse_args(&process.lock().unwrap().args);
		info!("{} | Spawning initial thread", name);
		info!("{} | Arguments: {:?}", name, args);

		// Spawn thread
		thread::spawn(move || {
			// Create & spawn child
			let mut child = Command::new(&process.lock().unwrap().path)
				.args(args)
				.stdout(std::process::Stdio::piped())
				.spawn().unwrap();

			// 1-second loop, reading and writing data to relevent struct
			loop {
				let process = process.lock().unwrap(); // Get lock
				// If user sent a signal, handle it
				match process.signal {
					ProcessSignal::None => {},
					_ => { child.kill(); break; },
				};
//				println!("{:?}", String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap());
				thread::sleep(SECOND);
			}

			// End of thread, must mean process is offline
			process.lock().unwrap().online = false;
		});
	}
}

//---------------------------------------------------------------------------------------------------- [ProcessSignal] Enum
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessSignal {
	None,
	Stop,
	Restart,
}

//---------------------------------------------------------------------------------------------------- [ProcessName] Enum
#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum ProcessName {
	P2Pool,
	XMRig,
}

impl std::fmt::Display for ProcessName {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", self)
	}
}
