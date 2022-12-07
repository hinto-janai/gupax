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

// Handling of [sudo] for XMRig.
// [zeroize] is used to wipe the memory after use.
// Only gets imported in [main.rs] for Unix.

use zeroize::Zeroize;
use std::sync::{Arc,Mutex};
use std::thread;
use log::*;

#[derive(Debug,Clone)]
pub struct SudoState {
	pub testing: bool, // Are we attempting a sudo test right now?
	pub success: bool, // Was the sudo test a success?
	pub hide: bool, // Are we hiding the password?
	pub msg: String, // The message shown to the user if unsuccessful
	pub pass: String, // The actual password wrapped in a [SecretVec]
}

impl SudoState {
	pub fn new() -> Self {
		Self {
			testing: false,
			success: false,
			hide: true,
			msg: "".to_string(),
			pass: String::with_capacity(256),
		}
	}

	// Swaps the pass with another 256-capacity String,
	// zeroizes the old and drops it.
	pub fn wipe(state: &Arc<Mutex<Self>>) {
		info!("Sudo | Wiping password with zeros and dropping from memory...");
		let mut new = String::with_capacity(256);
		let mut state = state.lock().unwrap();
		// new is now == old, and vice-versa.
		std::mem::swap(&mut new, &mut state.pass);
		// we're wiping & dropping the old pass here.
		new.zeroize();
		std::mem::drop(new);
		info!("Sudo ... Password Wipe OK");
	}

	pub fn test_sudo(state: Arc<Mutex<Self>>) {
		std::thread::spawn(move || {
			state.lock().unwrap().testing = true;
			info!("in test_sudo()");
			std::thread::sleep(std::time::Duration::from_secs(3));
			state.lock().unwrap().testing = false;
			if state.lock().unwrap().pass == "secret" {
				state.lock().unwrap().msg = "Correct!".to_string();
			} else {
				state.lock().unwrap().msg = "Incorrect password!".to_string();
			}
			Self::wipe(&state);
		});
	}
}
