use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::ui::widgets::{AsciiInfoWidget, VmListWidget};

/// Render the main menu screen
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status/help bar
        ])
        .split(area);

    // Render title
    render_title(app, chunks[0], frame);

    // Split main content: VM list on left, info on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Render VM list
    VmListWidget::new(app).render(main_chunks[0], frame.buffer_mut());

    // Render ASCII art and info
    let vm_name = app
        .selected_vm()
        .map(|vm| vm.display_name())
        .unwrap_or_else(|| "No VM selected".to_string());

    let os_info = app.selected_vm_info();
    let ascii_art = app.selected_vm_ascii();

    let notes = app.selected_vm().and_then(|vm| vm.notes.as_deref());

    AsciiInfoWidget {
        ascii_art,
        os_info: os_info.as_ref(),
        vm_name: &vm_name,
        scroll: app.info_scroll,
        notes,
    }
    .render(main_chunks[1], frame.buffer_mut());

    // Render help bar
    render_help_bar(app, chunks[2], frame);
}

fn render_title(app: &App, area: Rect, frame: &mut Frame) {
    // Format the library path, shortening home directory to ~
    let library_path = &app.config.vm_library_path;
    let display_path = if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = library_path.strip_prefix(&home) {
            format!("~/{}", stripped.display())
        } else {
            library_path.display().to_string()
        }
    } else {
        library_path.display().to_string()
    };

    let title_text = " VM Foundry ";
    let available_width = area.width.saturating_sub(4) as usize;
    let mut line_spans = vec![Span::styled(
        title_text,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];
    let used_width = UnicodeWidthStr::width(title_text);
    let path_width_budget = available_width.saturating_sub(used_width + 24);
    let medium_path_width_budget = available_width.saturating_sub(used_width + 14);

    let statuses = [
        format!(
            "{} VMs | {} running | Desktop QEMU Lab Manager | {}",
            app.vms.len(),
            app.running_vms.len(),
            display_path
        ),
        format!(
            "{} VMs | {} running | {}",
            app.vms.len(),
            app.running_vms.len(),
            ellipsize_left(&display_path, path_width_budget.max(12))
        ),
        format!(
            "{} | {} | {}",
            app.vms.len(),
            app.running_vms.len(),
            ellipsize_left(&display_path, medium_path_width_budget.max(8))
        ),
        format!("{} | {}", app.vms.len(), app.running_vms.len()),
    ];

    let status_text = statuses
        .into_iter()
        .find(|status| used_width + 1 + UnicodeWidthStr::width(status.as_str()) <= available_width)
        .map(|status| format!(" {} ", status));

    if let Some(status_text) = status_text {
        let status_width = UnicodeWidthStr::width(status_text.as_str());
        let space_width = available_width
            .saturating_sub(used_width + status_width)
            .max(1);
        line_spans.push(Span::raw(" ".repeat(space_width)));
        line_spans.push(Span::styled(status_text, Style::default().fg(Color::Gray)));
    }

    let title = Paragraph::new(vec![Line::from(line_spans)])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(title, area);
}

fn ellipsize_left(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let chars: Vec<char> = text.chars().collect();
    for start in 0..chars.len() {
        let candidate: String = chars[start..].iter().collect();
        let rendered = format!("...{}", candidate);
        if UnicodeWidthStr::width(rendered.as_str()) <= max_width {
            return rendered;
        }
    }

    "...".to_string()
}

fn render_help_bar(_app: &App, area: Rect, frame: &mut Frame) {
    let hints = vec![
        Span::styled(" [Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" Launch "),
        Span::styled(" [x]", Style::default().fg(Color::Yellow)),
        Span::raw(" Stop "),
        Span::styled(" [m]", Style::default().fg(Color::Yellow)),
        Span::raw(" Manage "),
        Span::styled(" [c]", Style::default().fg(Color::Yellow)),
        Span::raw(" Create "),
        Span::styled(" [i]", Style::default().fg(Color::Yellow)),
        Span::raw(" Import "),
        Span::styled(" [s]", Style::default().fg(Color::Yellow)),
        Span::raw(" Settings "),
        Span::styled(" [/]", Style::default().fg(Color::Yellow)),
        Span::raw(" Search "),
        Span::styled(" [?]", Style::default().fg(Color::Yellow)),
        Span::raw(" Help "),
        Span::styled(" [q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit "),
    ];

    let help = Paragraph::new(Line::from(hints))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(help, area);
}
