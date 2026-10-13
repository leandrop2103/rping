//! Saída no terminal e formatação de apresentação para o ping.

pub mod style;

use crate::cli::TimestampMode;
use crate::stats::StatsSummary;
use std::io::{self, Write};
use std::net::IpAddr;
use std::time::Duration;
use style::Style;

/// Formatador de saída no terminal para eventos de ping com suporte a estilos ANSI.
pub struct Printer {
    style: Style,
    quiet: bool,
    audible: bool,
    timestamp_mode: TimestampMode,
}

impl Printer {
    /// Cria uma nova instância do formatador de saída.
    #[must_use]
    pub fn new(no_color: bool, quiet: bool, audible: bool, timestamp_mode: TimestampMode) -> Self {
        Self {
            style: Style::new(no_color),
            quiet,
            audible,
            timestamp_mode,
        }
    }

    /// Emite sinal sonoro (beep/bell) se a opção estiver habilitada.
    pub fn ring_bell(&self) {
        if self.audible {
            print!("\x07");
            let _ = io::stdout().flush();
        }
    }

    /// Imprime o cabeçalho inicial da sessão de ping.
    ///
    /// # Errors
    /// Retorna `io::Error` se a escrita na saída padrão falhar.
    pub fn print_header(
        &self,
        target_name: &str,
        resolved_ip: IpAddr,
        payload_bytes: usize,
    ) -> io::Result<()> {
        let total_bytes = payload_bytes + 28; // 20 bytes IP + 8 bytes ICMP
        let mut out = io::stdout().lock();

        let ping_label = self.style.bold_cyan("PING");
        let target_str = self.style.color_host(target_name);
        let ip_str = self.style.cyan(&format!("({resolved_ip})"));
        let bytes_str = self.style.bold_cyan(&payload_bytes.to_string());
        let total_str = self.style.dim(&format!("({total_bytes})"));

        let header =
            format!("{ping_label} {target_str} {ip_str} {bytes_str}{total_str} bytes de dados.");
        writeln!(out, "{header}")?;
        out.flush()
    }

    fn timestamp_prefix(&self) -> String {
        match self.timestamp_mode {
            TimestampMode::None => String::new(),
            TimestampMode::Unix => self.style.color_timestamp(&current_unix_timestamp()),
            TimestampMode::Human => self.style.color_timestamp(&current_human_timestamp()),
        }
    }

    /// Imprime uma resposta de eco bem-sucedida.
    ///
    /// # Errors
    /// Retorna `io::Error` se a escrita na saída padrão falhar.
    pub fn print_reply(
        &self,
        bytes: usize,
        from_ip: IpAddr,
        seq: u16,
        ttl: u8,
        rtt: Duration,
    ) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        self.ring_bell();

        let rtt_ms = rtt.as_secs_f64() * 1000.0;
        let colored_bytes = self.style.color_bytes(bytes);
        let colored_ip = self.style.color_host(&from_ip.to_string());
        let colored_seq = self.style.color_seq(seq);
        let colored_ttl = self.style.color_ttl(ttl);
        let colored_rtt = self.style.color_rtt(rtt_ms);
        let ts_prefix = self.timestamp_prefix();

        let mut out = io::stdout().lock();
        writeln!(
            out,
            "{ts_prefix}{colored_bytes} de {colored_ip}: {colored_seq} {colored_ttl} tempo={colored_rtt}"
        )?;
        out.flush()
    }

    /// Imprime uma notificação de tempo limite esgotado.
    ///
    /// # Errors
    /// Retorna `io::Error` se a escrita na saída padrão falhar.
    pub fn print_timeout(&self, seq: u16) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        let ts_prefix = self.timestamp_prefix();
        let msg = format!("Tempo limite esgotado para icmp_seq {seq}");
        let mut out = io::stdout().lock();
        writeln!(out, "{ts_prefix}{}", self.style.bold_red(&msg))?;
        out.flush()
    }

    /// Imprime uma mensagem de erro ICMP (ex: TTL expirado, destino inalcançável).
    ///
    /// # Errors
    /// Retorna `io::Error` se a escrita na saída padrão falhar.
    pub fn print_error_reply(
        &self,
        from: Option<IpAddr>,
        seq: u16,
        message: &str,
    ) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        let ts_prefix = self.timestamp_prefix();
        let prefix = from.map_or_else(
            || format!("icmp_seq={seq}: {message}"),
            |ip| format!("De {ip} icmp_seq={seq}: {message}"),
        );

        let mut out = io::stdout().lock();
        writeln!(out, "{ts_prefix}{}", self.style.bold_yellow(&prefix))?;
        out.flush()
    }

    /// Imprime o bloco de resumo com estatísticas finais acumuladas.
    ///
    /// # Errors
    /// Retorna `io::Error` se a escrita na saída padrão falhar.
    pub fn print_summary(&self, summary: &StatsSummary) -> io::Result<()> {
        let mut out = io::stdout().lock();

        let target_str = self.style.color_host(&summary.target);
        let header = format!("--- estatísticas de ping para {target_str} ---");
        writeln!(out, "\n{}", self.style.bold_cyan(&header))?;

        let trans_str = self.style.color_transmitted(summary.transmitted);
        let recv_str = self
            .style
            .color_received(summary.received, summary.transmitted);
        let loss_colored = self.style.color_loss(summary.packet_loss_percent);
        let time_str = self.style.format_duration(summary.total_duration);

        writeln!(
            out,
            "{trans_str}, {recv_str}, {loss_colored} de perda de pacotes, tempo total {time_str}"
        )?;

        if let (Some(min), Some(avg), Some(max), Some(mdev)) = (
            summary.rtt_min,
            summary.rtt_avg,
            summary.rtt_max,
            summary.rtt_mdev,
        ) {
            let stats_line = self.style.color_rtt_stats(min, avg, max, mdev);
            writeln!(out, "{stats_line}")?;
        }

        out.flush()
    }
}

#[cfg(windows)]
fn current_human_timestamp() -> String {
    #[repr(C)]
    struct Win32SystemTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(lpSystemTime: *mut Win32SystemTime);
    }

    let mut st = std::mem::MaybeUninit::<Win32SystemTime>::uninit();
    unsafe {
        GetLocalTime(st.as_mut_ptr());
        let st = st.assume_init();
        format!(
            "{:02}/{:02}-{:02}:{:02}:{:02}",
            st.day, st.month, st.hour, st.minute, st.second
        )
    }
}

#[cfg(not(windows))]
fn current_human_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = now.as_secs();
    let s = total_secs % 60;
    let m = (total_secs / 60) % 60;
    let h = (total_secs / 3600) % 24;
    format!("00/00-{h:02}:{m:02}:{s:02}")
}

fn current_unix_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:06}", now.as_secs(), now.subsec_micros())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_unix_timestamp_format() {
        let ts = current_unix_timestamp();
        assert!(ts.contains('.'));
        let parts: Vec<&str> = ts.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 6);
    }

    #[test]
    fn test_current_human_timestamp_format() {
        let ts = current_human_timestamp();
        // dd/mm-hh:mm:ss -> length 14, contains '/' and '-' and ':'
        assert_eq!(ts.len(), 14);
        assert_eq!(&ts[2..3], "/");
        assert_eq!(&ts[5..6], "-");
        assert_eq!(&ts[8..9], ":");
        assert_eq!(&ts[11..12], ":");
    }

    #[test]
    fn test_timestamp_prefix_disabled() {
        let printer = Printer::new(false, false, false, TimestampMode::None);
        assert_eq!(printer.timestamp_prefix(), "");
    }

    #[test]
    fn test_timestamp_prefix_human() {
        let printer = Printer::new(true, false, false, TimestampMode::Human);
        let prefix = printer.timestamp_prefix();
        assert!(prefix.starts_with('['));
        assert!(prefix.ends_with("] "));
    }
}
