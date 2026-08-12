from pathlib import Path
import py_compile
import tomllib

root = Path(__file__).resolve().parents[1]

# Active config must be valid and intentionally minimal in AI Dev Team overlay mode.
with (root / ".codex" / "config.toml").open("rb") as f:
    active_config = tomllib.load(f)

assert "mcp_servers" not in active_config, "Active project config must not duplicate global AI Dev Team MCP servers"
assert "hooks" not in active_config, "Active project config must not register hooks by default"
assert "agents" not in active_config, "Do not override global AI Dev Team agent/concurrency settings in project config"

# Optional snippets must also parse, even though they are not auto-loaded.
for optional in (root / ".codex" / "hooks.optional.toml", root / ".codex" / "mcp.optional.toml"):
    with optional.open("rb") as f:
        tomllib.load(f)

# Only project-domain specialists are active.
active_agent_names = set()
for p in (root / ".codex" / "agents").glob("*.toml"):
    with p.open("rb") as f:
        data = tomllib.load(f)
    for required in ("name", "description", "developer_instructions"):
        assert data.get(required), f"{p}: missing {required}"
    name = data["name"]
    active_agent_names.add(name)
    assert name.startswith("mathcad_"), f"{p}: active project agent must be Mathcad-specific"

expected = {
    "mathcad_format_forensics",
    "mathcad_parser_engineer",
    "mathcad_math_semantics",
    "mathcad_word_openxml",
}
assert active_agent_names == expected, f"Unexpected active agents: {active_agent_names ^ expected}"

# Optional fallback agents are valid but inactive because they live outside .codex/agents/.
for p in (root / ".codex" / "agents-optional").glob("*.toml"):
    with p.open("rb") as f:
        data = tomllib.load(f)
    for required in ("name", "description", "developer_instructions"):
        assert data.get(required), f"{p}: missing {required}"

# Optional hook scripts must compile.
for p in (root / ".codex" / "hooks-optional").glob("*.py"):
    py_compile.compile(str(p), doraise=True)

# Design remains intentionally owner-supplied.
assert (root / "docs" / "DESIGN.md").read_bytes() == b"", "DESIGN.md must remain empty until owner supplies design"

# Compatibility doc must be present and referenced from root instructions.
compat = root / "docs" / "AI_DEV_TEAM_COMPATIBILITY.md"
assert compat.exists(), "Missing AI Dev Team compatibility policy"
assert "AI_DEV_TEAM_COMPATIBILITY.md" in (root / "AGENTS.md").read_text(encoding="utf-8")

print("context pack: OK (AI Dev Team overlay mode)")
