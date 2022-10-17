# Gupax - WORK IN PROGRESS
![banner.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/png/banner.png)
**Gupax** (*guh-picks*) is a cross-platform GUI for mining [**Monero**](https://github.com/monero-project/monero) on the decentralized [**P2Pool**](https://github.com/SChernykh/p2pool), using the dedicated [**XMRig**](https://github.com/xmrig/xmrig) miner for max hashrate.

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

## Goal
**Gupax is:**
* A simple GUI solution to P2Pool mining with max hashrate
* External mining software so Monero GUI isn't plagued with anti-virus issues
* Fast/lightweight because the context for this software is a ***mining*** computer

**Gupax is not:**
* For advanced mining setups
* A Monero wallet
* A Monero node

Monero GUI + Gupax = Easy, decentralized, max hashrate Monero mining.

## Build
Optimized:
```
cargo build --profile optimized
```
Optimized for your specific CPU (15%~ speed increase, depending on your CPU):
```
RUSTFLAGS="-C target-cpu=native" cargo build --profile optimized
```

Add `--target x86_64-pc-windows-gnu` to build for Windows.
