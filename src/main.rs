use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use map::WorldMap;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        canvas::*,
        Block, Widget,
    },
    Frame,
};

// How many map units are moved per step of zoom
const ZOOM_STEP_SIZE: i32 = 2;
const PAN_STEP_SIZE: i32 = 1;

use color_eyre::{eyre::WrapErr, Result};

mod map;
mod tui;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore the terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}

#[derive(Debug)]
struct Viewport {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            min_x: -180.,
            max_x: 180.,
            min_y: -90.,
            max_y: 90.,
        }
    }
}

impl Viewport {
    fn zoom(&mut self, z: i32) {
        self.min_x += (z * ZOOM_STEP_SIZE) as f64;
        self.min_y += (z * ZOOM_STEP_SIZE / 2) as f64;
        self.max_x -= (z * ZOOM_STEP_SIZE) as f64;
        self.max_y -= (z * ZOOM_STEP_SIZE / 2) as f64;
    }
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    viewport: Viewport,
    /// last seen mouse clicking position
    last_mouse_drag_position: Option<(u16, u16)>,
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle event failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed: \n{key_event:#?}")),
            Event::Mouse(mouse_event) => self
                .handle_mouse_event(mouse_event)
                .wrap_err_with(|| format!("handling mouse event failed: \n{mouse_event:#?}")),

            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.increment_zoom()?,
            KeyCode::Down => self.decrement_zoom()?,
            KeyCode::Char('w') => self.pan_up()?,
            KeyCode::Char('a') => self.pan_left()?,
            KeyCode::Char('s') => self.pan_down()?,
            KeyCode::Char('d') => self.pan_right()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        match mouse_event.kind {
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some((column, row)) = &self.last_mouse_drag_position {
                    let vertical_delta =
                        f64::from(i32::from(mouse_event.row).wrapping_sub(i32::from(*row))) * 0.2;
                    let horizontal_delta =
                        f64::from(i32::from(mouse_event.column).wrapping_sub(i32::from(*column)))
                            * 0.2;
                    self.viewport.max_x -= horizontal_delta;
                    self.viewport.min_x -= horizontal_delta;
                    self.viewport.max_y += vertical_delta;
                    self.viewport.min_y += vertical_delta;
                }
                self.last_mouse_drag_position = Some((mouse_event.column, mouse_event.row));
            }
            MouseEventKind::Up(_) => {
                // Dragging finishes
                self.last_mouse_drag_position = None;
            }
            MouseEventKind::ScrollUp => self.increment_zoom()?,
            MouseEventKind::ScrollDown => self.decrement_zoom()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_zoom(&mut self) -> Result<()> {
        self.viewport.zoom(1);
        Ok(())
    }

    fn decrement_zoom(&mut self) -> Result<()> {
        self.viewport.zoom(-1);
        Ok(())
    }

    fn pan_up(&mut self) -> Result<()> {
        self.viewport.max_y += PAN_STEP_SIZE as f64;
        self.viewport.min_y += PAN_STEP_SIZE as f64;
        Ok(())
    }
    fn pan_left(&mut self) -> Result<()> {
        self.viewport.max_x -= PAN_STEP_SIZE as f64;
        self.viewport.min_x -= PAN_STEP_SIZE as f64;
        Ok(())
    }
    fn pan_down(&mut self) -> Result<()> {
        self.viewport.max_y -= PAN_STEP_SIZE as f64;
        self.viewport.min_y -= PAN_STEP_SIZE as f64;
        Ok(())
    }
    fn pan_right(&mut self) -> Result<()> {
        self.viewport.max_x += PAN_STEP_SIZE as f64;
        self.viewport.min_x += PAN_STEP_SIZE as f64;
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Map ".bold());
        let instructions = Title::from(Line::from(vec![
            " Zoom In ".into(),
            "<Up>".blue().bold(),
            " Zoom Out ".into(),
            "<Down>".blue().bold(),
            " Pan around ".into(),
            "<w,a,s,d>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let canvas = Canvas::default()
            .block(block)
            .x_bounds([self.viewport.min_x, self.viewport.max_x])
            .y_bounds([self.viewport.min_y, self.viewport.max_y])
            .paint(|ctx| {
                ctx.draw(&WorldMap {
                    resolution: map::WorldResolution::High,
                    color: ratatui::style::Color::Blue,
                });
                ctx.layer()
            });

        canvas.render(area, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
