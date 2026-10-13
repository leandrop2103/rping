# rping

Uma reimplementação moderna, ultrarrápida e com saída colorida do utilitário tradicional `ping` (iputils/Mike Muuss `ping.c`) escrita em **Rust**.

Projetado especificamente para rodar nativamente e com máxima performance no **Windows** e **Linux**, sem exigir permissões de Administrador/root no Windows através da API oficial **Windows IP Helper** (`iphlpapi.dll`).

---

## 🚀 Destaques e Arquitetura

- **100% Cross-Platform Sem Gambiarras**:
  - **Windows**: Utiliza `IcmpSendEcho` (IPv4) e `Icmp6SendEcho2` (IPv6) via FFI seguro e tipado (`iphlpapi.dll`). Funciona para usuários padrão sem elevação UAC ("Executar como Administrador").
  - **Linux / Unix**: Suporte nativo via sockets ICMP datagram/raw.
- **Saída Colorida Inteligente**:
  - Destaque dinâmico de latência (RTT):
    - 🟢 **Verde**: `< 50ms` (conexão ultrarrápida)
    - 🟡 **Amarelo**: `50ms - 150ms` (latência moderada)
    - 🔴 **Vermelho**: `> 150ms` ou perda de pacotes (latência alta)
  - Estatísticas finais estilizadas com taxas de perda coloridas.
  - Habilitação automática de sequências ANSI no Windows Terminal, PowerShell e CMD (`ENABLE_VIRTUAL_TERMINAL_PROCESSING`).
  - Respeita o padrão `NO_COLOR` e desativa escape codes automaticamente quando redirecionado para pipes/arquivos.
- **Hot-Path Zero-Allocation**:
  - Buffers de I/O reutilizados na memória heap/stack sem alocações intermediárias dentro do loop de ping.
  - Cálculos matemáticos precisos de desvio padrão (`mdev`), média, mínimo e máximo usando instruções `mul_add` (FMA).
- **Asynchronous & Responsive**:
  - Orquestração não-bloqueante com `tokio` (interval pacing, resolução DNS e sinal Ctrl+C limpo que sempre imprime o resumo antes de sair).
- **Otimização Extrema de Binário**:
  - Release com `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` e runtime estático (`target-feature=+crt-static`).
  - Binário final autônomo com menos de **900 KB**.

---

## 📦 Instalação e Compilação

### Compilar Release Otimizado:
```powershell
cargo build --release
```

O binário final estará em:
```powershell
target\release\rping.exe
```

---

## 💻 Uso e Opções da CLI

```text
Uso: rping.exe [OPÇÕES] <DESTINATION>

Argumentos:
  <DESTINATION>              Hostname ou endereço IP (ex: google.com, 1.1.1.1, ::1)

Opções:
  -c, --count <COUNT>        Número de pacotes ECHO_REQUEST a enviar
  -i, --interval <SECONDS>   Intervalo em segundos entre envios [padrão: 1.0]
  -s, --size <BYTES>         Tamanho da carga útil (payload) em bytes [padrão: 56]
  -t, --ttl <TTL>            Time To Live (TTL) do pacote [padrão: 64]
  -W, --timeout <SECONDS>    Tempo limite de espera por resposta em segundos [padrão: 1.0]
  -w, --deadline <SECONDS>   Encerra o programa após este tempo limite total
  -4, --ipv4                 Força resolução para IPv4
  -6, --ipv6                 Força resolução para IPv6
  -a, --audible              Emite sinal sonoro (beep) em cada resposta recebida
  -q, --quiet                Modo silencioso (imprime apenas início e sumário final)
  -v, --verbose              Habilita diagnóstico e tracing estruturado
      --no-color             Desativa cores ANSI na saída
  -h, --help                 Exibe ajuda
  -V, --version              Exibe versão
```

### Exemplos Práticos

```powershell
# Ping simples com 4 pacotes (colorido)
.\target\release\rping.exe -c 4 google.com

# Ping rápido a cada 200ms com timeout curto
.\target\release\rping.exe -c 10 -i 0.2 -W 0.5 1.1.1.1

# Ping contínuo (Ctrl+C encerra e mostra estatísticas)
.\target\release\rping.exe 8.8.8.8

# Forçar resolução IPv6
.\target\release\rping.exe -6 google.com
```

---

## 🧪 Testes e Benchmarks

Executar testes unitários e de integração:
```powershell
cargo test
```

Executar benchmarks com Criterion:
```powershell
cargo bench
```

Verificação rigorosa com Clippy pedantic:
```powershell
cargo clippy --all-targets --all-features -- -D warnings
```
