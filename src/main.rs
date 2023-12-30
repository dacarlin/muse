use std::{error::Error, io};
use walkdir::WalkDir;
use regex::Regex;
use std::ffi::OsStr;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use id3::{Tag, TagLike};

// Helper function to check if a file name has a .mp3 extension
fn is_mp3_file(file_name: &OsStr, mp3_regex: &Regex) -> bool {
    file_name.to_str().map_or(false, |s| {
        mp3_regex.is_match(s.to_lowercase().as_str())
    })
}

struct App<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
    title: &'a str, 
}

impl<'a> App<'a> {
    fn new(items: Vec<Vec<&'a str>>, title: &'a str) -> App<'a> {
        App {
            state: TableState::default(),
            items: items, 
            title: title, 
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn play_track(&mut self) {
        self.state.selected(); 
    }
}

struct Track<'a> {
    file_path: &'a str, 
    artist: &'a str, 
    album: &'a str, 
    index: u8, 
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // read in data 
    // Define a regular expression for matching MP3 files
    let dir_path = "test_dir";
    let mp3_regex = Regex::new(r"\.mp3$").unwrap();

    // Recursively walk through the directory
    let mut tags: Vec<Tag> = Vec::new(); 
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        // Check if the entry is a file and its extension matches the MP3 regex
        if entry.file_type().is_file() && is_mp3_file(&entry.file_name(), &mp3_regex) {
            let tag = Tag::read_from_path(entry.path())?;
            tags.push(tag); 
        }
    }

    // Create the view data 
    let mut items: Vec<Vec<&str>> = Vec::new(); 
    for tag in &tags {
        let pkg = vec![tag.title().unwrap(), tag.artist().unwrap(), tag.album().unwrap()]; 
        items.push(pkg); 
    }

    // create app and run it
    let app = App::new(items, dir_path);
    let res = run_app(&mut terminal, app);

    // restore terminal
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter => app.play_track(), 
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default();
    let header_cells = ["Track", "Artist", "Album"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().underlined()));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });

    let t = Table::new(
        rows,
        [
            Constraint::Percentage(42),
            Constraint::Percentage(34),
            Constraint::Percentage(24),
        ],
    )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!("Muse: Loaded from {}", app.title)))
        .highlight_style(selected_style)
        .highlight_symbol("");
    let blk = Block::new(); 
    f.render_stateful_widget(t, rects[0], &mut app.state);
    f.render_widget(blk, rects[1]);
}
