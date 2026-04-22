# vm-foundry

A Rust TUI for managing desktop QEMU/KVM virtual machines with a VM library workflow, guided creation/import, GPU passthrough, networking controls, and a large catalog of pre-configured OS profiles.

This repository is maintained as a personal fork under the name **VM Foundry**. Fork-specific fixes and UI changes are tracked in [CHANGELOG.md](./CHANGELOG.md).

## Features

**VM Discovery & Organization**
- Automatically scans your VM library for directories containing `launch.sh` scripts
- Hierarchical organization by OS family and subcategory
- Parses QEMU launch scripts to extract configuration details
- Live process monitoring for running VMs
- Search and filter support from the main TUI

**VM Creation & Import**
- 5-step creation wizard for new VMs
- 120+ pre-configured OS profiles with tuned QEMU defaults
- Existing disk workflows for copying or moving prepared `qcow2` images into the library
- Guided import from libvirt (`virsh`) XML and Quickemu `.conf` files
- Custom OS entries with user metadata

**GPU, Display, and Devices**
- Single-GPU passthrough
- Multi-GPU passthrough with Looking Glass integration
- PCI passthrough screen for GPUs, USB controllers, NVMe devices, and more
- USB passthrough management with persistent configuration
- Shared folders via virtio-9p
- Direct `launch.sh` editing from the TUI

**Storage, Snapshots, and Networking**
- qcow2 creation with snapshot support
- Snapshot create/restore/delete operations
- Network backend selection: user/SLIRP, passt, bridge, or none
- Port forwarding presets for SSH, RDP, HTTP, HTTPS, and VNC
- Bridge setup guidance and bridge-helper checks

**Usability**
- Vim-style navigation plus arrows and mouse support
- Multiple boot modes including normal, install, custom media, recovery, and floppy
- Settings screen with persistent configuration
- OS metadata, notes, and ASCII art in the main UI

## Screenshots

```text
 VM Foundry (QEMU VM Library in ~/vm-space)
┌─────────────────────────────────────────────────────────────────────┐
│ ┌─────────────────────────┐  ┌────────────────────────────────────┐ │
│ │ VMs (35)                │  │       _    _ _           _        │ │
│ │ ──────────────────────  │  │      | |  | (_)         | |       │ │
│ │ 🪟 Microsoft            │  │      | |/\| |_ _ __   __| | ___   │ │
│ │   ▼ DOS                 │  │      \  /\  / | '_ \ / _` |/ _ \  │ │
│ │     > MS-DOS 6.22   [*] │  │       \/  \/|_|_| |_|\__,_|\___/  │ │
│ │     > Windows 3.11      │  │                                   │ │
│ │   ▼ Windows 9x          │  │   Windows 95 OSR2.5               │ │
│ │     > Windows 95        │  │   Microsoft | August 1995 | i386  │ │
│ │     > Windows 98        │  │                                   │ │
│ │ 🐧 Linux                │  │   The OS that changed everything  │ │
│ │   ▼ Debian-based        │  │   with the Start Menu, taskbar,   │ │
│ │     > Debian 12         │  │   and 32-bit computing for all.   │ │
│ │     > Ubuntu 24.04      │  │                                   │ │
│ └─────────────────────────┘  └────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│ [Enter] Launch  [m] Manage  [c] Create  [s] Settings  [?] Help     │
└─────────────────────────────────────────────────────────────────────┘
```

## Installation

### Package Install

**AUR (Arch / Arch-derived)**

```bash
paru -S vm-foundry
yay -S vm-foundry
```

**crates.io**

```bash
cargo install vm-foundry
```

**Binary Packages**

Pre-built packages (DEB, RPM, AppImage, tarball) can be published from [GitHub Releases](https://github.com/BlindMint/vm-foundry/releases).

### Build From Source

```bash
git clone https://github.com/BlindMint/vm-foundry.git
cd vm-foundry
cargo build --release
```

The built binary will be at `target/release/vm-foundry`.

### Install a Local Build Safely

If you already have an installed package, do not overwrite `/usr/bin/vm-foundry` directly. That path is package-managed.

Install your locally built binary to `/usr/local/bin` instead:

```bash
sudo install -Dm755 target/release/vm-foundry /usr/local/bin/vm-foundry
```

That keeps your custom build separate from any package-managed upstream binary while still allowing `/usr/local/bin/vm-foundry` to take precedence on most systems.

To confirm which binary will run:

```bash
type -a vm-foundry
```

## Requirements

**Runtime**
- QEMU (`qemu-system-*`)
- `qemu-img`
- `libudev`

**Build**
- Rust 1.70+
- `libudev-dev` on Debian/Ubuntu or `systemd-libs` on Arch/Fedora

**Optional**
- OVMF/edk2 for UEFI boot support
- `virt-viewer` for SPICE-app display
- `passt` for the passt network backend
- Looking Glass client for multi-GPU passthrough
- `polkit` for bridge networking and other privileged actions

## Usage

### TUI Mode

```bash
vm-foundry
```

### CLI Commands

```bash
# List all VMs
vm-foundry list

# Launch a VM
vm-foundry launch windows-95
vm-foundry launch windows-95 --install
vm-foundry launch windows-95 --cdrom /path/to/image.iso

# View VM configuration
vm-foundry info windows-95

# Manage snapshots
vm-foundry snapshot windows-95 list
vm-foundry snapshot windows-95 create my-snapshot
vm-foundry snapshot windows-95 restore my-snapshot
vm-foundry snapshot windows-95 delete my-snapshot

# List available QEMU emulators
vm-foundry emulators
```

## Key Bindings

### Main Menu

| Key | Action |
|-----|--------|
| `j/k` or `Down/Up` | Navigate VM list |
| `Enter` | Launch selected VM |
| `m` | Open management menu |
| `x` | Stop VM (if running) |
| `c` | Open VM creation wizard |
| `i` | Open VM import wizard |
| `s` | Open settings |
| `/` | Search/filter VMs |
| `?` | Show help |
| `PgUp/PgDn` | Scroll info panel |
| `Esc` | Back / Cancel |
| `q` | Quit |

### VM Management

| Key | Action |
|-----|--------|
| `j/k` or `Down/Up` | Navigate menu |
| `Enter` | Select menu option |
| `e` | Edit launch script |
| `u` | Configure USB passthrough |

Management menu options include:
- Boot options
- Snapshots
- USB passthrough
- PCI passthrough
- Shared folders
- Network settings
- Multi-GPU passthrough
- Single-GPU passthrough
- Change display
- Edit notes
- Rename VM
- Stop / force stop
- Reset VM
- Delete VM
- Edit raw configuration

### Create Wizard

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next/previous field |
| `Enter` | Select / Continue |
| `n` | Next step |
| `p` | Previous step |
| `Esc` | Cancel wizard |

## Configuration

Settings are stored in `~/.config/vm-foundry/config.toml` and can also be edited from the TUI settings screen.

```toml
# VM library location
vm_library_path = "~/vm-space"

# Default values for new VMs
default_memory_mb = 4096
default_cpu_cores = 2
default_disk_size_gb = 64
default_display = "gtk"
default_enable_kvm = true

# Behavior
confirm_before_launch = true

# Multi-GPU passthrough (Looking Glass)
enable_multi_gpu_passthrough = false
default_ivshmem_size_mb = 64
show_gpu_warnings = true
looking_glass_client_path = ""
looking_glass_auto_launch = true

# Single GPU passthrough
single_gpu_enabled = false
single_gpu_auto_tty = false
single_gpu_dm_override = ""
```

## VM Library Structure

VMs are expected in your library directory (default `~/vm-space/`) with a structure like:

```text
~/vm-space/
├── windows-95/
│   ├── launch.sh
│   └── disk.qcow2
├── linux-debian/
│   ├── launch.sh
│   ├── disk.qcow2
│   └── install.iso
└── macos-tiger/
    ├── launch.sh
    └── disk.qcow2
```

`vm-foundry` parses `launch.sh` to discover and manage existing VMs, and it can generate new scripts from the creation wizard.

## OS Profiles

The creation wizard includes 120+ pre-configured profiles across major families including:

- Microsoft
- Apple
- Linux
- BSD
- Unix
- IBM
- Commodore
- Be / Haiku
- NeXT
- Research
- Alternative
- Retro
- Mobile
- Infrastructure
- Utilities
- Other / catch-all

Each profile includes QEMU defaults such as emulator, machine type, CPU model, VGA, audio, network adapter, disk interface, and compatibility notes.

## Metadata Customization

**OS metadata**

Override or add OS metadata in `~/.config/vm-foundry/metadata/`:

```toml
[my-custom-os]
name = "My Custom OS"
publisher = "My Company"
release_date = "2024-01-01"
architecture = "x86_64"

[my-custom-os.blurb]
short = "A brief description"
long = "A longer description with history and details."

[my-custom-os.fun_facts]
facts = ["Fact 1", "Fact 2"]
```

**ASCII art**

Add custom ASCII art in `~/.config/vm-foundry/ascii/`.

**QEMU profiles**

Override profiles in `~/.config/vm-foundry/qemu_profiles.toml`.

## Cross-Distribution Notes

`vm-foundry` automatically detects OVMF/UEFI firmware paths across major Linux distributions, including Arch, Debian/Ubuntu, Fedora/RHEL, and NixOS-compatible layouts.

## Project Status

This fork is maintained for practical desktop QEMU workflows. See [CHANGELOG.md](./CHANGELOG.md) for fork-specific fixes, UI improvements, and upstream history.

## Contributing

Contributions are welcome. If you find a bug or have an idea for an improvement, open an issue or submit a pull request.

### Help Wanted: ASCII Art

As a TUI application, `vm-foundry` benefits from strong terminal aesthetics. Useful contributions include:

- Logo or banner art for the startup screen
- Small ASCII/block-style icons for menus and status views

## License

MIT
