// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022 hinto-janai
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

// These are general QoL macros, nothing too scary, I promise.
//
// | MACRO   | PURPOSE                                       | EQUIVALENT CODE                                            |
// |---------|-----------------------------------------------|------------------------------------------------------------|
// | lock    | Lock an [Arc<Mutex>]                          | a.lock().unwrap()                                          |
// | lock2   | Lock a field inside a struct, both Arc<Mutex> | a.lock().unwrap().b.lock().unwrap()                        |
// | arc_mut | Create a new [Arc<Mutex>]                     | std::sync::Arc::new(std::sync::Mutex::new(my_value))       |
// | sleep   | Sleep the current thread for x milliseconds   | std::thread::sleep(std::time::Duration::from_millis(1000)) |
// | flip    | Flip a bool in place                          | my_bool = !my_bool                                         |
//
// Hopefully the long ass code on the right justifies usage of macros :D
//
// [lock2!()] works like this: "lock2!(my_first, my_second)"
// and expects it be a [Struct]-[field] relationship, e.g:
//
//     let my_first = Arc::new(Mutex::new(Struct {
//         my_second: Arc::new(Mutex::new(true)),
//     }));
//     lock2!(my_first, my_second);
//
// The equivalent code is: "my_first.lock().unwrap().my_second.lock().unwrap()" (see? this is long as hell)

// Locks and unwraps an [Arc<Mutex<T>]
macro_rules! lock {
	($arc_mutex:expr) => {
		$arc_mutex.lock().unwrap()
	};
}
pub(crate) use lock;

// Locks and unwraps a field of a struct, both of them being [Arc<Mutex>]
// Yes, I know this is bad code.
macro_rules! lock2 {
	($arc_mutex:expr, $arc_mutex_two:ident) => {
		$arc_mutex.lock().unwrap().$arc_mutex_two.lock().unwrap()
	};
}
pub(crate) use lock2;

// Creates a new [Arc<Mutex<T>]
macro_rules! arc_mut {
	($arc_mutex:expr) => {
		std::sync::Arc::new(std::sync::Mutex::new($arc_mutex))
	};
}
pub(crate) use arc_mut;

// Sleeps a [std::thread] using milliseconds
macro_rules! sleep {
    ($millis:expr) => {
		std::thread::sleep(std::time::Duration::from_millis($millis))
    };
}
pub(crate) use sleep;

// Flips a [bool] in place
macro_rules! flip {
	($b:expr) => {
		match $b {
			true|false => $b = !$b,
		}
	};
}
pub(crate) use flip;

//---------------------------------------------------------------------------------------------------- TESTS
#[cfg(test)]
mod test {
	#[test]
	fn lock() {
		use std::sync::{Arc,Mutex};
		let arc_mutex = Arc::new(Mutex::new(false));
		*lock!(arc_mutex) = true;
		assert!(*lock!(arc_mutex) == true);
	}

	#[test]
	fn lock2() {
		struct Ab {
			a: Arc<Mutex<bool>>,
		}
		use std::sync::{Arc,Mutex};
		let arc_mutex = Arc::new(Mutex::new(
			Ab {
				a: Arc::new(Mutex::new(false)),
			}
		));
		*lock2!(arc_mutex,a) = true;
		assert!(*lock2!(arc_mutex,a) == true);
	}

	#[test]
	fn arc_mut() {
		let a = arc_mut!(false);
		assert!(*lock!(a) == false);
	}

	#[test]
	fn flip() {
		let mut b = true;
		flip!(b);
		assert!(b == false);
	}
}
