# rping

Uma reimplementação moderna, ultrarrápida e com saída colorida em **pt-BR** do utilitário tradicional `ping` (iputils/Mike Muuss `ping.c`) escrita em **Rust**.

Projetado especificamente para rodar nativamente e com máxima performance no **Windows** e **Linux**, sem exigir permissões de Administrador/root no Windows através da API oficial **Windows IP Helper** (`iphlpapi.dll`).

---

## 🚀 Destaques e Arquitetura

- **100% Cross-Platform Sem Privilégios Elevados**:
  - **Windows**: Utiliza `IcmpSendEcho` (IPv4) e `Icmp6SendEcho2` (IPv6) via FFI seguro e tipado (`iphlpapi.dll`). Funciona para usuários padrão sem elevação UAC ("Executar como Administrador").
  - **Linux / Unix**: Suporte nativo via sockets ICMP datagram/raw.
- **Saída 100% em Português do Brasil (pt-BR)**:
  - Mensagens, argumentos, erros e estatísticas traduzidos e adaptados com clareza.
- **Saída Colorida Inteligente**:
  - Destaque dinâmico de latência (RTT):
    - 🟢 **Verde**: `< 50ms` (conexão ultrarrápida)
    - 🟡 **Amarelo**: `50ms - 150ms` (latência moderada)
    - 🔴 **Vermelho**: `> 150ms` ou perda de pacotes (latência alta)
  - Estatísticas finais estilizadas com taxas de perda coloridas.
  - Habilitação automática de sequências ANSI no Windows Terminal, PowerShell e CMD (`ENABLE_VIRTUAL_TERMINAL_PROCESSING`).
  - Respeita o padrão `NO_COLOR` e desativa escape codes automaticamente quando redirecionado para pipes/arquivos.
- **Salva Automaticamente na Raiz do Projeto**:
  - Os scripts de compilação `build.ps1` (PowerShell) e `build.cmd` (CMD) compilam a versão otimizada e depositam imediatamente `rping.exe` e `ping.exe` na pasta raiz.
- **Hot-Path Zero-Allocation**:
  - Buffers de I/O reutilizados na memória heap/stack sem alocações intermediárias dentro do loop de ping.
  - Cálculos matemáticos precisos de desvio padrão (`mdev`), média, mínimo e máximo usando instruções FMA `mul_add`.
- **Asynchronous & Responsive**:
  - Orquestração não-bloqueante com `tokio` (interval pacing, resolução DNS e sinal Ctrl+C limpo que sempre imprime o resumo antes de sair).
- **Otimização Extrema de Binário**:
  - Release com `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` e runtime estático (`target-feature=+crt-static`).
  - Binário final autônomo com menos de **900 KB**.

---

## 📦 Compilação e Salvamento na Raiz

Para compilar e salvar os binários executáveis (`rping.exe` e `ping.exe`) diretamente na **pasta raiz**:

### No PowerShell:
```powershell
.\build.ps1
```

### No Prompt de Comando (CMD):
```cmd
build.cmd
```

Ou usando Cargo diretamente:
```powershell
cargo build --release
Copy-Item target\release\rping.exe .\rping.exe -Force
Copy-Item target\release\rping.exe .\ping.exe -Force
```

---

## 💻 Uso e Opções da CLI

```text
Uso: rping.exe [OPÇÕES] <DESTINO>

Argumentos:
  <DESTINO>                  Nome de host ou endereço IP de destino para pingar (ex: google.com, 1.1.1.1, ::1)

Opções:
  -c, --count <CONTAGEM>     Número de pacotes ECHO_REQUEST a enviar
  -i, --interval <SEGUNDOS>  Intervalo de espera em segundos entre o envio de cada pacote [padrão: 1.0]
  -s, --size <BYTES>         Número de bytes de dados da carga útil (payload) a enviar [padrão: 56]
  -t, --ttl <TTL>            Tempo de vida (TTL) do pacote IP em saltos (1-255) [padrão: 64]
  -W, --timeout <SEGUNDOS>   Tempo limite de espera por resposta em segundos [padrão: 1.0]
  -w, --deadline <SEGUNDOS>  Encerra após o tempo limite (deadline) em segundos, independentemente da contagem
  -4, --ipv4                 Força resolução de nomes para IPv4
  -6, --ipv6                 Força resolução de nomes para IPv6
  -a, --audible              Saída audível (emite sinal sonoro / beep a cada resposta recebida)
  -q, --quiet                Modo silencioso; exibe apenas o cabeçalho inicial e o resumo final
  -v, --verbose              Modo detalhado com registros de diagnóstico estruturados
      --no-color             Desativa a saída colorida no terminal
  -h, --help                 Exibe ajuda
  -V, --version              Exibe versão
```

### Exemplos Práticos

```powershell
# Executar a partir da raiz (4 pacotes)
.\rping.exe -c 4 google.com

# Ou usando o alias ping.exe na raiz
.\ping.exe -c 4 127.0.0.1

# Ping rápido a cada 200ms com timeout curto
.\rping.exe -c 10 -i 0.2 -W 0.5 1.1.1.1

# Ping contínuo (Ctrl+C encerra e mostra estatísticas)
.\rping.exe 8.8.8.8

# Forçar resolução IPv6
.\rping.exe -6 google.com
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
