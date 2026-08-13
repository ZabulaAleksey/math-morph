from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

from validate_fixtures import validate_fixtures

REQUIRED_FILES = (
    "AGENTS.md",
    "README.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "package.json",
    "pnpm-workspace.yaml",
    "pnpm-lock.yaml",
    "specs/README.md",
    "specs/system.spec.md",
    "docs/ARCHITECTURE.md",
    "docs/API.md",
    "docs/DEPENDENCIES.md",
    "docs/DECISIONS.md",
    "docs/DESIGN.md",
    "docs/FORMATS.md",
    "docs/AI_PLAN.md",
    "docs/AI_STATUS.md",
    "docs/PRIVACY.md",
    "docs/PROMPTS.md",
    "docs/ROADMAP.md",
    "docs/SECURITY.md",
    "docs/TECH_STACK.md",
    "docs/TESTING.md",
    "docs/TRACEABILITY.md",
    "docs/CONTEXT_COMPATIBILITY.md",
    "crates/mathcad-parser/AGENTS.md",
    "crates/math-engine/AGENTS.md",
    "crates/exporter-docx/AGENTS.md",
    "apps/web/AGENTS.md",
    "services/api/AGENTS.md",
    "tests/AGENTS.md",
    "services/api/pyproject.toml",
    "services/api/uv.lock",
    "apps/web/package.json",
    "scripts/validate_fixtures.py",
    "tests/fixtures/README.md",
    "tests/fixtures/manifest.json",
)

CONTEXT_CONTRACTS = {
    "AGENTS.md": ("Маршрутизация контекста", "Инварианты проекта"),
    "docs/DESIGN.md": (
        "Пользовательский интерфейс ещё не реализован",
        "визуальная система владельцем продукта не утверждена",
    ),
    "crates/mathcad-parser/AGENTS.md": ("Parser знает формат",),
    "crates/math-engine/AGENTS.md": ("Вычисление и представление разделены",),
    "crates/exporter-docx/AGENTS.md": ("редактируемыми структурами Word/OMML",),
    "apps/web/AGENTS.md": ("docs/DESIGN.md",),
    "services/api/AGENTS.md": ("/api/v1",),
    "tests/AGENTS.md": ("регрессионный тест или fixture",),
}

LEGACY_PATHS = (
    ".agents/skills-optional",
    ".codex/agents-optional",
    ".codex/hooks-optional",
    ".gitignore.context-pack.example",
    "PACK_MANIFEST.json",
    "README_CONTEXT_PACK.md",
    "docs/AI_DEV_TEAM_COMPATIBILITY.md",
    "docs/CONTEXT_POLICY.md",
    "docs/PROGRESS.md",
    "docs/SPECIFICATION.md",
    "docs/HOOKS.md",
    "docs/MCP.md",
    "docs/SKILLS.md",
    "docs/SUBAGENTS.md",
    ".codex/hooks.optional.toml",
    ".codex/mcp.optional.toml",
)

EXPECTED_AGENTS = {
    "mathcad_format_forensics",
    "mathcad_parser_engineer",
    "mathcad_math_semantics",
    "mathcad_word_openxml",
}

EXPECTED_SKILLS = {
    "mathcad-conversion-regression",
    "mathcad-format-forensics",
    "mathcad-security-overlay",
}

EXPECTED_CRATES = {
    "crates/mathcad-parser": "mathcad-parser",
    "crates/math-engine": "math-engine",
    "crates/exporter-docx": "exporter-docx",
}

CANONICAL_DOCUMENTS = (
    "README.md",
    "specs/README.md",
    "specs/system.spec.md",
    "docs/API.md",
    "docs/ARCHITECTURE.md",
    "docs/CONTEXT_COMPATIBILITY.md",
    "docs/DECISIONS.md",
    "docs/DEPENDENCIES.md",
    "docs/DESIGN.md",
    "docs/FORMATS.md",
    "docs/AI_PLAN.md",
    "docs/AI_STATUS.md",
    "docs/PRIVACY.md",
    "docs/PROMPTS.md",
    "docs/ROADMAP.md",
    "docs/SECURITY.md",
    "docs/TECH_STACK.md",
    "docs/TESTING.md",
    "docs/TRACEABILITY.md",
)

MARKDOWN_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
IGNORED_MARKDOWN_PARTS = {
    ".cache",
    ".git",
    ".next",
    ".pnpm-store",
    ".venv",
    "dist",
    "node_modules",
    "target",
}


def _load_toml(path: Path, errors: list[str]) -> dict:
    try:
        with path.open("rb") as stream:
            return tomllib.load(stream)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        errors.append(f"invalid TOML {path}: {exc}")
        return {}


def _load_json(path: Path, errors: list[str]) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(f"invalid JSON {path}: {exc}")
        return {}


def _validate_markdown_links(root: Path, errors: list[str]) -> None:
    for markdown in root.rglob("*.md"):
        if any(part in IGNORED_MARKDOWN_PARTS for part in markdown.relative_to(root).parts):
            continue
        try:
            contents = markdown.read_text(encoding="utf-8")
        except OSError as exc:
            errors.append(f"cannot read Markdown {markdown}: {exc}")
            continue
        for raw_target in MARKDOWN_LINK.findall(contents):
            target = raw_target.strip().strip("<>").split("#", 1)[0]
            if not target or "://" in target or target.startswith("mailto:"):
                continue
            resolved = (markdown.parent / target).resolve()
            if not resolved.exists():
                relative = markdown.relative_to(root)
                errors.append(f"broken Markdown link in {relative}: {raw_target}")


def validate_project(root: Path) -> list[str]:
    root = root.resolve()
    errors: list[str] = []

    for relative in REQUIRED_FILES:
        if not (root / relative).is_file():
            errors.append(f"missing required file: {relative}")

    for relative in LEGACY_PATHS:
        if (root / relative).exists():
            errors.append(f"legacy path must be removed: {relative}")

    for relative, markers in CONTEXT_CONTRACTS.items():
        path = root / relative
        if not path.is_file():
            continue
        contents = path.read_text(encoding="utf-8")
        if not contents.strip():
            errors.append(f"context contract must not be empty: {relative}")
            continue
        for marker in markers:
            if marker not in contents:
                errors.append(f"context contract {relative} is missing marker: {marker}")

    for relative in CANONICAL_DOCUMENTS:
        path = root / relative
        if path.is_file() and not path.read_text(encoding="utf-8").strip():
            errors.append(f"canonical document must not be empty: {relative}")

    spec = root / "specs/system.spec.md"
    if spec.is_file():
        spec_text = spec.read_text(encoding="utf-8")
        for requirement in ("NFR-FOUNDATION-001", "NFR-CONTEXT-001", "AC-FOUNDATION-001"):
            if requirement not in spec_text:
                errors.append(f"system SPEC is missing requirement: {requirement}")

    config = root / ".codex/config.toml"
    if config.is_file():
        active_config = _load_toml(config, errors)
        for forbidden in ("mcp_servers", "hooks", "agents"):
            if forbidden in active_config:
                errors.append(f"project config must not override global {forbidden}")

    actual_agents: set[str] = set()
    for path in (root / ".codex/agents").glob("*.toml"):
        data = _load_toml(path, errors)
        name = data.get("name")
        if name:
            actual_agents.add(name)
        for field in ("name", "description", "developer_instructions"):
            if not data.get(field):
                errors.append(f"{path.relative_to(root)} is missing {field}")
    if actual_agents != EXPECTED_AGENTS:
        errors.append(f"unexpected active agents: {sorted(actual_agents ^ EXPECTED_AGENTS)}")

    skills_root = root / ".agents/skills"
    actual_skills = {path.name for path in skills_root.iterdir() if path.is_dir()} if skills_root.is_dir() else set()
    if actual_skills != EXPECTED_SKILLS:
        errors.append(f"unexpected active Skills: {sorted(actual_skills ^ EXPECTED_SKILLS)}")

    cargo_path = root / "Cargo.toml"
    if cargo_path.is_file():
        cargo = _load_toml(cargo_path, errors)
        members = set(cargo.get("workspace", {}).get("members", []))
        if members != set(EXPECTED_CRATES):
            errors.append(f"unexpected Cargo workspace members: {sorted(members ^ set(EXPECTED_CRATES))}")
        for relative, expected_name in EXPECTED_CRATES.items():
            manifest_path = root / relative / "Cargo.toml"
            source_path = root / relative / "src/lib.rs"
            if not manifest_path.is_file() or not source_path.is_file():
                errors.append(f"incomplete Rust crate scaffold: {relative}")
                continue
            crate = _load_toml(manifest_path, errors)
            if crate.get("package", {}).get("name") != expected_name:
                errors.append(f"unexpected crate name in {relative}")
        if cargo.get("workspace", {}).get("package", {}).get("rust-version") != "1.88":
            errors.append("Cargo workspace rust-version must equal 1.88")

    toolchain_path = root / "rust-toolchain.toml"
    if toolchain_path.is_file():
        toolchain = _load_toml(toolchain_path, errors)
        if toolchain.get("toolchain", {}).get("channel") != "1.88.0":
            errors.append("rust-toolchain.toml channel must equal 1.88.0")

    cargo_lock_path = root / "Cargo.lock"
    if cargo_lock_path.is_file():
        cargo_lock = _load_toml(cargo_lock_path, errors)
        locked_packages = {package.get("name") for package in cargo_lock.get("package", [])}
        missing_workspace_crates = set(EXPECTED_CRATES.values()) - locked_packages
        if missing_workspace_crates:
            errors.append(
                f"Cargo.lock is missing workspace crates: {sorted(missing_workspace_crates)}"
            )

    root_package = root / "package.json"
    if root_package.is_file():
        package = _load_json(root_package, errors)
        if package.get("private") is not True:
            errors.append("root Node workspace must be private")

    web_package = root / "apps/web/package.json"
    if web_package.is_file():
        web = _load_json(web_package, errors)
        dependencies = web.get("dependencies", {})
        for dependency in ("next", "react", "react-dom"):
            if dependency not in dependencies:
                errors.append(f"web scaffold is missing dependency: {dependency}")
        if not (root / "apps/web/app/layout.tsx").is_file() or not (root / "apps/web/app/page.tsx").is_file():
            errors.append("web scaffold is missing required App Router files")

    api_manifest = root / "services/api/pyproject.toml"
    if api_manifest.is_file():
        api = _load_toml(api_manifest, errors)
        if api.get("project", {}).get("name") != "math-morph-api":
            errors.append("unexpected Python API package name")
        if not (root / "services/api/src/math_morph_api/__init__.py").is_file():
            errors.append("Python API package scaffold is incomplete")

    _validate_markdown_links(root, errors)
    errors.extend(f"fixtures: {error}" for error in validate_fixtures(root))
    return errors


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    errors = validate_project(root)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("math-morph project: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
