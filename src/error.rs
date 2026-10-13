//! Tipos de erro para o utilitário de ping.

use std::net::IpAddr;
use thiserror::Error;

/// Tipo de erro principal representando qualquer falha durante a execução do ping.
#[derive(Debug, Error)]
pub enum PingError {
    /// Resolução de DNS ou nome de host falhou.
    #[error("falha ao resolver o destino '{0}': {1}")]
    HostResolutionFailed(String, String),

    /// Nenhum endereço IP válido foi encontrado para o destino.
    #[error("nenhum endereço IP encontrado para o destino '{0}'")]
    NoAddressFound(String),

    /// A família de endereços do destino não coincide com a versão forçada (-4 ou -6).
    #[error("endereço resolvido '{0}' não corresponde à versão de IP solicitada")]
    AddressFamilyMismatch(IpAddr),

    /// Falha ao criar o identificador/socket ICMP do sistema.
    #[error("falha ao criar socket/identificador ICMP: {0}")]
    HandleCreationFailed(String),

    /// Tempo limite de resposta da sonda esgotado.
    #[error("tempo limite esgotado para icmp_seq {0}")]
    Timeout(u16),

    /// Tempo de vida (TTL) expirado no trânsito.
    #[error("tempo de vida (TTL) expirado no trânsito a partir de {0}")]
    TtlExpired(IpAddr),

    /// Destino inalcançável com causa específica.
    #[error("destino inalcançável: {0}")]
    DestinationUnreachable(String),

    /// O tamanho da carga útil excede o máximo permitido.
    #[error("tamanho da carga útil de {requested} bytes excede o máximo permitido ({max} bytes)")]
    PayloadTooLarge {
        /// Tamanho solicitado da carga útil em bytes.
        requested: usize,
        /// Tamanho máximo permitido da carga útil em bytes.
        max: usize,
    },

    /// Ocorreu um erro de E/S subjacente.
    #[error("erro de E/S: {0}")]
    Io(#[from] std::io::Error),

    /// Erro específico do sistema operacional.
    #[error("erro do sistema operacional: {0}")]
    OsError(String),
}

/// Apelido de conveniência para Results que retornam [`PingError`].
pub type Result<T> = std::result::Result<T, PingError>;
