//! Zero-dependency ANSI style and terminal color configuration.

use std::io::IsTerminal;

/// ANSI styling sequences.
pub struct Style {
    enabled: bool,
}

impl Style {
    /// Determines whether coloring should be enabled based on user flags, env vars, and terminal TTY.
    #[must_use]
    pub fn new(force_no_color: bool) -> Self {
        let no_color_env = std::env::var_os("NO_COLOR").is_some();
        let is_term = std::io::stdout().is_terminal();

        #[cfg(windows)]
        let _ = enable_windows_ansi();

        let enabled = !force_no_color && !no_color_env && is_term;
        Self { enabled }
    }

    /// Returns whether colors are enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Wraps text in bold cyan.
    #[must_use]
    pub fn bold_cyan(&self, text: &str) -> String {
        self.format("\x1b[1;36m", text)
    }

    /// Wraps text in bold green.
    #[must_use]
    pub fn bold_green(&self, text: &str) -> String {
        self.format("\x1b[1;32m", text)
    }

    /// Wraps text in bold yellow.
    #[must_use]
    pub fn bold_yellow(&self, text: &str) -> String {
        self.format("\x1b[1;33m", text)
    }

    /// Wraps text in bold red.
    #[must_use]
    pub fn bold_red(&self, text: &str) -> String {
        self.format("\x1b[1;31m", text)
    }

    /// Wraps text in bold magenta.
    #[must_use]
    pub fn bold_magenta(&self, text: &str) -> String {
        self.format("\x1b[1;35m", text)
    }

    /// Wraps text in dim/gray.
    #[must_use]
    pub fn dim(&self, text: &str) -> String {
        self.format("\x1b[2;37m", text)
    }

    /// Wraps text in bold white.
    #[must_use]
    pub fn bold(&self, text: &str) -> String {
        self.format("\x1b[1m", text)
    }

    /// Applies color to an RTT value based on latency thresholds:
    /// - Green: < 50ms (fast)
    /// - Yellow: 50ms .. 150ms (moderate)
    /// - Red: >= 150ms (slow)
    #[must_use]
    pub fn color_rtt(&self, millis: f64) -> String {
        let text = format!("{millis:.3} ms");
        if !self.enabled {
            return text;
        }

        if millis < 50.0 {
            format!("\x1b[1;32m{text}\x1b[0m")
        } else if millis < 150.0 {
            format!("\x1b[1;33m{text}\x1b[0m")
        } else {
            format!("\x1b[1;31m{text}\x1b[0m")
        }
    }

    /// Wraps text in normal cyan.
    #[must_use]
    pub fn cyan(&self, text: &str) -> String {
        self.format("\x1b[36m", text)
    }

    /// Formata a quantidade de bytes com destaque (ciano brilhante).
    #[must_use]
    pub fn color_bytes(&self, bytes: usize) -> String {
        if self.enabled {
            format!("\x1b[1;36m{bytes} bytes\x1b[0m")
        } else {
            format!("{bytes} bytes")
        }
    }

    /// Formata o host ou endereço IP de origem (magenta brilhante).
    #[must_use]
    pub fn color_host(&self, host: &str) -> String {
        if self.enabled {
            format!("\x1b[1;35m{host}\x1b[0m")
        } else {
            host.to_string()
        }
    }

    /// Formata o campo de sequência ICMP (azul).
    #[must_use]
    pub fn color_seq(&self, seq: u16) -> String {
        if self.enabled {
            format!("\x1b[34micmp_seq=\x1b[1;34m{seq}\x1b[0m")
        } else {
            format!("icmp_seq={seq}")
        }
    }

    /// Formata o campo Time To Live (TTL) (amarelo).
    #[must_use]
    pub fn color_ttl(&self, ttl: u8) -> String {
        if self.enabled {
            format!("\x1b[33mttl=\x1b[1;33m{ttl}\x1b[0m")
        } else {
            format!("ttl={ttl}")
        }
    }

    /// Formata os valores de RTT nas estatísticas com números destacados em ciano.
    #[must_use]
    pub fn color_rtt_stats(&self, min: f64, avg: f64, max: f64, mdev: f64) -> String {
        if self.enabled {
            format!(
                "rtt mín/méd/máx/mdev = \x1b[1;36m{min:.3}\x1b[0m/\x1b[1;36m{avg:.3}\x1b[0m/\x1b[1;36m{max:.3}\x1b[0m/\x1b[1;36m{mdev:.3}\x1b[0m ms"
            )
        } else {
            format!("rtt mín/méd/máx/mdev = {min:.3}/{avg:.3}/{max:.3}/{mdev:.3} ms")
        }
    }

    /// Formata a contagem de pacotes transmitidos com destaque (ciano brilhante).
    #[must_use]
    pub fn color_transmitted(&self, count: u64) -> String {
        if self.enabled {
            format!("\x1b[1;36m{count} pacotes transmitidos\x1b[0m")
        } else {
            format!("{count} pacotes transmitidos")
        }
    }

    /// Formata a contagem de pacotes recebidos com cor condicional de status.
    #[must_use]
    pub fn color_received(&self, received: u64, transmitted: u64) -> String {
        if !self.enabled {
            return format!("{received} recebidos");
        }

        if received == transmitted && transmitted > 0 {
            format!("\x1b[1;32m{received} recebidos\x1b[0m")
        } else if received > 0 {
            format!("\x1b[1;33m{received} recebidos\x1b[0m")
        } else {
            format!("\x1b[1;31m{received} recebidos\x1b[0m")
        }
    }

    /// Formata uma duração de tempo de forma legível em pt-BR:
    /// - < 60s: em segundos com duas casas decimais, ex: "7.51s"
    /// - >= 60s e < 3600s: "1m e 15s" (ou "1m" se segundos forem zero)
    /// - >= 3600s: "1h, 4m e 3s"
    #[must_use]
    pub fn format_duration(&self, duration: std::time::Duration) -> String {
        let total_secs = duration.as_secs();
        let formatted = if total_secs < 60 {
            format!("{:.2}s", duration.as_secs_f64())
        } else if total_secs < 3600 {
            let m = total_secs / 60;
            let s = total_secs % 60;
            if s == 0 {
                format!("{m}m")
            } else {
                format!("{m}m e {s}s")
            }
        } else {
            let h = total_secs / 3600;
            let m = (total_secs % 3600) / 60;
            let s = total_secs % 60;
            match (m, s) {
                (0, 0) => format!("{h}h"),
                (0, _) => format!("{h}h e {s}s"),
                (_, 0) => format!("{h}h e {m}m"),
                (_, _) => format!("{h}h, {m}m e {s}s"),
            }
        };

        if self.enabled {
            format!("\x1b[1;35m{formatted}\x1b[0m")
        } else {
            formatted
        }
    }

    /// Applies color to a packet loss percentage:
    /// - Green: 0%
    /// - Yellow: 0.1% .. 20%
    /// - Red: > 20%
    #[must_use]
    pub fn color_loss(&self, loss_pct: f64) -> String {
        let text = format!("{loss_pct:.1}%");
        if !self.enabled {
            return text;
        }

        if loss_pct < f64::EPSILON {
            format!("\x1b[1;32m{text}\x1b[0m")
        } else if loss_pct <= 20.0 {
            format!("\x1b[1;33m{text}\x1b[0m")
        } else {
            format!("\x1b[1;31m{text}\x1b[0m")
        }
    }

    fn format(&self, escape_seq: &str, text: &str) -> String {
        if self.enabled {
            format!("{escape_seq}{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

/// Enables ANSI virtual terminal processing on Windows consoles if running on Windows.
#[cfg(windows)]
fn enable_windows_ansi() -> bool {
    use std::os::windows::raw::HANDLE;

    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5; // (DWORD)-11
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> HANDLE;
        fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: u32) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() || handle == -1isize as HANDLE {
            return false;
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &raw mut mode) == 0 {
            return false;
        }
        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_duration_sub_minute() {
        let style = Style { enabled: false };
        assert_eq!(style.format_duration(Duration::from_millis(7510)), "7.51s");
        assert_eq!(style.format_duration(Duration::from_millis(350)), "0.35s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let style = Style { enabled: false };
        assert_eq!(style.format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(style.format_duration(Duration::from_secs(75)), "1m e 15s");
        assert_eq!(style.format_duration(Duration::from_secs(124)), "2m e 4s");
    }

    #[test]
    fn test_format_duration_hours() {
        let style = Style { enabled: false };
        assert_eq!(style.format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(style.format_duration(Duration::from_secs(3605)), "1h e 5s");
        assert_eq!(style.format_duration(Duration::from_mins(61)), "1h e 1m");
        assert_eq!(
            style.format_duration(Duration::from_secs(3843)),
            "1h, 4m e 3s"
        );
    }

    #[test]
    fn test_color_transmitted_and_received() {
        let style = Style { enabled: false };
        assert_eq!(style.color_transmitted(8), "8 pacotes transmitidos");
        assert_eq!(style.color_received(8, 8), "8 recebidos");

        let style_colored = Style { enabled: true };
        assert!(style_colored
            .color_transmitted(8)
            .contains("8 pacotes transmitidos"));
        assert!(style_colored.color_received(8, 8).contains("\x1b[1;32m"));
        assert!(style_colored.color_received(4, 8).contains("\x1b[1;33m"));
        assert!(style_colored.color_received(0, 8).contains("\x1b[1;31m"));
    }
}
