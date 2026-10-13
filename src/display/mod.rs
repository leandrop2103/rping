//! Saída no terminal e formatação de apresentação para o ping.

pub mod style;

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
}

impl Printer {
    /// Cria uma nova instância do formatador de saída.
    #[must_use]
    pub fn new(no_color: bool, quiet: bool, audible: bool) -> Self {
        Self {
            style: Style::new(no_color),
            quiet,
            audible,
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

        let header = format!("{ping_label} {target_str} {ip_str} {bytes_str}{total_str} bytes de dados.");
        writeln!(out, "{header}")?;
        out.flush()
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

        let mut out = io::stdout().lock();
        writeln!(
            out,
            "{colored_bytes} de {colored_ip}: {colored_seq} {colored_ttl} tempo={colored_rtt}"
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

        let msg = format!("Tempo limite esgotado para icmp_seq {seq}");
        let mut out = io::stdout().lock();
        writeln!(out, "{}", self.style.bold_red(&msg))?;
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

        let prefix = from.map_or_else(
            || format!("icmp_seq={seq}: {message}"),
            |ip| format!("De {ip} icmp_seq={seq}: {message}"),
        );

        let mut out = io::stdout().lock();
        writeln!(out, "{}", self.style.bold_yellow(&prefix))?;
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

        let total_ms = summary.total_duration.as_millis();
        let loss_colored = self.style.color_loss(summary.packet_loss_percent);

        writeln!(
            out,
            "{} pacotes transmitidos, {} recebidos, {} de perda de pacotes, tempo total {}ms",
            summary.transmitted, summary.received, loss_colored, total_ms
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
