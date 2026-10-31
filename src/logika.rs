//! Logika murni aplikasi ATM.
//!
//! Modul ini sengaja tidak bergantung pada ratatui maupun crossterm,
//! sehingga mudah diuji dan sangat ringan.

/// Jumlah digit maksimum yang boleh diketik (999.999.999.999.999 masih muat di `u64`).
pub const MAKS_DIGIT: usize = 15;

/// Saldo awal setiap kali aplikasi dijalankan: Rp 10.000.000.
pub const SALDO_AWAL: u64 = 10_000_000;

/// Pilihan pada menu utama (urutan sama dengan `Keadaan::pilihan`).
pub const MENU: [&str; 2] = ["Cek saldo", "Ambil uang"];

/// Layar yang sedang tampil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layar {
    /// Menu utama: Cek saldo / Ambil uang.
    #[default]
    Menu,
    /// Menampilkan saldo saat ini.
    CekSaldo,
    /// Pengguna mengetik jumlah Rupiah yang ingin diambil.
    AmbilUang,
    /// Struk penarikan setelah berhasil.
    Hasil,
}

/// Keadaan (state) aplikasi.
#[derive(Debug)]
pub struct Keadaan {
    pub layar: Layar,
    /// Indeks pilihan menu yang disorot (0 = Cek saldo, 1 = Ambil uang).
    pub pilihan: usize,
    /// Saldo saat ini dalam Rupiah.
    pub saldo: u64,
    /// Hanya berisi digit '0'..='9', tanpa nol di depan.
    pub input: String,
    /// Jumlah yang baru saja ditarik (untuk struk).
    pub jumlah_ditarik: Option<u64>,
    pub pesan_error: Option<&'static str>,
    pub selesai: bool,
}

impl Default for Keadaan {
    fn default() -> Self {
        Self {
            layar: Layar::Menu,
            pilihan: 0,
            saldo: SALDO_AWAL,
            input: String::new(),
            jumlah_ditarik: None,
            pesan_error: None,
            selesai: false,
        }
    }
}

impl Keadaan {
    /// Tombol Atas: pindah sorotan menu ke atas.
    pub fn naik(&mut self) {
        if self.layar == Layar::Menu && self.pilihan > 0 {
            self.pilihan -= 1;
        }
    }

    /// Tombol Bawah: pindah sorotan menu ke bawah.
    pub fn turun(&mut self) {
        if self.layar == Layar::Menu && self.pilihan + 1 < MENU.len() {
            self.pilihan += 1;
        }
    }

    /// Menambah satu digit (hanya di layar AmbilUang). Karakter lain diabaikan.
    pub fn ketik(&mut self, c: char) {
        if self.layar != Layar::AmbilUang || !c.is_ascii_digit() {
            return;
        }
        if self.input.is_empty() && c == '0' {
            return; // abaikan nol di depan
        }
        if self.input.len() >= MAKS_DIGIT {
            return;
        }
        self.input.push(c);
        self.pesan_error = None;
    }

    /// Menghapus digit terakhir (hanya di layar AmbilUang).
    pub fn hapus(&mut self) {
        if self.layar == Layar::AmbilUang {
            self.input.pop();
            self.pesan_error = None;
        }
    }

    /// Tombol Enter: pilih menu, konfirmasi penarikan, atau kembali ke menu.
    pub fn selanjutnya(&mut self) {
        match self.layar {
            Layar::Menu => {
                if self.pilihan == 0 {
                    self.layar = Layar::CekSaldo;
                } else {
                    self.input.clear();
                    self.pesan_error = None;
                    self.layar = Layar::AmbilUang;
                }
            }
            Layar::CekSaldo | Layar::Hasil => self.kembali_ke_menu(),
            Layar::AmbilUang => match tarik_tunai(self.saldo, &self.input) {
                Ok(jumlah) => {
                    self.saldo -= jumlah;
                    self.jumlah_ditarik = Some(jumlah);
                    self.pesan_error = None;
                    self.layar = Layar::Hasil;
                }
                Err(pesan) => self.pesan_error = Some(pesan),
            },
        }
    }

    /// Tombol Esc: kembali ke menu dari layar mana pun (di menu tidak berefek).
    pub fn sebelumnya(&mut self) {
        if self.layar != Layar::Menu {
            self.kembali_ke_menu();
        }
    }

    /// Tombol Q / Ctrl+C.
    pub fn keluar(&mut self) {
        self.selesai = true;
    }

    fn kembali_ke_menu(&mut self) {
        self.layar = Layar::Menu;
        self.input.clear();
        self.jumlah_ditarik = None;
        self.pesan_error = None;
    }
}

/// Membaca jumlah Rupiah dari teks (padanan `scanf("%llu", &jumlah)` di C,
/// tetapi dengan validasi). Jumlah harus bilangan bulat lebih dari nol.
pub fn baca_jumlah(teks: &str) -> Result<u64, &'static str> {
    let teks = teks.trim();
    if teks.is_empty() {
        return Err("Jumlah belum diisi");
    }
    if !teks.chars().all(|c| c.is_ascii_digit()) {
        return Err("Hanya angka yang diperbolehkan");
    }
    match teks.parse::<u64>() {
        Ok(0) => Err("Jumlah harus lebih dari Rp 0"),
        Ok(n) => Ok(n),
        Err(_) => Err("Jumlah terlalu besar"),
    }
}

/// Memvalidasi penarikan: jumlah harus valid dan tidak boleh melebihi saldo.
pub fn tarik_tunai(saldo: u64, teks: &str) -> Result<u64, &'static str> {
    let jumlah = baca_jumlah(teks)?;
    if jumlah > saldo {
        return Err("Saldo tidak cukup");
    }
    Ok(jumlah)
}

/// Memberi titik pemisah ribuan pada deretan digit: "1500000" -> "1.500.000".
fn kelompokkan(digit: &str) -> String {
    let panjang = digit.len();
    let mut hasil = String::with_capacity(panjang + panjang / 3);
    for (i, c) in digit.chars().enumerate() {
        if i > 0 && (panjang - i) % 3 == 0 {
            hasil.push('.');
        }
        hasil.push(c);
    }
    hasil
}

/// Contoh: `1500000` -> "Rp 1.500.000".
pub fn format_rupiah(jumlah: u64) -> String {
    format!("Rp {}", kelompokkan(&jumlah.to_string()))
}

/// Tampilan input yang sedang diketik, contoh: "15000" -> "Rp 15.000"; "" -> "Rp ".
pub fn format_input(input: &str) -> String {
    format!("Rp {}", kelompokkan(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ketik_semua(k: &mut Keadaan, teks: &str) {
        for c in teks.chars() {
            k.ketik(c);
        }
    }

    /// Keadaan yang sudah berada di layar AmbilUang.
    fn di_ambil_uang() -> Keadaan {
        let mut k = Keadaan::default();
        k.turun();
        k.selanjutnya();
        assert_eq!(k.layar, Layar::AmbilUang);
        k
    }

    #[test]
    fn saldo_awal_sepuluh_juta() {
        let k = Keadaan::default();
        assert_eq!(SALDO_AWAL, 10_000_000);
        assert_eq!(k.saldo, 10_000_000);
        assert_eq!(format_rupiah(k.saldo), "Rp 10.000.000");
        assert_eq!(k.layar, Layar::Menu);
    }

    #[test]
    fn format_rupiah_memakai_titik_ribuan() {
        assert_eq!(format_rupiah(0), "Rp 0");
        assert_eq!(format_rupiah(5), "Rp 5");
        assert_eq!(format_rupiah(12), "Rp 12");
        assert_eq!(format_rupiah(999), "Rp 999");
        assert_eq!(format_rupiah(1_000), "Rp 1.000");
        assert_eq!(format_rupiah(50_000), "Rp 50.000");
        assert_eq!(format_rupiah(1_500_000), "Rp 1.500.000");
        assert_eq!(format_rupiah(999_999_999_999_999), "Rp 999.999.999.999.999");
    }

    #[test]
    fn format_input_menangani_teks_kosong_dan_sebagian() {
        assert_eq!(format_input(""), "Rp ");
        assert_eq!(format_input("1"), "Rp 1");
        assert_eq!(format_input("15000"), "Rp 15.000");
    }

    #[test]
    fn baca_jumlah_valid() {
        assert_eq!(baca_jumlah("1"), Ok(1));
        assert_eq!(baca_jumlah("  250000 "), Ok(250_000));
    }

    #[test]
    fn baca_jumlah_tidak_valid() {
        assert_eq!(baca_jumlah(""), Err("Jumlah belum diisi"));
        assert_eq!(baca_jumlah("   "), Err("Jumlah belum diisi"));
        assert_eq!(baca_jumlah("0"), Err("Jumlah harus lebih dari Rp 0"));
        assert_eq!(baca_jumlah("12a"), Err("Hanya angka yang diperbolehkan"));
        assert_eq!(baca_jumlah("-5"), Err("Hanya angka yang diperbolehkan"));
        assert_eq!(baca_jumlah("1.000"), Err("Hanya angka yang diperbolehkan"));
        assert_eq!(
            baca_jumlah("99999999999999999999"),
            Err("Jumlah terlalu besar")
        );
    }

    #[test]
    fn tarik_tunai_memeriksa_saldo() {
        assert_eq!(tarik_tunai(10_000_000, "1500000"), Ok(1_500_000));
        assert_eq!(tarik_tunai(10_000_000, "10000000"), Ok(10_000_000));
        assert_eq!(
            tarik_tunai(10_000_000, "10000001"),
            Err("Saldo tidak cukup")
        );
        assert_eq!(tarik_tunai(0, "1"), Err("Saldo tidak cukup"));
        assert_eq!(tarik_tunai(10_000_000, ""), Err("Jumlah belum diisi"));
    }

    #[test]
    fn menu_dimulai_di_cek_saldo_dan_sorotan_dibatasi() {
        let mut k = Keadaan::default();
        assert_eq!(k.pilihan, 0);
        k.naik(); // sudah paling atas
        assert_eq!(k.pilihan, 0);
        k.turun();
        assert_eq!(k.pilihan, 1);
        k.turun(); // sudah paling bawah
        assert_eq!(k.pilihan, 1);
        k.naik();
        assert_eq!(k.pilihan, 0);
    }

    #[test]
    fn cek_saldo_lalu_kembali_ke_menu() {
        let mut k = Keadaan::default();
        k.selanjutnya();
        assert_eq!(k.layar, Layar::CekSaldo);
        assert_eq!(k.saldo, SALDO_AWAL); // cek saldo tidak mengubah saldo
        k.selanjutnya();
        assert_eq!(k.layar, Layar::Menu);

        k.selanjutnya();
        k.sebelumnya();
        assert_eq!(k.layar, Layar::Menu);
    }

    #[test]
    fn ketik_hanya_diterima_di_layar_ambil_uang() {
        let mut k = Keadaan::default();
        ketik_semua(&mut k, "123");
        assert_eq!(k.input, ""); // di Menu diabaikan

        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "1a5-,.x0");
        assert_eq!(k.input, "150");
    }

    #[test]
    fn nol_di_depan_diabaikan() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "000");
        assert_eq!(k.input, "");
        ketik_semua(&mut k, "1005");
        assert_eq!(k.input, "1005");
    }

    #[test]
    fn panjang_input_dibatasi() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, &"9".repeat(MAKS_DIGIT + 5));
        assert_eq!(k.input.len(), MAKS_DIGIT);
        // nilai maksimum harus selalu bisa diparse
        assert!(baca_jumlah(&k.input).is_ok());
    }

    #[test]
    fn backspace_menghapus_digit_terakhir() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "123");
        k.hapus();
        assert_eq!(k.input, "12");
        k.hapus();
        k.hapus();
        k.hapus(); // sudah kosong, tidak boleh panik
        assert_eq!(k.input, "");
    }

    #[test]
    fn enter_dengan_input_kosong_menampilkan_error() {
        let mut k = di_ambil_uang();
        k.selanjutnya();
        assert_eq!(k.layar, Layar::AmbilUang);
        assert_eq!(k.pesan_error, Some("Jumlah belum diisi"));
        // mengetik lagi menghapus pesan error
        k.ketik('5');
        assert_eq!(k.pesan_error, None);
    }

    #[test]
    fn alur_lengkap_ambil_uang() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "1500000");
        k.selanjutnya();
        assert_eq!(k.layar, Layar::Hasil);
        assert_eq!(k.jumlah_ditarik, Some(1_500_000));
        assert_eq!(k.saldo, 8_500_000);

        // di layar Hasil, mengetik diabaikan
        k.ketik('9');
        assert_eq!(k.input, "1500000");

        // Enter di layar Hasil = kembali ke menu
        k.selanjutnya();
        assert_eq!(k.layar, Layar::Menu);
        assert_eq!(k.input, "");
        assert_eq!(k.jumlah_ditarik, None);
        assert_eq!(k.saldo, 8_500_000); // saldo tetap berkurang
    }

    #[test]
    fn saldo_tidak_cukup_ditolak_dan_saldo_tidak_berubah() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "10000001");
        k.selanjutnya();
        assert_eq!(k.layar, Layar::AmbilUang);
        assert_eq!(k.pesan_error, Some("Saldo tidak cukup"));
        assert_eq!(k.saldo, SALDO_AWAL);
    }

    #[test]
    fn tarik_seluruh_saldo_lalu_penarikan_berikutnya_ditolak() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "10000000");
        k.selanjutnya();
        assert_eq!(k.layar, Layar::Hasil);
        assert_eq!(k.saldo, 0);

        k.selanjutnya(); // kembali ke menu
        k.turun();
        k.selanjutnya(); // Ambil uang lagi
        ketik_semua(&mut k, "1000");
        k.selanjutnya();
        assert_eq!(k.pesan_error, Some("Saldo tidak cukup"));
        assert_eq!(k.saldo, 0);
    }

    #[test]
    fn esc_di_ambil_uang_kembali_ke_menu_dan_mengosongkan_input() {
        let mut k = di_ambil_uang();
        ketik_semua(&mut k, "123");
        k.sebelumnya();
        assert_eq!(k.layar, Layar::Menu);
        assert_eq!(k.input, "");
        assert_eq!(k.saldo, SALDO_AWAL);
    }

    #[test]
    fn keluar_menandai_selesai() {
        let mut k = Keadaan::default();
        assert!(!k.selesai);
        k.keluar();
        assert!(k.selesai);
    }
}
