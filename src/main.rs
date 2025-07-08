use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend, // KORRIGIERT: Unnötiger `Backend`-Trait-Import entfernt
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use serde::Deserialize;
use std::{
    env,
    error::Error,
    fs,
    io::{self, Stdout},
    time::{Duration, Instant},
};

#[derive(Deserialize, Debug, Clone)]
struct Slot {
    name: String,
    duration_minutes: u64,
}

impl Slot {
    fn duration(&self) -> Duration {
        Duration::from_secs(self.duration_minutes * 60)
    }
}

struct App {
    slots: Vec<Slot>,
    current_slot_index: usize,
    slot_start_time: Option<Instant>,
    paused: bool,
    should_quit: bool,
    time_elapsed_when_paused: Duration,
}

impl App {
    fn new(slots: Vec<Slot>) -> App {
        App {
            slots,
            current_slot_index: 0,
            slot_start_time: Some(Instant::now()),
            paused: false,
            should_quit: false,
            time_elapsed_when_paused: Duration::from_secs(0),
        }
    }

    fn toggle_pause(&mut self) {
        if self.is_finished() {
            return;
        }

        self.paused = !self.paused;
        if self.paused {
            if let Some(start_time) = self.slot_start_time {
                self.time_elapsed_when_paused = start_time.elapsed();
            }
        } else {
            if let Some(_start_time) = self.slot_start_time {
                self.slot_start_time = Some(Instant::now() - self.time_elapsed_when_paused);
            }
        }
    }

    fn get_current_elapsed_time(&self) -> Duration {
        if self.paused {
            self.time_elapsed_when_paused
        } else if let Some(start_time) = self.slot_start_time {
            start_time.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }

    fn reset_and_start_slot(&mut self) {
        self.paused = false;
        self.time_elapsed_when_paused = Duration::from_secs(0);
        self.slot_start_time = Some(Instant::now());
    }

    fn next_slot(&mut self) {
        if self.current_slot_index < self.slots.len() - 1 {
            self.current_slot_index += 1;
            self.reset_and_start_slot();
        } else {
            self.slot_start_time = None;
        }
    }

    fn previous_slot(&mut self) {
        if self.current_slot_index > 0 {
            self.current_slot_index -= 1;
            self.reset_and_start_slot();
        }
    }

    fn update(&mut self) {
        if self.paused || self.is_finished() {
            return;
        }

        let elapsed = self.get_current_elapsed_time();
        let current_slot_duration = self.slots[self.current_slot_index].duration();
        if elapsed >= current_slot_duration {
            self.next_slot();
        }
    }

    fn is_finished(&self) -> bool {
        self.current_slot_index >= self.slots.len() - 1 && self.slot_start_time.is_none()
    }

    fn total_time_remaining(&self) -> Duration {
        if self.is_finished() {
            return Duration::from_secs(0);
        }

        let current_slot_elapsed = self.get_current_elapsed_time();
        let current_slot_duration = self.slots[self.current_slot_index].duration();
        let current_slot_remaining = current_slot_duration.saturating_sub(current_slot_elapsed);

        let future_slots_duration: Duration = self.slots[self.current_slot_index + 1..]
            .iter()
            .map(|s| s.duration())
            .sum();

        current_slot_remaining + future_slots_duration
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let executable_name = args.get(0).map_or("chronobolt", |s| s.as_str());
        eprintln!("Fehler: Es wurde keine JSON-Datei angegeben.");
        eprintln!("Verwendung: {} <pfad/zur/timetable.json>", executable_name);
        return Err("Keine Eingabedatei".into());
    }
    let file_path = &args[1];

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let json_content = fs::read_to_string(file_path)
        .map_err(|e| format!("Konnte Datei '{}' nicht lesen: {}", file_path, e))?;
    let slots: Vec<Slot> = serde_json::from_str(&json_content)
        .map_err(|e| format!("Konnte JSON aus '{}' nicht parsen: {}", file_path, e))?;

    let app = App::new(slots);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        app.update();
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char(' ') => app.toggle_pause(),
                    KeyCode::Char('n') => app.next_slot(),
                    KeyCode::Char('p') => app.previous_slot(),
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(8),
        ])
        .split(f.area());

    draw_header(f, main_layout[0]);
    draw_slot_list(f, main_layout[1], app);
    draw_timers(f, main_layout[2], app);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let text = vec![Line::from(vec![
        Span::styled("ChronoBolt ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        // KORRIGIERT: Das Blitz-Symbol wird direkt als Unicode-Zeichen eingefügt.
        Span::styled("⚡ ", Style::default().fg(Color::Yellow)),
        Span::raw(symbols::DOT),
        Span::styled(" (q)uit ", Style::default().fg(Color::Red)),
        Span::raw(symbols::DOT),
        Span::styled(" (space)pause/resume ", Style::default().fg(Color::Cyan)),
        Span::raw(symbols::DOT),
        Span::styled(" (n)ext ", Style::default().fg(Color::Green)),
        Span::raw(symbols::DOT),
        Span::styled(" (p)revious", Style::default().fg(Color::Green)),
    ])];
    let header = Paragraph::new(text)
        .block(Block::default().borders(Borders::BOTTOM).title("Steuerung"));
    f.render_widget(header, area);
}

fn draw_slot_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.slots.iter().enumerate().map(|(i, slot)| {
        let (prefix, style) = if i < app.current_slot_index {
            (Span::styled("✔ ", Style::default().fg(Color::Green)), Style::default().fg(Color::DarkGray))
        } else if i == app.current_slot_index && !app.is_finished() {
            let symbol = if app.paused { "⏸ " } else { "▶ " };
            (Span::styled(symbol, Style::default().fg(Color::Yellow)), Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        } else {
            (Span::raw("  "), Style::default())
        };

        let content = Line::from(vec![
            prefix,
            Span::raw(format!("{:<30}", slot.name)),
            Span::raw(format!(" ({} min)", slot.duration_minutes)),
        ]);

        ListItem::new(content).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Zeitplan"));
    f.render_widget(list, area);
}

fn draw_timers(f: &mut Frame, area: Rect, app: &App) {
    let timer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let current_slot = &app.slots[app.current_slot_index];
    let elapsed = app.get_current_elapsed_time();
    let total = current_slot.duration();
    let remaining = total.saturating_sub(elapsed);
    let ratio = (elapsed.as_secs_f64() / total.as_secs_f64()).min(1.0);

    let status_text = if app.is_finished() {
        "Fertig!".to_string()
    } else if app.paused {
        format!("Pausiert bei {}", format_duration(remaining))
    } else {
        format!("Verbleibend: {}", format_duration(remaining))
    };

    let current_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!("Aktueller Slot: {}", current_slot.name)))
        .gauge_style(Style::default().fg(Color::Yellow).bg(Color::Black))
        .label(status_text)
        .ratio(if app.is_finished() { 1.0 } else { ratio });
    f.render_widget(current_gauge, timer_layout[0]);

    let total_duration: Duration = app.slots.iter().map(|s| s.duration()).sum();
    let total_remaining = app.total_time_remaining();
    let total_elapsed = total_duration.saturating_sub(total_remaining);
    let total_ratio = if total_duration.as_secs() > 0 {
        (total_elapsed.as_secs_f64() / total_duration.as_secs_f64()).min(1.0)
    } else {
        1.0
    };

    let total_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Gesamtzeit"))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .label(format!("Verbleibend: {}", format_duration(total_remaining)))
        .ratio(total_ratio);
    f.render_widget(total_gauge, timer_layout[1]);
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}