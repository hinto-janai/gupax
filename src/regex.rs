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

// Some regexes used throughout Gupax.

use regex::Regex;

//---------------------------------------------------------------------------------------------------- [Regexes] struct
// General purpose Regexes, mostly used in the GUI.
#[derive(Clone, Debug)]
pub struct Regexes {
	pub name: Regex,
	pub address: Regex,
	pub ipv4: Regex,
	pub domain: Regex,
	pub port: Regex,
}

impl Regexes {
	pub fn new() -> Self {
		Regexes {
			name: Regex::new("^[A-Za-z0-9-_.]+( [A-Za-z0-9-_.]+)*$").unwrap(),
			address: Regex::new("^4[A-Za-z1-9]+$").unwrap(), // This still needs to check for (l, I, o, 0)
			ipv4: Regex::new(r#"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$"#).unwrap(),
			domain: Regex::new(r#"^(([a-zA-Z]{1})|([a-zA-Z]{1}[a-zA-Z]{1})|([a-zA-Z]{1}[0-9]{1})|([0-9]{1}[a-zA-Z]{1})|([a-zA-Z0-9][a-zA-Z0-9-_]{1,61}[a-zA-Z0-9]))\.([a-zA-Z]{2,6}|[a-zA-Z0-9-]{2,30}\.[a-zA-Z]{2,3})$"#).unwrap(),
			port: Regex::new(r#"^([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$"#).unwrap(),
		}
	}

	// Check if a Monero address is correct.
	// This actually only checks for length & Base58, and doesn't do any checksum validation
	// (the last few bytes of a Monero address are a Keccak hash checksum) so some invalid addresses can trick this function.
	pub fn addr_ok(&self, address: &str) -> bool {
		address.len() == 95 && Regex::is_match(&self.address, address) && !address.contains('0') && !address.contains('O') && !address.contains('l')
	}
}


//---------------------------------------------------------------------------------------------------- [P2poolRegex]
// Meant for parsing the output of P2Pool and finding payouts and total XMR found.
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
#[derive(Clone,Debug)]
pub struct P2poolRegex {
	pub payout: regex::Regex,
	pub float: regex::Regex,
	pub date: regex::Regex,
	pub block: regex::Regex,
	pub int: regex::Regex,
}

impl P2poolRegex {
	pub fn new() -> Self {
		Self {
			payout: regex::Regex::new("payout of [0-9].[0-9]+ XMR").unwrap(),
			float: regex::Regex::new("[0-9].[0-9]+").unwrap(),
			date: regex::Regex::new("[0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+.[0-9]+").unwrap(),
			block: regex::Regex::new("block [0-9]+").unwrap(),
			int: regex::Regex::new("[0-9]+").unwrap(),
		}
	}
}
