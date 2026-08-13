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


if __name__ == "__main__":
    unittest.main()
