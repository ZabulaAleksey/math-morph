import json
import subprocess
import sys
from pathlib import Path

MAX_PROGRESS = 900
MAX_CONTEXT = 1800

def read_input():
    try:
        return json.load(sys.stdin)
    except Exception:
        return {}

def git_root(cwd: str) -> Path:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=cwd or None,
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        return Path(out)
    except Exception:
        return Path(cwd or ".").resolve()

data = read_input()
root = git_root(data.get("cwd", "."))
progress = root / "docs" / "PROGRESS.md"
progress_text = ""
if progress.exists():
    try:
        raw = progress.read_text(encoding="utf-8", errors="replace")
        # Prefer current section and cap aggressively.
        idx = raw.find("## Current")
        if idx >= 0:
            raw = raw[idx:]
        progress_text = raw[:MAX_PROGRESS].strip()
    except Exception:
        pass

context = (
    "Use progressive context loading: root AGENTS first; read only domain docs/local AGENTS needed for the current task. "
    "Do not bulk-read ROADMAP/PROMPTS. Security-relevant work must consult docs/SECURITY.md; UI work consults DESIGN.md if non-empty. "
    "Use subagents only when independent analysis justifies them."
)
if progress_text:
    context += "\nCurrent progress excerpt:\n" + progress_text
context = context[:MAX_CONTEXT]

print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": context,
    }
}, ensure_ascii=False))
