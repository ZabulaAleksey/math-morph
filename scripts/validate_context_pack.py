from pathlib import Path
import json
import py_compile
import tomllib

root = Path(__file__).resolve().parents[1]

assert not (root / "KARKAS").exists(), "Legacy KARKAS scaffold must not coexist with the canonical context pack"

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

# Canonical context documents and the distribution manifest must remain navigable.
for required in (
    "docs/SPECIFICATION.md",
    "docs/ARCHITECTURE.md",
    "docs/DECISIONS.md",
    "docs/PROGRESS.md",
    "docs/TRACEABILITY.md",
):
    assert (root / required).exists(), f"Missing canonical context document: {required}"

manifest = json.loads((root / "PACK_MANIFEST.json").read_text(encoding="utf-8"))
manifest_paths = [entry["path"] for entry in manifest["files"]]
assert len(manifest_paths) == len(set(manifest_paths)), "PACK_MANIFEST.json contains duplicate paths"
assert all(not path.startswith("KARKAS/") for path in manifest_paths), "Manifest must not reference legacy KARKAS paths"
for path in manifest_paths:
    assert (root / path).is_file(), f"Manifest path is missing: {path}"

# Human-readable inventories must match the active/optional filesystem split.
subagents_doc = (root / "docs" / "SUBAGENTS.md").read_text(encoding="utf-8")
for name in expected:
    assert name in subagents_doc, f"Active agent missing from docs/SUBAGENTS.md: {name}"

skills_doc = (root / "docs" / "SKILLS.md").read_text(encoding="utf-8")
for skill_dir in (root / ".agents" / "skills").iterdir():
    if skill_dir.is_dir():
        assert skill_dir.name in skills_doc, f"Active Skill missing from docs/SKILLS.md: {skill_dir.name}"

hooks_doc = (root / "docs" / "HOOKS.md").read_text(encoding="utf-8")
assert ".codex/hooks-optional/" in hooks_doc, "Hooks documentation must point to the optional hook directory"

mcp_doc = (root / "docs" / "MCP.md").read_text(encoding="utf-8")
assert ".codex/mcp.optional.toml" in mcp_doc, "MCP documentation must identify the inactive optional template"

print("context pack: OK (AI Dev Team overlay mode)")
