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

        let header = format!(
            "PING {target_name} ({resolved_ip}) {payload_bytes}({total_bytes}) bytes de dados."
        );
        writeln!(out, "{}", self.style.bold_cyan(&header))?;
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
        let colored_rtt = self.style.color_rtt(rtt_ms);

        let mut out = io::stdout().lock();
        writeln!(
            out,
            "{bytes} bytes de {from_ip}: icmp_seq={seq} ttl={ttl} tempo={colored_rtt}"
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

        let header = format!("--- estatísticas de ping para {} ---", summary.target);
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
            let stats_line =
                format!("rtt mín/méd/máx/mdev = {min:.3}/{avg:.3}/{max:.3}/{mdev:.3} ms");
            writeln!(out, "{}", self.style.bold(&stats_line))?;
        }

        out.flush()
    }
}
