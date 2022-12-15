# WORK IN PROGRESS - ETA: December 25th, 2022
![banner.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/banner.png)
Gupax is a (Windows|macOS|Linux) GUI for mining [**Monero**](https://github.com/monero-project/monero) on [**P2Pool**](https://github.com/SChernykh/p2pool), using [**XMRig**](https://github.com/xmrig/xmrig).

**To see a 1-minute video on how to download and run Gupax: [click here.](#Video)**

## Contents
* [What is Monero/P2Pool/XMRig/Gupax?](#what-is-monero-p2pool-xmrig-and-gupax)
* [How-To](#How-To)
	- [Video](#Video)
	- [Text](#Text)
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
	- [How much memory does Gupax use?](#how-much-memory-does-gupax-use)
	- [How is sudo handled? (on macOS/Linux)](#how-is-sudo-handled-on-macos-linux)
	- [Why does Gupax need to be Admin? (on Windows)](#why-does-gupax-need-to-be-admin-on-windows)

## What is Monero/P2Pool/XMRig/Gupax?
**Monero** is a secure, private, and untraceable cryptocurrency.

The **[Monero GUI](https://github.com/monero-project/monero-gui)** software lets you run a **Monero node** (among other things). A Monero node connects you to other peers and lets you download Monero's [blockchain](https://en.wikipedia.org/wiki/Blockchain).

***[More info here.](https://github.com/monero-project/monero)***

---

**P2Pool** is software that lets you create/join **decentralized peer-to-peer Monero mining pools.**

P2Pool as a concept was [first developed for Bitcoin](https://en.bitcoin.it/wiki/P2Pool) but was [never fully realized](https://github.com/p2pool/p2pool) due to many limitations. These limitations were fixed when SChernykh rewrote P2Pool from scratch for Monero. P2Pool combines the best of solo mining and traditional pool mining:

* ***It's decentralized:*** There's no central server that can be shutdown or pool admin that controls your hashrate
* ***It's permissionless:*** It's peer-to-peer so there's no one to decide who can and cannot mine on the pool
* ***It's trustless:*** Funds are never in custody, all pool blocks pay out to miners directly and immediately
* **0% transaction fee, 0 payout fee, immediate ~0.0003 XMR minimum payout**

***[More info here.](https://github.com/SChernykh/p2pool)***

---

**XMRig** is an optimized miner which **mines Monero at higher speeds.**

Both Monero and P2Pool have built in miners but XMRig is quite faster than both of them. Due to issues like [anti-virus flagging](https://github.com/monero-project/monero-gui/pull/3829#issuecomment-1018191461), it is not feasible to integrate XMRig directly into Monero or P2Pool, however, XMRig is still freely available for anyone to download with the caveat being: you have to set it up yourself.

***[More info here.](https://github.com/xmrig/xmrig)***

---

**Gupax** is a GUI that helps with configuring, updating, and managing P2Pool & XMRig (both originally CLI-only).

***Recap:***
1. **XMRig** mines to **P2Pool** which fetchs blocks from a **Monero node**
2. **Monero GUI** runs the ***Monero node***
3. **Gupax** runs ***P2Pool/XMRig***

![stack.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/diagram.png)

With Monero GUI managing the Monero node on one side and Gupax managing P2Pool/XMRig on the other, it is (hopefully) very easy for anyone to start mining Monero at **max hashrate in a decentralized, permissionless, and trustless manner**.

## How-To
### Video
### Text

## Simple
### Gupax

---

### P2Pool

---

### XMRig

## Advanced
### Verifying

---

### Command Line

---

### Tor/Arti

---

### Logs

---

### Disk

---

### Swapping P2Pool/XMRig

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
| DNS | DNS connections will usually be handled by your OS (or whatever custom DNS setup you have). If using Tor, DNS requests [*should*](https://tpo.pages.torproject.net/core/doc/rust/arti/) be routed through the Tor network automatically | Same as above | Same as above |
| Community Monero Nodes | Connecting to with P2Pool, measuring ping latency | `[P2Pool Simple]` tab | [`node.rs`](https://github.com/hinto-janaiyo/gupax/blob/main/src/node.rs) |

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
Although Gupax uses a temporary folder (`gupax_update_[A-Za-z0-9]`) to store temporary downloaded files, there aren't measures in place to revert an upgrade once the file swapping has actually started. If you quit Gupax anytime before the `Upgrading packages` phase (after metadata, download, extraction), you will technically be safe but this is not recommended as it is risky, especially since these updates can be very fast.

If you started an update, you should let it finish. If the update has been stuck for a *long* time, it may be worth quitting Gupax. The worst that can happen is that your `Gupax/P2Pool/XMRig` binaries may be moved/deleted. Those can be easily redownloaded. Your actual `Gupax` user data (settings, custom nodes, pools, etc) is never touched.

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
