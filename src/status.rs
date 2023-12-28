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

use crate::{
	PubP2poolApi,
	PubXmrigApi,
	ImgP2pool,
	ImgXmrig,
	constants::*,
	Sys,
	Hash,
	Submenu,
	macros::*,
	GupaxP2poolApi,
	PayoutView,
	human::HumanNumber,
	Benchmark,
};
use std::sync::{Arc,Mutex};
use log::*;
use egui::{
	Label,RichText,TextStyle,
	TextStyle::Monospace,
	TextStyle::Name,
	TextEdit,
	SelectableLabel,Hyperlink,
	Slider,ProgressBar,Spinner,
};

impl crate::disk::Status {
#[inline(always)]
pub fn show(&mut self, sys: &Arc<Mutex<Sys>>, p2pool_api: &Arc<Mutex<PubP2poolApi>>, xmrig_api: &Arc<Mutex<PubXmrigApi>>, p2pool_img: &Arc<Mutex<ImgP2pool>>, xmrig_img: &Arc<Mutex<ImgXmrig>>, p2pool_alive: bool, xmrig_alive: bool, max_threads: usize, gupax_p2pool_api: &Arc<Mutex<GupaxP2poolApi>>, benchmarks: &[Benchmark], width: f32, height: f32, _ctx: &egui::Context, ui: &mut egui::Ui) {
	//---------------------------------------------------------------------------------------------------- [Processes]
	if self.submenu == Submenu::Processes {
	let width = (width/3.0)-(SPACE*1.666);
	let min_height = height - SPACE;
	let height = height/25.0;
	ui.horizontal(|ui| {
	// [Gupax]
	ui.group(|ui| { ui.vertical(|ui| {
		debug!("Status Tab | Rendering [Gupax]");
		ui.set_min_height(min_height);
		ui.add_sized([width, height], Label::new(RichText::new("[Gupax]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("Gupax is online");
		let sys = lock!(sys);
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_GUPAX_UPTIME);
		ui.add_sized([width, height], Label::new(sys.gupax_uptime.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_CPU_USAGE);
		ui.add_sized([width, height], Label::new(sys.gupax_cpu_usage.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("Gupax Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_MEMORY_USAGE);
		ui.add_sized([width, height], Label::new(sys.gupax_memory_used_mb.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_USAGE);
		ui.add_sized([width, height], Label::new(sys.system_cpu_usage.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("System Memory").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_MEMORY);
		ui.add_sized([width, height], Label::new(sys.system_memory.to_string()));
		ui.add_sized([width, height], Label::new(RichText::new("System CPU Model").underline().color(BONE))).on_hover_text(STATUS_GUPAX_SYSTEM_CPU_MODEL);
		ui.add_sized([width, height], Label::new(sys.system_cpu_model.to_string()));
		drop(sys);
	})});
	// [P2Pool]
	ui.group(|ui| { ui.vertical(|ui| {
		debug!("Status Tab | Rendering [P2Pool]");
		ui.set_enabled(p2pool_alive);
		ui.set_min_height(min_height);
		ui.add_sized([width, height], Label::new(RichText::new("[P2Pool]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("P2Pool is online").on_disabled_hover_text("P2Pool is offline");
		ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
		let height = height/1.4;
		let api = lock!(p2pool_api);
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_P2POOL_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", api.uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("Shares Found").underline().color(BONE))).on_hover_text(STATUS_P2POOL_SHARES);
		ui.add_sized([width, height], Label::new(format!("{}", api.shares_found)));
		ui.add_sized([width, height], Label::new(RichText::new("Payouts").underline().color(BONE))).on_hover_text(STATUS_P2POOL_PAYOUTS);
		ui.add_sized([width, height], Label::new(format!("Total: {}", api.payouts)));
		ui.add_sized([width, height], Label::new(format!("[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]", api.payouts_hour, api.payouts_day, api.payouts_month)));
		ui.add_sized([width, height], Label::new(RichText::new("XMR Mined").underline().color(BONE))).on_hover_text(STATUS_P2POOL_XMR);
		ui.add_sized([width, height], Label::new(format!("Total: {:.13} XMR", api.xmr)));
		ui.add_sized([width, height], Label::new(format!("[{:.7}/hour]\n[{:.7}/day]\n[{:.7}/month]", api.xmr_hour, api.xmr_day, api.xmr_month)));
		ui.add_sized([width, height], Label::new(RichText::new("Hashrate (15m/1h/24h)").underline().color(BONE))).on_hover_text(STATUS_P2POOL_HASHRATE);
		ui.add_sized([width, height], Label::new(format!("[{} H/s] [{} H/s] [{} H/s]", api.hashrate_15m, api.hashrate_1h, api.hashrate_24h)));
		ui.add_sized([width, height], Label::new(RichText::new("Miners Connected").underline().color(BONE))).on_hover_text(STATUS_P2POOL_CONNECTIONS);
		ui.add_sized([width, height], Label::new(format!("{}", api.connections)));
		ui.add_sized([width, height], Label::new(RichText::new("Effort").underline().color(BONE))).on_hover_text(STATUS_P2POOL_EFFORT);
		ui.add_sized([width, height], Label::new(format!("[Average: {}] [Current: {}]", api.average_effort, api.current_effort)));
		let img = lock!(p2pool_img);
		ui.add_sized([width, height], Label::new(RichText::new("Monero Node").underline().color(BONE))).on_hover_text(STATUS_P2POOL_MONERO_NODE);
		ui.add_sized([width, height], Label::new(format!("[IP: {}]\n[RPC: {}] [ZMQ: {}]", &img.host, &img.rpc, &img.zmq)));
		ui.add_sized([width, height], Label::new(RichText::new("Sidechain").underline().color(BONE))).on_hover_text(STATUS_P2POOL_POOL);
		ui.add_sized([width, height], Label::new(&img.mini));
		ui.add_sized([width, height], Label::new(RichText::new("Address").underline().color(BONE))).on_hover_text(STATUS_P2POOL_ADDRESS);
		ui.add_sized([width, height], Label::new(&img.address));
		drop(img);
		drop(api);
	})});
	// [XMRig]
	ui.group(|ui| { ui.vertical(|ui| {
		debug!("Status Tab | Rendering [XMRig]");
		ui.set_enabled(xmrig_alive);
		ui.set_min_height(min_height);
		ui.add_sized([width, height], Label::new(RichText::new("[XMRig]").color(LIGHT_GRAY).text_style(TextStyle::Name("MonospaceLarge".into())))).on_hover_text("XMRig is online").on_disabled_hover_text("XMRig is offline");
		let api = lock!(xmrig_api);
		ui.add_sized([width, height], Label::new(RichText::new("Uptime").underline().color(BONE))).on_hover_text(STATUS_XMRIG_UPTIME);
		ui.add_sized([width, height], Label::new(format!("{}", api.uptime)));
		ui.add_sized([width, height], Label::new(RichText::new("CPU Load (10s/60s/15m)").underline().color(BONE))).on_hover_text(STATUS_XMRIG_CPU);
		ui.add_sized([width, height], Label::new(format!("{}", api.resources)));
		ui.add_sized([width, height], Label::new(RichText::new("Hashrate (10s/60s/15m)").underline().color(BONE))).on_hover_text(STATUS_XMRIG_HASHRATE);
		ui.add_sized([width, height], Label::new(format!("{}", api.hashrate)));
		ui.add_sized([width, height], Label::new(RichText::new("Difficulty").underline().color(BONE))).on_hover_text(STATUS_XMRIG_DIFFICULTY);
		ui.add_sized([width, height], Label::new(format!("{}", api.diff)));
		ui.add_sized([width, height], Label::new(RichText::new("Shares").underline().color(BONE))).on_hover_text(STATUS_XMRIG_SHARES);
		ui.add_sized([width, height], Label::new(format!("[Accepted: {}] [Rejected: {}]", api.accepted, api.rejected)));
		ui.add_sized([width, height], Label::new(RichText::new("Pool").underline().color(BONE))).on_hover_text(STATUS_XMRIG_POOL);
		ui.add_sized([width, height], Label::new(&lock!(xmrig_img).url));
		ui.add_sized([width, height], Label::new(RichText::new("Threads").underline().color(BONE))).on_hover_text(STATUS_XMRIG_THREADS);
		ui.add_sized([width, height], Label::new(format!("{}/{}", &lock!(xmrig_img).threads, max_threads)));
		drop(api);
	})});
	});
	//---------------------------------------------------------------------------------------------------- [P2Pool]
	} else if self.submenu == Submenu::P2pool {
	let api = lock!(gupax_p2pool_api);
	let text = height / 25.0;
	let log = height / 2.8;
	// Payout Text + PayoutView buttons
	ui.group(|ui| {
		ui.horizontal(|ui| {
			let width = (width/3.0)-(SPACE*4.0);
			ui.add_sized([width, text], Label::new(RichText::new(format!("Total Payouts: {}", api.payout)).underline().color(LIGHT_GRAY))).on_hover_text(STATUS_SUBMENU_PAYOUT);
			ui.separator();
			ui.add_sized([width, text], Label::new(RichText::new(format!("Total XMR: {}", api.xmr)).underline().color(LIGHT_GRAY))).on_hover_text(STATUS_SUBMENU_XMR);
			let width = width / 4.0;
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Latest, "Latest")).on_hover_text(STATUS_SUBMENU_LATEST).clicked() {
				self.payout_view = PayoutView::Latest;
			}
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Oldest, "Oldest")).on_hover_text(STATUS_SUBMENU_OLDEST).clicked() {
				self.payout_view = PayoutView::Oldest;
			}
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Biggest, "Biggest")).on_hover_text(STATUS_SUBMENU_BIGGEST).clicked() {
				self.payout_view = PayoutView::Biggest;
			}
			ui.separator();
			if ui.add_sized([width, text], SelectableLabel::new(self.payout_view == PayoutView::Smallest, "Smallest")).on_hover_text(STATUS_SUBMENU_SMALLEST).clicked() {
				self.payout_view = PayoutView::Smallest;
			}
		});
		ui.separator();
		// Actual logs
		egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
			egui::ScrollArea::vertical().stick_to_bottom(self.payout_view == PayoutView::Oldest).max_width(width).max_height(log).auto_shrink([false; 2]).show_viewport(ui, |ui, _| {
				ui.style_mut().override_text_style = Some(Name("MonospaceLarge".into()));
				match self.payout_view {
					PayoutView::Latest   => ui.add_sized([width, log], TextEdit::multiline(&mut api.log_rev.as_str())),
					PayoutView::Oldest   => ui.add_sized([width, log], TextEdit::multiline(&mut api.log.as_str())),
					PayoutView::Biggest  => ui.add_sized([width, log], TextEdit::multiline(&mut api.payout_high.as_str())),
					PayoutView::Smallest => ui.add_sized([width, log], TextEdit::multiline(&mut api.payout_low.as_str())),
				};
			});
		});
	});
	drop(api);
	// Payout/Share Calculator
	let button = (width/20.0)-(SPACE*1.666);
	ui.group(|ui| { ui.horizontal(|ui| {
		ui.set_min_width(width-SPACE);
		if ui.add_sized([button*2.0, text], SelectableLabel::new(!self.manual_hash, "Automatic")).on_hover_text(STATUS_SUBMENU_AUTOMATIC).clicked() {self.manual_hash = false; }
		ui.separator();
		if ui.add_sized([button*2.0, text], SelectableLabel::new(self.manual_hash, "Manual")).on_hover_text(STATUS_SUBMENU_MANUAL).clicked() { self.manual_hash = true; }
		ui.separator();
		ui.set_enabled(self.manual_hash);
		if ui.add_sized([button, text], SelectableLabel::new(self.hash_metric == Hash::Hash, "Hash")).on_hover_text(STATUS_SUBMENU_HASH).clicked() { self.hash_metric = Hash::Hash; }
		ui.separator();
		if ui.add_sized([button, text], SelectableLabel::new(self.hash_metric == Hash::Kilo, "Kilo")).on_hover_text(STATUS_SUBMENU_KILO).clicked() { self.hash_metric = Hash::Kilo; }
		ui.separator();
		if ui.add_sized([button, text], SelectableLabel::new(self.hash_metric == Hash::Mega, "Mega")).on_hover_text(STATUS_SUBMENU_MEGA).clicked() { self.hash_metric = Hash::Mega; }
		ui.separator();
		if ui.add_sized([button, text], SelectableLabel::new(self.hash_metric == Hash::Giga, "Giga")).on_hover_text(STATUS_SUBMENU_GIGA).clicked() { self.hash_metric = Hash::Giga; }
		ui.separator();
		ui.spacing_mut().slider_width = button*11.5;
		ui.add_sized([button*14.0, text], Slider::new(&mut self.hashrate, 1.0..=1_000.0));
	})});
	// Actual stats
	ui.set_enabled(p2pool_alive);
	let text = height / 25.0;
	let width = (width/3.0)-(SPACE*1.666);
	let min_height = ui.available_height()/1.3;
	let api = lock!(p2pool_api);
	ui.horizontal(|ui| {
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, text], Label::new(RichText::new("Monero Difficulty").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_MONERO_DIFFICULTY);
		ui.add_sized([width, text], Label::new(api.monero_difficulty.as_str()));
		ui.add_sized([width, text], Label::new(RichText::new("Monero Hashrate").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_MONERO_HASHRATE);
		ui.add_sized([width, text], Label::new(api.monero_hashrate.as_str()));
		ui.add_sized([width, text], Label::new(RichText::new("P2Pool Difficulty").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_DIFFICULTY);
		ui.add_sized([width, text], Label::new(api.p2pool_difficulty.as_str()));
		ui.add_sized([width, text], Label::new(RichText::new("P2Pool Hashrate").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_HASHRATE);
		ui.add_sized([width, text], Label::new(api.p2pool_hashrate.as_str()));
	})});
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		if self.manual_hash {
			let hashrate          = Hash::convert_to_hash(self.hashrate, self.hash_metric) as u64;
			let p2pool_share_mean = PubP2poolApi::calculate_share_or_block_time(hashrate, api.p2pool_difficulty_u64);
			let solo_block_mean   = PubP2poolApi::calculate_share_or_block_time(hashrate, api.monero_difficulty_u64);
			ui.add_sized([width, text], Label::new(RichText::new("Manually Inputted Hashrate").underline().color(BONE)));
			ui.add_sized([width, text], Label::new(format!("{} H/s", HumanNumber::from_u64(hashrate))));
			ui.add_sized([width, text], Label::new(RichText::new("P2Pool Block Mean").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_BLOCK_MEAN);
			ui.add_sized([width, text], Label::new(api.p2pool_block_mean.to_string()));
			ui.add_sized([width, text], Label::new(RichText::new("Your P2Pool Share Mean").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_SHARE_MEAN);
			ui.add_sized([width, text], Label::new(p2pool_share_mean.to_string()));
			ui.add_sized([width, text], Label::new(RichText::new("Your Solo Block Mean").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_SOLO_BLOCK_MEAN);
			ui.add_sized([width, text], Label::new(solo_block_mean.to_string()));
		} else {
			ui.add_sized([width, text], Label::new(RichText::new("Your P2Pool Hashrate").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_P2POOL_HASHRATE);
			ui.add_sized([width, text], Label::new(format!("{} H/s", api.hashrate_1h)));
			ui.add_sized([width, text], Label::new(RichText::new("P2Pool Block Mean").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_BLOCK_MEAN);
			ui.add_sized([width, text], Label::new(api.p2pool_block_mean.to_string()));
			ui.add_sized([width, text], Label::new(RichText::new("Your P2Pool Share Mean").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_SHARE_MEAN);
			ui.add_sized([width, text], Label::new(api.p2pool_share_mean.to_string()));
			ui.add_sized([width, text], Label::new(RichText::new("Your Solo Block Mean").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_SOLO_BLOCK_MEAN);
			ui.add_sized([width, text], Label::new(api.solo_block_mean.to_string()));
		}
	})});
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		if self.manual_hash {
			let hashrate            = Hash::convert_to_hash(self.hashrate, self.hash_metric) as u64;
			let user_p2pool_percent = PubP2poolApi::calculate_dominance(hashrate, api.p2pool_hashrate_u64);
			let user_monero_percent = PubP2poolApi::calculate_dominance(hashrate, api.monero_hashrate_u64);
			ui.add_sized([width, text], Label::new(RichText::new("P2Pool Miners").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_MINERS);
			ui.add_sized([width, text], Label::new(api.miners.as_str()));
			ui.add_sized([width, text], Label::new(RichText::new("P2Pool Dominance").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_DOMINANCE);
			ui.add_sized([width, text], Label::new(api.p2pool_percent.as_str()));
			ui.add_sized([width, text], Label::new(RichText::new("Your P2Pool Dominance").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_P2POOL_DOMINANCE);
			ui.add_sized([width, text], Label::new(user_p2pool_percent.as_str()));
			ui.add_sized([width, text], Label::new(RichText::new("Your Monero Dominance").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_MONERO_DOMINANCE);
			ui.add_sized([width, text], Label::new(user_monero_percent.as_str()));
		} else {
			ui.add_sized([width, text], Label::new(RichText::new("P2Pool Miners").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_MINERS);
			ui.add_sized([width, text], Label::new(api.miners.as_str()));
			ui.add_sized([width, text], Label::new(RichText::new("P2Pool Dominance").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_P2POOL_DOMINANCE);
			ui.add_sized([width, text], Label::new(api.p2pool_percent.as_str()));
			ui.add_sized([width, text], Label::new(RichText::new("Your P2Pool Dominance").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_P2POOL_DOMINANCE);
			ui.add_sized([width, text], Label::new(api.user_p2pool_percent.as_str()));
			ui.add_sized([width, text], Label::new(RichText::new("Your Monero Dominance").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_MONERO_DOMINANCE);
			ui.add_sized([width, text], Label::new(api.user_monero_percent.as_str()));
		}
	})});
	});
	// Tick bar
	ui.add_sized([ui.available_width(), text], Label::new(api.calculate_tick_bar())).on_hover_text(STATUS_SUBMENU_PROGRESS_BAR);
	drop(api);
	//---------------------------------------------------------------------------------------------------- [Benchmarks]
	} else if self.submenu == Submenu::Benchmarks {
	debug!("Status Tab | Rendering [Benchmarks]");
	let text = height / 20.0;
	let double = text * 2.0;
	let log = height / 3.0;

	// [0], The user's CPU (most likely).
	let cpu = &benchmarks[0];
	ui.horizontal(|ui| {
	let width = (width/2.0)-(SPACE*1.666);
	let min_height = log;
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, text], Label::new(RichText::new("Your CPU").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_CPU);
		ui.add_sized([width, text], Label::new(cpu.cpu.as_str()));
		ui.add_sized([width, text], Label::new(RichText::new("Total Benchmarks").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_BENCHMARKS);
		ui.add_sized([width, text], Label::new(format!("{}", cpu.benchmarks)));
		ui.add_sized([width, text], Label::new(RichText::new("Rank").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_RANK);
		ui.add_sized([width, text], Label::new(format!("{}/{}", cpu.rank, &benchmarks.len())));
	})});
	ui.group(|ui| { ui.vertical(|ui| {
		ui.set_min_height(min_height);
		ui.add_sized([width, text], Label::new(RichText::new("High Hashrate").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_HIGH);
		ui.add_sized([width, text], Label::new(format!("{} H/s", HumanNumber::from_f32(cpu.high))));
		ui.add_sized([width, text], Label::new(RichText::new("Average Hashrate").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_AVERAGE);
		ui.add_sized([width, text], Label::new(format!("{} H/s", HumanNumber::from_f32(cpu.average))));
		ui.add_sized([width, text], Label::new(RichText::new("Low Hashrate").underline().color(BONE))).on_hover_text(STATUS_SUBMENU_YOUR_LOW);
		ui.add_sized([width, text], Label::new(format!("{} H/s", HumanNumber::from_f32(cpu.low))));
	})})
	});

	// User's CPU hashrate comparison (if XMRig is alive).
	ui.scope(|ui| {
		if xmrig_alive {
			let api = lock!(xmrig_api);
			let percent = (api.hashrate_raw / cpu.high) * 100.0;
			let human = HumanNumber::to_percent(percent);
			if percent > 100.0 {
				ui.add_sized([width, double], Label::new(format!("Your CPU's is faster than the highest benchmark! It is [{}] faster @ {}!", human, api.hashrate)));
				ui.add_sized([width, text], ProgressBar::new(1.0));
			} else if api.hashrate_raw == 0.0 {
				ui.add_sized([width, text], Label::new("Measuring hashrate..."));
				ui.add_sized([width, text], Spinner::new().size(text));
				ui.add_sized([width, text], ProgressBar::new(0.0));
			} else {
				ui.add_sized([width, double], Label::new(format!("Your CPU's hashrate is [{}] of the highest benchmark @ {}", human, api.hashrate)));
				ui.add_sized([width, text], ProgressBar::new(percent / 100.0));
			}
		} else {
			ui.set_enabled(xmrig_alive);
			ui.add_sized([width, double], Label::new("XMRig is offline. Hashrate cannot be determined."));
			ui.add_sized([width, text], ProgressBar::new(0.0));
		}
	});

	// Comparison
	ui.group(|ui| {
		ui.add_sized([width, text], Hyperlink::from_label_and_url("Other CPUs", "https://xmrig.com/benchmark")).on_hover_text(STATUS_SUBMENU_OTHER_CPUS);
	});

	egui::ScrollArea::both()
	.scroll_bar_visibility(egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible)
	.max_width(width)
	.max_height(height)
	.auto_shrink([false; 2])
	.show_viewport(ui, |ui, _| {
		let width = width / 20.0;
		let (cpu, bar, high, average, low, rank, bench) = (
			width*10.0,
			width*3.0,
			width*2.0,
			width*2.0,
			width*2.0,
			width,
			width*2.0,
		);
		ui.group(|ui| {
			ui.horizontal(|ui| {
				ui.add_sized([cpu, double], Label::new("CPU")).on_hover_text(STATUS_SUBMENU_OTHER_CPU);
				ui.separator();
				ui.add_sized([bar, double], Label::new("Relative")).on_hover_text(STATUS_SUBMENU_OTHER_RELATIVE);
				ui.separator();
				ui.add_sized([high, double], Label::new("High")).on_hover_text(STATUS_SUBMENU_OTHER_HIGH);
				ui.separator();
				ui.add_sized([average, double], Label::new("Average")).on_hover_text(STATUS_SUBMENU_OTHER_AVERAGE);
				ui.separator();
				ui.add_sized([low, double], Label::new("Low")).on_hover_text(STATUS_SUBMENU_OTHER_LOW);
				ui.separator();
				ui.add_sized([rank, double], Label::new("Rank")).on_hover_text(STATUS_SUBMENU_OTHER_RANK);
				ui.separator();
				ui.add_sized([bench, double], Label::new("Benchmarks")).on_hover_text(STATUS_SUBMENU_OTHER_BENCHMARKS);
			});
		});

		for benchmark in benchmarks[1..].iter() {
			ui.group(|ui| { ui.horizontal(|ui| {
				ui.add_sized([cpu, text], Label::new(benchmark.cpu.as_str()));
				ui.separator();
				ui.add_sized([bar, text], ProgressBar::new(benchmark.percent / 100.0)).on_hover_text(HumanNumber::to_percent(benchmark.percent).as_str());
				ui.separator();
				ui.add_sized([high, text], Label::new(HumanNumber::to_hashrate(benchmark.high).as_str()));
				ui.separator();
				ui.add_sized([average, text], Label::new(HumanNumber::to_hashrate(benchmark.average).as_str()));
				ui.separator();
				ui.add_sized([low, text], Label::new(HumanNumber::to_hashrate(benchmark.low).as_str()));
				ui.separator();
				ui.add_sized([rank, text], Label::new(HumanNumber::from_u16(benchmark.rank).as_str()));
				ui.separator();
				ui.add_sized([bench, text], Label::new(HumanNumber::from_u16(benchmark.benchmarks).as_str()));
			})});
		}
	});
	}
}
}
