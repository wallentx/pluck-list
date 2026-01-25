use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};
use std::{
    env,
    fs,
    io::{self, BufRead},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to read, or stdin if not provided
    file: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ActiveBuffer {
    Prompt,
    Modified,
    New,
    SingleList,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PluckMode {
    TopDown,
    BottomUp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AppState {
    Init,
    ModeSelect,
    CountInput(PluckMode),
    MatchInput,
    ApplyPluck,
    PostPluckModeSelect,
    SaveAs(ActiveBuffer),
    ConfirmOverwrite(PathBuf, ActiveBuffer),
    Message(String, Box<AppState>),
    Error(String, Box<AppState>),
}

struct ListBuffer {
    items: Vec<String>,
    state: ListState,
    scroll_offset: usize,
    viewport_height: usize,
}

impl ListBuffer {
    fn new(items: Vec<String>) -> Self {
        Self { 
            items, 
            state: ListState::default(),
            scroll_offset: 0,
            viewport_height: 0,
        }
    }

    fn max_offset(&self) -> usize {
        if self.viewport_height == 0 {
            return self.items.len().saturating_sub(1);
        }
        self.items.len().saturating_sub(self.viewport_height)
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.scroll_offset < self.max_offset() {
            self.scroll_offset += 1;
        }
        *self.state.offset_mut() = self.scroll_offset;
        self.state.select(None);
    }

    fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
        *self.state.offset_mut() = self.scroll_offset;
        self.state.select(None);
    }

    fn scroll_page(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        if delta > 0 {
            self.scroll_offset = (self.scroll_offset + delta as usize).min(self.max_offset());
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub((-delta) as usize);
        }
        *self.state.offset_mut() = self.scroll_offset;
        self.state.select(None);
    }
}

struct App {
    state: AppState,
    active_buffer: ActiveBuffer,
    modified_list: ListBuffer,
    new_list: ListBuffer,
    input_path: Option<PathBuf>,
    prompt_input: String,
    input_cursor_position: usize,
    mode_index: usize,
    preview_indices: Vec<usize>,
    post_pluck: bool,
}

impl App {
    fn new(items: Vec<String>, input_path: Option<PathBuf>) -> Self {
        Self {
            state: AppState::Init,
            active_buffer: ActiveBuffer::Prompt,
            modified_list: ListBuffer::new(items),
            new_list: ListBuffer::new(Vec::new()),
            input_path,
            prompt_input: String::new(),
            input_cursor_position: 0,
            mode_index: 0,
            preview_indices: Vec::new(),
            post_pluck: false,
        }
    }

    fn is_split_view(&self) -> bool {
        self.post_pluck
    }

    fn insert_char(&mut self, c: char) {
        if self.input_cursor_position >= self.prompt_input.len() {
            self.prompt_input.push(c);
        } else {
            self.prompt_input.insert(self.input_cursor_position, c);
        }
        self.input_cursor_position += 1;
    }

    fn delete_char(&mut self) {
        if self.input_cursor_position > 0 {
            if self.input_cursor_position >= self.prompt_input.len() {
                self.prompt_input.pop();
            } else {
                self.prompt_input.remove(self.input_cursor_position - 1);
            }
            self.input_cursor_position -= 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.input_cursor_position > 0 {
            self.input_cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.input_cursor_position < self.prompt_input.len() {
            self.input_cursor_position += 1;
        }
    }

    fn move_cursor_home(&mut self) {
        self.input_cursor_position = 0;
    }

    fn move_cursor_end(&mut self) {
        self.input_cursor_position = self.prompt_input.len();
    }

    fn cycle_active_buffer(&mut self) {
        if self.is_split_view() {
            self.active_buffer = match self.active_buffer {
                ActiveBuffer::Prompt => ActiveBuffer::Modified,
                ActiveBuffer::Modified => ActiveBuffer::New,
                ActiveBuffer::New => ActiveBuffer::Prompt,
                _ => ActiveBuffer::Prompt,
            };
        } else {
            self.active_buffer = match self.active_buffer {
                ActiveBuffer::Prompt => ActiveBuffer::SingleList,
                _ => ActiveBuffer::Prompt,
            };
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let items = if let Some(path) = args.file.clone() {
        let content = fs::read_to_string(path)?;
        content.lines().map(|s| s.to_string()).collect()
    } else {
        let stdin = io::stdin();
        stdin.lock().lines().collect::<Result<Vec<_>, _>>()?
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new(items, args.file);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
    loop {
        if app.state == AppState::Init {
            app.state = AppState::ModeSelect;
            app.active_buffer = ActiveBuffer::Prompt;
        }
        if app.state == AppState::ApplyPluck && app.active_buffer == ActiveBuffer::Prompt {
            app.state = AppState::PostPluckModeSelect;
        }

        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }
                match key.code {
                    KeyCode::Char('q') => {
                        match app.state {
                            AppState::MatchInput | AppState::SaveAs(_) => handle_key_event(app, key)?,
                            _ => return Ok(()),
                        }
                    }
                    KeyCode::Tab => app.cycle_active_buffer(),
                    _ => handle_key_event(app, key)?,
                }
            }
        }
    }
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    // Navigation keys (arrows, pgup/pgdn)
    match key.code {
        KeyCode::Up => {
            if app.active_buffer == ActiveBuffer::SingleList
                || app.active_buffer == ActiveBuffer::Modified
            {
                app.modified_list.previous();
            } else if app.active_buffer == ActiveBuffer::New {
                app.new_list.previous();
            }
        }
        KeyCode::Down => {
            if app.active_buffer == ActiveBuffer::SingleList
                || app.active_buffer == ActiveBuffer::Modified
            {
                app.modified_list.next();
            } else if app.active_buffer == ActiveBuffer::New {
                app.new_list.next();
            }
        }
        KeyCode::PageUp => {
            if app.active_buffer == ActiveBuffer::SingleList
                || app.active_buffer == ActiveBuffer::Modified
            {
                app.modified_list.scroll_page(-10);
            } else if app.active_buffer == ActiveBuffer::New {
                app.new_list.scroll_page(-10);
            }
        }
        KeyCode::PageDown => {
            if app.active_buffer == ActiveBuffer::SingleList
                || app.active_buffer == ActiveBuffer::Modified
            {
                app.modified_list.scroll_page(10);
            } else if app.active_buffer == ActiveBuffer::New {
                app.new_list.scroll_page(10);
            }
        }
        KeyCode::Char('s') => {
            if app.active_buffer == ActiveBuffer::Modified && app.input_path.is_some() {
                save_list(app, app.input_path.clone().unwrap(), ActiveBuffer::Modified)?;
                return Ok(());
            }
        }
        KeyCode::Char('S') => {
            if app.active_buffer == ActiveBuffer::Modified || app.active_buffer == ActiveBuffer::New
            {
                app.state = AppState::SaveAs(app.active_buffer);
                app.active_buffer = ActiveBuffer::Prompt;
                app.prompt_input.clear();
                app.input_cursor_position = 0;
                return Ok(());
            }
        }
        _ => {}
    }

    // State specific handling
    let current_state = app.state.clone();
    match current_state {
        AppState::ModeSelect | AppState::PostPluckModeSelect => {
            if app.active_buffer == ActiveBuffer::Prompt {
                match key.code {
                    KeyCode::Left | KeyCode::Up => {
                        if app.mode_index > 0 {
                            app.mode_index -= 1;
                        }
                    }
                    KeyCode::Right | KeyCode::Down => {
                        if app.mode_index < 2 {
                            app.mode_index += 1;
                        }
                    }
                    KeyCode::Enter => {
                        app.prompt_input.clear();
                        app.input_cursor_position = 0;
                        app.state = match app.mode_index {
                            0 => AppState::CountInput(PluckMode::TopDown),
                            1 => AppState::CountInput(PluckMode::BottomUp),
                            2 => {
                                update_preview(app);
                                AppState::MatchInput
                            }
                            _ => current_state,
                        };
                        app.active_buffer = ActiveBuffer::Prompt;
                    }
                    _ => {}
                }
            }
        }
        AppState::CountInput(mode) => {
            if app.active_buffer == ActiveBuffer::Prompt {
                match key.code {
                    KeyCode::Char(c) if c.is_digit(10) => {
                        app.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    KeyCode::Home => app.move_cursor_home(),
                    KeyCode::End => app.move_cursor_end(),
                    KeyCode::Enter => {
                        if let Ok(n) = app.prompt_input.parse::<usize>() {
                            apply_pluck(app, mode, n);
                            app.prompt_input.clear();
                            app.input_cursor_position = 0;
                        } else if !app.prompt_input.is_empty() {
                            app.state = AppState::Error(
                                "Invalid number".to_string(),
                                Box::new(AppState::CountInput(mode)),
                            );
                        }
                    }
                    KeyCode::Esc => {
                        app.state = if !app.post_pluck {
                            AppState::ModeSelect
                        } else {
                            AppState::PostPluckModeSelect
                        };
                        app.prompt_input.clear();
                        app.input_cursor_position = 0;
                    }
                    _ => {}
                }
            }
        }
        AppState::MatchInput => {
            if app.active_buffer == ActiveBuffer::Prompt {
                match key.code {
                    KeyCode::Char(c) => {
                        app.insert_char(c);
                        update_preview(app);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                        update_preview(app);
                    }
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    KeyCode::Home => app.move_cursor_home(),
                    KeyCode::End => app.move_cursor_end(),
                    KeyCode::Enter => {
                        apply_match_pluck(app);
                        app.prompt_input.clear();
                        app.input_cursor_position = 0;
                    }
                    KeyCode::Esc => {
                        app.state = if !app.post_pluck {
                            AppState::ModeSelect
                        } else {
                            AppState::PostPluckModeSelect
                        };
                        app.prompt_input.clear();
                        app.input_cursor_position = 0;
                        app.preview_indices.clear();
                    }
                    _ => {}
                }
            }
        }
        AppState::SaveAs(target) => {
            if app.active_buffer == ActiveBuffer::Prompt {
                match key.code {
                    KeyCode::Char(c) => {
                        app.insert_char(c);
                        app.state = AppState::SaveAs(target);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                        app.state = AppState::SaveAs(target);
                    }
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    KeyCode::Home => app.move_cursor_home(),
                    KeyCode::End => app.move_cursor_end(),
                    KeyCode::Enter => {
                        let path = PathBuf::from(&app.prompt_input);
                        if path.exists() {
                            app.state = AppState::ConfirmOverwrite(path, target);
                        } else {
                            if let Err(e) = save_list(app, path.clone(), target) {
                                app.state = AppState::Error(
                                    e.to_string(),
                                    Box::new(AppState::PostPluckModeSelect),
                                );
                            } else {
                                app.state = AppState::Message(
                                    format!("Successfully saved to {}", path.display()),
                                    Box::new(AppState::PostPluckModeSelect),
                                );
                                app.prompt_input.clear();
                                app.input_cursor_position = 0;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.state = AppState::PostPluckModeSelect;
                        app.prompt_input.clear();
                        app.input_cursor_position = 0;
                    }
                    _ => app.state = AppState::SaveAs(target),
                }
            } else {
                app.state = AppState::SaveAs(target);
            }
        }
        AppState::ConfirmOverwrite(path, target) => {
            if key.code == KeyCode::Char('y') || key.code == KeyCode::Char('Y') {
                if let Err(e) = save_list(app, path.clone(), target) {
                    app.state = AppState::Error(
                        e.to_string(),
                        Box::new(AppState::PostPluckModeSelect),
                    );
                } else {
                    app.state = AppState::Message(
                        format!("Successfully saved to {}", path.display()),
                        Box::new(AppState::PostPluckModeSelect),
                    );
                    app.prompt_input.clear();
                }
            } else if key.code == KeyCode::Char('n') || key.code == KeyCode::Char('N') || key.code == KeyCode::Esc {
                app.state = AppState::SaveAs(target);
            } else {
                app.state = AppState::ConfirmOverwrite(path, target);
            }
        }
        AppState::Message(_, return_to) => {
            if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                app.state = *return_to;
            }
        }
        AppState::Error(_, return_to) => {
            if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                app.state = *return_to;
            }
        }
        _ => {}
    }

    Ok(())
}

fn update_preview(app: &mut App) {
    app.preview_indices.clear();
    if app.prompt_input.is_empty() {
        return;
    }

    let pattern = &app.prompt_input;
    if let Ok(re) = regex::RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
    {
        for (i, item) in app.modified_list.items.iter().enumerate() {
            if re.is_match(item) {
                app.preview_indices.push(i);
            }
        }
    } else {
        // Fallback to simple string match if regex is invalid
        for (i, item) in app.modified_list.items.iter().enumerate() {
            if item.to_lowercase().contains(&pattern.to_lowercase()) {
                app.preview_indices.push(i);
            }
        }
    }
}

fn apply_match_pluck(app: &mut App) {
    let mut new_items = Vec::new();
    let mut remaining_items = Vec::new();

    let preview_set: std::collections::HashSet<usize> =
        app.preview_indices.iter().cloned().collect();

    let old_items = std::mem::take(&mut app.modified_list.items);
    for (i, item) in old_items.into_iter().enumerate() {
        if preview_set.contains(&i) {
            new_items.push(item);
        } else {
            remaining_items.push(item);
        }
    }

    app.modified_list = ListBuffer::new(remaining_items);
    app.new_list = ListBuffer::new(new_items);
    app.preview_indices.clear();
    app.state = AppState::ApplyPluck;
    app.post_pluck = true;
    app.active_buffer = ActiveBuffer::New;
}

fn save_list(app: &App, path: PathBuf, target: ActiveBuffer) -> io::Result<()> {
    let list = if target == ActiveBuffer::Modified {
        &app.modified_list
    } else {
        &app.new_list
    };

    let content = list.items.join("\n");
    fs::write(path, content)?;
    Ok(())
}

fn apply_pluck(app: &mut App, mode: PluckMode, n: usize) {
    let len = app.modified_list.items.len();
    let n = n.min(len);

    let plucked = match mode {
        PluckMode::TopDown => app.modified_list.items.drain(0..n).collect::<Vec<_>>(),
        PluckMode::BottomUp => app
            .modified_list
            .items
            .drain((len - n)..len)
            .collect::<Vec<_>>(),
    };

    app.new_list = ListBuffer::new(plucked);
    app.state = AppState::ApplyPluck;
    app.post_pluck = true;
    app.active_buffer = ActiveBuffer::New;
}

fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .split(f.area());

    let top_area = main_layout[0];
    let prompt_area = main_layout[1];
    let legend_area = main_layout[2];

    // Render top buffer(s)
    if !app.is_split_view() {
        render_list(
            f,
            top_area,
            &mut app.modified_list,
            "List",
            app.active_buffer == ActiveBuffer::SingleList,
            &app.preview_indices,
        );
    } else {
        let split_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(top_area);

        render_list(
            f,
            split_layout[0],
            &mut app.modified_list,
            "Modified List",
            app.active_buffer == ActiveBuffer::Modified,
            &app.preview_indices,
        );
        render_list(
            f,
            split_layout[1],
            &mut app.new_list,
            "New List",
            app.active_buffer == ActiveBuffer::New,
            &[],
        );
    }

    // Render prompt buffer
    let prompt_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.active_buffer == ActiveBuffer::Prompt {
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inactive_text_style = if app.active_buffer == ActiveBuffer::Prompt {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    match &app.state {
        AppState::Init => {} // Handled automatically in run_app
        AppState::ModeSelect | AppState::PostPluckModeSelect => {
            let options = ["Top-down", "Bottom-up", "String match"];
            let mut text = vec![Line::from("How would you like to pluck from this list?")];
            for (i, &opt) in options.iter().enumerate() {
                if i == app.mode_index {
                    text.push(Line::from(Span::styled(
                        format!("[{}]", opt),
                        Style::default()
                            .fg(if app.active_buffer == ActiveBuffer::Prompt { Color::Yellow } else { Color::Gray })
                            .add_modifier(Modifier::BOLD),
                    )));
                } else {
                    text.push(Line::from(format!(" {}", opt)));
                }
            }
            let paragraph = Paragraph::new(text).block(prompt_block).style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
        }
        AppState::CountInput(_) => {
            let prefix = "Enter number of lines to pluck: ";
            let paragraph = Paragraph::new(Line::from(format!(
                "{}{}",
                prefix,
                app.prompt_input
            )))
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
            if app.active_buffer == ActiveBuffer::Prompt {
                f.set_cursor_position((
                    prompt_area.x + 1 + prefix.len() as u16 + app.input_cursor_position as u16,
                    prompt_area.y + 1,
                ));
            }
        }
        AppState::MatchInput => {
            let prefix = "Match (regex supported): ";
            let paragraph = Paragraph::new(Line::from(format!(
                "{}{}",
                prefix,
                app.prompt_input
            )))
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
            if app.active_buffer == ActiveBuffer::Prompt {
                f.set_cursor_position((
                    prompt_area.x + 1 + prefix.len() as u16 + app.input_cursor_position as u16,
                    prompt_area.y + 1,
                ));
            }
        }
        AppState::ApplyPluck => {
            let paragraph = Paragraph::new(Line::from(format!(
                "Plucked {} lines. Focus input buffer to continue.",
                app.new_list.items.len()
            )))
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
        }
        AppState::SaveAs(_) => {
            let cwd = env::current_dir().unwrap_or_default();
            let prefix = "Save as: ";
            let paragraph = Paragraph::new(vec![
                Line::from(format!("{}{}", prefix, app.prompt_input)),
                Line::from(Span::styled(
                    format!("in {}", cwd.display()),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
            if app.active_buffer == ActiveBuffer::Prompt {
                f.set_cursor_position((
                    prompt_area.x + 1 + prefix.len() as u16 + app.input_cursor_position as u16,
                    prompt_area.y + 1,
                ));
            }
        }
        AppState::ConfirmOverwrite(path, _) => {
            let paragraph = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("File '{}' exists. Overwrite? (y/n)", path.display()),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]))
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
        }
        AppState::Message(msg, _) => {
            let paragraph = Paragraph::new(Line::from(Span::styled(
                msg,
                Style::default().fg(Color::Green),
            )))
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
        }
        AppState::Error(msg, _) => {
            let paragraph = Paragraph::new(Line::from(Span::styled(
                msg,
                Style::default().fg(Color::Red),
            )))
            .block(prompt_block)
            .style(inactive_text_style);
            f.render_widget(paragraph, prompt_area);
        }
    }

    // Render legend
    render_legend(f, legend_area, app);
}

fn render_list(
    f: &mut Frame,
    area: Rect,
    buffer: &mut ListBuffer,
    title_prefix: &str,
    active: bool,
    highlights: &[usize],
) {
    let title = format!("{} ({})", title_prefix, buffer.items.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_type(BorderType::Rounded)
        .border_style(if active {
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let inner_area = block.inner(area);
    buffer.viewport_height = inner_area.height as usize;

    let highlight_set: std::collections::HashSet<_> = highlights.iter().cloned().collect();
    let items: Vec<ListItem> = buffer
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if highlight_set.contains(&i) {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if !active {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let list = List::new(items).block(block);

    *buffer.state.offset_mut() = buffer.scroll_offset;
    f.render_stateful_widget(list, area, &mut buffer.state);

    let total_items = buffer.items.len();
    let offset = buffer.state.offset();

    let mut scrollbar_state = ScrollbarState::new(total_items.saturating_sub(buffer.viewport_height))
        .position(offset)
        .viewport_content_length(buffer.viewport_height);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .thumb_symbol("█")
        .track_symbol(Some("║"));

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_legend(f: &mut Frame, area: Rect, app: &App) {
    let mut spans = Vec::new();

    fn add_entry(spans: &mut Vec<Span>, key: &str, label: &str, enabled: bool) {
        let color = if enabled {
            Color::Magenta
        } else {
            Color::DarkGray
        };
        spans.push(Span::styled(format!(" {} ", key), color));
        spans.push(Span::styled(format!("{}  ", label), color));
    }

    add_entry(&mut spans, "Tab", "Cycle Buffer", true);

    let s_enabled = app.active_buffer == ActiveBuffer::Modified && app.input_path.is_some();
    add_entry(&mut spans, "s", "Save file", s_enabled);

    let s_caps_enabled =
        app.active_buffer == ActiveBuffer::Modified || app.active_buffer == ActiveBuffer::New;
    add_entry(&mut spans, "S", "Save As", s_caps_enabled);

    let nav_enabled = app.active_buffer != ActiveBuffer::Prompt;
    add_entry(&mut spans, "Arrows/PgUp/PgDn", "Navigate", nav_enabled);

    let enter_enabled = app.active_buffer == ActiveBuffer::Prompt;
    add_entry(&mut spans, "↵", "Select Option", enter_enabled);

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
