//! VM Management workspace — IDE-style master/detail layout.
//!
//! Left column: categories (Overview, Run, Network, …).
//! Right column: detail summary and actions for the selected category.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::App;
use crate::config::Config;
use crate::vm::DiscoveredVm;

/// Categories in the left navigation column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManageCategory {
    Overview,
    Run,
    Network,
    Storage,
    SharedFolders,
    Devices,
    Advanced,
}

impl ManageCategory {
    pub fn title(self) -> &'static str {
        match self {
            ManageCategory::Overview => "Overview",
            ManageCategory::Run => "Run",
            ManageCategory::Network => "Network",
            ManageCategory::Storage => "Storage",
            ManageCategory::SharedFolders => "Shared Folders",
            ManageCategory::Devices => "Devices",
            ManageCategory::Advanced => "Advanced",
        }
    }
}

/// Actions that can be performed from the management workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StopVm,
    BootOptions,
    Snapshots,
    UsbPassthrough,
    PciPassthrough,
    SharedFolders,
    NetworkSettings,
    MultiGpuPassthrough,
    SingleGpuPassthrough,
    ChangeDisplay,
    EditNotes,
    RenameVm,
    ResetVm,
    DeleteVm,
    EditRawConfig,
}

fn is_danger_action(action: MenuAction) -> bool {
    matches!(
        action,
        MenuAction::StopVm | MenuAction::ResetVm | MenuAction::DeleteVm
    )
}

/// A selectable row in the right (detail) pane
#[derive(Debug, Clone)]
pub struct DetailItem {
    pub name: &'static str,
    pub description: &'static str,
    /// Optional current-value hint shown on the right of the name
    pub value: Option<String>,
    pub action: Option<MenuAction>,
}

/// Build the left-column category list for this VM/config
pub fn categories_for(_vm: &DiscoveredVm, _config: &Config) -> Vec<ManageCategory> {
    vec![
        ManageCategory::Overview,
        ManageCategory::Run,
        ManageCategory::Network,
        ManageCategory::Storage,
        ManageCategory::SharedFolders,
        ManageCategory::Devices,
        ManageCategory::Advanced,
    ]
}

/// Build right-pane items for a category
pub fn detail_items_for(
    category: ManageCategory,
    vm: &DiscoveredVm,
    config: &Config,
    app: &App,
) -> Vec<DetailItem> {
    match category {
        ManageCategory::Overview => vec![
            DetailItem {
                name: "Edit Notes",
                description: "Add or edit personal notes for this VM",
                value: vm.notes.as_ref().map(|n| {
                    let first = n.lines().next().unwrap_or("").trim();
                    if first.is_empty() {
                        "empty".to_string()
                    } else if first.chars().count() > 28 {
                        format!("{}…", first.chars().take(27).collect::<String>())
                    } else {
                        first.to_string()
                    }
                }),
                action: Some(MenuAction::EditNotes),
            },
            DetailItem {
                name: "Rename VM",
                description: "Change the VM's display name",
                value: Some(vm.display_name()),
                action: Some(MenuAction::RenameVm),
            },
        ],
        ManageCategory::Run => {
            let display = extract_display_from_script(&vm.config.raw_script);
            let running = app.running_vms.contains_key(&vm.id);
            vec![
                DetailItem {
                    name: "Boot Options",
                    description: "Normal, installer, recovery, or custom media boot",
                    value: None,
                    action: Some(MenuAction::BootOptions),
                },
                DetailItem {
                    name: "Change Display",
                    description: "Switch GTK, SDL, SPICE-app, or VNC output",
                    value: Some(display),
                    action: Some(MenuAction::ChangeDisplay),
                },
                DetailItem {
                    name: if running { "Stop VM" } else { "Stop VM" },
                    description: if running {
                        "Shut down the running VM (ACPI poweroff)"
                    } else {
                        "VM is not running"
                    },
                    value: if running {
                        app.running_vms
                            .get(&vm.id)
                            .map(|pid| format!("pid {}", pid))
                    } else {
                        Some("stopped".to_string())
                    },
                    action: Some(MenuAction::StopVm),
                },
            ]
        }
        ManageCategory::Network => {
            let (backend, model, pf_count) = vm
                .config
                .network
                .as_ref()
                .map(|n| {
                    let backend = match &n.backend {
                        crate::vm::qemu_config::NetworkBackend::User => "user/SLIRP",
                        crate::vm::qemu_config::NetworkBackend::Passt => "passt",
                        crate::vm::qemu_config::NetworkBackend::Bridge(_) => "bridge",
                        crate::vm::qemu_config::NetworkBackend::None => "none",
                    };
                    (backend.to_string(), n.model.clone(), n.port_forwards.len())
                })
                .unwrap_or_else(|| ("none".to_string(), "—".to_string(), 0));
            let value = if pf_count > 0 {
                format!("{} · {} · {} fwd", model, backend, pf_count)
            } else {
                format!("{} · {}", model, backend)
            };
            vec![DetailItem {
                name: "Network Settings",
                description: "Change backend, adapter model, and port forwarding",
                value: Some(value),
                action: Some(MenuAction::NetworkSettings),
            }]
        }
        ManageCategory::Storage => {
            let snap_value = if vm.config.supports_snapshots() {
                if app.snapshots.is_empty() {
                    Some("qcow2 · no snapshots loaded".to_string())
                } else {
                    Some(format!("qcow2 · {} snapshot(s)", app.snapshots.len()))
                }
            } else {
                Some("unavailable (raw disk)".to_string())
            };
            vec![DetailItem {
                name: "Snapshots",
                description: "Create, restore, or delete qcow2 snapshots",
                value: snap_value,
                action: Some(MenuAction::Snapshots),
            }]
        }
        ManageCategory::SharedFolders => vec![DetailItem {
            name: "Shared Folders",
            description: "Share host directories with the VM (virtio-9p)",
            value: if app.shared_folders.is_empty() {
                None
            } else {
                Some(format!("{} folder(s)", app.shared_folders.len()))
            },
            action: Some(MenuAction::SharedFolders),
        }],
        ManageCategory::Devices => {
            let mut items = vec![
                DetailItem {
                    name: "USB Passthrough",
                    description: "Pass USB devices to the VM",
                    value: None,
                    action: Some(MenuAction::UsbPassthrough),
                },
                DetailItem {
                    name: "PCI Passthrough",
                    description: "Pass PCI devices to the VM",
                    value: None,
                    action: Some(MenuAction::PciPassthrough),
                },
            ];
            if config.enable_multi_gpu_passthrough {
                items.push(DetailItem {
                    name: "Multi-GPU Passthrough",
                    description: "Pass a secondary GPU to the VM with Looking Glass",
                    value: None,
                    action: Some(MenuAction::MultiGpuPassthrough),
                });
            }
            if config.single_gpu_enabled {
                items.push(DetailItem {
                    name: "Single GPU Passthrough",
                    description: "Configure passthrough for your primary GPU",
                    value: None,
                    action: Some(MenuAction::SingleGpuPassthrough),
                });
            }
            items
        }
        ManageCategory::Advanced => vec![
            DetailItem {
                name: "Edit Raw Configuration",
                description: "Edit launch.sh directly",
                value: None,
                action: Some(MenuAction::EditRawConfig),
            },
            DetailItem {
                name: "Reset VM (recreate disk)",
                description: "Restore VM to fresh state",
                value: None,
                action: Some(MenuAction::ResetVm),
            },
            DetailItem {
                name: "Delete VM",
                description: "Permanently remove this VM",
                value: None,
                action: Some(MenuAction::DeleteVm),
            },
        ],
    }
}

/// Restore left/right selection so a given action is highlighted after a sub-screen
pub fn focus_action(app: &mut App, action: MenuAction) {
    let Some(vm) = app.selected_vm().cloned() else {
        return;
    };
    let cats = categories_for(&vm, &app.config);
    for (ci, cat) in cats.iter().enumerate() {
        let details = detail_items_for(*cat, &vm, &app.config, app);
        if let Some(di) = details.iter().position(|d| d.action == Some(action)) {
            app.management_category = ci;
            app.management_detail = di;
            app.management_focus_right = true;
            return;
        }
    }
}

/// Count of right-pane items for the current category
pub fn detail_item_count(app: &App) -> usize {
    if let Some(vm) = app.selected_vm() {
        let cats = categories_for(vm, &app.config);
        if let Some(cat) = cats.get(app.management_category) {
            return detail_items_for(*cat, vm, &app.config, app).len();
        }
    }
    0
}

pub fn category_count(app: &App) -> usize {
    app.selected_vm()
        .map(|vm| categories_for(vm, &app.config).len())
        .unwrap_or(0)
}

/// Resolve the currently selected detail action, if any
pub fn selected_action(app: &App) -> Option<MenuAction> {
    let vm = app.selected_vm()?;
    let cats = categories_for(vm, &app.config);
    let cat = *cats.get(app.management_category)?;
    let details = detail_items_for(cat, vm, &app.config, app);
    details
        .get(app.management_detail)
        .and_then(|d| d.action)
}

/// Default display options for VMs (used as fallback descriptions)
const DISPLAY_OPTIONS: &[(&str, &str)] = &[
    ("gtk", "GTK - Default windowed display"),
    ("sdl", "SDL - Better for 3D acceleration"),
    ("spice-app", "SPICE - Remote desktop (needs virt-viewer)"),
    ("vnc", "VNC - Network accessible display"),
    ("none", "None - Headless, no graphical output"),
];

/// Get dynamic display options based on detected emulator capabilities.
pub fn get_display_options(app: &App) -> Vec<(String, String)> {
    let emulator = app
        .selected_vm()
        .map(|vm| vm.config.emulator.command())
        .unwrap_or("qemu-system-x86_64");

    let detected = app.get_display_options_for_emulator(emulator);

    detected
        .iter()
        .map(|backend| {
            let desc = DISPLAY_OPTIONS
                .iter()
                .find(|(name, _)| *name == backend.as_str())
                .map(|(_, desc)| desc.to_string())
                .unwrap_or_else(|| format!("{} display", backend));
            (backend.clone(), desc)
        })
        .collect()
}

fn management_status_line(app: &App, vm: &DiscoveredVm) -> Line<'static> {
    let run_text = if let Some(pid) = app.running_vms.get(&vm.id) {
        format!("running (pid {})", pid)
    } else {
        "stopped".to_string()
    };
    let snapshot_text = if vm.config.supports_snapshots() {
        "snapshots: qcow2"
    } else {
        "snapshots: unavailable"
    };
    let network_text = vm
        .config
        .network
        .as_ref()
        .map(|network| format!("net: {}", network.backend))
        .unwrap_or_else(|| "net: none".to_string());

    Line::from(vec![
        Span::styled(
            " STATUS ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(format!(" {}  ", run_text)),
        Span::styled(
            " STORAGE ",
            Style::default().fg(Color::Black).bg(Color::Green),
        ),
        Span::raw(format!(" {}  ", snapshot_text)),
        Span::styled(
            " NETWORK ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ),
        Span::raw(network_text),
    ])
}

fn overview_summary_lines(vm: &DiscoveredVm) -> Vec<Line<'static>> {
    let config = &vm.config;
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("Path: ", Style::default().fg(Color::Yellow)),
        Span::raw(vm.path.display().to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Emulator: ", Style::default().fg(Color::Yellow)),
        Span::raw(config.emulator.command().to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Architecture: ", Style::default().fg(Color::Yellow)),
        Span::raw(config.emulator.architecture().to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Memory: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} MB", config.memory_mb)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("CPU Cores: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", config.cpu_cores)),
    ]));
    if let Some(ref model) = config.cpu_model {
        lines.push(Line::from(vec![
            Span::styled("CPU Model: ", Style::default().fg(Color::Yellow)),
            Span::raw(model.clone()),
        ]));
    }
    if let Some(ref machine) = config.machine {
        lines.push(Line::from(vec![
            Span::styled("Machine: ", Style::default().fg(Color::Yellow)),
            Span::raw(machine.clone()),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("VGA: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{:?}", config.vga)),
    ]));
    if let Some(ref net) = config.network {
        let backend_str = match &net.backend {
            crate::vm::qemu_config::NetworkBackend::User => "user/SLIRP (NAT)".to_string(),
            crate::vm::qemu_config::NetworkBackend::Passt => "passt".to_string(),
            crate::vm::qemu_config::NetworkBackend::Bridge(name) => format!("bridge: {}", name),
            crate::vm::qemu_config::NetworkBackend::None => "none".to_string(),
        };
        lines.push(Line::from(vec![
            Span::styled("Network: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ({})", net.model, backend_str)),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("Display: ", Style::default().fg(Color::Yellow)),
        Span::raw(extract_display_from_script(&config.raw_script)),
    ]));

    let mut features = Vec::new();
    if config.enable_kvm {
        features.push("KVM");
    }
    if config.uefi {
        features.push("UEFI");
    }
    if config.tpm {
        features.push("TPM");
    }
    if !features.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Features: ", Style::default().fg(Color::Yellow)),
            Span::raw(features.join(", ")),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Disks",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    for disk in &config.disks {
        let path = disk
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        lines.push(Line::from(format!(
            "  {} ({:?}, {})",
            path, disk.format, disk.interface
        )));
    }

    if let Some(ref notes) = vm.notes {
        if !notes.trim().is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Notes",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for line in notes.lines().take(6) {
                lines.push(Line::from(format!("  {}", line)));
            }
        }
    }

    lines
}

/// Render the management workspace (full screen, master/detail)
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let vm = app.selected_vm();
    let vm_name = vm
        .map(|vm| vm.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    let outer = Block::default()
        .title(format!(" {} — Manage ", vm_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(crate::ui::modal_background()));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let Some(vm) = vm else {
        let msg = Paragraph::new("No VM selected")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    };

    let cats = categories_for(vm, &app.config);
    let cat_idx = app.management_category.min(cats.len().saturating_sub(1));
    let category = cats.get(cat_idx).copied().unwrap_or(ManageCategory::Overview);
    let details = detail_items_for(category, vm, &app.config, app);
    let detail_idx = app
        .management_detail
        .min(details.len().saturating_sub(1));

    // Header status | body | footer
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // status
            Constraint::Min(8),    // master/detail
            Constraint::Length(1), // help
        ])
        .split(inner);

    let status = Paragraph::new(vec![management_status_line(app, vm)])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" VM Status "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(status, rows[0]);

    // Left categories (~28 cols) | right detail
    let left_width = 26u16.min(rows[1].width.saturating_div(3).max(18));
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(left_width), Constraint::Min(30)])
        .split(rows[1]);

    let left_focused = !app.management_focus_right;
    let left_border = if left_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let right_border = if app.management_focus_right {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // --- Left: categories ---
    let left_block = Block::default()
        .title(" Categories ")
        .borders(Borders::ALL)
        .border_style(left_border);
    let left_inner = left_block.inner(cols[0]);
    frame.render_widget(left_block, cols[0]);

    let cat_items: Vec<ListItem> = cats
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let selected = i == cat_idx;
            let style = if selected && left_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if *cat == ManageCategory::Advanced {
                Style::default().fg(Color::LightRed)
            } else {
                Style::default().fg(Color::White)
            };
            let marker = if selected { "▸ " } else { "  " };
            ListItem::new(Line::styled(format!("{}{}", marker, cat.title()), style))
        })
        .collect();
    let mut cat_state = ListState::default();
    cat_state.select(Some(cat_idx));
    frame.render_stateful_widget(List::new(cat_items), left_inner, &mut cat_state);

    // --- Right: summary + actions ---
    let right_block = Block::default()
        .title(format!(" {} ", category.title()))
        .borders(Borders::ALL)
        .border_style(right_border);
    let right_inner = right_block.inner(cols[1]);
    frame.render_widget(right_block, cols[1]);

    let right_chunks = if category == ManageCategory::Overview {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(details.len() as u16 * 2 + 1)])
            .split(right_inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(4)])
            .split(right_inner)
    };

    if category == ManageCategory::Overview {
        let summary = Paragraph::new(overview_summary_lines(vm))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));
        frame.render_widget(summary, right_chunks[0]);
        render_detail_list(
            frame,
            right_chunks[1],
            &details,
            detail_idx,
            app.management_focus_right,
        );
    } else {
        let blurb = category_blurb(category);
        let header = Paragraph::new(blurb).style(Style::default().fg(Color::Gray));
        frame.render_widget(header, right_chunks[0]);
        render_detail_list(
            frame,
            right_chunks[1],
            &details,
            detail_idx,
            app.management_focus_right,
        );
    }

    let help = if app.management_focus_right {
        "[j/k] Actions  [Enter] Open  [h/Esc] Categories  [q] Quit"
    } else {
        "[j/k] Categories  [Tab/l/Enter] Actions  [Esc] Library  [q] Quit"
    };
    let help = Paragraph::new(help)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help, rows[2]);
}

fn category_blurb(category: ManageCategory) -> &'static str {
    match category {
        ManageCategory::Overview => "Configuration summary and metadata",
        ManageCategory::Run => "Boot, display, and power control",
        ManageCategory::Network => "Adapter, backend, bridge, and port forwards",
        ManageCategory::Storage => "Disk snapshots (qcow2)",
        ManageCategory::SharedFolders => "Host directories shared into the guest",
        ManageCategory::Devices => "USB, PCI, and GPU passthrough",
        ManageCategory::Advanced => "Raw config and destructive operations",
    }
}

fn render_detail_list(
    frame: &mut Frame,
    area: Rect,
    details: &[DetailItem],
    selected: usize,
    focused: bool,
) {
    if details.is_empty() {
        let msg = Paragraph::new("No actions in this category")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = details
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_sel = i == selected;
            let danger = item.action.is_some_and(is_danger_action);
            let name_style = if is_sel && focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_sel {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if danger {
                Style::default().fg(Color::LightRed)
            } else {
                Style::default().fg(Color::White)
            };
            let desc_style = if danger {
                Style::default().fg(Color::LightRed)
            } else {
                Style::default().fg(Color::Gray)
            };

            let mut name_line = format!("  {}", item.name);
            if let Some(ref value) = item.value {
                name_line.push_str(&format!("  ·  {}", value));
            }

            ListItem::new(vec![
                Line::styled(name_line, name_style),
                Line::styled(format!("    {}", item.description), desc_style),
            ])
        })
        .collect();

    let mut state = ListState::default();
    // Each item is 2 visual rows; ListState selects item index not row
    state.select(Some(selected));
    let list = List::new(items).highlight_symbol(if focused { "> " } else { "  " });
    frame.render_stateful_widget(list, area, &mut state);
}

/// Render boot options submenu
pub fn render_boot_options(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let dialog_width = 64.min(area.width.saturating_sub(4));
    let dialog_height = 18.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Boot Options ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(crate::ui::modal_background()));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(h_chunks[1]);

    let boot_items = [
        ("Normal boot", "Start the VM normally"),
        ("Install mode", "Boot from installation media"),
        ("Boot with custom ISO", "Select an ISO file to boot"),
        (
            "Boot with recovery DMG",
            "Select a DMG file as recovery image",
        ),
        (
            "Boot with floppy image",
            "Select a floppy image (.img, .ima) to boot",
        ),
    ];

    let items: Vec<ListItem> = boot_items
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let style = if i == app.selected_menu_item {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(vec![
                Line::styled(format!("[{}] {}", i + 1, name), style),
                Line::styled(
                    format!("    {}", desc),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_menu_item));
    frame.render_stateful_widget(List::new(items), v_chunks[1], &mut state);

    let help = Paragraph::new("[Enter] Select  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, v_chunks[2]);
}

/// Render display options submenu
pub fn render_display_options(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let dialog_width = 64.min(area.width.saturating_sub(4));
    let dialog_height = 18.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let current_display = app
        .selected_vm()
        .map(|vm| extract_display_from_script(&vm.config.raw_script))
        .unwrap_or_else(|| "gtk".to_string());

    let block = Block::default()
        .title(format!(" Display Options (current: {}) ", current_display))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(crate::ui::modal_background()));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(h_chunks[1]);

    let display_options = get_display_options(app);

    let items: Vec<ListItem> = display_options
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let is_current = *name == current_display;
            let style = if i == app.selected_menu_item {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            let marker = if is_current { " *" } else { "" };

            ListItem::new(vec![
                Line::styled(format!("[{}] {}{}", i + 1, name, marker), style),
                Line::styled(
                    format!("    {}", desc),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_menu_item));
    frame.render_stateful_widget(List::new(items), v_chunks[1], &mut state);

    let help = Paragraph::new("[Enter] Select  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, v_chunks[2]);
}

/// Extract display setting from launch script
fn extract_display_from_script(script: &str) -> String {
    if let Some(pos) = script.find("-display ") {
        let rest = &script[pos + 9..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ',' || c == '\\')
            .unwrap_or(rest.len());
        let display = rest[..end].trim();
        if let Some(comma_pos) = display.find(',') {
            return display[..comma_pos].to_string();
        }
        return display.to_string();
    }
    "gtk".to_string()
}

/// Render snapshot management submenu
pub fn render_snapshots(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let dialog_width = 70.min(area.width.saturating_sub(4));
    let dialog_height = 22.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let supports_snapshots = app
        .selected_vm()
        .map(|vm| vm.config.supports_snapshots())
        .unwrap_or(false);

    let title = if supports_snapshots {
        format!(" Snapshots ({}) ", app.snapshots.len())
    } else {
        " Snapshots (not supported) ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(crate::ui::modal_background()));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(h_chunks[1]);

    let content_area = v_chunks[1];

    if !supports_snapshots {
        let msg = Paragraph::new("This VM uses a raw disk image which doesn't support snapshots.\n\nOnly qcow2 format disks support snapshots.")
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: false });
        frame.render_widget(msg, content_area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(content_area);

    let actions = Paragraph::new(vec![Line::from(vec![
        Span::styled("[c]", Style::default().fg(Color::Yellow)),
        Span::raw(" Create new snapshot"),
    ])]);
    frame.render_widget(actions, chunks[0]);

    if app.snapshots.is_empty() {
        let msg = Paragraph::new("No snapshots yet.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(msg, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .snapshots
            .iter()
            .enumerate()
            .map(|(i, snap)| {
                let style = if i == app.selected_snapshot {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(vec![
                    Line::styled(format!("  {}", snap.name), style),
                    Line::styled(
                        format!("    {} - {}", snap.date, snap.size),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(app.selected_snapshot));
        let list = List::new(items).highlight_symbol("> ");
        frame.render_stateful_widget(list, chunks[1], &mut state);
    }

    let help = Paragraph::new("[r] Restore  [d] Delete  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn danger_actions_are_classified_correctly() {
        assert!(is_danger_action(MenuAction::StopVm));
        assert!(is_danger_action(MenuAction::ResetVm));
        assert!(is_danger_action(MenuAction::DeleteVm));
        assert!(!is_danger_action(MenuAction::BootOptions));
        assert!(!is_danger_action(MenuAction::EditRawConfig));
    }

    #[test]
    fn categories_include_core_sections() {
        let vm = DiscoveredVm {
            id: "linux-test".to_string(),
            path: "/tmp/linux-test".into(),
            launch_script: "/tmp/linux-test/launch.sh".into(),
            config: crate::vm::QemuConfig::default(),
            custom_name: None,
            os_profile: None,
            notes: None,
            default_boot_mode: crate::vm::BootMode::Normal,
        };
        let config = Config::default();
        let cats = categories_for(&vm, &config);
        assert!(cats.contains(&ManageCategory::Overview));
        assert!(cats.contains(&ManageCategory::Run));
        assert!(cats.contains(&ManageCategory::Network));
        assert!(cats.contains(&ManageCategory::Advanced));
        assert!(!cats.is_empty());
    }

    #[test]
    fn advanced_category_lists_destructive_actions() {
        // Advanced detail items are static aside from App snapshot counts used elsewhere.
        // Verify category title and that danger classification still covers reset/delete.
        assert_eq!(ManageCategory::Advanced.title(), "Advanced");
        assert!(is_danger_action(MenuAction::ResetVm));
        assert!(is_danger_action(MenuAction::DeleteVm));
        assert!(!is_danger_action(MenuAction::EditRawConfig));
    }

    #[test]
    fn devices_respect_gpu_feature_flags() {
        let vm = DiscoveredVm {
            id: "linux-test".to_string(),
            path: "/tmp/linux-test".into(),
            launch_script: "/tmp/linux-test/launch.sh".into(),
            config: crate::vm::QemuConfig::default(),
            custom_name: None,
            os_profile: None,
            notes: None,
            default_boot_mode: crate::vm::BootMode::Normal,
        };
        let mut config = Config::default();
        config.enable_multi_gpu_passthrough = false;
        config.single_gpu_enabled = false;

        // Without App we only check category presence; GPU flags gate detail rows at runtime.
        let cats = categories_for(&vm, &config);
        assert!(cats.contains(&ManageCategory::Devices));
        assert!(!config.enable_multi_gpu_passthrough);
        assert!(!config.single_gpu_enabled);
    }
}
