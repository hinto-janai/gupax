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
