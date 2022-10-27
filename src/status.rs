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
use egui::{containers::*, *};

// Main data structure for the Status tab
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {

}

impl Status {
	pub fn show(app: &mut App, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
	    let color = if ui.visuals().dark_mode {
	        Color32::from_additive_luminance(196)
	    } else {
	        Color32::from_black_alpha(240)
	    };

	    Frame::canvas(ui.style()).show(ui, |ui| {
	        ui.ctx().request_repaint();
	        let time = ui.input().time;

	        let desired_size = ui.available_width() * vec2(1.0, 0.3);
	        let (_id, rect) = ui.allocate_space(desired_size);

	        let to_screen =
	            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

	        let mut shapes = vec![];

	        for &mode in &[2, 3, 5] {
	            let mode = mode as f64;
	            let n = 120;
	            let speed = 1.5;

	            let points: Vec<Pos2> = (0..=n)
	                .map(|i| {
	                    let t = i as f64 / (n as f64);
	                    let amp = (time * speed * mode).sin() / mode;
	                    let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
	                    to_screen * pos2(t as f32, y as f32)
	                })
	                .collect();

	            let thickness = 10.0 / mode as f32;
	            shapes.push(epaint::Shape::line(points, Stroke::new(thickness, color)));
	        }

	        ui.painter().extend(shapes);
	    });
		ui.label("WIP");
		ui.label("Enjoy these cool lines.");
	}
}
