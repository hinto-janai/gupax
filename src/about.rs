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

use crate::App;
use egui::special_emojis::GITHUB;

// Main data structure for the About tab
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct About {
}

impl About {
	pub fn show(app: &mut App, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.add_space(10.0);
		ui.vertical_centered(|ui| {
			let space = ui.available_height()/2.2;
			app.banner.show(ui);
			ui.label("Gupax (guh-picks) is a cross-platform GUI for mining");
			ui.hyperlink_to("[Monero]", "https://www.github.com/monero-project/monero");
			ui.label("on the decentralized");
			ui.hyperlink_to("[P2Pool]", "https://www.github.com/SChernykh/p2pool");
			ui.label("using the dedicated");
			ui.hyperlink_to("[XMRig]", "https://www.github.com/xmrig/xmrig");
			ui.label("miner for max hashrate");

			ui.add_space(ui.available_height()/2.4);

			ui.hyperlink_to("Powered by egui", "https://github.com/emilk/egui");
			ui.hyperlink_to(format!("{} {}", GITHUB, "Gupax made by hinto-janaiyo"), "https://www.github.com/hinto-janaiyo/gupax");
			ui.label("egui is licensed under MIT & Apache-2.0");
			ui.label("Gupax, P2Pool, and XMRig are licensed under GPLv3");
		});
	}
}
