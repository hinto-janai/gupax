# v0.7.0
## Updates
*

## Fixes
*


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
