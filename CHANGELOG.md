# v1.0.1
## Fixes
* macOS: Added warning (and solution) if `Gupax/P2Pool/XMRig` were quarantined by [`Gatekeeper`](https://support.apple.com/en-us/HT202491)
* P2Pool/XMRig: Added a red `Start` button on errors (bad PATH, invalid file, etc) and a solution in the tooltip
* P2Pool/XMRig: Fixed processes sometimes not starting after entering a custom PATH
* P2Pool: Fixed custom node selection sometimes using old values after save
* Miscellaneous UI changes and fixes

## Bundled Versions
* [`P2Pool v2.6`](https://github.com/SChernykh/p2pool/releases/tag/v2.6)
* [`XMRig v6.18.1`](https://github.com/xmrig/xmrig/releases/tag/v6.18.1)


---


# v1.0.0
[Download here](https://github.com/hinto-janaiyo/gupax/releases/latest) or at https://gupax.io.

[Watch a 3-minute setup guide here.](https://github.com/hinto-janaiyo/gupax#How-To)

## Changes
* Optimized PTY output handling (less memory usage)
* Added `Select random node`, `<- Last`, `Next ->` buttons in `P2Pool Simple`
* Added `Debug Info` screen (`D` key on `About` tab)
* Added Linux distro build profile
* Added `21` unit tests
* Misc fixes/optimizations

## Bundled Versions
* [`P2Pool v2.6`](https://github.com/SChernykh/p2pool/releases/tag/v2.6)
* [`XMRig v6.18.1`](https://github.com/xmrig/xmrig/releases/tag/v6.18.1)


---


# v0.9.0
## Beta Release
* Connected `[Start/Stop/Restart]` buttons to actual processes:
	- Current state (settings that may or may not be saved) are passed to the process when (re)starting
	- Uptime & Exit status display when process is stopped
	- Added colors for process state:
		```
		GREEN  | Process is online and healthy
		YELLOW | Process is in the middle of (re)starting/stopping
		RED    | Process is offline, and failed when exiting
		GRAY   | Process is offline
		```
* Added keyboard shortcuts:
	```
	*--------------------------------------*
	|             Key shortcuts            |
	|--------------------------------------|
	|             F11 | Fullscreen         |
	|          Escape | Quit screen        |
	|      Left/Right | Switch Tabs        |
	|              Up | Start/Restart      |
	|            Down | Stop               |
	|               S | Save               |
	|               R | Reset              |
	*--------------------------------------*
	```
* Added `PTY` (actual terminals) for P2Pool/XMRig:
	- Scrollable logs up to 500k bytes (6000~ lines) before refresh
	- All STDOUT/STDERR relayed to GUI (buffered, lazily read)
	- `Advanced` tabs have input (STDIN) relayed to process (buffered, 1~sec delay)
* Added `sudo` screen for XMRig (macOS/Linux):
	- Tests password for validity
	- Starts XMRig with `sudo` for MSR mod & hugepages
	- Wipes password memory with zeros after usage
* Added `Status` tab:
	- Refreshes all stats once per second
	- Gupax/System stats
	- P2Pool stats via API file
	- XMRig stats via HTTP API
* Added `Simple` XMRig tab:
	- Console
	- Thread slider
	- Pause on active slider (Windows/macOS only)
* Added `Advanced` XMRig tab:
	- Includes all simple features
	- STDIN input
	- Manual pool database, select/add/edit/delete a custom `Name/IP/Port/RigID` (max 1000 pools), saved at:
	    - Windows: `C:\Users\USER\AppData\Roaming\Gupax\pool.toml`
	    - macOS: `/Users/USER/Library/Application Support/Gupax/pool.toml`
	    - Linux: `/home/USER/.local/share/gupax/pool.toml`
	- Overriding command arguments
	- Manual Monero address option
	- HTTP API IP/Port option
	- TLS option
	- Keepalive option
* Added `Simple` Gupax tab:
	- Package updater
	- `Auto-update` setting
	- `Update-via-Tor` setting
	- `Ask-before-quit` setting
	- `Save-before-quit` setting
	- `Auto-P2Pool` setting (starts P2Pool on Gupax startup)
	- `Auto-XMRig` setting (starts XMRig on Gupax startup)
* Added `Advanced` Gupax tab:
	- Includes all simple features
	- P2Pool binary path selector
	- XMRig binary path selector
	- Gupax window width/height adjuster
	- Startup Tab selector
* Added plowsof to community nodes:
	- Plowsof1: `IP: node.monerodevs.org, RPC: 18089, ZMQ: 18084`
	- Plowsof2: `IP: node2.monerodevs.org, RPC: 18089, ZMQ: 18084`
* Default resolution change `1280x720, 16:9` -> `1280x960, 4:3`
* Added fade-in/out of black when resizing resolution
* Added more internal documentation (`src/README.md`)
* Added many, many `info` & `debug` logs (accessible via env variable `RUST_LOG`)
* Bunch of fixes, optimizations, etc.


---


## v0.5.0
## Prototype Release
* Added `Simple` P2Pool tab:
	- Monero address input with valid address check (base58 regex)
	- [Community Monero node selector](https://github.com/hinto-janaiyo/gupax/tree/main/README.md#community-monero-nodes) 
	- Community node ping button (asynchronous `JSON-RPC` calls to all nodes)
	- Color coded list after ping:
		```
		<300ms = GREEN
		<1000ms = YELLOW
		<5000ms = RED
		>5000ms = BLACK
		```
	- `Auto-select` - Pick the fastest node after ping automatically
	- `Auto-ping` - Automatically ping nodes on Gupax startup
* Added `Advanced` P2Pool tab:
	- Manual node database, select/add/edit/delete a custom `Name/IP/RPC/ZMQ` (max 1000 nodes), saved at:
	    - Windows: `C:\Users\USER\AppData\Roaming\Gupax\node.toml`
	    - macOS: `/Users/USER/Library/Application Support/Gupax/node.toml`
	    - Linux: `/home/USER/.local/share/gupax/node.toml`
	- Overriding command arguments to P2Pool
	- P2Pool main/mini toggle
	- Out/In Peers slider
	- Log level slider
* Added command arguments:
	```
	--help         Print this help message
	--version      Print version and build info
	--state        Print Gupax state
	--nodes        Print the manual node list
	--no-startup   Disable all auto-startup settings for this instance
	--reset-state  Reset all Gupax state (your settings)
	--reset-nodes  Reset the manual node list in the [P2Pool] tab
	--reset-all    Reset both the state and the manual node list
	--ferris       Print an extremely cute crab
	```
* Added fullscreen GUI error handler (Error message + UI buttons for response, Yes/No, Quit, etc)
* Added a native `File Explorer/Finder/GTK` file selector for picking P2Pool/XMRig binary path in `Gupax` tab
* Added detailed console log levels `RUST_LOG=(trace|debug|info|warn|error) ./gupax`
* [Added new PGP key](https://github.com/hinto-janaiyo/gupax/blob/main/pgp/hinto-janaiyo.asc)
* Created website (HTML/CSS only, no JavaScript): https://gupax.io


---


## v0.1.0
## Prototype Release
* Added package updater (by default, via Tor using [`Arti`](https://blog.torproject.org/arti_100_released/))
* Added [custom icons per OS](https://github.com/hinto-janaiyo/gupax/tree/main/images/icons) (File Explorer, Taskbar, Finder, App header, etc)
* Added Monero node [`JSON-RPC ping`](https://github.com/hinto-janaiyo/gupax/blob/main/src/node.rs) system, not yet in GUI
* Added `F11` fullscreen toggle
* Implemented `Ask before quit`
* Implemented `Auto-save`
* Binaries for all platforms (Windows, macOS, Linux)
* Added state file to save settings:
    - Windows: `C:\Users\USER\AppData\Roaming\Gupax\gupax.toml`
    - macOS: `/Users/USER/Library/Application Support/Gupax/gupax.toml`
    - Linux: `/home/USER/.local/share/gupax/gupax.toml`


---


## v0.0.1
## Prototype Release
* Functional (slightly buggy) GUI external
* Elements have state (buttons, sliders, etc)
* No internals, no connections, no processes
* Only binaries for x86_64 Windows/Linux for now
