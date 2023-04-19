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

// Some regexes used throughout Gupax.

use regex::Regex;
use lazy_static::lazy_static;

//---------------------------------------------------------------------------------------------------- Lazy
lazy_static! {
	pub static ref REGEXES:      Regexes     = Regexes::new();
	pub static ref P2POOL_REGEX: P2poolRegex = P2poolRegex::new();
	pub static ref XMRIG_REGEX:  XmrigRegex  = XmrigRegex::new();
}

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
	fn new() -> Self {
		Self {
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
	pub fn addr_ok(address: &str) -> bool {
		address.len() == 95 && REGEXES.address.is_match(address) && !address.contains('0') && !address.contains('O') && !address.contains('l')
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
	pub date: Regex,
	pub payout: Regex,
	pub payout_float: Regex,
	pub block: Regex,
	pub block_int: Regex,
	pub block_comma: Regex,
	pub synchronized: Regex,
}

impl P2poolRegex {
	fn new() -> Self {
		Self {
			date: Regex::new("[0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+.[0-9]+").unwrap(),
			payout: Regex::new("payout of [0-9].[0-9]+ XMR").unwrap(), // Assumes 12 digits after the dot.
			payout_float: Regex::new("[0-9].[0-9]{12}").unwrap(), // Assumes 12 digits after the dot.
			block: Regex::new("block [0-9]{7}").unwrap(), // Monero blocks will be 7 digits for... the next 10,379 years
			block_int: Regex::new("[0-9]{7}").unwrap(),
			block_comma: Regex::new("[0-9],[0-9]{3},[0-9]{3}").unwrap(),
			synchronized: Regex::new("SYNCHRONIZED").unwrap(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- XMRig regex.
#[derive(Debug)]
pub struct XmrigRegex {
	pub not_mining: Regex,
	pub new_job: Regex,
}

impl XmrigRegex {
	fn new() -> Self {
		Self {
			not_mining: Regex::new("no active pools, stop mining").unwrap(),
			new_job: Regex::new("new job").unwrap(),
		}
	}
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
	#[test]
	fn build_regexes() {
		use regex::Regex;
		let r = crate::Regexes::new();
		assert!(Regex::is_match(&r.name, "_this_ is... a n-a-m-e."));
		assert!(Regex::is_match(&r.address, "44hintoFpuo3ugKfcqJvh5BmrsTRpnTasJmetKC4VXCt6QDtbHVuixdTtsm6Ptp7Y8haXnJ6j8Gj2dra8CKy5ewz7Vi9CYW"));
		assert!(Regex::is_match(&r.ipv4, "192.168.1.2"));
		assert!(Regex::is_match(&r.ipv4, "127.0.0.1"));
		assert!(Regex::is_match(&r.domain, "my.node.com"));
		assert!(Regex::is_match(&r.domain, "my.monero-node123.net"));
		assert!(Regex::is_match(&r.domain, "www.my-node.org"));
		assert!(Regex::is_match(&r.domain, "www.my-monero-node123.io"));
		for i in 1..=65535 {
			assert!(Regex::is_match(&r.port, &i.to_string()));
		}
		assert!(!Regex::is_match(&r.port, "0"));
		assert!(!Regex::is_match(&r.port, "65536"));
	}

	#[test]
	fn build_p2pool_regex() {
		use regex::Regex;
		let r = crate::P2poolRegex::new();
		let text = "NOTICE  2022-11-11 11:11:11.1111 P2Pool You received a payout of 0.111111111111 XMR in block 1111111";
		let text2 = "2022-11-11 11:11:11.1111 | 0.111111111111 XMR | Block 1,111,111";
		let text3 = "NOTICE  2020-12-11 12:35:41.3150 SideChain SYNCHRONIZED";
		assert_eq!(r.payout.find(text).unwrap().as_str(),       "payout of 0.111111111111 XMR");
		assert_eq!(r.payout_float.find(text).unwrap().as_str(), "0.111111111111");
		assert_eq!(r.date.find(text).unwrap().as_str(),         "2022-11-11 11:11:11.1111");
		assert_eq!(r.block.find(text).unwrap().as_str(),        "block 1111111");
		assert_eq!(r.block_int.find(text).unwrap().as_str(),    "1111111");
		assert_eq!(r.block_comma.find(text2).unwrap().as_str(),  "1,111,111");
		assert_eq!(r.synchronized.find(text3).unwrap().as_str(),  "SYNCHRONIZED");
	}
}
