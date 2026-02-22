use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    prelude::*,
    style::{Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::fs::read_to_string;

use chrono::{NaiveDate, TimeZone, Utc};

use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};

use std::collections::HashMap;
use std::io;
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

#[derive(Debug)]
struct CursorPos {
    x: u16,
    y: u16,
}

struct App {
    events: HashMap<NaiveDate, Vec<icalendar::Event>>,
    scroll_offset: usize,
    cursor_pos: CursorPos,
    screen_height: u16,
    screen_width: u16,
    visible_days: u16,
    visible_day_width: u16,
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
            cursor_pos: CursorPos {
                x: CLOCK_WIDTH,
                y: 0,
            },
            exit: false,
            screen_width: 0,
            screen_height: 0,
            visible_days: 7,
            visible_day_width: 0,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.screen_height = frame.area().height;
        self.screen_width = frame.area().width;

        let _weekly_n = 7 + 1;
        let _daily_n = 62;
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

        let area = frame.area();

        self.visible_day_width = (area.width - CLOCK_WIDTH) / self.visible_days;

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

            for days in 0..self.visible_days {
                let day_rect = Rect {
                    x: (area.x + CLOCK_WIDTH) + (self.visible_day_width * days),
                    y: area.y + screen_row as u16,
                    width: self.visible_day_width,
                    height: 1,
                };
                let test_string = "test string hello hello";
                let test_label = format!(
                    "{}|",
                    truncate(&test_string, (self.visible_day_width - 1) as usize)
                );
                frame.render_widget(Paragraph::new(test_label), day_rect);
            }

            let clock_label = format!("{:02}:{:02}|", hour, minute);
            frame.render_widget(Paragraph::new(clock_label), clock_rect);

            // check for cursor mistakes
            if (self.cursor_pos.x < CLOCK_WIDTH) {
                self.cursor_pos.x = CLOCK_WIDTH
            }
            // draw cursor once per iteration
            let cursor_rec = Rect {
                x: self.cursor_pos.x,
                y: self.cursor_pos.y,
                width: 1,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new("x").style(Style::default().fg(Color::Red)),
                cursor_rec,
            );
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
            KeyCode::Char(';') => self.scroll_right(),
            KeyCode::Char('j') => self.scroll_left(),

            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn scroll_down(&mut self) {
        // if the cursor is at the end of the screen. Scroll the screen
        if (self.cursor_pos.y + 1) >= self.screen_height {
            if self.scroll_offset + 1 > TOTAL_ROWS {
                return;
            }
            self.scroll_offset += 1;
            return;
        }
        // else its inside move cursor pos with 1
        self.cursor_pos.y += 1;
    }
    fn scroll_up(&mut self) {
        // if the cursor is at the top of the screen. Scroll the screen
        if self.cursor_pos.y == 0 {
            if self.scroll_offset == 0 {
                return;
            }
            self.scroll_offset -= 1;
            return;
        }
        // else its inside move cursor pos with 1
        self.cursor_pos.y -= 1;
    }
    fn scroll_right(&mut self) {
        let end_of_days = CLOCK_WIDTH + (self.visible_day_width * self.visible_days);
        let pos_after = self.cursor_pos.x + self.visible_day_width;
        if pos_after >= end_of_days {
            return;
        }
        self.cursor_pos.x += self.visible_day_width;
    }

    fn scroll_left(&mut self) {
        if self.cursor_pos.x <= CLOCK_WIDTH {
            return;
        }
        self.cursor_pos.x -= self.visible_day_width;
    }
}

impl Widget for &App {
    fn render(self, _area: Rect, _buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);
        let _outer_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
    }
}

fn main() -> io::Result<()> {
    let mut app = App::new();
    let dt = Utc.with_ymd_and_hms(2025, 09, 04, 07, 15, 00).unwrap();
    let d = dt.date_naive();
    ratatui::run(|terminal| app.run(terminal))
}

// main function for testing stuff
// fn main() {
//     let app = App::new();
//
//     for (date, events) in &app.events {
//         println!("{}", date)
//     }
//
//     let dt = Utc.with_ymd_and_hms(2025, 09, 04, 07, 15, 00).unwrap();
//     let d = dt.date_naive();
//     println!("{:?}", d);
//     let eventsTest = app.events.get(&d);
//     let Some(events) = eventsTest else {
//         println!("events empty");
//         return;
//     };
//     for event in events {
//         println!("{:?}", event.get_start())
//     }
// }
