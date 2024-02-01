# v1.3.6
## Changes
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- Removed `bunkernet.ddns.net`
	- Removed `ru.poiuty.com`
	- Added `xmr{1,2,3}.rs.me` (thanks @SChernykh [#80](https://github.com/hinto-janai/gupax/pull/80))

## Bundled Versions
* [`P2Pool v3.10`](https://github.com/SChernykh/p2pool/releases/tag/v3.10)
* [`XMRig v6.21.0`](https://github.com/xmrig/xmrig/releases/tag/v6.21.0)


---


# v1.3.5
## Fixes
* Fix flickering `0s` XMRig uptime (thanks @Tomoyukiryu & @Burner8 [#77](https://github.com/hinto-janai/gupax/pull/77))

## Bundled Versions
* [`P2Pool v3.10`](https://github.com/SChernykh/p2pool/releases/tag/v3.10)
* [`XMRig v6.21.0`](https://github.com/xmrig/xmrig/releases/tag/v6.21.0)


---


# v1.3.4
## Fixes
* Domain parsing is more relaxed, allows subdomains with longer TLDs (thanks @soupslurpr [#67](https://github.com/hinto-janai/gupax/pull/67))
* ANSI escape sequences in Windows P2Pool/XMRig terminal output ([#71](https://github.com/hinto-janai/gupax/pull/71))
* P2Pool appearing green (synchronized) on false-positives ([#75](https://github.com/hinto-janai/gupax/pull/75))

## Bundled Versions
* [`P2Pool v3.10`](https://github.com/SChernykh/p2pool/releases/tag/v3.10)
* [`XMRig v6.21.0`](https://github.com/xmrig/xmrig/releases/tag/v6.21.0)


---


# v1.3.3
## Changes
* Crashes will now create a file on disk with debug information ([#59](https://github.com/hinto-janai/gupax/pull/59))
	| OS       | Path |
	|----------|------|
	| Windows  | `C:\Users\USER\AppData\Roaming\Gupax\crash.txt`           |
	| macOS    | `/Users/USER/Library/Application Support/Gupax/crash.txt` |
	| Linux    | `/home/USER/.local/share/gupax/crash.txt`                 |
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- Removed `xmr.theuplink.net`

## Fixes
* P2Pool `[Simple]`'s backup hosts option will only include green/yellow nodes (<300ms ping) ([#65](https://github.com/hinto-janai/gupax/pull/65))
* P2Pool ping now verifies node is synchronized ([#63](https://github.com/hinto-janai/gupax/pull/63))
* XMRig `[Simple]` tab slider overflow ([#60](https://github.com/hinto-janai/gupax/pull/60))
* P2Pool `[Simple]` tab height overflow (https://github.com/hinto-janai/gupax/commit/b4a4e83457c8fc353e75ddf6c284bd41422f0db4)

## Bundled Versions
* [`P2Pool v3.9`](https://github.com/SChernykh/p2pool/releases/tag/v3.9)
* [`XMRig v6.21.0`](https://github.com/xmrig/xmrig/releases/tag/v6.21.0)


---


# v1.3.2
## Updates
* Added window scaling option (`0.1..2.0` pixel scaling multiplier)
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- Removed `oracle.netrix.cc`

## Bundled Versions
* [`P2Pool v3.7`](https://github.com/SChernykh/p2pool/releases/tag/v3.7)
* [`XMRig v6.20.0`](https://github.com/xmrig/xmrig/releases/tag/v6.20.0)


---


# v1.3.1
## Fixes
* Auto update issue ([#40](https://github.com/hinto-janai/gupax/issues/40))
* macOS launch issue ([#39](https://github.com/hinto-janai/gupax/issues/39))

## Bundled Versions
* [`P2Pool v3.5`](https://github.com/SChernykh/p2pool/releases/tag/v3.5)
* [`XMRig v6.20.0`](https://github.com/xmrig/xmrig/releases/tag/v6.20.0)


---


# v1.3.0
## Updates
* Added P2Pool [backup host support](https://github.com/SChernykh/p2pool/blob/master/docs/COMMAND_LINE.MD#multiple-backup-hosts). `[Simple]` will fallback to next fastest nodes, `[Advanced]` will fallback to all other nodes in list.
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- Added `sf.xmr.support`
	- Added `node.cryptocano.de`
	- Added `bunkernet.ddns.net`
	- Added `oracle.netrix.cc`
	- Added `fbx.tranbert.com`
	- Removed `xmr.aa78i2efsewr0neeknk.xyz`
	- Removed `node.yeetin.me`
	- Removed `monero2.10z.com.ar`
	- Removed `node.moneroworld.com`
	- Removed `de.poiuty.com`
	- Removed `m1.poiuty.com`
	- Removed `reynald.ro`
	- Removed `monero.homeqloud.com`
	- Removed `xmr.foxpro.su`
	- Removed `radishfields.hopto.org`
	- Removed `node.sethforprivacy.com`

## Bundled Versions
* [`P2Pool v3.5`](https://github.com/SChernykh/p2pool/releases/tag/v3.5)
* [`XMRig v6.20.0`](https://github.com/xmrig/xmrig/releases/tag/v6.20.0)


---


# v1.2.3
## Updates
* Added ARM (Apple Silicon) macOS releases (bundle includes ARM P2Pool/XMRig)
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- Added `xmr.support`
	- Added `xmr.theuplink.net`

## Fixes
* Fixed macOS Tor+TLS issue, updates are now via Tor by default ([#28](https://github.com/hinto-janai/gupax/issues/28))
* Fixed undisplayable ANSI codes in P2Pool's terminal ([#24](https://github.com/hinto-janai/gupax/issues/34))

## Bundled Versions
* [`P2Pool v3.4`](https://github.com/SChernykh/p2pool/releases/tag/v3.4)
* [`XMRig v6.19.3`](https://github.com/xmrig/xmrig/releases/tag/v6.19.3)


---


# v1.2.2
## Updates
* **UI:** Changed overall style (all text is monospace, darker theme, rounded corners)
* **P2Pool:** Color status is now `ORANGE` until synchronized
* **XMRig:** Color status is now `ORANGE` when not mining
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- Added `node.sethforprivacy.com`
	- Added `node.moneroworld.com`
	- Added `node.yeetin.me`
	- Added `xmr.foxpro.su`

## Fixes
* Fixed `[Status]` P2Pool stats overflowing sometimes
* Added help messages on config loading issues
* Fixed rare crash upon bad config data

## Bundled Versions
* [`P2Pool v3.3`](https://github.com/SChernykh/p2pool/releases/tag/v3.3)
* [`XMRig v6.19.2`](https://github.com/xmrig/xmrig/releases/tag/v6.19.2)


---


# v1.2.1
## Fixes
* Small internal change to be compatible with `P2Pool v3.2`

## Bundled Versions
* [`P2Pool v3.2`](https://github.com/SChernykh/p2pool/releases/tag/v3.2)
* [`XMRig v6.19.1`](https://github.com/xmrig/xmrig/releases/tag/v6.19.1)


---


# v1.2.0
## Updates
* Added [`Benchmarks`](https://github.com/hinto-janai/gupax#Status-1) submenu in `Status` tab
	- Your hashrate vs others with the same CPU
	- List of similar CPUs and their stats
	- Data source: [Here](https://github.com/hinto-janai/xmrig-benchmarks) & [here](https://xmrig.com/benchmark)
* [Remote Node](https://github.com/hinto-janai/gupax#remote-monero-nodes) changes:
	- `+ ADD` p2pool.uk
	- `+ ADD` xmr.aa78i2efsewr0neeknk.xyz
	- `+ ADD` monero.jameswillhoite.com
	- `- REMOVE` fbx.tranbert.com

## Bundled Versions
* [`P2Pool v3.1`](https://github.com/SChernykh/p2pool/releases/tag/v3.1)
* [`XMRig v6.19.0`](https://github.com/xmrig/xmrig/releases/tag/v6.19.0)

## PGP Key Change
Please use the new key to verify releases, [found here](https://github.com/hinto-janai/gupax/blob/main/pgp/hinto-janai.asc), [or here.](https://gupax.io/hinto)

<details>
<summary>Verification</summary>

```
-----BEGIN PGP SIGNED MESSAGE-----
Hash: SHA256

I'm slightly shortening my alias from
`hinto-janaiyo` to `hinto-janai` and changing to
this PGP key: 31C5145AAFA5A8DF1C1DB2A6D47CE05FA175A499

This message is signed with the old key,
2A8F883E016FED0380287FAFB1C5A64B80691E45

Sorry for the inconvenience.
-----BEGIN PGP SIGNATURE-----

iHUEARYIAB0WIQQqj4g+AW/tA4Aof6+xxaZLgGkeRQUCZBTRkQAKCRCxxaZLgGke
RWb4AP0T/n+2XlssCcUHh/6cNq67isJ0v10Hi/drmTPLKvNKjgEAqNavi6+sB1NQ
Eh6+zWpoydVGzFdEkE5XKmnQ1dm/GQ8=
=g9YL
-----END PGP SIGNATURE-----
```
</details>


---


# v1.1.2
## Fixes
* **Windows:** Fixed Gupax crashing on certain CPU-based graphics (integrated + basic drivers)
* **Windows:** Fixed P2Pool Advanced command inputs being ignored
* **P2Pool/XMRig:** Fixed parsing of `localhost` into `127.0.0.1`
* **P2Pool/XMRig:** Current (non-saved) text-box values are now used instead of "old" selected values for custom nodes/pools
* **Log:** Only Gupax console logs will be printed (libraries filtered out)

## Bundled Versions
* [`P2Pool v3.0`](https://github.com/SChernykh/p2pool/releases/tag/v3.0)
* [`XMRig v6.19.0`](https://github.com/xmrig/xmrig/releases/tag/v6.19.0)


---


# v1.1.1
## Updates
* **Remote Nodes:** Replaced `[Community Monero Nodes]` with known ZMQ-enabled [Remote Nodes](https://github.com/hinto-janai/gupax#remote-monero-nodes). List is sourced from this [daily-updated list based off uptime](https://github.com/hinto-janai/monero-nodes). **This should fix most P2Pool connection related issues.**
* **P2Pool:** Added warning in `[P2Pool Simple]` tab about privacy/practical downsides when using remote nodes; Hyperlinks to [Running a Local Monero Node](https://github.com/hinto-janai/gupax#running-a-local-monero-node).

## Fixes
* **Ping:** Fixed ping end lag; Remote node pings are as fast as the slowest ping instead of always taking 10 seconds flat
* **UI:** Top/Bottom bars are smaller, fixes some UI overflowing or being cramped

## Bundled Versions
* [`P2Pool v3.0`](https://github.com/SChernykh/p2pool/releases/tag/v3.0)
* [`XMRig v6.18.1`](https://github.com/xmrig/xmrig/releases/tag/v6.18.1)


---


# v1.1.0
## Updates
* **Status:** [Added P2Pool submenu](https://github.com/hinto-janai/gupax#Status)
	- Total payouts across all time
	- Total XMR mined across all time
	- Formatted log lines of ALL payouts (date, amount, block) with sorting options
	- Automatic/Manual calculator for average share/block time
	- P2Pool/Monero stats for difficulty/hashrate/dominance
* **Status:** Added more process stats
	- P2Pool
		- Current Monero node IP/RPC/ZMQ
		- Current Sidechain
		- Current Monero address
	- XMRig
		- Current Pool IP
		- Current thread usage
* **Key Shortcut:** Added two shortcuts
	- `C | Left Submenu`
	- `V | Left Submenu`
* **Command Line:** Added two flags
	- `--payouts         Print the P2Pool payout log, payout count, and total XMR mined`
	- `--reset-payouts   Reset the permanent P2Pool stats that appear in the [Status] tab`
	
## Fixes
* **macOS:** Added warning (and solution) if `Gupax/P2Pool/XMRig` were quarantined by [`Gatekeeper`](https://support.apple.com/en-us/HT202491)
* **P2Pool/XMRig:** Added a red `Start` button on errors (bad PATH, invalid file, etc) and a solution in the tooltip
* **P2Pool/XMRig:** Fixed processes sometimes not starting after entering a custom PATH
* **P2Pool:** Fixed custom node selection sometimes using old values after save
* Miscellaneous UI changes and fixes

## Bundled Versions
* [`P2Pool v2.7`](https://github.com/SChernykh/p2pool/releases/tag/v2.7)
* [`XMRig v6.18.1`](https://github.com/xmrig/xmrig/releases/tag/v6.18.1)


---


# v1.0.0
[Download here](https://github.com/hinto-janai/gupax/releases/latest) or at https://gupax.io.

[Watch a 3-minute setup guide here.](https://github.com/hinto-janai/gupax#How-To)

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
	- [Community Monero node selector](https://github.com/hinto-janai/gupax/tree/main/README.md#community-monero-nodes) 
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
	```
* Added fullscreen GUI error handler (Error message + UI buttons for response, Yes/No, Quit, etc)
* Added a native `File Explorer/Finder/GTK` file selector for picking P2Pool/XMRig binary path in `Gupax` tab
* Added detailed console log levels `RUST_LOG=(trace|debug|info|warn|error) ./gupax`
* [Added new PGP key](https://github.com/hinto-janai/gupax/blob/main/pgp/hinto-janai.asc)
* Created website (HTML/CSS only, no JavaScript): https://gupax.io


---


## v0.1.0
## Prototype Release
* Added package updater (by default, via Tor using [`Arti`](https://blog.torproject.org/arti_100_released/))
* Added [custom icons per OS](https://github.com/hinto-janai/gupax/tree/main/images/icons) (File Explorer, Taskbar, Finder, App header, etc)
* Added Monero node [`JSON-RPC ping`](https://github.com/hinto-janai/gupax/blob/main/src/node.rs) system, not yet in GUI
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
