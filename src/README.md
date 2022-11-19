# Gupax source files
* [Structure](#Structure)
* [Bootstrap](#Bootstrap)
* [State](#State)
* [Scale](#Scale)

## Structure
| File/Folder    | Purpose |
|----------------|---------|
| `constants.rs` | General constants needed in Gupax
| `disk.rs`      | Code for writing to disk: `state.toml`, `node.toml`; This holds the structs for mutable [State]
| `ferris.rs`    | Cute crab bytes
| `gupax.rs`     | `Gupax` tab
| `main.rs`      | `App/Tab/State` + misc data/functions
| `node.rs`      | Community node ping code for the `P2Pool` simple tab
| `p2pool.rs`    | `P2Pool` tab
| `status.rs`    | `Status` tab
| `update.rs`    | Update code for the `Gupax` tab
| `xmrig.rs`     | `XMRig` tab

## Bootstrap
This is how Gupax works internally when starting up, divided into 3 sections.

1. **INIT**
	- Initialize custom console logging with `log`, `env_logger`
	- Initialize misc data (structs, text styles, thread count, images, etc)
	- Check for admin privilege (for XMRig)
	- Start initializing main `App` struct
	- Parse command arguments
	- Attempt to read disk files `state.toml`, `node.toml`
	- If errors were found, pop-up window
	
2. **AUTO**
	- If `auto_update` == `true`, spawn auto-updating thread
	- If `auto_select` == `true`, spawn community node ping thread

3. **MAIN**
	- All data should be initialized at this point, either via `state.toml` or default options
	- Start `App` frame
	- Do `App` stuff
	- If `ask_before_quit` == `true`, ask before quitting
	- Kill processes, kill connections, exit

## State
Internal state is saved in the "OS data folder" as `state.toml`, using the [TOML](https://github.com/toml-lang/toml) format. If not found, a default `state.toml` file will be created. Given a slightly corrupted state file, Gupax will attempt to merge it with a new default one. This will most likely happen if the internal data structure of `state.toml` is changed in the future (e.g removing an outdated setting). Merging silently in the background is a good non-interactive way to handle this. If Gupax can't read/write to disk at all, or if there are any other big issues, it will show an un-recoverable error window.

| OS       | Data Folder                              | Example                                                   |
|----------|----------------------------------------- |-----------------------------------------------------------|
| Windows  | `{FOLDERID_LocalAppData}`                | C:\Users\Alice\AppData\Roaming\Gupax\gupax.toml           |
| macOS    | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support/Gupax/gupax.toml |
| Linux    | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share/gupax/gupax.toml                 |

## Scale
Every frame, the max available `[width, height]` are calculated, and those are used as a baseline for the Top/Bottom bars, containing the tabs and status bar. After that, all available space is given to the middle ui elements. The scale is calculated every frame so that all elements can scale immediately as the user adjusts it; this doesn't take as much CPU as you might think since frames are only rendered on user interaction. Some elements are subtracted a fixed number because the `ui.seperator()`s add some fixed space which needs to be accounted for.

```
Main [App] outer frame (default: [1280.0, 720.0])
   ├─ TopPanel     = height: 1/12th
   ├─ BottomPanel  = height: 1/20th
   ├─ CentralPanel = height: the rest
```
