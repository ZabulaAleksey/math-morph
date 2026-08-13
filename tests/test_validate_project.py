from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "scripts"))

from validate_project import validate_project  # noqa: E402


class ProjectValidatorTests(unittest.TestCase):
    def test_current_repository_is_valid(self) -> None:
        self.assertEqual(validate_project(PROJECT_ROOT), [])

    def test_legacy_progress_document_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            (root / "docs").mkdir()
            (root / "docs/PROGRESS.md").write_text("# Legacy\n", encoding="utf-8")

            errors = validate_project(root)

        self.assertIn("legacy path must be removed: docs/PROGRESS.md", errors)

    def test_obsolete_optional_agent_directory_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            obsolete = root / ".codex/agents-optional/mathcad-qa-fallback.toml"
            obsolete.parent.mkdir(parents=True)
            obsolete.write_text('name = "mathcad_qa_fallback"\n', encoding="utf-8")

            errors = validate_project(root)

        self.assertIn("legacy path must be removed: .codex/agents-optional", errors)

    def test_each_context_contract_rejects_empty_content(self) -> None:
        contracts = (
            "AGENTS.md",
            "docs/DESIGN.md",
            "crates/mathcad-parser/AGENTS.md",
            "crates/math-engine/AGENTS.md",
            "crates/exporter-docx/AGENTS.md",
            "apps/web/AGENTS.md",
            "services/api/AGENTS.md",
            "tests/AGENTS.md",
        )

        for relative in contracts:
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as temporary_directory:
                root = Path(temporary_directory)
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("\n", encoding="utf-8")

                errors = validate_project(root)

                self.assertIn(f"context contract must not be empty: {relative}", errors)

    def test_context_contract_rejects_missing_required_marker(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            target = root / "services/api/AGENTS.md"
            target.parent.mkdir(parents=True)
            target.write_text("# Правила API\n", encoding="utf-8")

            errors = validate_project(root)

        self.assertIn("context contract services/api/AGENTS.md is missing marker: /api/v1", errors)

    def test_canonical_document_rejects_empty_content(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            target = root / "docs/SECURITY.md"
            target.parent.mkdir(parents=True)
            target.write_text("\n", encoding="utf-8")

            errors = validate_project(root)

        self.assertIn("canonical document must not be empty: docs/SECURITY.md", errors)


if __name__ == "__main__":
    unittest.main()
