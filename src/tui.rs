use crate::{block_user, BotScore};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{self, Stdout};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Results,
    ConfirmBlock,
    // Blocking state can be handled within ConfirmBlock or transiently
}

pub struct AppState {
    pub followers_scanned: u32,
    pub threshold: u32,
    pub flagged: Vec<BotScore>,
    pub selected_index: usize,
    pub blocked: Vec<String>,
    pub view: ViewMode,
    pub list_state: ListState,
    pub dry_run: bool,
    pub mode: String, // "Manual" usually
}

impl AppState {
    pub fn new(flagged: Vec<BotScore>, scanned_count: u32, threshold: u32, dry_run: bool) -> Self {
        let mut list_state = ListState::default();
        if !flagged.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            followers_scanned: scanned_count,
            threshold,
            flagged,
            selected_index: 0,
            blocked: Vec::new(),
            view: ViewMode::Results,
            list_state,
            dry_run,
            mode: "Manual".to_string(),
        }
    }

    fn next(&mut self) {
        if self.flagged.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.flagged.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn previous(&mut self) {
        if self.flagged.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.flagged.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn block_current(&mut self) {
        if let Some(selected) = self.flagged.get(self.selected_index) {
            if !self.blocked.contains(&selected.login) {
                // In a real app, we would perform the API call here or signal it.
                // For this structure, we might need a callback or just mark it as blocked visually
                // and assume the caller handles it, or we handle it here if we pass the client.
                // Given the constraints, let's just mark it as blocked in the list for now.
                // But the requirement says "After confirmation: Replace row with [BLOCKED], Remove from flagged list".

                // If we remove it from the list, the index shifts.
                // Let's remove it.
                let blocked_login = selected.login.clone();
                self.blocked.push(blocked_login);
                self.flagged.remove(self.selected_index);

                // Adjust selection
                if self.flagged.is_empty() {
                    self.list_state.select(None);
                } else if self.selected_index >= self.flagged.len() {
                    self.selected_index = self.flagged.len() - 1;
                    self.list_state.select(Some(self.selected_index));
                }
            }
        }
        self.view = ViewMode::Results;
    }
}

pub async fn run_tui(mut app: AppState, client: reqwest::Client) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app, &client).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut AppState,
    client: &reqwest::Client,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.view {
                        ViewMode::Results => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Down => app.next(),
                            KeyCode::Up => app.previous(),
                            KeyCode::Enter => {
                                if !app.flagged.is_empty() {
                                    app.view = ViewMode::ConfirmBlock;
                                }
                            }
                            _ => {}
                        },
                        ViewMode::ConfirmBlock => match key.code {
                            KeyCode::Char('y') | KeyCode::Enter => {
                                let mut success = true;
                                if !app.dry_run {
                                    if let Some(item) = app.flagged.get(app.selected_index) {
                                        success = block_user(client, &item.login).await;
                                    }
                                }
                                if success {
                                    app.block_current();
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                                app.view = ViewMode::Results;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Summary
            Constraint::Min(10),   // Main split
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Header
    let header_text = format!(
        " StarReaper v0.3.0 ─ GitHub Signal Purification Engine {}",
        if app.dry_run { "[DRY RUN]" } else { "" }
    );
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Summary
    let summary_text = format!(
        " Scanned: {}   Flagged: {}   Blocked: {}   Threshold: {}   Mode: {}",
        app.followers_scanned,
        app.flagged.len(),
        app.blocked.len(),
        app.threshold,
        app.mode
    );
    let summary = Paragraph::new(summary_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(summary, chunks[1]);

    // Main Split
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Left Panel: Flagged Accounts
    let items: Vec<ListItem> = app
        .flagged
        .iter()
        .map(|item| {
            let severity = if item.score >= 6 {
                "CRITICAL"
            } else if item.score >= 4 {
                "HIGH"
            } else {
                "MEDIUM"
            };

            let color = match severity {
                "CRITICAL" => Color::Red, // DarkRed might be too dark
                "HIGH" => Color::Red,
                "MEDIUM" => Color::Yellow,
                _ => Color::White,
            };

            let content = format!(" {:<15} score: {:<2} {}", item.login, item.score, severity);

            ListItem::new(content).style(Style::default().fg(color))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Flagged Accounts "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, main_chunks[0], &mut app.list_state.clone());

    // Right Panel: Details
    if let Some(selected) = app.flagged.get(app.selected_index) {
        let reasons_formatted = selected
            .reasons
            .iter()
            .map(|r| format!(" • {}", r))
            .collect::<Vec<_>>()
            .join("\n");
        let details_text = format!(
            " Login: {}\n Followers: {}\n Following: {}\n Repos: {}\n Age: {}\n\n Reasons:\n{}",
            selected.login,
            selected.profile.followers,
            selected.profile.following,
            selected.profile.public_repos,
            selected
                .profile
                .created_at
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default(),
            reasons_formatted
        );

        let details = Paragraph::new(details_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Profile Risk Details "),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(details, main_chunks[1]);
    } else {
        let details = Paragraph::new(" No account selected").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Profile Risk Details "),
        );
        frame.render_widget(details, main_chunks[1]);
    }

    // Footer
    let footer_text = " ↑↓ Navigate   ENTER Block   Q Quit";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[3]);

    // Overlay for Confirmation
    if app.view == ViewMode::ConfirmBlock {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let area = centered_rect(60, 20, frame.area());
        frame.render_widget(Clear, area); // Clear the background
        frame.render_widget(block, area);

        let confirm_text = if let Some(item) = app.flagged.get(app.selected_index) {
            format!(" Block user {}? (y/n)", item.login)
        } else {
            " Confirm action? (y/n)".to_string()
        };

        let text = Paragraph::new(confirm_text)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        // Center the text vertically in the popup
        let vertical_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(area);

        frame.render_widget(text, vertical_center[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
