use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    sytle::stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
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
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::result<()> {
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
            KeyCode::Enter => self.selanjutnya(),
            KeyCode::Esc => self.sebelumnya(),
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(vec![
            " Selanjutnya ".into(),
            "<Enter>".blue().bold(),
            " Sebelumnya ".into(),
            "<Esc>".blue().bold,
            " Keluar ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let atm_text = Text::from(vec![Line::from(vec![
            "Cek saldo  : ".into(),
            "Ambil uang : ".into(),

        ])]);

        Paragraph::new(atm_text)
            .centered()
            .block(block)
            .render(area, buf);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn selanjutnya(&mut self) {
        self.enter = true;
    }

    fn sebelumnya(&mut self) {
        self.escape = true;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━━━ ATM Bersama ━━━━━━━━━━━━━━━━━━━┓",
            "┃                                                   ┃",
            "┃                    Cek saldo  :                   ┃",
            "┃                    Ambil Uang :                   ┃",
            "┃                                                   ┃",
            "┗━ Selanjutnya <Enter> Sebelumnya <Esc> Keluar <Q> ━┛",
        ]);
        let title_style = Style::new().bold();
        let atm_style = Style::new().blue();
        let key_style = Style::new().white().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_ew!(buf, expected);
    }

    #[test]
    fn handle_key_event() {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Enter.into());
        assert_eq!(app.selanjutnya, enter);

        app.handle_key_event(KeyCode::Esc.into());
        assert_eq!(app.sebelumnya, esc);

        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);
    }
}
