from __future__ import annotations

import hashlib
import sys
import tempfile
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT / "scripts"))

from validate_mathtype_compatibility import (  # noqa: E402
    validate_compatibility_document,
    validate_mathtype_compatibility,
)


class MathTypeCompatibilityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.document = (
            PROJECT_ROOT / "docs/MATHTYPE_COMPATIBILITY.md"
        ).read_text(encoding="utf-8")
        self.golden_names = {
            path.name
            for path in (PROJECT_ROOT / "crates/exporter-mathml/tests/golden").glob(
                "*.mathml"
            )
        }

    def test_current_compatibility_document_matches_golden_inventory(self) -> None:
        self.assertEqual(validate_mathtype_compatibility(PROJECT_ROOT), [])

    def test_missing_and_duplicate_cases_are_rejected(self) -> None:
        row = (
            "| `add.mathml` | `mrow(mi, mo(+), mn)` | `DOCUMENTED` | "
            "`NOT_RUN` | `NOT_RUN` | `NOT_RUN` |"
        )
        missing = self.document.replace(f"{row}\n", "", 1)
        duplicate = self.document.replace(row, f"{row}\n{row}", 1)

        self.assertIn(
            "missing compatibility cases: ['add.mathml']",
            validate_compatibility_document(missing, self.golden_names),
        )
        self.assertIn(
            "duplicate compatibility cases: ['add.mathml']",
            validate_compatibility_document(duplicate, self.golden_names),
        )

    def test_unknown_status_is_rejected(self) -> None:
        invalid = self.document.replace(
            "| `add.mathml` | `mrow(mi, mo(+), mn)` | `DOCUMENTED` |",
            "| `add.mathml` | `mrow(mi, mo(+), mn)` | `ASSUMED` |",
            1,
        )

        errors = validate_compatibility_document(invalid, self.golden_names)

        self.assertTrue(
            any(error.startswith("unknown static coverage status") for error in errors)
        )

    def test_not_run_cannot_be_reported_as_verified(self) -> None:
        invalid = self.document.replace(
            "**Общий статус:** `UNVERIFIED`", "**Общий статус:** `VERIFIED`", 1
        )

        self.assertIn(
            "overall status must be UNVERIFIED while any live evidence is NOT_RUN",
            validate_compatibility_document(invalid, self.golden_names),
        )

    def test_pass_without_versioned_evidence_is_rejected(self) -> None:
        invalid = self.document.replace("`NOT_RUN`", "`PASS`").replace(
            "**Общий статус:** `UNVERIFIED`", "**Общий статус:** `VERIFIED`", 1
        )

        errors = validate_compatibility_document(invalid, self.golden_names)

        self.assertIn(
            "missing live evidence record: add.mathml/WEB_IMPORT",
            errors,
        )

    def test_fail_cannot_be_reported_as_verified(self) -> None:
        invalid = self.document.replace("`NOT_RUN`", "`FAIL`").replace(
            "**Общий статус:** `UNVERIFIED`", "**Общий статус:** `VERIFIED`", 1
        )

        errors = validate_compatibility_document(invalid, self.golden_names)

        self.assertIn(
            "overall status must be INCOMPATIBLE while any live evidence is FAIL",
            errors,
        )

    def _document_with_web_pass(self, artifact_hash: str) -> str:
        document = self.document.replace(
            "| `add.mathml` | `mrow(mi, mo(+), mn)` | `DOCUMENTED` | `NOT_RUN` |",
            "| `add.mathml` | `mrow(mi, mo(+), mn)` | `DOCUMENTED` | `PASS` |",
            1,
        )
        evidence_row = (
            "| `add.mathml` | `WEB_IMPORT` | `PASS` | `MathType Web 7.25.5` | "
            "`Windows 11 24H2 / Chrome 140.0.0.0` | `2026-08-20` | "
            "`WEB_SET_MATHML` | "
            f"`tests/evidence/mathtype/add-web.json#sha256={artifact_hash}` |"
        )
        separator = "|---|---|---|---|---|---|---|---|"
        return document.replace(separator, f"{separator}\n{evidence_row}", 1)

    def test_valid_versioned_evidence_record_is_accepted(self) -> None:
        artifact = b'{"case":"add.mathml","result":"PASS"}\n'
        artifact_hash = hashlib.sha256(artifact).hexdigest()
        document = self._document_with_web_pass(artifact_hash)

        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            evidence = root / "tests/evidence/mathtype/add-web.json"
            evidence.parent.mkdir(parents=True)
            evidence.write_bytes(artifact)

            self.assertEqual(
                validate_compatibility_document(document, self.golden_names, root), []
            )

    def test_invalid_provenance_fields_and_artifact_are_rejected(self) -> None:
        document = self._document_with_web_pass("0" * 64)
        invalid = (
            document.replace("MathType Web 7.25.5", "unknown", 1)
            .replace("Windows 11 24H2 / Chrome 140.0.0.0", "unknown", 1)
            .replace(
                "| `2026-08-20` | `WEB_SET_MATHML` |",
                "| `2026-99-99` | `unknown` |",
                1,
            )
        )

        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            evidence = root / "tests/evidence/mathtype/add-web.json"
            evidence.parent.mkdir(parents=True)
            evidence.write_bytes(b"different artifact\n")
            errors = validate_compatibility_document(
                invalid, self.golden_names, root
            )

        self.assertTrue(any(error.startswith("invalid product version") for error in errors))
        self.assertTrue(any(error.startswith("invalid platform") for error in errors))
        self.assertTrue(any(error.startswith("invalid evidence date") for error in errors))
        self.assertTrue(any(error.startswith("invalid import method") for error in errors))
        self.assertTrue(any(error.startswith("evidence hash mismatch") for error in errors))

    def test_duplicate_evidence_and_overall_status_are_rejected(self) -> None:
        document = self._document_with_web_pass("0" * 64)
        evidence_row = next(
            line
            for line in document.splitlines()
            if line.startswith("| `add.mathml` | `WEB_IMPORT`")
        )
        duplicate = document.replace(evidence_row, f"{evidence_row}\n{evidence_row}", 1)
        duplicate += "\n**Общий статус:** `VERIFIED`\n"

        errors = validate_compatibility_document(duplicate, self.golden_names)

        self.assertIn("duplicate live evidence record: add.mathml/WEB_IMPORT", errors)
        self.assertTrue(
            any(
                error.startswith(
                    "compatibility document must contain exactly one overall status"
                )
                for error in errors
            )
        )


if __name__ == "__main__":
    unittest.main()
