# Documentação Técnica e Guia Completo do rping

> Reimplementação moderna, de alta performance e com saída colorida em Rust do clássico utilitário `ping` (Mike Muuss / iputils).

---

## Sumário
1. [Visão Geral e Arquitetura](#1-visão-geral-e-arquitetura)
2. [Pré-requisitos](#2-pré-requisitos)
3. [Passo a Passo: Compilação e Build](#3-passo-a-passo-compilação-e-build)
4. [Passo a Passo: Instalação](#4-passo-a-passo-instalação)
5. [Passo a Passo: Uso e Comandos](#5-passo-a-passo-uso-e-comandos)
6. [Estrutura Interna do Código (Módulos)](#6-estrutura-interna-do-código-módulos)
7. [Mecanismos Técnicos e Otimizações](#7-mecanismos-técnicos-e-otimizações)
8. [Testes, Benchmarks e Qualidade](#8-testes-benchmarks-e-qualidade)
9. [Publicação e Distribuição (WinGet & GitHub Releases)](#9-publicação-e-distribuição-winget--github-releases)

---

## 1. Visão Geral e Arquitetura

O utilitário tradicional `ping` no Windows ou Linux frequentemente possui limitações:
- No Windows padrão, requer privilégios elevados se implementar sockets raw clássicos (`SOCK_RAW`).
- A saída no Windows é monocromática e rígida.
- As ferramentas convencionais possuem diferenças de sintaxe entre Windows (`-n`, `-l`) e Linux (`-c`, `-s`).

O **rping** resolve todas essas dores:
- **Sem elevação UAC**: No Windows, utiliza a API do Windows IP Helper (`iphlpapi.dll`) via funções seguras de FFI (`IcmpCreateFile`, `IcmpSendEcho` para IPv4 e `Icmp6CreateFile`, `Icmp6SendEcho2` para IPv6). Qualquer usuário padrão pode executar sem permissões de Administrador.
- **Sintaxe Híbrida**: Aceita tanto as flags do ecossistema Linux (`-c`, `-s`) quanto do Windows (`-n`, `-l`).
- **Terminal Moderno e Colorido**: Interpretação de latência em tempo real com faixas de cores verde, amarela e vermelha, e habilitação automática de Virtual Terminal Processing no console do Windows.
- **Tratamento de Sinais Gracioso**: Interrupções via `Ctrl+C` capturam o sinal e sempre exibem o sumário estatístico completo antes de encerrar.

---

## 2. Pré-requisitos

Para compilar e desenvolver o projeto do zero:
- **Rust Toolchain**: Rust 1.70 ou superior (`rustup default stable`).
- **Target**: `x86_64-pc-windows-msvc` (ou `x86_64-unknown-linux-gnu` para Linux).
- **Git**: Para controle de versão.
- **PowerShell** ou **CMD**: Para execução dos scripts utilitários de automação.

---

## 3. Passo a Passo: Compilação e Build

O projeto conta com scripts prontos para facilitar o fluxo de compilação sem necessidade de lembrar parâmetros avançados do Cargo.

### Método 1: Via Script PowerShell (Recomendado no Windows)
Abra o PowerShell na pasta do projeto e execute:
```powershell
.\build.ps1
```
Esse script executa:
1. `cargo build --release` com as flags de otimização máxima.
2. Copia o binário resultante para a raiz do repositório como `rping.exe`.
3. Copia uma réplica adicional nomeada como `ping.exe`.

### Método 2: Via Script Batch (Prompt de Comando / CMD)
```cmd
build.cmd
```

### Método 3: Compilação Manual via Cargo
```powershell
# Compilação em modo release com todas as otimizações
cargo build --release

# Opcional: copiar para a raiz
Copy-Item target\release\rping.exe .\rping.exe -Force
Copy-Item target\release\rping.exe .\ping.exe -Force
```

---

## 4. Passo a Passo: Instalação

### Opção A: Instalação Oficial via WinGet (Windows Package Manager)
Após aprovação do pacote comunitário:
```powershell
winget install leandrop2103.rping
```
O WinGet baixa o binário portátil e o disponibiliza globalmente no PATH.

### Opção B: Instalação Local Automatizada
O projeto inclui um script `instalar.cmd` que solicita elevação de privilégios automaticamente e copia o executável para uma pasta do sistema (`C:\Windows` ou diretório de sistema):
```cmd
instalar.cmd
```

### Opção C: Manualmente no PATH do Usuário
1. Crie uma pasta, por exemplo: `C:\bin`.
2. Adicione `C:\bin` à variável de ambiente `PATH`.
3. Mova `rping.exe` para `C:\bin`.

---

## 5. Passo a Passo: Uso e Comandos

### Sintaxe Básica
```text
rping [OPÇÕES] <DESTINO>
```

### Principais Parâmetros e Opções
| Parâmetro | Equivalente Windows | Descrição | Padrão |
|---|---|---|---|
| `-c, --count <N>` | `-n <N>` | Número de pacotes a enviar | Infinito |
| `-i, --interval <SEG>` | — | Intervalo entre pacotes em segundos | `1.0` |
| `-s, --size <BYTES>` | `-l <BYTES>` | Tamanho da carga útil (payload) | `56` bytes |
| `-T, --ttl <VAL>` | — | Tempo de vida do pacote (TTL) | `64` |
| `-W, --timeout <SEG>` | — | Timeout de espera para cada resposta | `1.0` s |
| `-w, --deadline <SEG>` | — | Limite total de tempo para execução | Nenhum |
| `-t, --timestamp` | — | Imprime timestamp UNIX epoch em microssegundos | Desativado |
| `-th, --th` | — | Imprime timestamp humano (`dd/mm-hh:mm:ss`) | Desativado |
| `-4, --ipv4` | — | Força resolução exclusiva para IPv4 | Automático |
| `-6, --ipv6` | — | Força resolução exclusiva para IPv6 | Automático |
| `-a, --audible` | — | Emite aviso sonoro (beep) em cada resposta | Desativado |
| `-q, --quiet` | — | Modo silencioso (apenas cabeçalho e resumo) | Desativado |
| `--no-color` | — | Desativa cores ANSI | Auto |

### Exemplos de Execução
```powershell
# 1. Ping padrão contínuo
.\rping.exe google.com

# 2. Ping com limite de pacotes e timestamp legível
.\rping.exe -th -c 5 1.1.1.1

# 3. Estilo tradicional do Windows
.\ping.exe -n 4 -l 64 8.8.8.8

# 4. Forçando IPv6 com payload personalizado
.\rping.exe -6 -s 128 cloudflare.com
```

---

## 6. Estrutura Interna do Código (Módulos)

O código-fonte em [`src`](file:///d:/rust/ping/src) é decomposto de maneira modular:

```text
src/
├── app.rs          # Orquestrador do ciclo de vida: loop de envio, temporização e cancelamento
├── cli.rs          # Definições de argumentos e flags usando `clap`
├── error.rs        # Tipos de erro unificados (enum `PingError`) e conversões
├── lib.rs          # Exportação dos submódulos e API pública
├── main.rs         # Ponto de entrada, configuração de logs e chamada de execução
├── stats.rs        # Coletor estatístico: rtt min, max, avg, perda e cálculo de desvio padrão (mdev)
├── display/        # Camada de apresentação e estilização
│   ├── mod.rs      # Formatação do cabeçalho, linhas de eco e sumário final
│   └── style.rs    # Regras de colorização ANSI e suporte ao padrão NO_COLOR
└── engine/         # Camada de abstração de rede ICMP
    ├── mod.rs      # Trait unificada `PingEngine`
    ├── unix.rs     # Implementação para Linux/Unix via datagram/raw sockets
    └── windows.rs  # Implementação Windows via IP Helper API (IcmpSendEcho / Icmp6SendEcho2)
```

---

## 7. Mecanismos Técnicos e Otimizações

1. **FFI Seguro no Windows (`engine/windows.rs`)**:
   - Não usa `SOCK_RAW`, o que contorna a exigência de UAC Administrator.
   - Carrega `IcmpCreateFile` e `IcmpSendEcho` alocando buffers alinhados para as estruturas nativas `ICMP_ECHO_REPLY` e `IP_OPTION_INFORMATION`.
2. **Zero-Allocation no Hot-Path**:
   - Os buffers de envio e recebimento de pacotes são pré-alocados fora do loop e reutilizados a cada iteração.
3. **Cálculo de Desvio Padrão (`mdev`) de Alta Precisão**:
   - Implementado em [`src/stats.rs`](file:///d:/rust/ping/src/stats.rs) com fórmula incremental de variância online com `mul_add` (FMA).
4. **Otimizações no `Cargo.toml`**:
   - `opt-level = "z"` (foco em tamanho e densidade de cache).
   - `lto = "fat"` (Link-Time Optimization total entre crates).
   - `codegen-units = 1` (otimização interprocedural máxima).
   - `panic = "abort"` (remove o custo das tabelas de stack unwinding).
   - `strip = true` (remove símbolos de debug desnecessários no binário final).
   - `crt-static` (runtime C vinculado estaticamente, tornando o binário livre de dependências de DLLs externas).

---

## 8. Testes, Benchmarks e Qualidade

Para garantir a confiabilidade do código:

```powershell
# Executar todos os testes unitários e de integração
cargo test

# Executar linter estrito do Rust (clippy)
cargo clippy --all-targets --all-features -- -D warnings

# Executar suíte de benchmarks (Criterion)
cargo bench
```

---

## 9. Publicação e Distribuição (WinGet & GitHub Releases)

O pipeline de entrega contínua do projeto é estruturado em:
1. **GitHub Releases**:
   - Ao criar uma nova tag (ex: `v0.1.0`), o binário é empacotado em `.zip` e publicado junto com seu hash SHA-256.
2. **Manifesto do WinGet**:
   - Localizado em `manifests/l/leandrop2103/rping/0.1.0/`.
   - Contém o identificador `leandrop2103.rping`, a URL de download da release e o instalador do tipo `portable`.
   - Submetido via Pull Request para o repositório oficial [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).
