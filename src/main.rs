use color_eyre::Result;
use crossterm::event::read;
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
use std::fs::read_to_string;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};

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
const TOTAL_ROWS: usize = 96;
const TOTAL_COLS: usize = 8;
const CLOCK_WIDTH: u16 = 6;

struct App {
    events: HashMap<NaiveDate, Vec<icalendar::Event>>,
    scroll_offset: usize,
    exit: bool,
}
fn truncate(s: &str, max_width: usize) -> String {
    if s.chars().count() > max_width {
        let truncated: String = s.chars().take(max_width - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

fn parse_calendar_file() -> HashMap<NaiveDate, Vec<icalendar::Event>> {
    let mut calendarHashmap: HashMap<NaiveDate, Vec<icalendar::Event>> = HashMap::new();

    let contents = read_to_string("schedule.ics").unwrap();

    let parsed_calendar: Calendar = contents.parse().unwrap();

    for component in &parsed_calendar.components {
        if let CalendarComponent::Event(event) = component {
            let Some(timestamp) = event.get_start() else {
                continue;
            };
            let DatePerhapsTime::DateTime(dt) = timestamp else {
                continue;
            };
            let CalendarDateTime::Utc(dtt) = dt else {
                continue;
            };

            calendarHashmap
                .entry(dtt.with_timezone(&chrono::Local).date_naive())
                .or_insert_with(Vec::new)
                .push(event.clone());
        }
    }
    return calendarHashmap;
}

impl App {
    pub fn new() -> Self {
        App {
            events: parse_calendar_file(),
            scroll_offset: 0,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let weekly_n = 7 + 1;
        let daily_n = 62;
        // let weekly_layout_constraint = (0..weekly_n)
        //     .map(|_| Constraint::Ratio(1, weekly_n))
        //     .collect::<Vec<_>>();

        // println!(
        //     "Terminal size: {} row x {} cols;",
        //     frame.area().height.to_string(),
        //     frame.area().width.to_string()
        // );

        // let daily_layout_constraint = (0..daily_n)
        //     .map(|_| Constraint::Ratio(1, daily_n))
        //     .collect::<Vec<_>>();
        //
        // let week_layout = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints(weekly_layout_constraint)
        //     .split(frame.area());
        //
        // let daily_layout = (0..weekly_n)
        //     .map(|x| {
        //         Layout::default()
        //             .direction(Direction::Vertical)
        //             .constraints(daily_layout_constraint.clone())
        //             .split(week_layout[x as usize])
        //     })
        //     .collect::<Vec<_>>();

        let visible_rows = frame.area().height as usize;
        let visible_days = 7;

        let area = frame.area();

        let width_per_day = (area.width - CLOCK_WIDTH) / visible_days;

        for screen_row in 0..visible_rows {
            let logical_row = screen_row + self.scroll_offset;
            let hour: usize = 1 * (logical_row / 4);
            let minute: usize = 15 * logical_row % 60;

            if logical_row >= 96 {
                break;
            }

            let clock_rect = Rect {
                x: area.x,
                y: area.y + screen_row as u16,
                width: CLOCK_WIDTH,
                height: 1,
            };

            for days in 0..visible_days {
                let day_rect = Rect {
                    x: (area.x + CLOCK_WIDTH) + (width_per_day * days),
                    y: area.y + screen_row as u16,
                    width: width_per_day,
                    height: 1,
                };
                let test_string = "test string hello hello";
                let test_label =
                    format!("{}|", truncate(&test_string, (width_per_day - 1) as usize));
                frame.render_widget(Paragraph::new(test_label), day_rect);
            }

            let clock_label = format!("{:02}:{:02}|", hour, minute);
            frame.render_widget(Paragraph::new(clock_label), clock_rect);
        }
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
            KeyCode::Char('k') => self.scroll_down(),
            KeyCode::Char('l') => self.scroll_up(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn scroll_down(&mut self) {
        if self.scroll_offset + 1 > TOTAL_ROWS {
            return;
        }
        self.scroll_offset += 1
    }
    fn scroll_up(&mut self) {
        if self.scroll_offset == 0 {
            return;
        }
        self.scroll_offset -= 1
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

// fn main() -> io::Result<()> {
//     ratatui::run(|terminal| App::new().run(terminal))
// }
//

// main function for testing stuff
fn main() {
    let app = App::new();

    // for (date, events) in &app.events {
    //     println!("{}", date)
    // }

    let dt = Utc.with_ymd_and_hms(2025, 09, 04, 07, 15, 00).unwrap();
    let d = dt.date_naive();
    println!("{:?}", d);
    let eventsTest = app.events.get(&d);
    let Some(events) = eventsTest else {
        println!("events empty");
        return;
    };
    for event in events {
        println!("{:?}", event.get_start())
    }
}
