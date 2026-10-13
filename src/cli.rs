//! Análise de argumentos de linha de comando e configuração.

use clap::Parser;

/// Utilitário de ping moderno e de alto desempenho com saída colorida e suporte multiplataforma.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "rping",
    author = "Leandro",
    version,
    about = "Utilitário de ping multiplataforma de alto desempenho escrito em Rust",
    long_about = "Reimplementação moderna do utilitário ping (iputils) em Rust para Windows e Linux.\nPossui hot-path sem alocações, ICMP não privilegiado no Windows, saída colorida e estatísticas completas."
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Nome de host ou endereço IP de destino para pingar
    #[arg(value_name = "DESTINO")]
    pub destination: String,

    /// Número de pacotes `ECHO_REQUEST` a enviar
    #[arg(short = 'c', long = "count", value_name = "CONTAGEM")]
    pub count: Option<u64>,

    /// Intervalo de espera em segundos entre o envio de cada pacote
    #[arg(
        short = 'i',
        long = "interval",
        default_value_t = 1.0,
        value_name = "SEGUNDOS"
    )]
    pub interval: f64,

    /// Número de bytes de dados da carga útil (payload) a enviar
    #[arg(short = 's', long = "size", default_value_t = 56, value_name = "BYTES")]
    pub size: usize,

    /// Tempo de vida (TTL) do pacote IP em saltos (1-255)
    #[arg(short = 'T', long = "ttl", default_value_t = 64, value_name = "TTL")]
    pub ttl: u8,

    /// Exibe carimbo de data/hora UNIX epoch em cada resposta
    #[arg(short = 't', long = "timestamp")]
    pub timestamp: bool,

    /// Exibe carimbo de data/hora em formato humano (dd/mm-hh:mm:ss) em cada resposta
    #[arg(long = "th", alias = "timestamp-human")]
    pub timestamp_human: bool,

    /// Tempo limite de espera por resposta em segundos
    #[arg(
        short = 'W',
        long = "timeout",
        default_value_t = 1.0,
        value_name = "SEGUNDOS"
    )]
    pub timeout: f64,

    /// Encerra após o tempo limite (deadline) em segundos, independentemente da contagem de pacotes
    #[arg(short = 'w', long = "deadline", value_name = "SEGUNDOS")]
    pub deadline: Option<f64>,

    /// Força resolução de nomes para IPv4
    #[arg(short = '4', long = "ipv4", conflicts_with = "ipv6")]
    pub ipv4: bool,

    /// Força resolução de nomes para IPv6
    #[arg(short = '6', long = "ipv6", conflicts_with = "ipv4")]
    pub ipv6: bool,

    /// Saída audível (emite sinal sonoro / beep a cada resposta recebida)
    #[arg(short = 'a', long = "audible")]
    pub audible: bool,

    /// Modo silencioso; exibe apenas o cabeçalho inicial e o resumo final
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Modo detalhado com registros de diagnóstico estruturados
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Desativa a saída colorida no terminal
    #[arg(long = "no-color")]
    pub no_color: bool,
}

/// Modo de exibição de carimbo de data/hora (timestamp).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimestampMode {
    /// Sem carimbo de data/hora.
    #[default]
    None,
    /// Timestamp numérico Unix Epoch com microssegundos (`[1728834921.123456]`).
    Unix,
    /// Timestamp legível em formato humano `dd/mm-hh:mm:ss` (`[13/10-14:08:17]`).
    Human,
}

impl Cli {
    /// Determina o modo de timestamp configurado nos argumentos.
    #[must_use]
    pub const fn timestamp_mode(&self) -> TimestampMode {
        if self.timestamp_human {
            TimestampMode::Human
        } else if self.timestamp {
            TimestampMode::Unix
        } else {
            TimestampMode::None
        }
    }

    /// Analisa uma sequência arbitrária de argumentos, normalizando `-th` para `--th`.
    #[must_use]
    pub fn parse_from_args<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let normalized = args.into_iter().map(|item| {
            let s: std::ffi::OsString = item.into();
            if s == "-th" {
                std::ffi::OsString::from("--th")
            } else {
                s
            }
        });
        Self::parse_from(normalized)
    }

    /// Analisa os argumentos da linha de comando, normalizando atalhos como `-th` para `--th`.
    #[must_use]
    pub fn parse_args() -> Self {
        Self::parse_from_args(std::env::args_os())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp_flags() {
        let cli = Cli::parse_from_args(["rping", "-t", "1.1.1.1"]);
        assert!(cli.timestamp);
        assert!(!cli.timestamp_human);
        assert_eq!(cli.timestamp_mode(), TimestampMode::Unix);

        let cli_th = Cli::parse_from_args(["rping", "-th", "1.1.1.1"]);
        assert!(cli_th.timestamp_human);
        assert_eq!(cli_th.timestamp_mode(), TimestampMode::Human);

        let cli_long_th = Cli::parse_from_args(["rping", "--th", "1.1.1.1"]);
        assert!(cli_long_th.timestamp_human);
        assert_eq!(cli_long_th.timestamp_mode(), TimestampMode::Human);
    }
}
