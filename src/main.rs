use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    prelude::*,
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    symbols::border,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal, Frame,
};

use std::io;
use std::{collections::HashMap, fmt::Alignment};
// Main calendar datastructure
#[derive(Debug, Default)]
struct KallEvent {
    title: String,
    body: String,
    start: String,
    end: String,
}

#[derive(Debug, Default)]
struct App {
    events: HashMap<String, Vec<KallEvent>>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let weekly_n = 7 + 1;
        let daily_n = 16;
        let weekly_layout_constraint = (0..weekly_n)
            .map(|_| Constraint::Ratio(1, weekly_n))
            .collect::<Vec<_>>();

        let daily_layout_constraint = (0..daily_n)
            .map(|_| Constraint::Ratio(1, daily_n))
            .collect::<Vec<_>>();

        let week_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(weekly_layout_constraint)
            .split(frame.area());

        let daily_layout = (0..weekly_n)
            .map(|x| {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(daily_layout_constraint.clone())
                    .split(week_layout[x as usize])
            })
            .collect::<Vec<_>>();

        let time_widget = Paragraph::new("Time").block(Block::new().borders(Borders::ALL));
        frame.render_widget(time_widget, week_layout[0]);

        for x in 0..daily_n {
            let widget = Paragraph::new(x.to_string()).alignment(HorizontalAlignment::Center);
            frame.render_widget(widget, daily_layout[0][x as usize]);
        }

        frame.render_widget(Block::new().borders(Borders::ALL), week_layout[1]);

        frame.render_widget(Block::new().borders(Borders::ALL), week_layout[2]);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);
        let outer_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
