//---------------------------------------------------------------------------------------------------- Use
use crate::constants::{COMMIT, GUPAX_VERSION, OS_NAME, P2POOL_VERSION, XMRIG_VERSION};

//----------------------------------------------------------------------------------------------------
#[cold]
#[inline(never)]
/// Set custom panic hook.
pub(crate) fn set_panic_hook(now: std::time::Instant) {
    std::panic::set_hook(Box::new(move |panic_info| {
        // Set stack-trace.
        let stack_trace = std::backtrace::Backtrace::force_capture();
        let args = std::env::args_os();
        let uptime = now.elapsed().as_secs_f32();

        // Re-format panic info.
        let panic_info = format!(
            "{panic_info:#?}

info:
   OS      | {OS_NAME}
   args    | {args:?}
   commit  | {COMMIT}
   gupax   | {GUPAX_VERSION}
   p2pool  | {P2POOL_VERSION} (bundled)
   xmrig   | {XMRIG_VERSION} (bundled)
   uptime  | {uptime} seconds

stack backtrace:\n{stack_trace}",
        );

        // Attempt to write panic info to disk.
        match crate::disk::get_gupax_data_path() {
            Ok(mut path) => {
                path.push("crash.txt");
                match std::fs::write(&path, &panic_info) {
                    Ok(_) => {
                        eprintln!("\nmass_panic!() - Saved panic log to: {}\n", path.display())
                    }
                    Err(e) => eprintln!("\nmass_panic!() - Could not save panic log: {e}\n"),
                }
            }
            Err(e) => eprintln!("panic_hook PATH error: {e}"),
        }

        // Exit all threads.
        benri::mass_panic!(panic_info);
    }));
}
