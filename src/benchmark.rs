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

// This file contains backend code for handling XMRig's benchmark data:
//     - HTTP(s) fetchs to [https://xmrig.com]
//     - (De)serialization of JSON data
//     - (De)serialization of CPU topology XML (./xmrig --export topology)

use serde::{Serialize, Deserialize};
use std::fmt::Write;

// Input: Full [&str] of XMRig's [topology.xml] file
// Output: The CPU name formatted so it's usable as the endpoint, e.g: [AMD+Ryzen+9+5950X+16-Core+Processor]
fn cpu_name_from_xml(xml: &str) -> Option<String> {
	// A minimal matching struct for the [CPUModel] <info> field in the XML file
	#[derive(Debug, Serialize, Deserialize, PartialEq)]
	struct Info {      // <info [...] />
		name: String,  // name="CPUModel"
		value: String, // value="Ryzen ..."
	}

	// Regex to find matching field
	let regex = regex::Regex::new("\"CPUModel\"").unwrap();

	for line in xml.lines() {
		if !regex.is_match(&line) { continue }

		// If found, attempt to serialize XML proper
		if let Ok(info) = serde_xml_rs::from_str::<Info>(&line) {
			// Return early if empty
			if info.value.is_empty() {
				return None
			}
			// If serialized, turn whitespaces into '+'
			let words: Vec<&str> = info.value.split_whitespace().collect();
			let last_word = words.len();
			let mut result = String::new();
			let mut n = 1;
			for word in words.iter() {
				match n == last_word {
					false => write!(result, "{}+", word),
					true  => write!(result, "{}", word),
				};
				n += 1;
			}
			return Some(result)
		}
	}

	// If loop didn't return early, return none
	None
}

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
	#[test]
	fn get_cpu_from_xml() {
		let string =
r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE topology SYSTEM "hwloc2.dtd">
<topology version="2.0">
  <object type="Machine" os_index="0" cpuset="0xffffffff" complete_cpuset="0xffffffff" allowed_cpuset="0xffffffff" nodeset="0x00000001" complete_nodeset="0x00000001" allowed_nodeset="0x00000001" gp_index="1">
    <info name="DMIBIOSVendor" value="American Megatrends International, LLC."/>
    <info name="Backend" value="Linux"/>
    <info name="LinuxCgroup" value="/"/>
    <info name="OSName" value="Linux"/>
    <info name="CPUModel" value="AMD Ryzen 9 5950X 16-Core Processor            "/>
    <info name="CPUStepping" value="0"/>
"#;
		assert_eq!(crate::benchmark::cpu_name_from_xml(&string).unwrap(), "AMD+Ryzen+9+5950X+16-Core+Processor");
	}
}
