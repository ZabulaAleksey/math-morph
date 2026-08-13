from __future__ import annotations

import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "scripts"))

from validate_fixtures import validate_fixtures  # noqa: E402


class FixtureValidatorTests(unittest.TestCase):
    def _copy_fixture_tree(self, root: Path) -> Path:
        target = root / "tests/fixtures"
        shutil.copytree(PROJECT_ROOT / "tests/fixtures", target)
        return target

    def test_current_fixture_tree_is_valid(self) -> None:
        self.assertEqual(validate_fixtures(PROJECT_ROOT), [])

    def test_unlisted_fixture_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            fixture_root = self._copy_fixture_tree(root)
            (fixture_root / "corrupted/unlisted.xmcd").write_text("<broken>", encoding="utf-8")

            errors = validate_fixtures(root)

        self.assertIn("fixture file is missing from manifest: corrupted/unlisted.xmcd", errors)

    def test_parent_traversal_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            fixture_root = self._copy_fixture_tree(root)
            manifest_path = fixture_root / "manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["fixtures"][0]["path"] = "xmcd/../outside.xmcd"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            errors = validate_fixtures(root)

        self.assertIn("unsafe or unclassified fixture path: xmcd/../outside.xmcd", errors)

    def test_unknown_manifest_field_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            fixture_root = self._copy_fixture_tree(root)
            manifest_path = fixture_root / "manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["fixtures"][0]["surprise"] = True
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            errors = validate_fixtures(root)

        self.assertIn("fixture[0] has unexpected fields: ['surprise']", errors)

    def test_mismatched_extension_can_be_declared_for_detector_regression(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            fixture_root = self._copy_fixture_tree(root)
            source = fixture_root / "xmcd/minimal-worksheet30.xmcd"
            target = fixture_root / "xmcd/minimal-worksheet30.mcdx"
            source.rename(target)
            manifest_path = fixture_root / "manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["fixtures"][0]["path"] = "xmcd/minimal-worksheet30.mcdx"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            errors = validate_fixtures(root)

        self.assertEqual(errors, [])

    def test_non_scalar_enums_are_reported_without_exception(self) -> None:
        for field in ("format", "expected_status"):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temporary_directory:
                root = Path(temporary_directory)
                fixture_root = self._copy_fixture_tree(root)
                manifest_path = fixture_root / "manifest.json"
                manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
                manifest["fixtures"][0][field] = []
                manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

                errors = validate_fixtures(root)

                self.assertTrue(any("unsupported" in error for error in errors))

    def test_nested_readme_must_be_listed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            fixture_root = self._copy_fixture_tree(root)
            (fixture_root / "security/README.md").write_text("unlisted", encoding="utf-8")

            errors = validate_fixtures(root)

        self.assertIn("fixture file is missing from manifest: security/README.md", errors)


if __name__ == "__main__":
    unittest.main()
