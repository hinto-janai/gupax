# external
These are some external git repositories for various purposes:
| Repo | Purpose             |
|------|---------------------|
| egui | [external/egui/crates/eframe/src/native/run.rs] line 41: [.with_srgb(true)]. This line causes a [panic!] inside a Windows VM, from a Linux host. The only change is [.with_srgb()] is set to [false]. This is only used for testing, since ironically, this crashes bare-metal Windows. |
| rust-runas | This contains some interesting code that could be used as an alternative to running processes with elevated privilege in Windows for Gupax |
