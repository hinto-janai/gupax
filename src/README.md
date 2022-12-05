# Gupax source
* [Structure](#Structure)
* [Thread Model](#Thread-Model)
* [Bootstrap](#Bootstrap)
* [Disk](#Disk)
* [Scale](#Scale)
* [Naming Scheme](#naming-scheme)

## Structure
| File/Folder  | Purpose |
|--------------|---------|
| constants.rs | General constants needed in Gupax
| disk.rs      | Code for writing to disk: `state.toml/node.toml/pool.toml`; This holds the structs for the [State] struct
| ferris.rs    | Cute crab bytes
| gupax.rs     | `Gupax` tab
| helper.rs   | The "helper" thread that runs for the entire duration Gupax is alive. All the processing that needs to be done without blocking the main GUI thread runs here, including everything related to handling P2Pool/XMRig
| main.rs      | The main `App` struct that holds all data + misc data/functions
| node.rs      | Community node ping code for the `P2Pool` simple tab
| p2pool.rs    | `P2Pool` tab
| status.rs    | `Status` tab
| update.rs    | Update code for the `Gupax` tab
| xmrig.rs     | `XMRig` tab

## Thread Model
![thread_model.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/thread_model.png)

Note: The process I/O model depends on if the `[Simple]` or `[Advanced]` version is used.

`[Simple]` has:
	- 1 OS thread for the watchdog (API fetching, watching signals, etc)
	- 1 OS thread (with 2 tokio tasks) for STDOUT/STDERR
	- No pseudo terminal allocated
	- No STDIN pipe

`[Advanced]` has:
	- 1 OS thread for the watchdog (API fetching, watching signals, relaying STDIN)
	- 1 OS thread for a PTY-Child combo (combines STDOUT/STDERR for me, nice!)
	- A PTY (pseudo terminal) whose underlying type is abstracted with the [`portable_pty`](https://docs.rs/portable-pty/) library

The reason `[Advanced]` is non-async is because P2Pool requires a `TTY` to take STDIN. The PTY library used, [`portable_pty`](https://docs.rs/portable-pty/), doesn't implement async traits. There seem to be tokio PTY libraries, but they are Unix-specific. Having separate PTY code for Windows/Unix is also a big pain. Since the threads will be sleeping most of the time (the pipes are lazily read and buffered), it's fine. Ideally, any I/O should be a tokio task, though.

## Bootstrap
This is how Gupax works internally when starting up:

1. **INIT**
	- Initialize custom console logging with `log`, `env_logger`
	- Initialize misc data (structs, text styles, thread count, images, etc)
	- Start initializing main `App` struct
	- Parse command arguments
	- Attempt to read disk files
	- If errors were found, set the `panic` error screen
	
2. **AUTO**
	- If `auto_update` == `true`, spawn auto-updating thread
	- If `auto_select` == `true`, spawn community node ping thread

3. **MAIN**
	- All data should be initialized at this point, either via `state.toml` or default options
	- Start `App` frame
	- Do `App` stuff
	- If `ask_before_quit` == `true`, ask before quitting
	- Kill processes, kill connections, exit

## Disk
Long-term state is saved onto the disk in the "OS data folder", using the [TOML](https://github.com/toml-lang/toml) format. If not found, default files will be created. Given a slightly corrupted state file, Gupax will attempt to merge it with a new default one. This will most likely happen if the internal data structure of `state.toml` is changed in the future (e.g removing an outdated setting). Merging silently in the background is a good non-interactive way to handle this. The node/pool database cannot be merged, and if given a corrupted file, Gupax will show an un-recoverable error screen. If Gupax can't read/write to disk at all, or if there are any other big issues, it will show an un-recoverable error screen.

| OS       | Data Folder                              | Example                                        |
|----------|----------------------------------------- |------------------------------------------------|
| Windows  | `{FOLDERID_LocalAppData}`                | C:\Users\Alice\AppData\Roaming\Gupax           |
| macOS    | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support/Gupax |
| Linux    | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share/gupax                 |

The current files saved to disk:
* `state.toml` Gupax state/settings
* `node.toml` The manual node database used for P2Pool advanced
* `pool.toml` The manual pool database used for XMRig advanced

Arti (Tor) also needs to save cache and state. It uses the same file/folder conventions.

## Scale
Every frame, the max available `[width, height]` are calculated, and those are used as a baseline for the Top/Bottom bars, containing the tabs and status bar. After that, all available space is given to the middle ui elements. The scale is calculated every frame so that all elements can scale immediately as the user adjusts it; this doesn't take as much CPU as you might think since frames are only rendered on user interaction. Some elements are subtracted a fixed number because the `ui.seperator()`'s add some fixed space which needs to be accounted for.

```
Main [App] outer frame (default: [1280.0, 800.0], 16:10 aspect ratio)
   ├─ TopPanel     = height: 1/12th
   ├─ BottomPanel  = height: 1/20th
   ├─ CentralPanel = height: the rest
```

## Naming Scheme
This is the internal naming scheme used by Gupax when updating/creating default folders/etc:

Windows:
- Gupax: `Gupax.exe`
- P2Pool: `P2Pool\p2pool.exe`
- XMRig: `XMRig\xmrig.exe`

macOS:
- Gupax: `Gupax.app/.../Gupax` (Gupax is packaged as an `.app` on macOS)
- P2Pool: `p2pool/p2pool`
- XMRig: `xmrig/xmrig`

Linux:
- Gupax: `gupax`
- P2Pool: `p2pool/p2pool`
- XMRig: `xmrig/xmrig`

These have to be packaged exactly with these names because the update code is case-sensitive. If an exact match is not found, it will error.

Package naming schemes:
- `gupax` - gupax-vX.X.X-(windows|macos|linux)-x64(standalone|bundle).(zip|tar.gz)
- `p2pool` - p2pool-vX.X.X-(windows|macos|linux)-x64.(zip|tar.gz)
- `xmrig` - xmrig-X.X.X-(msvc-win64|macos-x64|linux-static-x64).(zip|tar.gz)

Exceptions (there are always exceptions...):
- XMRig doesn't have a [v], so it is [xmrig-6.18.0-...]
- XMRig separates the hash and signature
- P2Pool hashes are in UPPERCASE
