import json
import re
import sys

if hasattr(sys.stdin, "reconfigure"):
    sys.stdin.reconfigure(encoding="utf-8")
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

def deny(reason: str):
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
    raise SystemExit(0)

try:
    data = json.load(sys.stdin)
except Exception:
    raise SystemExit(0)

name = str(data.get("tool_name", ""))
inputs = data.get("tool_input") or {}
command = str(inputs.get("command", ""))
low = command.lower()

# Obvious destructive repository/system commands. Guardrail only; CI/app controls remain authoritative.
destructive = [
    r"git\s+reset\s+--hard\b",
    r"git\s+clean\s+-[^\n]*[xX]",
    r"git\s+push\b[^\n]*(--force|-f\b)",
    r"rm\s+-rf\s+/(?:\s|$)",
    r"format\s+[a-z]:",
    r"diskpart\b",
]
for pat in destructive:
    if re.search(pat, low):
        deny("Blocked by repository guardrail: destructive command requires an explicit manual decision outside the default agent flow.")

# Prevent direct reads of common secret/private-key files through shell commands.
secret_read = [
    r"\b(cat|type|get-content|gc)\b[^\n]*(?:^|[\\/\s])\.env(?:\s|$)",
    r"\b(cat|type|get-content|gc)\b[^\n]*(id_rsa|id_ed25519|\.pem|\.p12|\.pfx)(?:\s|$)",
]
for pat in secret_read:
    if re.search(pat, low):
        deny("Blocked by repository guardrail: do not read raw secret/private-key files into model/tool context. Use environment/secret-manager metadata instead.")

if name == "apply_patch":
    # Stop accidental insertion of literal private keys or tracked .env secret files.
    if "begin private key" in low or "begin rsa private key" in low:
        deny("Blocked: patch appears to contain private-key material.")
    if re.search(r"\+\+\+\s+b/\.env(?:\s|$)", command, flags=re.IGNORECASE):
        deny("Blocked: do not commit a tracked .env file. Use .env.example without secrets.")

# Exit 0 with no output = allow.
