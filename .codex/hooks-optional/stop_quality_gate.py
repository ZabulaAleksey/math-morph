import json
import subprocess
import sys
from pathlib import Path

if hasattr(sys.stdin, "reconfigure"):
    sys.stdin.reconfigure(encoding="utf-8")
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

try:
    data = json.load(sys.stdin)
except Exception:
    data = {}

# Never create an infinite continuation loop.
if data.get("stop_hook_active"):
    print(json.dumps({"continue": True}))
    raise SystemExit(0)

cwd = data.get("cwd") or "."
try:
    root = subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], cwd=cwd,
        stderr=subprocess.DEVNULL, text=True
    ).strip()
    changed = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=root,
        stderr=subprocess.DEVNULL, text=True
    ).splitlines()
except Exception:
    print(json.dumps({"continue": True}))
    raise SystemExit(0)

if not changed:
    print(json.dumps({"continue": True}))
    raise SystemExit(0)

paths = []
for line in changed:
    if len(line) >= 4:
        paths.append(line[3:].strip().replace("\\", "/"))

code_prefixes = ("apps/", "services/", "crates/", "packages/", "sdk/", "infra/")
code_changed = any(p.startswith(code_prefixes) for p in paths)
progress_changed = any(p == "docs/PROGRESS.md" for p in paths)

if code_changed and not progress_changed:
    print(json.dumps({
        "decision": "block",
        "reason": (
            "Before finishing this turn, do one concise completion pass: run/report the relevant tests or checks for the changed code, "
            "update docs/PROGRESS.md if this is a completed meaningful stage, and mention any known limitations. Do not broaden scope."
        )
    }, ensure_ascii=False))
else:
    print(json.dumps({"continue": True}))
