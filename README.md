# Gupax - WORK IN PROGRESS
![banner.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/banner.png)
**Gupax** (*guh-picks*) is a cross-platform GUI for mining [**Monero**](https://github.com/monero-project/monero) on [**P2Pool**](https://github.com/SChernykh/p2pool), using [**XMRig**](https://github.com/xmrig/xmrig).

## Contents
* [What is Monero, P2Pool, XMRig, and Gupax?](##what-is-monero-p2pool-xmrig-and-gupax)
* [Community Monero Nodes](#community-monero-nodes)
* [Demo](#Demo)
* [Implementation](#Implementation)
* [Planned](#Planned)
* [Goals](#Goals)
* [Build](#Build)

## What is Monero, P2Pool, XMRig, and Gupax?
**Monero** is a secure, private, and untraceable cryptocurrency.

The **[Monero GUI](https://github.com/monero-project/monero-gui)** software lets you run a **Monero node** (among other things). A Monero node connects you to other peers and lets you download Monero's [blockchain](https://en.wikipedia.org/wiki/Blockchain). But you already knew all of this, right?

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

**XMRig** is an optimized miner which lets you **mine Monero at higher speeds.**

Both Monero and P2Pool have built in miners but XMRig is quite faster than both of them. Due to issues like [anti-virus flagging](https://github.com/monero-project/monero-gui/pull/3829#issuecomment-1018191461), it is not feasible to integrate XMRig directly into Monero or P2Pool, however, XMRig is still freely available for anyone to download. The issue is: you have to manually set it up yourself.

***[More info here.](https://github.com/xmrig/xmrig)***

---

**Gupax** is a GUI that helps with configuring, updating, and managing P2Pool & XMRig (both originally CLI-only).

***Recap:***
1. **XMRig** mines to **P2Pool** which fetchs blocks from a **Monero node**
2. **Monero GUI** runs the ***Monero node***
3. **Gupax** runs ***P2Pool/XMRig***

![stack.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/diagram.png)

With Monero GUI managing the Monero node on one side and Gupax managing P2Pool/XMRig on the other, it is (hopefully) very easy for anyone to start mining Monero at **max hashrate in a decentralized, permissionless, and trustless manner**.

## Community Monero Nodes
| Name           | IP/Domain                        | RPC Port |
|----------------|----------------------------------|----------|
| C3pool         | node.c3pool.com                  | 18081    |
| Cake           | xmr-node.cakewallet.com          | 18081    |
| CakeEu         | xmr-node-eu.cakewallet.com       | 18081    |
| CakeUk         | xmr-node-uk.cakewallet.com       | 18081    |
| CakeUs         | xmr-node-usa-east.cakewallet.com | 18081    |
| Feather1       | selsta1.featherwallet.net        | 18081    |
| Feather2       | selsta2.featherwallet.net        | 18081    |
| MajesticBankIs | node.majesticbank.is             | 18089    |
| MajesticBankSu | node.majesticbank.su             | 18089    |
| Monerujo       | nodex.monerujo.io                | 18081    |
| Rino           | node.community.rino.io           | 18081    |
| Seth           | node.sethforprivacy.com          | 18089    |
| Singapore      | node.supportxmr.com              | 18081    |
| SupportXmr     | node.supportxmr.ir               | 18081    |
| SupportXmrIr   | singapore.node.xmr.pm            | 18089    |
| XmrVsBeast     | p2pmd.xmrvsbeast.com             | 18081    |
**Note: All have ZMQ port on 18083**

## Demo
https://user-images.githubusercontent.com/101352116/194763334-d8e936c9-a71e-474e-ac65-3a339b96a9d2.mp4

<details>
<summary>Click me to load images!</summary>

![about.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/tabs/about.png)
![status.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/tabs/status.png)
![gupax.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/tabs/gupax.png)
![p2pool.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/tabs/p2pool.png)
![xmrig.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/tabs/xmrig.png)

</details>


## Implementation
- **OS:** Gupax will be tested for Windows, macOS, and Linux. Maybe the BSDs
- **Docs:** All Gupax usage will have documentation on GitHub; General P2Pool/XMRig info will also be included
- **Packaging:** Gupax will be packaged in a bundled zip/tar that includes P2Pool/XMRig, and as a standalone binary that expects you to bring your own P2Pool/XMRig. Both will be the same binary, only difference being the first will include all necessary components. Maybe an installer as well
- **Efficiency:** The context for Gupax is a ***mining*** machine, it would be too ironic if it impacted the hashrate performance, and so, Gupax uses the very lightweight [Rust egui library](https://github.com/emilk/egui). By default egui is an "immediate mode" GUI, meaning frames are rendered 60x/sec. This is turned off in Gupax so frames are only rendered upon user interaction. This allows for a fast and lightweight GUI. For context, it uses around 5x less CPU when switching around tabs compared to Monero GUI

## Planned
- **Community Node:** An option to use a trusted community Monero node instead of your own. At a small privacy cost, this allows users to immediately start mining on P2Pool without downloading the entire chain
- **Update:** Built-in update/upgrader for Gupax/P2Pool/XMRig and an (opt-in) auto-updater that runs at startup
- **Config:** All the basic configurations you would expect with P2Pool/XMRig (main, mini, peers, thread count, etc)
- **Status:** Status tab displaying mining statistics using P2Pool & XMRig's APIs

## Goals
**Gupax is:**
* A simple GUI solution to P2Pool mining with max hashrate
* External mining software so Monero GUI isn't plagued with anti-virus issues
* Fast/lightweight because the context for this software is a ***mining*** computer

**Gupax is not:**
* A Monero node/wallet

## Build
```
cargo build --release
```
On macOS, if you want the binary to have an icon in `Finder`, you must install [`cargo-bundle`](https://github.com/burtonageo/cargo-bundle) and compile with:
```
cargo bundle --release
```

The `build.rs` file in the repo root sets the icon in `File Explorer` for Windows. The taskbar icon & App frame icon (for all OS's) get set at runtime using pre-compiled bytes in [`src/constants.rs`](https://github.com/hinto-janaiyo/gupax/blob/main/src/constants.rs) from [`images`](https://github.com/hinto-janaiyo/gupax/blob/main/images).
