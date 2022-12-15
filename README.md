# WORK IN PROGRESS - ETA: December 25th, 2022
![banner.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/banner.png)
Gupax is a (Windows|macOS|Linux) GUI for mining [**Monero**](https://github.com/monero-project/monero) on [**P2Pool**](https://github.com/SChernykh/p2pool), using [**XMRig**](https://github.com/xmrig/xmrig).

**To see a 3-minute video on how to download and run Gupax: [click here.](#How-To)**

## Contents
* [What is Monero/P2Pool/XMRig/Gupax?](#what-is-monero-p2pool-xmrig-and-gupax)
* [How-To](#How-To)
* [Simple](#Simple)
	- [Gupax](#Gupax)
	- [P2Pool](#P2Pool)
	- [XMRig](#XMRig)
* [Advanced](#Advanced)
	- [Verifying](#Verifying)
	- [Command Line](#Command-Line)
	- [Resolution](#Resolution)
	- [Tor/Arti](#TorArti)
	- [Logs](#Logs)
	- [Disk](#Disk)
	- [Swapping P2Pool/XMRig](#Swapping-P2PoolXMRig)
	- [Gupax](#Gupax)
	- [P2Pool](#P2Pool)
	- [XMRig](#XMRig)
* [Connections](#Connections)
* [Community Monero Nodes](#community-monero-nodes)
* [Build](#Build)
	- [General Info](#General-Info)
	- [Linux](#Linux)
	- [macOS](#macOS)
	- [Windows](#Windows)
* [FAQ](#FAQ)
	- [Where are updates downloaded from?](#where-are-updates-downloaded-from)
	- [Can I quit mid-update?](#can-i-quit-mid-update)
	- [Bundled vs Standalone](#bundled-vs-standalone)
	- [How much memory does Gupax use?](#how-much-memory-does-gupax-use)
	- [How is sudo handled? (on macOS/Linux)](#how-is-sudo-handled-on-macoslinux)
	- [Why does Gupax need to be Admin? (on Windows)](#why-does-gupax-need-to-be-admin-on-windows)

## What is Monero/P2Pool/XMRig/Gupax?
**Monero** is a secure, private, and untraceable cryptocurrency.

The [Monero GUI](https://github.com/monero-project/monero-gui) software lets you run a Monero node (among other things). A Monero node connects you to other peers and lets you download Monero's [blockchain](https://en.wikipedia.org/wiki/Blockchain).

***[More info here.](https://github.com/monero-project/monero)***

---

**P2Pool** is software that lets you create/join decentralized peer-to-peer Monero mining pools.

P2Pool as a concept was [first developed for Bitcoin](https://en.bitcoin.it/wiki/P2Pool) but was [never fully realized](https://github.com/p2pool/p2pool) due to many limitations. These limitations were fixed when SChernykh rewrote P2Pool from scratch for Monero. P2Pool combines the best of solo mining and traditional pool mining:

* ***It's decentralized:*** There's no central server that can be shutdown or pool admin that controls your hashrate
* ***It's permissionless:*** It's peer-to-peer so there's no one to decide who can and cannot mine on the pool
* ***It's trustless:*** Funds are never in custody, all pool blocks pay out to miners directly and immediately
* **0% transaction fee, 0 payout fee, immediate ~0.0003 XMR minimum payout**

***[More info here.](https://github.com/SChernykh/p2pool)***

---

**XMRig** is an optimized miner which mines Monero at higher speeds.

Both Monero and P2Pool have built in miners but XMRig is quite faster than both of them. Due to issues like [anti-virus flagging](https://github.com/monero-project/monero-gui/pull/3829#issuecomment-1018191461), it is not feasible to integrate XMRig directly into Monero or P2Pool, however, XMRig is still freely available for anyone to download with the caveat being: you have to set it up yourself.

***[More info here.](https://github.com/xmrig/xmrig)***

---

**Gupax** is a GUI that helps with configuring, updating, and managing P2Pool & XMRig (both originally CLI-only).

***Recap:***
1. **XMRig** mines to **P2Pool** which fetchs blocks from a **Monero node**
2. **Monero GUI** runs the ***Monero node***
3. **Gupax** runs ***P2Pool/XMRig***

![local.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/local.png)

By default, though, Gupax will use a [Community Monero Node](#community-monero-nodes) so you don't even have to run your own full Monero node to start mining on P2Pool:

![community.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/community.png)

## How-To
https://user-images.githubusercontent.com/101352116/207978455-6ffdc0cc-204c-4594-9a2f-e10c505745bc.mp4

1. [Download the bundled version of Gupax for your OS here](https://github.com/hinto-janaiyo/gupax/releases) or from [gupax.io](https://gupax.io/downloads)
2. Extract somewhere (Desktop, Documents, etc)
3. Launch Gupax
4. Input your Monero address in the `[P2Pool]` tab
5. Select a [`Community Monero Node`](#community-monero-nodes) that you trust
5. Start P2Pool
6. Start XMRig

You are now mining to your own instance of P2Pool, welcome to the world of decentralized peer-to-peer mining!

_Notes:_
- _[What is bundled? What is standalone?](#bundled-vs-standalone)_
- _If you'd like to get deeper into the settings, see [Advanced.](#advanced)_

## Simple
### Gupax

---

### P2Pool

---

### XMRig

## Advanced
### Verifying
It is recommended to verify the hash and PGP signature of the download before using Gupax.

Download the [`SHA256SUM`](https://github.com/hinto-janaiyo/gupax/releases/latest) file, download and import my [`PGP key`](https://github.com/hinto-janaiyo/gupax/blob/main/pgp/hinto-janaiyo.asc), and verify:
```bash
sha256sum -c SHA256SUM
gpg --import hinto-janaiyo.asc
gpg --verify SHA256SUM
```

Q: How can I be sure the P2Pool/XMRig bundled with Gupax hasn't been tampered with?  
A: Verify the hash.

You can always compare the hash of the `P2Pool/XMRig` bundled with Gupax with the official hashes found here:
- https://github.com/SChernykh/p2pool/releases
- https://github.com/xmrig/xmrig/releases

Make sure the _version_ you are comparing against is correct. If they match, you can be sure they are the exact same. Verifying the PGP signature is also recommended:
- P2Pool - [`SChernykh.asc`](https://github.com/monero-project/gitian.sigs/blob/master/gitian-pubkeys/SChernykh.asc)
- XMRig - [`xmrig.asc`](https://github.com/xmrig/xmrig/blob/master/doc/gpg_keys/xmrig.asc)
 
---

### Command Line
Gupax comes with some command line options:
```
USAGE: ./gupax [--flag]

    --help         Print this help message
    --version      Print version and build info
    --state        Print Gupax state
    --nodes        Print the manual node list
    --no-startup   Disable all auto-startup settings for this instance
    --reset-state  Reset all Gupax state (your settings)
    --reset-nodes  Reset the manual node list in the [P2Pool] tab
    --reset-pools  Reset the manual pool list in the [XMRig] tab
    --reset-all    Reset the state, the manual node list, and the manual pool list
    --ferris       Print an extremely cute crab
```

By default, Gupax has `auto-update` & `auto-ping` enabled. This can only be turned off in the GUI which causes a chicken-and-egg problem. To get around this, start Gupax with `--no-startup`. This will disable all `auto` features for that instance.

---

### Resolution
The default resolution of Gupax is `1280x960` which is a `4:3` aspect ratio.

This can be changed by dragging the corner of the window itself or by using the resolution sliders in the `Gupax Advanced` tab. After a resolution change, Gupax will fade-in/out of black and will take a second to resize all the UI elements to scale correctly to the new resolution.

If you have changed your OS's pixel scaling, you may need to resize Gupax to see all UI correctly.

The minimum window size is: `640x480`  
The maximum window size is: `2560x1920`  
Fullscreen mode can also be entered by pressing `F11`.

---

### Tor/Arti
By default, Gupax updates via Tor. In particular, it uses [`Arti`](https://gitlab.torproject.org/tpo/core/arti), the official Rust implementation of Tor.

Instead of bootstrapping onto the Tor network every time, Arti saves state/cache about the Tor network (circuits, guards, etc) for later reuse onto the disk:

State:
| OS       | Data Folder                                                   |
|----------|---------------------------------------------------------------|
| Windows  | `C:\Users\USER\AppData\Local\torproject\Arti\data`            |
| macOS    | `/Users/USER/Library/Application Support/org.torproject.Arti` |
| Linux    | `/home/USER/.local/share/arti`                                |

Cache:
| OS       | Data Folder                                                   |
|----------|---------------------------------------------------------------|
| Windows  | `C:\Users\USER\AppData\Local\torproject\Arti\cache`           |
| macOS    | `/Users/USER/Library/Caches/org.torproject.Arti`              |
| Linux    | `/home/USER/.cache/arti`                                      |

---

### Disk
Long-term state is saved onto the disk in the "OS data folder", using the [TOML](https://github.com/toml-lang/toml) format. If not found, default files will be created.

Given a slightly corrupted `state.toml` file, Gupax will attempt to merge it with a new default one. This will most likely happen if the internal data structure of `state.toml` is changed in the future (e.g removing an outdated setting). The node/pool database cannot be merged.

If Gupax can't read/write to disk at all, or if there are any other big issues, it will show an un-recoverable error screen.

| OS       | Data Folder                              | Example                                         |
|----------|------------------------------------------|-------------------------------------------------|
| Windows  | `{FOLDERID_RoamingAppData}`              | `C:\Users\USER\AppData\Roaming\Gupax`           |
| macOS    | `$HOME`/Library/Application Support      | `/Users/USER/Library/Application Support/Gupax` |
| Linux    | `$XDG_DATA_HOME` or `$HOME`/.local/share | `/home/USER/.local/share/gupax`                 |

The current files saved to disk:
* `state.toml` Gupax state/settings
* `node.toml` The manual node database used for P2Pool advanced
* `pool.toml` The manual pool database used for XMRig advanced

---

### Logs
Gupax has console logs that show with increasing detail, what exactly it is is doing.

There are multiple log filter levels but by default, `INFO` and above are enabled.
To view more detailed console debug information, start Gupax with the environment variable `RUST_LOG` set to a log level like so:
```bash
RUST_LOG=(trace|debug|info|warn|error) ./gupax
```
For example:
```bash
RUST_LOG=debug ./gupax
```

In general:
- `ERROR` means something has gone wrong and that something will likely break
- `WARN` means something has gone wrong, but things will likely be fine
- `INFO` logs are general info about what Gupax (the GUI thread) is currently doing
- `DEBUG` logs are much more verbose and include what EVERY thread is doing (not just the main GUI thread)
- `TRACE` logs are insanely verbose and shows the logs of the libraries Gupax uses (HTTP connections, GUI polling, etc)

---

### Swapping P2Pool/XMRig
If you downloaded Gupax standalone and want to use your own `P2Pool/XMRig` binaries and/or want to swap them, you can:
- Edit the PATH in `Gupax Advanced` to point at the new binaries
- Change the binary itself

By default, Gupax will look for `P2Pool/XMRig` in folders next to itself:

Windows:
```
Gupax\
├─ Gupax.exe
├─ P2Pool\
│  ├─ p2pool.exe
├─ XMRig\
   ├─ xmrig.exe
```

macOS (Gupax is packaged as an `.app` on macOS):
```
Gupax.app/Contents/MacOS/
├─ gupax
├─ p2pool/
│  ├─ p2pool
├─ xmrig/
   ├─ xmrig
```

Linux:
```
gupax/
├─ gupax
├─ p2pool/
│  ├─ p2pool
├─ xmrig/
   ├─ xmrig
```

Be warned: Gupax will use your custom PATH/binary and will replace them if you use `Check for updates` in the `[Gupax]` tab.

---

### Gupax

---

### P2Pool

---

### XMRig

## Connections
For transparency, here's all the connections Gupax makes:

| Domain             | Why                                                   | When | Where |
|--------------------|-------------------------------------------------------|------|-------|
| https://github.com | Fetching metadata information on packages + download  | `[Gupax]` tab -> `Check for updates` | [`update.rs`](https://github.com/hinto-janaiyo/gupax/blob/main/src/update.rs) |
| Community Monero Nodes | Connecting to with P2Pool, measuring ping latency | `[P2Pool Simple]` tab | [`node.rs`](https://github.com/hinto-janaiyo/gupax/blob/main/src/node.rs) |
| DNS | DNS connections will usually be handled by your OS (or whatever custom DNS setup you have). If using Tor, DNS requests for updates [*should*](https://tpo.pages.torproject.net/core/doc/rust/arti/) be routed through the Tor network automatically | All of the above | All of the above |

## Community Monero Nodes
| Name                                                  | IP/Domain                        | RPC Port | ZMQ Port |
|-------------------------------------------------------|----------------------------------|----------|----------|
| [C3pool](https://www.c3pool.com)                      | node.c3pool.com                  | 18081    | 18083    |
| [Cake](https://cakewallet.com)                        | xmr-node.cakewallet.com          | 18081    | 18083    |
| [CakeEu](https://cakewallet.com)                      | xmr-node-eu.cakewallet.com       | 18081    | 18083    |
| [CakeUk](https://cakewallet.com)                      | xmr-node-uk.cakewallet.com       | 18081    | 18083    |
| [CakeUs](https://cakewallet.com)                      | xmr-node-usa-east.cakewallet.com | 18081    | 18083    |
| [Feather1](https://github.com/feather-wallet/feather) | selsta1.featherwallet.net        | 18081    | 18083    |
| [Feather2](https://github.com/feather-wallet/feather) | selsta2.featherwallet.net        | 18081    | 18083    |
| [MajesticBankIs](https://www.majesticbank.sc)         | node.majesticbank.is             | 18089    | 18083    |
| [MajesticBankSu](https://www.majesticbank.sc)         | node.majesticbank.su             | 18089    | 18083    |
| [Monerujo](https://www.monerujo.io)                   | nodex.monerujo.io                | 18081    | 18083    |
| [Plowsof1](https://github.com/plowsof)                | node.monerodevs.org              | 18089    | 18084    |
| [Plowsof2](https://github.com/plowsof)                | node2.monerodevs.org             | 18089    | 18084    |
| [Rino](https://cakewallet.com)                        | node.community.rino.io           | 18081    | 18083    |
| [Seth](https://github.com/sethforprivacy)             | node.sethforprivacy.com          | 18089    | 18083    |
| [SupportXmr](https://www.supportxmr.com)              | node.supportxmr.com              | 18081    | 18083    |
| [SupportXmrIr](https://www.supportxmr.com)            | node.supportxmr.ir               | 18089    | 18083    |
| [XmrVsBeast](https://xmrvsbeast.com)                  | p2pmd.xmrvsbeast.com             | 18081    | 18083    |

## Build
### General Info
You need [`cargo`](https://www.rust-lang.org/learn/get-started), Rust's build tool and package manager.

The `--release` profile in Gupax is set to prefer code performance & small binary sizes over compilation speed (see [`Cargo.toml`](https://github.com/hinto-janaiyo/gupax/blob/main/Cargo.toml)). Gupax itself (with all dependencies already built) takes around 1m30s to build (vs 10s on a normal `--release`) with a Ryzen 5950x.

---

### Linux
You'll need the development versions of libraries like `OpenSSL`, `SQLite`, and maybe some other ones already installed on your system. Read the compiler errors to see which ones are missing from your system and search around to see which packages you'll need to install depending on your distro.

After that, run:
```
cargo build --release
```

---

### macOS
You'll need [`Xcode`](https://developer.apple.com/xcode/).

On macOS, if you want the binary to have an icon, you must install [`cargo-bundle`](https://github.com/burtonageo/cargo-bundle) and compile with:
```
cargo bundle --release
```
This bundles Gupax into a `Gupax.app`, the way it comes in the pre-built tars for macOS.

---

### Windows
You'll need [`Visual Studio`](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup).

There is a `build.rs` file in the repo solely for Windows-specific things:
1. It sets the icon in `File Explorer`
2. It statically links `VCRUNTIME140.dll` into Gupax (the binary will not be portable without this)

After installing the development tools, run:
```
cargo build --release
```

This will build Gupax with the MSVC toolchain (`x86_64-pc-windows-msvc`). This is the recommended method and is how the pre-compiled release binaries are built.

## FAQ
### Where are updates downloaded from?
The latest versions are downloaded using the GitHub API.
* Gupax [`https://github.com/hinto-janaiyo/gupax`](https://github.com/hinto-janaiyo/gupax)
* P2Pool [`https://github.com/SChernykh/p2pool`](https://github.com/SChernykh/p2pool)
* XMRig [`https://github.com/xmrig/xmrig`](https://github.com/xmrig/xmrig)

GitHub's API blocks request that do not have an HTTP `User-Agent` header. [For privacy, Gupax randomly uses a recent version of a `Wget/Curl` user-agent.](https://github.com/hinto-janaiyo/gupax/blob/2b80aa027728ddd193bac2e77caa5ddb4323f8fd/src/update.rs#L134)

---

### Can I quit mid-update?
If you started an update, you should let it finish. If the update has been stuck for a *long* time, it may be worth quitting Gupax. The worst that can happen is that your `Gupax/P2Pool/XMRig` binaries may be moved/deleted. Those can be easily redownloaded. Your actual `Gupax` user data (settings, custom nodes, pools, etc) is never touched.

Although Gupax uses a temporary folder (`gupax_update_[A-Za-z0-9]`) to store temporary downloaded files, there aren't measures in place to revert an upgrade once the file swapping has actually started. If you quit Gupax anytime before the `Upgrading packages` phase (after metadata, download, extraction), you will technically be safe but this is not recommended as it is risky, especially since these updates can be very fast.

---

### Bundled vs Standalone
`Bundled` Gupax comes the latest version of P2Pool/XMRig already in the `zip/tar`.

`Standlone` only contains the Gupax executable.

---

### How much memory does Gupax use?
Gupax itself uses around 100-300 megabytes of memory.

Gupax also holds up to [500,000 bytes](https://github.com/hinto-janaiyo/gupax/blob/2b80aa027728ddd193bac2e77caa5ddb4323f8fd/src/helper.rs#L63) of log data from `P2Pool/XMRig` to display in the GUI terminals. These logs are reset once over capacity which takes around 1-2 hours.

Memory usage should *never* be above 400~ megabytes. If you see Gupax using more than this, please send a bug report.

---

### How is sudo handled? (on macOS/Linux)
[See here for more info.](https://github.com/hinto-janaiyo/gupax/tree/main/src#sudo)

---

### Why does Gupax need to be Admin? (on Windows)
[See here for more info.](https://github.com/hinto-janaiyo/gupax/tree/main/src#why-does-gupax-need-to-be-admin-on-windows)
