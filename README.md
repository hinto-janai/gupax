![banner.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/banner.png)
Gupax is a (Windows|macOS|Linux) GUI for mining [**Monero**](https://github.com/monero-project/monero) on [**P2Pool**](https://github.com/SChernykh/p2pool), using [**XMRig**](https://github.com/xmrig/xmrig).

**To see a 3-minute video on how to download and run Gupax: [click here.](#How-To)**

## Contents
* [What is Monero/P2Pool/XMRig/Gupax?](#what-is-monerop2poolxmriggupax)
* [How-To](#How-To)
* [Simple](#Simple)
	- [Status](#Status)
	- [Gupax](#Gupax)
	- [P2Pool](#P2Pool)
	- [XMRig](#XMRig)
* [Advanced](#Advanced)
	- [Verifying](#Verifying)
	- [Running a Local Monero Node](#running-a-local-monero-node)
	- [Command Line](#Command-Line)
	- [Key Shortcuts](#Key-Shortcuts)
	- [Resolution](#Resolution)
	- [Tor/Arti](#TorArti)
	- [Logs](#Logs)
	- [Disk](#Disk)
	- [Swapping P2Pool/XMRig](#Swapping-P2PoolXMRig)
	- [Status](#Status-1)
	- [Gupax](#Gupax-1)
	- [P2Pool](#P2Pool-1)
	- [XMRig](#XMRig-1)
* [Connections](#Connections)
* [Community Monero Nodes](#community-monero-nodes)
* [Build](#Build)
	- [General Info](#General-Info)
	- [Linux](#Linux)
		- [Building for a distribution](#building-for-a-distribution)
	- [macOS](#macOS)
	- [Windows](#Windows)
* [License](#License)
* [FAQ](#FAQ)
	- [Where are updates downloaded from?](#where-are-updates-downloaded-from)
	- [P2Pool connection errors](#p2pool-connection-errors)
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
The `Gupax/P2Pool/XMRig` tabs have two versions, `Simple` & `Advanced`.

`Simple` is for a minimal & working out-of-the-box configuration.

---

### Status
This tab has 2 submenus that show some info about running processes and P2Pool-specific stats.

**Processes:**

![processes.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/processes.png)

**P2Pool:**

This submenu shows:
- ***Permanent*** stats on all your payouts received via P2Pool & Gupax
- Payout sorting options
- Share/block time calculator

![payouts.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/payouts.png)

---

### Gupax
This tab has the updater and general Gupax settings.

If `Check for updates` is pressed, Gupax will update your `Gupax/P2Pool/XMRig` (if needed) using the [GitHub API](#where-are-updates-downloaded-from).

Below that, there are some general Gupax settings:
| Setting            | What does it do?  |
|--------------------|-------------------| 
| `Update via Tor`   | Causes updates to be fetched via the Tor network. Tor is embedded within Gupax; a Tor system proxy is not required
| `Auto-Update`      | Gupax will automatically check for updates at startup
| `Auto-P2Pool`      | Gupax will automatically start P2Pool at startup
| `Auto-XMRig`       | Gupax will automatically start XMRig at startup
| `Ask before quit`  | Gupax will ask before quitting (and notify if there are any updates/processes still alive)
| `Save before quit` | Gupax will automatically saved any un-saved setting on quit

---

### P2Pool
P2Pool Simple allows you to ping & connect to a [Community Monero Node](#community-monero-nodes) and start your own local P2Pool instance on the `Mini` sidechain.

To start P2Pool, first input the Monero address you'd like to receive payouts from. You must use a primary Monero address to mine on P2Pool (starts with a 4). It is highly recommended to create a new wallet since addresses are public on P2Pool!

**Warning: [There are negative privacy/security implications when using a Monero node not in your control.](https://www.getmonero.org/resources/moneropedia/remote-node.html)** Select a community node that you trust, or better yet, run your own node. If you'd like to manually specify a node to connect to, see [Advanced.](#advanced)

---

### XMRig
XMRig Simple has a log output box, a thread slider, and `Pause-on-active` setting.

If XMRig is started with `Pause-on-active` with a value greater than 0, XMRig will automatically pause for that many seconds if it detects any user activity (mouse movements, keyboard clicks). [This setting is only available on Windows/macOS.](https://xmrig.com/docs/miner/config/misc#pause-on-active)

**Windows:**  
Gupax will automatically launch XMRig with administrator privileges to activate [mining optimizations.](https://xmrig.com/docs/miner/randomx-optimization-guide) XMRig also needs a [signed WinRing0 driver (© 2007-2009 OpenLibSys.org)](https://xmrig.com/docs/miner/randomx-optimization-guide/msr#manual-configuration) to access MSR registers. This is the file next to XMRig called `WinRing0x64.sys`. This comes in the bundled version of Gupax. If missing/deleted, a copy is packaged with all [Windows XMRig releases.](https://github.com/xmrig/xmrig/releases/) A direct standalone version is also provided, [here.](https://github.com/xmrig/xmrig/blob/master/bin/WinRing0/WinRing0x64.sys)

**macOS/Linux:**  
Gupax will prompt for your `sudo` password to start XMRig with and do all the things above.

XMRig Simple will always mine to your own local P2Pool (`127.0.0.1:3333`), if you'd like to manually specify a pool to mine to, see [Advanced](#advanced).

## Advanced
### Verifying
It is recommended to verify the hash and PGP signature of the download before using Gupax.

Download the [`SHA256SUMS`](https://github.com/hinto-janaiyo/gupax/releases/latest) file, download and import my [`PGP key`](https://github.com/hinto-janaiyo/gupax/blob/main/pgp/hinto-janaiyo.asc), and verify:
```bash
sha256sum -c SHA256SUMS
gpg --import hinto-janaiyo.asc
gpg --verify SHA256SUMS
```

Q: How can I be sure the P2Pool/XMRig bundled with Gupax hasn't been tampered with?  
A: Verify the hash.

You can always compare the hash of the `P2Pool/XMRig` binaries bundled with Gupax with the hashes of the binaries found here:
- https://github.com/SChernykh/p2pool/releases
- https://github.com/xmrig/xmrig/releases

Make sure the _version_ you are comparing against is correct, and make sure you are comparing the _binary_ to the _binary_, not the `tar/zip`. If they match, you can be sure they are the exact same. Verifying the PGP signature is also recommended:
- P2Pool - [`SChernykh.asc`](https://github.com/monero-project/gitian.sigs/blob/master/gitian-pubkeys/SChernykh.asc)
- XMRig - [`xmrig.asc`](https://github.com/xmrig/xmrig/blob/master/doc/gpg_keys/xmrig.asc)
 
---

### Running a Local Monero Node
Running and using your own local Monero node improves privacy and security. It also means you won't be depending on one of the [Community Monero Nodes](#community-monero-nodes) provided by Gupax. This comes at the cost of downloading and syncing Monero's blockchain yourself (currently `155GB`).

If you'd like to run and use your own local Monero node for P2Pool, follow these steps:

![local_node.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/local_node.png)
1. In the Monero GUI, go to `Settings`
2. Go to the `Node` tab
3. Enable `Local node`
4. Enter `--zmq-pub=tcp://127.0.0.1:18083` into `Daemon startup flags`

After syncing the blockchain, you will now have your own Monero node. The 4th step enables `ZMQ`, which is extra Monero node functionality that is needed for P2Pool to work correctly.

[For much more detailed information on configuring a Monero node, click here.](https://monerodocs.org)

---

### Command Line
Gupax comes with some command line options:
```
USAGE: ./gupax [--flag]

    --help            Print this help message
    --version         Print version and build info
    --state           Print Gupax state
    --nodes           Print the manual node list
    --payouts         Print the P2Pool payout log, payout count, and total XMR mined
    --no-startup      Disable all auto-startup settings for this instance (auto-update, auto-ping, etc)
    --reset-state     Reset all Gupax state (your settings)
    --reset-nodes     Reset the manual node list in the [P2Pool] tab
    --reset-pools     Reset the manual pool list in the [XMRig] tab
    --reset-payouts   Reset the permanent P2Pool stats that appear in the [Status] tab
    --reset-all       Reset the state, manual node list, manual pool list, and P2Pool stats
    --ferris          Print an extremely cute crab
```

By default, Gupax has `auto-update` & `auto-ping` enabled. This can only be turned off in the GUI which causes a chicken-and-egg problem. To get around this, start Gupax with `--no-startup`. This will disable all `auto` features for that instance.

---

### Key Shortcuts
The letter keys (Z/X/C/V/S/R) will only work if nothing is in focus, i.e, you _are not_ editing a text box.

An ALT+F4 will also trigger the exit confirm screen (if enabled).
```
*---------------------------------------*
|             Key shortcuts             |
|---------------------------------------|
|             F11 | Fullscreen          |
|          Escape | Quit screen         |
|              Up | Start/Restart       |
|            Down | Stop                |
|               Z | Left Tab            |
|               X | Right Tab           |
|               C | Left Submenu        |
|               V | Right Submenu       |
|               S | Save                |
|               R | Reset               |
*---------------------------------------*
```

---

### Resolution
The default resolution of Gupax is `1280x960` which is a `4:3` aspect ratio.

This can be changed by dragging the corner of the window itself or by using the resolution sliders in the `Gupax Advanced` tab. After a resolution change, Gupax will fade-in/out of black and will take a second to resize all the UI elements to scale correctly to the new resolution.

If you have changed your OS's pixel scaling, you may need to resize Gupax to see all UI correctly.

The minimum window size is: `640x480`  
The maximum window size is: `3840x2160`  
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

If Gupax can't read/write to disk at all, or if there are any other big issues, it will show an unrecoverable error screen.

| OS       | Data Folder                              | Example                                         |
|----------|------------------------------------------|-------------------------------------------------|
| Windows  | `{FOLDERID_RoamingAppData}`              | `C:\Users\USER\AppData\Roaming\Gupax`           |
| macOS    | `$HOME`/Library/Application Support      | `/Users/USER/Library/Application Support/Gupax` |
| Linux    | `$XDG_DATA_HOME` or `$HOME`/.local/share | `/home/USER/.local/share/gupax`                 |

The current files saved to disk:
* `state.toml` Gupax state/settings
* `node.toml` The manual node database used for P2Pool advanced
* `pool.toml` The manual pool database used for XMRig advanced
* `p2pool/` The Gupax-P2Pool API files

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
- `ERROR` means something has gone wrong and that something will probably break
- `WARN` means something has gone wrong, but things will be fine
- `INFO` logs are general info about what Gupax (the GUI thread) is currently doing
- `DEBUG` logs are much more verbose and include what EVERY thread is doing (not just the main GUI thread)
- `TRACE` logs are insanely verbose and shows very low-level logs of the libraries Gupax uses (HTTP connections, GUI polling, etc)

---

### Swapping P2Pool/XMRig
If you want to use your own `P2Pool/XMRig` binaries you can:
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

---

### Status
The `[P2Pool]` submenu in this tab reads/writes to a file-based API in a folder called `p2pool` located in the [Gupax OS data directory:](#Disk)

| File     | Purpose                                                 | Specific Data Type |
|----------|---------------------------------------------------------|--------------------|
| `log`    | Formatted payout lines extracted from P2Pool            | Basically a `String` that needs to be parsed correctly
| `payout` | The total amount of payouts received via P2Pool & Gupax | `u64`
| `xmr`    | The total amount of XMR mined via P2Pool & Gupax        | Atomic Units represented with a `u64`

The `log` file contains formatted payout lines extracted from your P2Pool:
```
<DATE> <TIME> | <12_FLOATING_POINT> XMR | Block <HUMAN_READABLE_BLOCK>
```
e.g:
```
2023-01-01 00:00:00.0000 | 0.600000000000 XMR | Block 2,123,123
```

The `payout` file is just a simple count of how many payouts have been received.

The `xmr` file is the sum of XMR mined as [Atomic Units](https://web.getmonero.org/resources/moneropedia/atomic-units.html). The equation to represent this as XMR is:
```rust
XMR_ATOMIC_UNITS / 1,000,000,000,000
```
e.g:
```rust
600,000,000,000 / 1,000,000,000,000 = 0.6 XMR
```

Gupax interacts with these files in two ways:
- Reading the files when initially starting
- Appending/overwriting new data to the files upon P2Pool payout

These files shouldn't be written to manually or Gupax will report wrong data.

If you want to reset these stats, you can run:
```bash
./gupax --reset-payouts
```
or just manually delete the `p2pool` folder in the [Gupax OS data directory.](#Disk)

---

### Gupax
Along with the updater and settings mentioned in [Simple](#simple), `Gupax Advanced` allows you to change:
- The PATH of where Gupax looks for P2Pool/XMRig
- The selected tab on startup
- Gupax's resolution

**Warning:** Gupax will use your custom PATH/binary and will replace them if you use `Check for updates` in the `[Gupax]` tab. There are sanity checks in place, however. Your PATH MUST end in a value that _appears_ correct or else the updater will refuse to start:
| Binary   | Accepted values                  | Good PATH       | Bad PATH |
|----------|----------------------------------|-----------------|----------|
| `P2Pool` | `P2POOL, P2Pool, P2pool, p2pool` | `P2pool/p2pool` | `Documents/my_really_important_file`
| `XMRig`  | `XMRIG, XMRig, Xmrig, xmrig`     | `XMRig/XMRig`   | `Desktop/`

If using Windows, the PATH _must_ end with `.exe`.

---

### P2Pool
P2Pool Advanced has:
- Terminal input
- Overriding command arguments
- Manual node list
- P2Pool Main/Mini selection
- Out/In peer setting
- Log level setting

The overriding command arguments will completely override your Gupax settings and start P2Pool with those arguments. **Warning:** If using this setting, use `--no-color` and make sure to set `--data-api <PATH>` & `--local-api` so that the `[Status]` tab can work!

The manual node list allows you save and connect up-to 1000 custom Monero nodes:
| Data Field | Purpose                                                       | Limits                                                 | Max Length     |
|------------|---------------------------------------------------------------|--------------------------------------------------------|----------------|
| `Name`     | A unique name to identify this node (only for Gupax purposes) | Only `[A-Za-z0-9-_.]` and spaces allowed               | 30 characters  |
| `IP`       | The Monero Node IP to connect to with P2Pool                  | It must be a valid IPv4 address or a valid domain name | 255 characters |
| `RPC`      | The RPC port of the Monero node                               | `[1-65535]`                                            | 5 characters   | 
| `ZMQ`      | The ZMQ port of the Monero node                               | `[1-65535]`                                            | 5 characters   | 

The `Main/Mini` selector allows you to change which P2Pool sidechain you mine on:
| P2Pool Sidechain | Description                                                  | Use-case                                  |
|------------------|--------------------------------------------------------------|-------------------------------------------|
| `Main`           | More miners, finds blocks faster, has a higher difficulty    | Suitable for miners with MORE than 50kH/s |
| `Mini`           | Less miners, finds blocks slower, has a lower difficulty     | Suitable for miners with LESS than 50kH/s |

Given enough time, both `Main` and `Mini` will result in the same reward (as will solo mining):
| Mining Method    | Share Behavior                              | Payout/Output Behavior             | Example                                    | Total (it's the same) |
|------------------|---------------------------------------------|------------------------------------|--------------------------------------------|-----------------------|
| `P2Pool Main`    | LESS frequent shares that are MORE valuable | Results in MORE outputs worth LESS | 20 shares, 100 outputs worth `0.006 XMR`   | `0.6 XMR`             |
| `P2Pool Mini`    | MORE frequent shares that are LESS valuable | Results in LESS outputs worth MORE | 100 shares, 20 outputs worth `0.03 XMR`    | `0.6 XMR`             |
| `Solo mining`    | No shares, only payouts                     | 1 output                           | 1 output worth the block reward: `0.6 XMR` | `0.6 XMR`             |

In the end, it doesn't matter _too much_ which sidechain you pick, it will all average out. Getting LESS but more valuable outputs may be desired, however, since the transaction cost to combine all of them (`sweep_all`) will be cheaper due to being comprised of less outputs.

The remaining sliders control miscellaneous settings:
| Slider      | Purpose                                                     | Default | Min/Max Range |
|-------------|-------------------------------------------------------------|---------|---------------|
| `Out peers` | How many out-bound peers P2Pool will connect to             | `10`    | `10..450`     |
| `In peers`  | How many in-bound peers P2Pool will allow to connect to you | `10`    | `10..450`     |
| `Log level` | Verbosity of the P2Pool console log                         | `3`     | `0..6`        |


---

### XMRig
XMRig Advanced has:
- Terminal input
- Overriding command arguments
- Custom payout address
- CPU thread slider
- Manual pool list
- Custom HTTP API IP/Port
- TLS setting
- Keepalive setting

The overriding command arguments will completely override your Gupax settings and start XMRig with those arguments. **Warned:** If using this setting, use `[--no-color]` and make sure to set `[--http-host <IP>]` & `[--http-port <PORT>]` so that the `[Status]` tab can work!

The manual pool list allows you save and connect up-to 1000 custom Pools (regardless if P2Pool or not):
| Data Field | Purpose                                                       | Limits                                                 | Max Length     |
|------------|---------------------------------------------------------------|--------------------------------------------------------|----------------|
| `Name`     | A unique name to identify this pool (only for Gupax purposes) | Only `[A-Za-z0-9-_.]` and spaces allowed               | 30 characters  |
| `IP`       | The pool IP to connect to with XMRig                          | It must be a valid IPv4 address or a valid domain name | 255 characters |
| `Port`     | The port of the pool                                          | `[1-65535]`                                            | 5 characters   | 
| `Rig`      | An optional rig ID; This will be the name shown on the pool   | Only `[A-Za-z0-9-_]` and spaces allowed                | 30 characters  |

The HTTP API textboxes allow you to change to IP/Port XMRig's HTTP API opens up on:
| Data Field      | Purpose                                       | Default               | Limits                                                 | Max Length
|-----------------|-----------------------------------------------|-----------------------|--------------------------------------------------------|----------------|
| `HTTP API IP`   | The IP XMRig's HTTP API server will bind to   | `localhost/127.0.0.1` | It must be a valid IPv4 address or a valid domain name | 255 characters |
| `HTTP API Port` | The port XMRig's HTTP API server will bind to | `18088`               | `[1-65535]`                                            | 5 characters   |

The remaining buttons control miscellaneous settings (both are disabled by default, as P2Pool does not require them):
| Button           | Purpose                                                                   |
|------------------|---------------------------------------------------------------------------|
| `TLS Connection` | Enables SSL/TLS connections (needs pool support)                          |
| `Keepalive`      | Enables sending keepalive packets to prevent timeout (needs pool support) |

## Connections
For transparency, here's all the connections Gupax makes:

| Domain             | Why                                                   | When | Where |
|--------------------|-------------------------------------------------------|------|-------|
| https://github.com | Fetching metadata information on packages + download  | `[Gupax]` tab -> `Check for updates` | [`update.rs`](https://github.com/hinto-janaiyo/gupax/blob/main/src/update.rs) |
| Community Monero Nodes | Connecting to with P2Pool, measuring ping latency | `[P2Pool Simple]` tab | [`node.rs`](https://github.com/hinto-janaiyo/gupax/blob/main/src/node.rs) |
| DNS | DNS connections will usually be handled by your OS (or whatever custom DNS setup you have). If using Tor, DNS requests for updates [*should*](https://tpo.pages.torproject.net/core/doc/rust/arti/) be routed through the Tor network automatically | All of the above | All of the above |

## Community Monero Nodes
These are the community nodes used by Gupax in the `[P2Pool Simple]` tab. If you would like to have a node added/removed, please submit an [Issue](https://github.com/hinto-janaiyo/gupax/issues) with the reasoning.

In general, a suitable node needs to:
- Be fast
- Have good uptime
- Have RPC enabled
- Have ZMQ enabled
- Have an owner known by the general Monero community

| Name           | Owner                                             | Owner Type | IP/Domain                         | RPC Port | ZMQ Port |
|----------------|---------------------------------------------------|------------|-----------------------------------|----------|----------|
| C3pool         | [C3pool](https://www.c3pool.com)                  | Pool       | node.c3pool.com                   | 18081    | 18083    |
| Cake           | [Cake](https://cakewallet.com)                    | Wallet     | xmr-node.cakewallet.com           | 18081    | 18083    |
| CakeEu         | [Cake](https://cakewallet.com)                    | Wallet     | xmr-node-eu.cakewallet.com        | 18081    | 18083    |
| CakeUk         | [Cake](https://cakewallet.com)                    | Wallet     | xmr-node-uk.cakewallet.com        | 18081    | 18083    |
| CakeUs         | [Cake](https://cakewallet.com)                    | Wallet     | xmr-node-usa-east.cakewallet.com  | 18081    | 18083    |
| Feather1       | [Feather](https://featherwallet.org)              | Wallet     | selsta1.featherwallet.net         | 18081    | 18083    |
| Feather2       | [Feather](https://featherwallet.org)              | Wallet     | selsta2.featherwallet.net         | 18081    | 18083    |
| HashVault      | [HashVault](https://hashvault.pro)                | Pool       | nodes.hashvault.pro               | 18081    | 18083    |
| MajesticBankIs | [MajesticBank](https://www.majesticbank.sc)       | Exchange   | node.majesticbank.is              | 18089    | 18083    |
| MajesticBankSu | [MajesticBank](https://www.majesticbank.sc)       | Exchange   | node.majesticbank.su              | 18089    | 18083    |
| MoneroSeed1    | [Monero](https://github.com/monero-project/monero/blob/release-v0.18/src/p2p/net_node.inl#L708) | Seed Node | 176.9.0.187   | 18089 | 18083 |
| MoneroSeed2    | [Monero](https://github.com/monero-project/monero/blob/release-v0.18/src/p2p/net_node.inl#L715) | Seed Node | 51.79.173.165 | 18089 | 18083 |
| MoneroWorld1   | [Gingeropolous](https://github.com/Gingeropolous) | Individual | node.moneroworld.com              | 18089    | 18083    |
| MoneroWorld2   | [Gingeropolous](https://github.com/Gingeropolous) | Individual | uwillrunanodesoon.moneroworld.com | 18089    | 18083    |
| Monerujo       | [Monerujo](https://www.monerujo.io)               | Wallet     | nodex.monerujo.io                 | 18081    | 18083    |
| Plowsof1       | [Plowsof](https://github.com/plowsof)             | Individual | node.monerodevs.org               | 18089    | 18084    |
| Plowsof2       | [Plowsof](https://github.com/plowsof)             | Individual | node2.monerodevs.org              | 18089    | 18084    |
| Rino           | [Rino](https://rino.io)                           | Wallet     | node.community.rino.io            | 18081    | 18083    |
| Seth           | [Seth](https://github.com/sethforprivacy)         | Individual | node.sethforprivacy.com           | 18089    | 18083    |
| SupportXmr     | [SupportXMR](https://www.supportxmr.com)          | Pool       | node.supportxmr.com               | 18081    | 18083    |
| SupportXmrIr   | [SupportXMR](https://www.supportxmr.com)          | Pool       | node.supportxmr.ir                | 18089    | 18083    |
| XmrVsBeast     | [XMRvsBeast](https://xmrvsbeast.com)              | Pool       | p2pmd.xmrvsbeast.com              | 18081    | 18083    |

## Build
### General Info
You need [`cargo`](https://www.rust-lang.org/learn/get-started), Rust's build tool and package manager.

The `--release` profile in Gupax is set to prefer code performance & small binary sizes over compilation speed (see [`Cargo.toml`](https://github.com/hinto-janaiyo/gupax/blob/main/Cargo.toml)). Gupax itself (with all dependencies already built) takes around 1m30s to build (vs 10s on a normal `--release`) with a Ryzen 5950x.

There are `38` unit tests throughout the codebase files, you should probably run:
```
cargo test
```
before attempting a full build.

---

### Linux
You'll need the development versions of libraries like `OpenSSL`, `SQLite`, and maybe some other ones already installed on your system. Read the compiler errors to see which ones are missing from your system and search around to see which packages you'll need to install depending on your distro.

After that, run:
```
cargo build --release
```

The Linux release tars come with another file next to the Gupax binary: `Gupax.AppImage`. ***This is not an actual [AppImage](https://en.wikipedia.org/wiki/AppImage)***, it is just a text file that contains: `./gupax`. This allows users to double-click and execute Gupax in file explorers like `Nautilus` in Ubuntu/Debian.

### Building for a distribution
Gupax has a build flag for use as a package in a Linux distribution:
```
cargo build --release --features distro
```
This is the same as the `--release` profile, but with some changes:
| Change                                     | Reason |
|--------------------------------------------|--------|
| Built-in `Update` feature is disabled      | Updates should be handled by the native package manager
| Default `P2Pool/XMRig` path is `/usr/bin/` | `P2Pool/XMRig` exist in _[some](https://aur.archlinux.org)_ repositories, which means they'll be installed in `/usr/bin/`

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

## License
The GUI library Gupax uses is [egui](https://github.com/emilk/egui). It is licensed under [MIT](https://github.com/emilk/egui/blob/master/LICENSE-MIT) & [Apache 2.0.](https://github.com/emilk/egui/blob/master/LICENSE-APACHE)

[Many other libraries are used that have various licenses.](https://github.com/hinto-janaiyo/gupax/blob/master/Cargo.toml)

[Gupax](https://github.com/hinto-janaiyo/gupax/blob/master/LICENSE), [P2Pool](https://github.com/SChernykh/p2pool/blob/master/LICENSE), and [XMRig](https://github.com/xmrig/xmrig/blob/master/LICENSE) are licensed under the GNU General Public License v3.0.

## FAQ
### Where are updates downloaded from?
The latest versions are downloaded using GitHub's API.
* Gupax [`https://github.com/hinto-janaiyo/gupax`](https://github.com/hinto-janaiyo/gupax)
* P2Pool [`https://github.com/SChernykh/p2pool`](https://github.com/SChernykh/p2pool)
* XMRig [`https://github.com/xmrig/xmrig`](https://github.com/xmrig/xmrig)

GitHub's API blocks request that do not have an HTTP `User-Agent` header. [Gupax uses a random recent version of a `Wget/Curl` user-agent.](https://github.com/hinto-janaiyo/gupax/blob/e6bf49b309c64d29e50c0a1a185fcf0ebc05e7c7/src/update.rs#L134)

---

### P2Pool connection errors
**TL;DR: Run & use your own Monero Node.**

If you are using the [default P2Pool settings](#P2Pool) then you are using a [Community Monero Node](#community-monero-nodes). Using a community node is convenient but comes at the cost of privacy and reliability. You may encounter connections issues with these nodes that look like this:
```
2023-01-05 12:27:37.7962 P2PServer peer 23.233.96.72:37888 is ahead on mainchain (height 2792939, your height 2792936). Is your monerod stuck or lagging?
```
To fix this you can select a different community node, or better yet: [Run your own local Monero Node](#running-a-local-monero-node).

Running and using your own local Monero node improves privacy and ensures your connection is as stable as your own internet connection. This comes at the cost of downloading and syncing Monero's blockchain yourself (currently 155GB). If you have the disk space, consider using the [P2Pool Advanced](#p2pool-1) tab and connecting to your own Monero node.

For a simple guide, see the [Running a Local Monero Node](#running-a-local-monero-node) section.

---

### Can I quit mid-update?
If you started an update, you should let it finish. If the update has been stuck for a *long* time, quitting Gupax is probably okay. The worst that can happen is that your `Gupax/P2Pool/XMRig` binaries may be moved/deleted. Those can be easily redownloaded. Your actual `Gupax` user data (settings, custom nodes, pools, etc) is never touched.

Although Gupax uses a temporary folder (`gupax_update_[A-Za-z0-9]`) to store temporary downloaded files, there aren't measures in place to revert an upgrade once the file swapping has actually started. If you quit Gupax anytime before the `Upgrading packages` phase (after metadata, download, extraction), you will technically be safe but this is not recommended as it is risky, especially since these updates can be very fast.

---

### Bundled vs Standalone
`Bundled` Gupax comes the latest version of P2Pool/XMRig already in the `zip/tar`.

`Standlone` only contains the Gupax executable.

---

### How much memory does Gupax use?
Gupax itself uses around 100-400 megabytes of memory.

Gupax also holds up to [500,000 bytes](https://github.com/hinto-janaiyo/gupax/blob/e6bf49b309c64d29e50c0a1a185fcf0ebc05e7c7/src/helper.rs#L59) of log data from `P2Pool/XMRig` to display in the GUI terminals. These logs are reset once over capacity which takes around 1-4 hours.

Memory usage should *never* be above 500~ megabytes. If you see Gupax using more than this, please send a bug report.

---

### How is sudo handled? (on macOS/Linux)
[See here for more info.](https://github.com/hinto-janaiyo/gupax/tree/main/src#sudo)

---

### Why does Gupax need to be Admin? (on Windows)
[See here for more info.](https://github.com/hinto-janaiyo/gupax/tree/main/src#why-does-gupax-need-to-be-admin-on-windows)
