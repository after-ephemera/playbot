use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

use crate::db::{Database, TrackInfo};

enum InputMode {
    Normal,
    Editing,
}

enum ViewMode {
    List,
    Detail,
}

struct App {
    db: Database,
    tracks: Vec<TrackInfo>,
    list_state: ListState,
    search_query: String,
    input_mode: InputMode,
    view_mode: ViewMode,
    should_quit: bool,
    detail_scroll: u16,
}

impl App {
    fn new(db: Database) -> Result<Self> {
        let tracks = db.get_all_tracks()?;
        let mut list_state = ListState::default();
        if !tracks.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            db,
            tracks,
            list_state,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            view_mode: ViewMode::List,
            should_quit: false,
            detail_scroll: 0,
        })
    }

    fn scroll_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    fn reset_scroll(&mut self) {
        self.detail_scroll = 0;
    }

    fn next(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tracks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tracks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn update_search(&mut self) -> Result<()> {
        self.tracks = if self.search_query.is_empty() {
            self.db.get_all_tracks()?
        } else {
            self.db.search_tracks(&self.search_query)?
        };

        if !self.tracks.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }

        Ok(())
    }

    fn selected_track(&self) -> Option<&TrackInfo> {
        self.list_state.selected().and_then(|i| self.tracks.get(i))
    }
}

pub fn run(db: Database) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(db)?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('/') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('j') | KeyCode::Down => match app.view_mode {
                        ViewMode::List => app.next(),
                        ViewMode::Detail => app.scroll_down(),
                    },
                    KeyCode::Char('k') | KeyCode::Up => match app.view_mode {
                        ViewMode::List => app.previous(),
                        ViewMode::Detail => app.scroll_up(),
                    },
                    KeyCode::Char('l') | KeyCode::Right => match app.view_mode {
                        ViewMode::Detail => {
                            app.next();
                            app.reset_scroll();
                        },
                        _ => {}
                    },
                    KeyCode::Char('h') | KeyCode::Left => match app.view_mode {
                        ViewMode::Detail => {
                            app.previous();
                            app.reset_scroll();
                        },
                        _ => {}
                    },
                    KeyCode::Enter => match app.view_mode {
                        ViewMode::List => {
                            app.reset_scroll();
                            app.view_mode = ViewMode::Detail;
                        },
                        ViewMode::Detail => {
                            app.reset_scroll();
                            app.view_mode = ViewMode::List;
                        },
                    },
                    KeyCode::Esc => {
                        app.reset_scroll();
                        app.view_mode = ViewMode::List;
                    },
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.update_search()?;
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.update_search()?;
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_search_box(f, app, chunks[0]);

    match app.view_mode {
        ViewMode::List => render_track_list(f, app, chunks[1]),
        ViewMode::Detail => render_track_detail(f, app, chunks[1]),
    }

    render_help(f, app, chunks[2]);
}

fn render_search_box(f: &mut Frame, app: &App, area: Rect) {
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search"),
            ],
            Style::default(),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Searching: "),
                Span::styled(
                    app.search_query.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ],
            Style::default().fg(Color::Yellow),
        ),
    };

    let text = Text::from(Line::from(msg)).patch_style(style);
    let search = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search, area);
}

fn render_track_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .tracks
        .iter()
        .map(|track| {
            let content = Line::from(vec![
                Span::styled(
                    format!("{} ", track.track_name),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw("by "),
                Span::styled(
                    &track.artist_name,
                    Style::default().fg(Color::Green),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Tracks ({})", app.tracks.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_track_detail(f: &mut Frame, app: &App, area: Rect) {
    let track = match app.selected_track() {
        Some(t) => t,
        None => {
            let paragraph = Paragraph::new("No track selected")
                .block(Block::default().borders(Borders::ALL).title("Track Details"));
            f.render_widget(paragraph, area);
            return;
        }
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Track: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.track_name),
        ]),
        Line::from(vec![
            Span::styled("Artist: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.artist_name),
        ]),
        Line::from(vec![
            Span::styled("Album: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.album_name),
        ]),
    ];

    if !track.release_date.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Release Date: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.release_date),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!(
            "{}:{:02}",
            track.duration_ms / 60000,
            (track.duration_ms % 60000) / 1000
        )),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Popularity: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}/100", track.popularity)),
    ]));

    if !track.genres.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Genres: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.genres),
        ]));
    }

    if !track.producers.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Producers: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.producers),
        ]));
    }

    if !track.writers.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Writers: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&track.writers),
        ]));
    }

    if let Some(lyrics) = &track.lyrics {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Lyrics:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        for line in lyrics.lines() {
            lines.push(Line::from(line));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Track Details"))
        .wrap(Wrap { trim: true })
        .scroll((app.detail_scroll, 0));

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.view_mode {
        ViewMode::List => match app.input_mode {
            InputMode::Normal => {
                "j/k or Up/Down: Navigate | Enter: View Details | /: Search | q: Quit"
            }
            InputMode::Editing => "Type to search | Enter: Finish | Esc: Cancel",
        },
        ViewMode::Detail => "j/k: Scroll | h/l: Prev/Next Song | Enter/Esc: Back to List | q: Quit",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help, area);
}
