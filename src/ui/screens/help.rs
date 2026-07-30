use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Render the help screen
pub fn render(frame: &mut Frame) {
    let area = frame.area();
    let dialog_width = 78.min(area.width.saturating_sub(4));
    let dialog_height = 32.min(area.height.saturating_sub(4));

    let dialog_area = centered_rect(dialog_width, dialog_height, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(crate::ui::modal_background()));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let text = vec![
        section("Main"),
        key_line("j / k, arrows", "Move through lists and menus"),
        key_line("Enter", "Launch, confirm, or open the selected item"),
        key_line("Esc", "Back or cancel"),
        key_line("/", "Search VMs"),
        key_line("c", "Create a VM"),
        key_line("i", "Import a VM"),
        key_line("m", "Open the management menu"),
        key_line("s", "Open settings"),
        key_line("x", "Stop the selected VM"),
        key_line("?", "Open this help"),
        key_line("q", "Quit"),
        blank(),
        section("Create Wizard"),
        key_line("Tab", "Move between edit fields and the OS filter"),
        key_line("/", "Jump to the OS filter"),
        key_line("Space", "Toggle selected options"),
        key_line("Enter", "Confirm the current choice or advance"),
        key_line("Esc", "Back to the previous step"),
        key_line(
            "Busy overlay",
            "Long operations show a centered progress box",
        ),
        blank(),
        section("Management (IDE layout)"),
        key_line("j / k", "Move categories (left) or actions (right)"),
        key_line("Tab / l / Enter", "Focus the action pane"),
        key_line("h / Esc", "Back to categories, then library"),
        key_line("1-9", "Quick-select an action in the current category"),
        key_line(
            "Categories",
            "Overview, Run, Network, Storage, Shared Folders, Devices, Advanced",
        ),
        key_line(
            "Network",
            "Bridge screens highlight lab-safe vs exposed bridges",
        ),
        blank(),
        section("Settings"),
        key_line(
            "Enter / Space",
            "Toggle, select, or edit the current setting",
        ),
        key_line(
            "Headers",
            "Section headers are informational and skipped in navigation",
        ),
        key_line(
            "GPU modes",
            "Disabled, Multiple GPUs, and Single GPU are exclusive modes",
        ),
        blank(),
        section("Status"),
        key_line(
            "Top-right notifications",
            "Short-lived success, warning, and error messages",
        ),
        key_line(
            "Header",
            "Shows VM counts, running status, and library path",
        ),
        blank(),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::Gray),
        )),
    ];

    let para = Paragraph::new(text).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn blank() -> Line<'static> {
    Line::from("")
}

fn key_line(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:18}", key), Style::default().fg(Color::Green)),
        Span::raw(description.to_string()),
    ])
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
