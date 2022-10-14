# Gupax source files
* [Structure](#Structure)
* [Bootstrap](#Bootstrap)
* [State](#State)

## Structure
| File/Folder    | Purpose |
|----------------|---------|
| `about.rs`     | Struct/impl for `About` tab
| `constants.rs` | General constants needed in Gupax
| `gupax.rs`     | Struct/impl for `Gupax` tab
| `main.rs`      | Struct/enum/impl for `App/Tab/State`, init functions, main function
| `node.rs`      | Struct/impl for Community Nodes
| `p2pool.rs`    | Struct/impl for `P2Pool` tab
| `status.rs`    | Struct/impl for `Status` tab
| `toml.rs`      | Struct/impl for `gupax.toml`, the disk state
| `xmrig.rs`     | Struct/impl for `XMRig` tab

## Bootstrap
This is how Gupax works internally when starting up, divided into 3 sections.

1. **INIT**
	- Initialize custom console logging with `log`, `env_logger` || *warn!*
	- Initialize misc data (structs, text styles, thread count, images, etc) || *panic!*
	- Check for admin privilege (for XMRig) || *warn!*
	- Attempt to read `gupax.toml` || *warn!*, *initialize config with default options*
	- If errors were found, pop-up window
	
2. **AUTO**
	- If `auto_update` == `true`, pop-up auto-updating window || *info!*, *skip auto-update*
	- Multi-threaded GitHub API check on Gupax -> P2Pool -> XMRig || *warn!*, *skip auto-update*
	- Multi-threaded download if current version != new version || *warn!*, *skip auto-update*
	- After download, atomically replace current binaries with new || *warn!*, *skip auto-update*
	- Update version metadata || *warn!*, *skip auto-update*
	- If `auto_select` == `true`, ping community nodes and select fastest one || *warn!*

3. **MAIN**
	- All data must be initialized at this point, either via `gupax.toml` or default options || *panic!*
	- Start `App` frame || *panic!*
	- Write state to `gupax.toml` on user clicking `Save` (after checking input for correctness) || *warn!*
	- If `ask_before_quit` == `true`, check for running processes, unsaved state, and update connections before quitting
	- Kill processes, kill connections, exit

## State
Internal state is saved in the "OS data folder" as `gupax.toml`, using the [TOML](https://github.com/toml-lang/toml) format. If the version can't be parsed (not in the `vX.X.X` or `vX.X` format), the auto-updater will be skipped. [If not found, a default `gupax.toml` file will be created with `Toml::default`.](https://github.com/hinto-janaiyo/gupax/blob/main/src/toml.rs) Gupax will `panic!` if `gupax.toml` has IO or parsing issues.

| OS       | Data Folder                              | Example                                                   |
|----------|----------------------------------------- |-----------------------------------------------------------|
| Windows  | `{FOLDERID_LocalAppData}`                | C:\Users\Alice\AppData\Roaming\Gupax\gupax.toml           |
| macOS    | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support/Gupax/gupax.toml |
| Linux    | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share/gupax/gupax.toml                 |
