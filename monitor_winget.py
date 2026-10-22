#!/usr/bin/env python3
"""
Monitor automatizado da esteira de validação do WinGet no GitHub (microsoft/winget-pkgs).

Acompanha a Pull Request em tempo real, alertando a cada avanço com:
- Atualização em terminal colorido
- Alertas sonoros (bip do sistema Windows)
- Notificações nativas do Windows (Toast Notifications)
"""

import argparse
import datetime
import json
import os
import subprocess
import sys
import time

try:
    import winsound
    HAS_WINSOUND = True
except ImportError:
    HAS_WINSOUND = False

# Cores ANSI para o terminal
RESET = "\033[0m"
BOLD = "\033[1m"
DIM = "\033[2m"
GREEN = "\033[1;32m"
YELLOW = "\033[1;33m"
RED = "\033[1;31m"
CYAN = "\033[1;36m"
MAGENTA = "\033[1;35m"
BLUE = "\033[1;34m"

def now_str() -> str:
    return datetime.datetime.now().strftime("%d/%m-%H:%M:%S")

def send_windows_toast(title: str, message: str) -> None:
    """Envia uma notificação Toast nativa no Windows usando PowerShell/WinRT."""
    if os.name != "nt":
        return
    try:
        # Escapar aspas
        safe_title = title.replace('"', '`"')
        safe_msg = message.replace('"', '`"').replace("\n", " ")
        ps_cmd = f"""
        [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
        [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
        $template = @"
<toast>
    <visual>
        <binding template="ToastGeneric">
            <text>{safe_title}</text>
            <text>{safe_msg}</text>
        </binding>
    </visual>
</toast>
"@
        $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
        $xml.LoadXml($template)
        $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
        [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("WinGet Monitor").Show($toast)
        """
        subprocess.run(
            ["powershell", "-NoProfile", "-Command", ps_cmd],
            capture_output=True,
            timeout=5,
            check=False,
        )
    except Exception:
        pass

def play_alert(sound_type: str = "step") -> None:
    """Emite um alerta sonoro no Windows."""
    if not HAS_WINSOUND:
        return
    try:
        if sound_type == "success":
            winsound.MessageBeep(winsound.MB_ICONASTERISK)
            time.sleep(0.15)
            winsound.MessageBeep(winsound.MB_ICONASTERISK)
        elif sound_type == "error":
            winsound.MessageBeep(winsound.MB_ICONHAND)
        else:
            winsound.MessageBeep(winsound.MB_OK)
    except Exception:
        pass

def fetch_pr_data(pr_number: int, repo: str) -> dict:
    """Obtém os dados da PR diretamente via GitHub CLI (gh)."""
    cmd = [
        "gh",
        "pr",
        "view",
        str(pr_number),
        "--repo",
        repo,
        "--json",
        "title,url,state,statusCheckRollup,comments,mergeable",
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(res.stdout)

def format_status(status: str, conclusion: str) -> str:
    status_u = (status or "").upper()
    conc_u = (conclusion or "").upper()
    if conc_u == "SUCCESS":
        return f"{GREEN}✔ Concluído (Sucesso){RESET}"
    if conc_u in ("FAILURE", "TIMED_OUT"):
        return f"{RED}✖ Falhou ({conc_u}){RESET}"
    if conc_u == "SKIPPED":
        return f"{DIM}↷ Ignorado (Skipped){RESET}"
    if status_u == "IN_PROGRESS":
        return f"{YELLOW}⏳ Em Execução...{RESET}"
    if status_u == "QUEUED":
        return f"{BLUE}⋯ Na Fila (Queued){RESET}"
    return f"{DIM}{status_u} / {conc_u}{RESET}"

def run_monitor(pr_number: int, repo: str, interval: int, enable_toast: bool, enable_sound: bool):
    print(f"{CYAN}{BOLD}========================================================================{RESET}")
    print(f"{CYAN}{BOLD}       MONITOR DA ESTEIRA WINGET (microsoft/winget-pkgs)               {RESET}")
    print(f"{CYAN}{BOLD}========================================================================{RESET}")
    print(f"{BOLD}Repositório:{RESET} {repo}")
    print(f"{BOLD}Pull Request:{RESET} #{pr_number}")
    print(f"{BOLD}Intervalo de Checagem:{RESET} a cada {interval} segundos")
    print(f"{BOLD}Notificações do Windows (Toast):{RESET} {'Ativadas' if enable_toast else 'Desativadas'}")
    print(f"{BOLD}Alertas Sonoros:{RESET} {'Ativados' if enable_sound else 'Desativados'}")
    print(f"{DIM}Iniciado em: {now_str()} | Pressione Ctrl+C para encerrar{RESET}\n")

    prev_state = None
    prev_checks = {}
    prev_comment_count = 0
    cycle = 0

    while True:
        cycle += 1
        timestamp = now_str()

        try:
            data = fetch_pr_data(pr_number, repo)
        except subprocess.CalledProcessError as err:
            print(f"[{timestamp}] {RED}Aviso:{RESET} Erro temporário ao consultar GitHub CLI: {err.stderr.strip() or err}")
            time.sleep(interval)
            continue
        except Exception as err:
            print(f"[{timestamp}] {RED}Erro inesperado:{RESET} {err}")
            time.sleep(interval)
            continue

        state = data.get("state", "UNKNOWN")
        title = data.get("title", "")
        url = data.get("url", "")
        checks = data.get("statusCheckRollup", []) or []
        comments = data.get("comments", []) or []

        # Mapa de checks por nome único (ou workflow + name)
        current_checks = {}
        for c in checks:
            name = c.get("name") or "Unnamed"
            wf = c.get("workflowName") or ""
            key = f"{wf}::{name}" if wf else name
            current_checks[key] = {
                "name": name,
                "workflow": wf,
                "status": c.get("status") or "",
                "conclusion": c.get("conclusion") or "",
                "detailsUrl": c.get("detailsUrl") or "",
            }

        # 1. Mudança de Estado da PR (Ex: Aberta -> Mesclada)
        if prev_state is not None and state != prev_state:
            msg = f"A Pull Request #{pr_number} mudou de estado para: {state}!"
            print(f"\n[{timestamp}] {MAGENTA}{BOLD}★ EVENTO PRINCIPAL: {msg}{RESET}")
            if enable_toast:
                send_windows_toast("WinGet PR Atualizada!", msg)
            if enable_sound:
                play_alert("success" if state == "MERGED" else "step")

            if state == "MERGED":
                print(f"\n{GREEN}{BOLD}🎉 PARABÉNS! A PR FOI MESCLADA PELA MICROSOFT!{RESET}")
                print(f"{GREEN}O pacote leandrop2103.rping agora está ativo no repositório oficial do WinGet!{RESET}")
                print(f"{CYAN}Para testar em qualquer máquina Windows:{RESET}")
                print(f"   winget install leandrop2103.rping\n")
                break

        # 2. Novos comentários postados (bots da Microsoft ou revisores)
        if len(comments) > prev_comment_count and cycle > 1:
            diff = len(comments) - prev_comment_count
            new_comments = comments[-diff:]
            for com in new_comments:
                author = com.get("author", {}).get("login", "Desconhecido")
                body = com.get("body", "").strip()
                preview = (body[:200] + "...") if len(body) > 200 else body
                print(f"\n[{timestamp}] {BLUE}{BOLD}💬 NOVO COMENTÁRIO de @{author}:{RESET}")
                print(f"{DIM}{preview}{RESET}\n")

                if enable_toast:
                    send_windows_toast(f"Novo comentário de @{author}", preview)
                if enable_sound:
                    play_alert("step")

        # 3. Mudanças nos Checks / Esteira de Validação
        if cycle > 1:
            for key, curr in current_checks.items():
                if key not in prev_checks:
                    # Nova etapa iniciada
                    msg = f"Nova etapa iniciada: {curr['name']} ({curr['status']})"
                    print(f"[{timestamp}] {CYAN}▶ {msg}{RESET}")
                    if enable_sound:
                        play_alert("step")
                else:
                    old = prev_checks[key]
                    if curr["status"] != old["status"] or curr["conclusion"] != old["conclusion"]:
                        status_str = format_status(curr["status"], curr["conclusion"])
                        msg = f"Etapa [{curr['name']}]: {status_str}"
                        print(f"[{timestamp}] {YELLOW}⚡ AVANÇO:{RESET} {msg}")

                        if curr["conclusion"] in ("SUCCESS", "FAILURE"):
                            if enable_toast:
                                send_windows_toast("Avanço na Validação WinGet", f"{curr['name']}: {curr['conclusion']}")
                            if enable_sound:
                                play_alert("success" if curr["conclusion"] == "SUCCESS" else "error")

        # Exibição periódica do resumo
        # Identificar principais checks relevantes (excluindo os que foram skipped em massa)
        relevant_checks = [
            c for c in current_checks.values()
            if c["conclusion"].upper() != "SKIPPED" or c["name"] in ("license/cla", "ValidationPipeline", "PublishPipeline")
        ]

        if cycle == 1 or cycle % 10 == 0:
            print(f"\n[{timestamp}] {BOLD}--- Status Atual da PR #{pr_number} ({state}) ---{RESET}")
            print(f"{BOLD}Título:{RESET} {title}")
            print(f"{BOLD}URL:{RESET} {url}")
            print(f"{BOLD}Etapas em monitoramento ({len(relevant_checks)} ativas/relevantes de {len(current_checks)} totais):{RESET}")
            for c in relevant_checks:
                status_str = format_status(c["status"], c["conclusion"])
                print(f"  • {BOLD}{c['name']:<25}{RESET} -> {status_str}")
            print(f"{DIM}Aguardando {interval}s para próxima checagem...{RESET}\n")
        else:
            # Indicador de atividade na mesma linha
            sys.stdout.write(f"\r[{timestamp}] Ciclo #{cycle} | Estado: {state} | Checks: {len(current_checks)} | Aguardando {interval}s... ")
            sys.stdout.flush()

        prev_state = state
        prev_checks = current_checks
        prev_comment_count = len(comments)

        time.sleep(interval)

def main():
    parser = argparse.ArgumentParser(description="Monitor em tempo real da esteira WinGet da Microsoft")
    parser.add_argument("--pr", type=int, default=459533, help="Número da Pull Request (padrão: 459533)")
    parser.add_argument("--repo", type=str, default="microsoft/winget-pkgs", help="Repositório da PR")
    parser.add_argument("--interval", type=int, default=30, help="Intervalo em segundos entre verificações (padrão: 30)")
    parser.add_argument("--no-toast", action="store_true", help="Desativa notificações Toast do Windows")
    parser.add_argument("--no-sound", action="store_true", help="Desativa alertas sonoros")

    args = parser.parse_args()

    try:
        run_monitor(
            pr_number=args.pr,
            repo=args.repo,
            interval=args.interval,
            enable_toast=not args.no_toast,
            enable_sound=not args.no_sound,
        )
    except KeyboardInterrupt:
        print(f"\n\n{YELLOW}Monitoramento encerrado pelo usuário.{RESET}")

if __name__ == "__main__":
    main()
