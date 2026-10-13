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
