# Gupax source files. Development documentation is here.
* [Structure](#Structure)
* [Thread Model](#Thread-Model)
* [Bootstrap](#Bootstrap)
* [Scale](#Scale)
* [Naming Scheme](#naming-scheme)
* [Mining Stat Reference](#mining-stat-reference)
* [Sudo](#Sudo)
* [Why does Gupax need to be Admin? (on Windows)](#why-does-gupax-need-to-be-admin-on-windows)
	- [The issue](#the-issue)
	- [The requirements](#the-requirements)
	- [CMD's RunAs](#cmds-runas)
	- [PowerShell's Start-Process](#powershells-start-process)
	- [Win32's ShellExecuteW](#win32s-shellexecutew)
	- [Registry Edit](#registry-edit)
	- [Windows vs Unix](#windows-vs-unix)

## Structure
| File/Folder  | Purpose |
|--------------|---------|
| constants.rs | General constants used in Gupax
| disk.rs      | Code for writing to disk: `state.toml/node.toml/pool.toml`; This holds the structs for the [State] struct
| ferris.rs    | Cute crab bytes
| gupax.rs     | `Gupax` tab
| helper.rs    | The "helper" thread that runs for the entire duration Gupax is alive. All the processing that needs to be done without blocking the main GUI thread runs here, including everything related to handling P2Pool/XMRig
| human.rs     | Code for displaying human readable numbers & time
| macros.rs    | General `macros!()` used in Gupax
| main.rs      | The main `App` struct that holds all data + misc data/functions
| node.rs      | Community node ping code for the `P2Pool` simple tab
| p2pool.rs    | `P2Pool` tab
| regex.rs     | General regexes used in Gupax
| status.rs    | `Status` tab
| sudo.rs      | Code for handling `sudo` escalation for XMRig on Unix
| update.rs    | Update code for the `Gupax` tab
| xmr.rs       | Code for handling actual XMR, `AtomicUnit` & `PayoutOrd`
| xmrig.rs     | `XMRig` tab

## Thread Model
![thread_model.png](https://github.com/hinto-janaiyo/gupax/blob/main/images/thread_model.png)

Process's (both Simple/Advanced) have:
- 1 OS thread for the watchdog (API fetching, watching signals, etc)
- 1 OS thread for a PTY-Child combo (combines STDOUT/STDERR for me, nice!)
- A PTY (pseudo terminal) whose underlying type is abstracted with the [`portable_pty`](https://docs.rs/portable-pty/) library

The reason why STDOUT/STDERR is non-async is because P2Pool requires a `TTY` to take STDIN. The PTY library used, [`portable_pty`](https://docs.rs/portable-pty/), doesn't implement async traits. There seem to be tokio PTY libraries, but they are Unix-specific. Having separate PTY code for Windows/Unix is also a big pain. Since the threads will be sleeping most of the time (the pipes are lazily read and buffered), it's fine. Ideally, any I/O should be a tokio task, though.

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
	- If `auto_ping` == `true`, spawn community node ping thread
	- If `auto_p2pool` == `true`, spawn P2Pool
	- If `auto_xmrig` == `true`, spawn XMRig

3. **MAIN**
	- All data should be initialized at this point, either via `state.toml` or default options
	- Start `App` frame
	- Do `App` stuff
	- If `ask_before_quit` == `true`, ask before quitting
	- Kill processes, kill connections, exit

## Scale
Every frame, the max available `[width, height]` are calculated, and those are used as a baseline for the Top/Bottom bars, containing the tabs and status bar. After that, all available space is given to the middle ui elements. The scale is calculated every frame so that all elements can scale immediately as the user adjusts it; this doesn't take as much CPU as you might think since frames are only rendered on user interaction. Some elements are subtracted a fixed number because the `ui.seperator()`'s add some fixed space which needs to be accounted for.

```
Main [App] outer frame (default: [1280.0, 960.0], 4:3 aspect ratio)
   ├─ TopPanel     = height: 1/15
   ├─ BottomPanel  = height: 1/22
   ├─ CentralPanel = height: the rest
```

## Naming Scheme
This is the internal naming scheme used by Gupax when updating/creating default folders/etc:

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

When Gupax updates, it walks the directories of the extracted `zip/tar` searching for a valid file. These are the valid filenames Gupax will match against and assume is the new binary we're looking for:
- `[GUPAX, Gupax, gupax]`
- `[P2POOL, P2Pool, P2pool, p2pool]`
- `[XMRIG, XMRig, Xmrig, xmrig]`

Windows versions of Gupax also need the file to end with `.exe`.

The actual `zip/tar` matching is static, however. They have to be packaged exactly with the following naming scheme. If an exact match is not found, it will error:
- `gupax-vX.X.X-(windows|macos|linux)-x64-(bundle|standalone).(zip|tar.gz)`
- `p2pool-vX.X.X-(windows|macos|linux)-x64.(zip|tar.gz)`
- `xmrig-X.X.X-(msvc-win64|macos-x64|linux-static-x64).(zip|tar.gz)`

Exceptions (there are always exceptions...):
- XMRig doesn't have a [v], so it is [xmrig-6.18.0-...]
- XMRig separates the hash and signature
- P2Pool hashes are in UPPERCASE

## Mining Stat Reference
Some pseudo JSON for constants/equations needed for generating mining stats. They're here for easy reference, I was never good at math :)
```
block_time_in_seconds: {
	P2POOL_BLOCK_TIME: 10,
	MONERO_BLOCK_TIME: 120,
}

difficulty: {
	P2POOL_DIFFICULTY: (current_p2pool_hashrate * P2POOL_BLOCK_TIME),
	MONERO_DIFFICULTY: (current_monero_hashrate * MONERO_BLOCK_TIME),
}

hashrate_per_second: {
	P2POOL_HASHRATE: (P2POOL_DIFFICULTY / P2POOL_BLOCK_TIME),
	MONERO_HASHRATE: (MONERO_DIFFICULTY / MONERO_BLOCK_TIME),
}

mean_in_seconds: {
	P2POOL_BLOCK_MEAN: (MONERO_DIFF / P2POOL_HASHRATE),
	MY_SOLO_BLOCK_MEAN: (MONERO_DIFF / my_hashrate),
	MY_P2POOL_SHARE_MEAN: (P2POOL_DIFF / my_hashrate),
}
```

## Sudo
Unlike Windows, Unix (macOS/Linux) has a userland program that handles all the dirty details of privilege escalation: `sudo`.

`sudo` is used in Gupax to execute XMRig, to enable MSR mods and hugepages. After every use of `sudo`, the memory holding the `String` buffer containing the password is wiped with 0's using [`zeroize`](https://docs.rs/zeroize/) to make sure the compiler doesn't optimize away the wipe. Although memory *should* be safe, this prevents passive accidents (core-dumps revealing plain-text password) and active attacks (attackers accessing live process memory) from happening.

In general, secrets should be ephemeral or encrypted if not in use. I considered using [`secrets`](https://docs.rs/secrets/) to keep the password encrypted so that the user would only have to enter their password once, but simply wiping the memory was easier to implement and caused less worries of handling things incorrectly.

## Why does Gupax need to be Admin? (on Windows)
**TL;DR:** Because Windows.  

**Slightly more detailed TL;DR:** Rust does not have mature Win32 API wrapper libraries. Although Microsoft has an official ["Rust" library](https://github.com/microsoft/windows-rs), it is quite low-level and using it within Gupax would mean re-implementing a lot of Rust's STDLIB process module code.

If you are confused because you use Gupax on macOS/Linux, this is a Windows-only issue.

The following sections will go more into the technical issues I've encountered in trying to implement something that sounds pretty trivial: Starting a child process with elevated privilege, and getting a handle to it and its output. (it's a rant about windows).

---

### The issue
`XMRig` needs to be run with administrative privileges to enable MSR mods and hugepages. There are other ways of achieving this through pretty manual and technical efforts (which also gets more complicated due to OS differences) but in the best interest of Gupax's users, I always want to implement things so that it's **easy for the user.**

Users should not need to be familiar with MSRs to get max hashrate, this is something the program (me, Gupax!) should do for them.

---

### The requirements
Process's in Gupax need the following criteria met:
- I (as the parent process, Gupax) *must* have a direct handle to the process so that I can send SIGNALs
- I *must* have a handle to the process's STDOUT+STDERR so that I can actually relay output to the user
- I *really should* but don't absolutely need a handle to STDIN so that I can send input from the user

In the case of XMRig, **I absolutely must enable MSR's automatically for the user**, that's the whole point of XMRig, that's the point of an easy-to-use GUI.
Although I want XMRig with elevated rights, I don't want these side-effects:
- All of Gupax running as Admin
- P2Pool running as Admin

Here are the "solutions" I've attempted:

---

### CMD's RunAs
Window has a `runas` command, which allows for privilege escalation. Perfect! Spawn a shell and it's easy as running this:
```
runas /user:Administrator xmrig.exe [...]
```
...right?

The `Administrator` in this context is a legacy account, not meant to be touched, not really the `Admin` we are looking for, but more importantly: the password is not set, and the entire account is disabled by default. This means you cannot actually `runas` as *that* `Administrator`. Technically, all it would take is for the user to enabled the account and set a password. But that is already asking for too much, remember: that's my job, to make this **easy and automatic**. So this is a no-go, next.

---

### PowerShell's Start-Process
Window's `PowerShell` has a nice built-in called [`Start-Process`](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/start-process?view=powershell-7.3). This allows PowerShell to start... processes. In particular, I was intrigued by the all-in-one flag: `-Verb RunAs`, which runs the provided process with elevated permissions after a **UAC prompt.** That sounds perfect... except if you click that link you'll see 2 sets of syntax. IF you are escalating privilege, Microsoft puts a lot more retrictions on what you can do with this built-in, in particular:
- You CANNOT redirect STDOUT/STDERR/STDIN
- You CANNOT run the process in the current shell (a new PowerShell window will always open!)

I attempted some hacks like chaining non-admin PowerShell + admin PowerShell together, which made things overly complicated and meant I would be handling logic within these child PowerShell's which would be controlled via STDIN from Gupax code... Not very robust. I also tried just starting an admin PowerShell directly from Gupax, but that meant the user, upon clicking `[Start]` for XMRig, would see a UAC prompt to open PowerShell, which wasn't a good look. Eventually I gave up on PowerShell, next.

---

### Win32's ShellExecuteW
This was the first option I came across, but I intentionally ignored it due to many reasons. Microsoft has official Windows API bindings in [Rust](https://github.com/microsoft/windows-rs). That library has a couple problems:
1. All (the entire library) code requires `unsafe`
2. It's extremely low-level

The first one isn't actually as bad as it seems, this is Win32 so it's battle-tested. It's also extern C, so it makes sense it has to wrapped in `unsafe`.

The second one is the real issue. [ShellExecuteW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew) is a Win32 function that allows exactly what I need, starting a process with elevated privilege with the `runas` flag. It even shows the UAC to the user. But... that's it! No other functionality. The highly abstracted `Command` type in Rust's STDLIB actually uses [`CreateProcessW`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw), and due to type imcompatabilities, using `ShellExecuteW` on my own would mean re-implementing ALL the functionality Rust STDLIB gives, aka: handling STDOUT, STDERR, STDIN, sending SIGNALS, waiting on process, etc etc. I would be programming for "Windows", not "Rust". Okay... next.

---

### Registry Edit
To start a process in Windows with elevated escalation you can right-click -> `Run as Administrator`, but you can also set a permanent flag deeper in the file's options. In reality this sets a Registry Key with the absolute path to that executable and a `RUNASADMIN` flag. This allows Windows to know which programs to run as an admin. There is a Rust library called [`WinReg`](https://github.com/gentoo90/winreg-rs) that provides functionality to read/write to the Registry. Editing the Registry is akin to editing someone's `.bashrc`, it's a sin! But... if it means **automatically applying the MSR mod** and **better UX**, then yes I will. The flow would have been:
- User starts XMRig
- Gupax notices XMRig is not admin
- Gupax tells user
- Gupax gives option to AUTOMATICALLY edit registry
- Gupax also gives the option to show how to do it manually

This was the solution I would have gone with, but alas, the abstracted `Command` types I am using to start processes completely ignore this metadata. When Gupax starts XMRig, that `Run as Administrator` flag is completely ignored. Grrr... what options are left?

---

### Windows vs Unix
Unix (macOS/Linux) has a super nice, easy, friendly, not-completely-garbage userland program called: `sudo`. It is so extremely simple to use `sudo` as a sort of wrapper around XMRig since `sudo` isn't completely backwards and actually has valuable flags! No legacy `Administrator`, no UAC prompt, no shells within shells, no low-level system APIs, no messing with the user Registry. 

You get the user's password, you input it to `sudo` with `--stdin` and you execute XMRig with it. Simple, easy, nice. (Don't forget to zero the password memory, though).

With no other option left on Windows, I unfortunately have to fallback to the worst solution: shipping Gupax's binary to have `Administrator` metadata, so that it will automatically prompt users for UAC. This means all child process spawned by Gupax will ALSO have admin rights. Windows having one of the most complicated spaghetti privilege systems is ironically what led me to use the most unsecure option.

Depending on the privilege used, Gupax will error/panic:
- Windows: If not admin, warn the user about potential lower XMRig hashrate
- Unix: IF admin, panic! Don't allow anything. As it should be.

If you're reading this and have a solution (that isn't using Win32), please... please teach me. 
