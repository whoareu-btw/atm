mod logika;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

use logika::{Keadaan, Layar, MENU, format_input, format_rupiah};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default)]
pub struct App {
    keadaan: Keadaan,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.keadaan.selesai {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        // Pada raw mode, Ctrl+C tidak mengirim sinyal, jadi harus ditangani manual.
        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && key_event.code == KeyCode::Char('c')
        {
            self.keadaan.keluar();
            return;
        }

        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.keadaan.keluar(),
            KeyCode::Char(c) => self.keadaan.ketik(c),
            KeyCode::Backspace => self.keadaan.hapus(),
            KeyCode::Up => self.keadaan.naik(),
            KeyCode::Down => self.keadaan.turun(),
            KeyCode::Enter => self.keadaan.selanjutnya(),
            KeyCode::Esc => self.keadaan.sebelumnya(),
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let k = &self.keadaan;
        let judul = Line::from(" ATM Bersama ".bold());
        let petunjuk = Line::from(vec![
            " Selanjutnya ".into(),
            "<Enter>".blue(),
            " Sebelumnya ".into(),
            "<Esc>".blue(),
            " Keluar ".into(),
            "<Q> ".blue(),
        ]);
        let block = Block::bordered()
            .title(judul.centered())
            .title_bottom(petunjuk.centered())
            .border_set(border::THICK);

        let redup = Style::new().dark_gray();

        let isi: Vec<Line> = match k.layar {
            // Menu utama: Cek saldo / Ambil uang
            Layar::Menu => {
                let mut baris = vec![Line::from("Pilih transaksi:"), Line::raw("")];
                for (i, nama) in MENU.iter().enumerate() {
                    let (teks, gaya) = if i == k.pilihan {
                        (format!("> {:<10}", nama), Style::new().white())
                    } else {
                        (format!("  {:<10}", nama), Style::new())
                    };
                    baris.push(Line::from(Span::styled(teks, gaya)));
                }
                baris.push(Line::raw(""));
                baris.push(Line::from(Span::styled(
                    "Tombol Atas/Bawah, lalu Enter",
                    redup,
                )));
                baris
            }
            // Cek saldo
            Layar::CekSaldo => vec![
                Line::from("Saldo Anda saat ini:"),
                Line::raw(""),
                Line::from(Span::styled(
                    format_rupiah(k.saldo),
                    Style::new().white().bold(),
                )),
                Line::raw(""),
                Line::from(Span::styled("Tekan Enter/Esc untuk kembali ke menu", redup)),
            ],
            // Ambil uang: baca jumlah Rupiah
            Layar::AmbilUang => {
                let baris_bawah = match k.pesan_error {
                    Some(pesan) => Span::styled(pesan, Style::new().red().bold()),
                    None => Span::styled(format!("Saldo Anda: {}", format_rupiah(k.saldo)), redup),
                };
                vec![
                    Line::from("Masukkan jumlah:"),
                    Line::raw(""),
                    Line::from(Span::styled(
                        format!("{}_", format_input(&k.input)),
                        Style::new().blue(),
                    )),
                    Line::raw(""),
                    Line::from(baris_bawah),
                ]
            }
            // Struk penarikan
            Layar::Hasil => vec![
                Line::from("Penarikan berhasil"),
                Line::raw(""),
                Line::from(Span::styled(
                    format!(
                        "Anda mengambil: {}",
                        format_rupiah(k.jumlah_ditarik.unwrap_or(0))
                    ),
                    Style::new().blue().bold(),
                )),
                Line::from(format!("Sisa saldo: {}", format_rupiah(k.saldo))),
                Line::raw(""),
                Line::from(Span::styled("Tekan Enter/Esc untuk kembali ke menu", redup)),
            ],
        };

        // Pusatkan secara vertikal di dalam border (tinggi dalam = tinggi - 2).
        let tinggi_dalam = area.height.saturating_sub(2) as usize;
        let atas = tinggi_dalam.saturating_sub(isi.len()) / 2;
        let mut baris = vec![Line::raw(""); atas];
        baris.extend(isi);

        Paragraph::new(Text::from(baris))
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Menggabungkan seluruh isi buffer menjadi satu string untuk dicek.
    fn teks_layar(app: &App) -> String {
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 14));
        app.render(buf.area, &mut buf);
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    fn tekan(app: &mut App, kode: KeyCode) {
        app.handle_key_event(kode.into());
    }

    fn ketik(app: &mut App, teks: &str) {
        for c in teks.chars() {
            tekan(app, KeyCode::Char(c));
        }
    }

    #[test]
    fn menu_menampilkan_judul_dan_pilihan() {
        let teks = teks_layar(&App::default());
        assert!(teks.contains("ATM Bersama"));
        assert!(teks.contains("Pilih transaksi:"));
        assert!(teks.contains("> Cek saldo"));
        assert!(teks.contains("Ambil uang"));
        assert!(teks.contains("Selanjutnya"));
    }

    #[test]
    fn cek_saldo_menampilkan_saldo_awal_sepuluh_juta() {
        let mut app = App::default();
        tekan(&mut app, KeyCode::Enter);
        let teks = teks_layar(&app);
        assert!(teks.contains("Saldo Anda saat ini:"));
        assert!(teks.contains("Rp 10.000.000"));

        tekan(&mut app, KeyCode::Esc);
        assert!(teks_layar(&app).contains("Pilih transaksi:"));
    }

    #[test]
    fn alur_ambil_uang_dari_menu_sampai_struk() {
        let mut app = App::default();
        tekan(&mut app, KeyCode::Down);
        assert!(teks_layar(&app).contains("> Ambil uang"));
        tekan(&mut app, KeyCode::Enter);

        let teks = teks_layar(&app);
        assert!(teks.contains("Masukkan jumlah Rupiah:"));
        assert!(teks.contains("Saldo Anda: Rp 10.000.000"));

        ketik(&mut app, "1500000");
        assert!(teks_layar(&app).contains("Rp 1.500.000_"));

        tekan(&mut app, KeyCode::Enter);
        let teks = teks_layar(&app);
        assert!(teks.contains("Penarikan berhasil"));
        assert!(teks.contains("Anda mengambil: Rp 1.500.000"));
        assert!(teks.contains("Sisa saldo: Rp 8.500.000"));

        tekan(&mut app, KeyCode::Enter);
        assert!(teks_layar(&app).contains("Pilih transaksi:"));
    }

    #[test]
    fn saldo_tidak_cukup_menampilkan_pesan_error() {
        let mut app = App::default();
        tekan(&mut app, KeyCode::Down);
        tekan(&mut app, KeyCode::Enter);
        ketik(&mut app, "99999999");
        tekan(&mut app, KeyCode::Enter);
        assert!(teks_layar(&app).contains("Saldo tidak cukup"));
    }

    #[test]
    fn enter_tanpa_angka_menampilkan_pesan_error() {
        let mut app = App::default();
        tekan(&mut app, KeyCode::Down);
        tekan(&mut app, KeyCode::Enter);
        tekan(&mut app, KeyCode::Enter);
        assert!(teks_layar(&app).contains("Jumlah belum diisi"));
    }

    #[test]
    fn tombol_q_dan_ctrl_c_keluar() {
        let mut app = App::default();
        tekan(&mut app, KeyCode::Char('q'));
        assert!(app.keadaan.selesai);

        let mut app = App::default();
        app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.keadaan.selesai);
    }
}
