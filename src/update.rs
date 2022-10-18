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
//// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::State;

struct Update {
	new_gupax: String,
	new_p2pool: String,
	new_xmrig: String,
	path_gupax: String,
	path_p2pool: String,
	path_xmrig: String,
	updating: Arc<Mutex<bool>> // Is the update in progress?
	update_prog: u8, // Not an [f32] because [Eq] doesn't work
}

impl Update {
	fn new(path_p2pool: String, path_xmrig: String) -> Result<Self, Error> {
		let path_gupax = std::env::current_exe()?;
		Self {
			new_gupax: "?".to_string(),
			new_p2pool: "?".to_string(),
			new_xmrig: "?".to_string(),
			path_gupax,
			path_p2pool,
			path_xmrig,
			updating: Arc::new(Mutex::new(false)),
			update_prog: 0,
		}
	}

	fn update(state: &mut State) -> Result((), Error) {

	}

#[derive(Debug, Serialize, Deserialize)]
struct TagName {
	tag_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum Error {
	Io(std::io::Error),
	Serialize(toml::ser::Error),
	Deserialize(toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
enum Package {
	Gupax,
	P2pool,
	Xmrig,
}

