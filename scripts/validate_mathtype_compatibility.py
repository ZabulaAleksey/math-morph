from __future__ import annotations

from collections import Counter
from datetime import date as calendar_date
import hashlib
from pathlib import Path
import re
import sys


DOCUMENT = "docs/MATHTYPE_COMPATIBILITY.md"
GOLDEN_DIRECTORY = "crates/exporter-mathml/tests/golden"
STATIC_STATUSES = {"DOCUMENTED", "PARTIAL", "NOT_DOCUMENTED"}
LIVE_STATUSES = {"PASS", "FAIL", "NOT_RUN"}
REQUIRED_MARKERS = (
    "## 3. Источники и область утверждений",
    "## 5. Матрица",
    "## 6. Воспроизводимый live smoke",
    "## 7. Versioned live evidence records",
    "EquationBackend::MathType",
)
OVERALL_STATUS = re.compile(r"\*\*Общий статус:\*\* `([A-Z_]+)`")
EVIDENCE_SURFACES = ("WEB_IMPORT", "DESKTOP_IMPORT", "EDIT_ROUND_TRIP")
WEB_PRODUCT = re.compile(r"MathType Web \d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?\Z")
DESKTOP_PRODUCT = re.compile(r"MathType 7 \d+\.\d+(?:\.\d+){0,2}\Z")
WEB_PLATFORM = re.compile(
    r"(?:Windows|macOS|Linux) [^;/]+ / (?:Chrome|Edge|Firefox|Safari) \d+(?:\.\d+)+\Z"
)
DESKTOP_PLATFORM = re.compile(r"Windows [^;/]+ / Word \d+(?:\.\d+)+\Z")
EVIDENCE_REFERENCE = re.compile(
    r"(tests/evidence/mathtype/[0-9A-Za-z._/-]+)#sha256=([0-9a-f]{64})\Z"
)
SURFACE_METHODS = {
    "WEB_IMPORT": {"WEB_SET_MATHML"},
    "DESKTOP_IMPORT": {"SDK_TEXT_FILE", "OLE_MATHML_CLIPBOARD"},
    "EDIT_ROUND_TRIP": {
        "WEB_EDIT_SAVE_REOPEN_EXPORT",
        "DESKTOP_EDIT_SAVE_REOPEN_EXPORT",
    },
}


def validate_compatibility_document(
    contents: str, golden_names: set[str], evidence_root: Path | None = None
) -> list[str]:
    errors: list[str] = []
    for marker in REQUIRED_MARKERS:
        if marker not in contents:
            errors.append(f"compatibility document is missing marker: {marker}")

    rows: list[tuple[str, list[str]]] = []
    evidence_rows: list[tuple[str, list[str]]] = []
    for line in contents.splitlines():
        if not line.startswith("| `"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        case = cells[0].strip("`")
        if not case.endswith(".mathml"):
            continue
        if len(cells) == 6:
            rows.append((case, [cell.strip("`") for cell in cells[2:]]))
        elif len(cells) == 8:
            evidence_rows.append((case, [cell.strip("`") for cell in cells[1:]]))
        else:
            errors.append(f"invalid compatibility row: {line}")

    names = [case for case, _ in rows]
    counts = Counter(names)
    duplicates = sorted(name for name, count in counts.items() if count > 1)
    if duplicates:
        errors.append(f"duplicate compatibility cases: {duplicates}")

    actual = set(names)
    missing = sorted(golden_names - actual)
    unexpected = sorted(actual - golden_names)
    if missing:
        errors.append(f"missing compatibility cases: {missing}")
    if unexpected:
        errors.append(f"unexpected compatibility cases: {unexpected}")

    evidence_by_key: dict[tuple[str, str], list[str]] = {}
    for case, evidence in evidence_rows:
        surface, status, version, platform, date, method, artifact = evidence
        key = (case, surface)
        if key in evidence_by_key:
            errors.append(f"duplicate live evidence record: {case}/{surface}")
        evidence_by_key[key] = evidence
        if case not in golden_names:
            errors.append(f"live evidence references unknown case: {case}")
        if surface not in EVIDENCE_SURFACES:
            errors.append(f"unknown live evidence surface for {case}: {surface}")
        if status not in {"PASS", "FAIL"}:
            errors.append(f"invalid live evidence record status for {case}/{surface}: {status}")

        expected_methods = SURFACE_METHODS.get(surface, set())
        if method not in expected_methods:
            errors.append(f"invalid import method for {case}/{surface}: {method}")

        product_pattern = WEB_PRODUCT if surface == "WEB_IMPORT" else DESKTOP_PRODUCT
        platform_pattern = WEB_PLATFORM if surface == "WEB_IMPORT" else DESKTOP_PLATFORM
        if surface == "EDIT_ROUND_TRIP" and method.startswith("WEB_"):
            product_pattern = WEB_PRODUCT
            platform_pattern = WEB_PLATFORM
        if product_pattern.fullmatch(version) is None:
            errors.append(f"invalid product version for {case}/{surface}: {version}")
        if platform_pattern.fullmatch(platform) is None:
            errors.append(f"invalid platform for {case}/{surface}: {platform}")

        try:
            calendar_date.fromisoformat(date)
        except ValueError:
            errors.append(f"invalid evidence date for {case}/{surface}: {date}")

        artifact_match = EVIDENCE_REFERENCE.fullmatch(artifact)
        artifact_path = Path(artifact.split("#", 1)[0])
        if artifact_match is None or ".." in artifact_path.parts:
            errors.append(f"invalid evidence reference for {case}/{surface}: {artifact}")
        elif evidence_root is not None:
            relative_path, expected_hash = artifact_match.groups()
            evidence_path = evidence_root / relative_path
            if not evidence_path.is_file():
                errors.append(f"missing evidence artifact for {case}/{surface}: {relative_path}")
            else:
                actual_hash = hashlib.sha256(evidence_path.read_bytes()).hexdigest()
                if actual_hash != expected_hash:
                    errors.append(f"evidence hash mismatch for {case}/{surface}: {relative_path}")

    has_not_run = False
    has_fail = False
    expected_evidence_keys: set[tuple[str, str]] = set()
    for case, statuses in rows:
        static_status, *live_statuses = statuses
        if static_status not in STATIC_STATUSES:
            errors.append(f"unknown static coverage status for {case}: {static_status}")
        for surface, status in zip(EVIDENCE_SURFACES, live_statuses, strict=True):
            if status not in LIVE_STATUSES:
                errors.append(f"unknown live evidence status for {case}: {status}")
            has_not_run = has_not_run or status == "NOT_RUN"
            has_fail = has_fail or status == "FAIL"
            if status in {"PASS", "FAIL"}:
                key = (case, surface)
                expected_evidence_keys.add(key)
                record = evidence_by_key.get(key)
                if record is None:
                    errors.append(f"missing live evidence record: {case}/{surface}")
                elif record[1] != status:
                    errors.append(
                        f"matrix/evidence status mismatch for {case}/{surface}: "
                        f"{status} != {record[1]}"
                    )

    dangling_evidence = sorted(set(evidence_by_key) - expected_evidence_keys)
    for case, surface in dangling_evidence:
        errors.append(f"live evidence has no matrix result: {case}/{surface}")

    overall_statuses = OVERALL_STATUS.findall(contents)
    if not overall_statuses:
        errors.append("compatibility document is missing overall status")
    elif len(overall_statuses) != 1:
        errors.append(f"compatibility document must contain exactly one overall status: {overall_statuses}")
    elif has_not_run and overall_statuses[0] != "UNVERIFIED":
        errors.append("overall status must be UNVERIFIED while any live evidence is NOT_RUN")
    elif has_fail and overall_statuses[0] != "INCOMPATIBLE":
        errors.append("overall status must be INCOMPATIBLE while any live evidence is FAIL")
    elif rows and not has_not_run and not has_fail and overall_statuses[0] != "VERIFIED":
        errors.append("overall status must be VERIFIED when every live evidence status is PASS")

    return errors


def validate_mathtype_compatibility(root: Path) -> list[str]:
    document = root / DOCUMENT
    golden_directory = root / GOLDEN_DIRECTORY
    if not document.is_file():
        return [f"missing compatibility document: {DOCUMENT}"]
    if not golden_directory.is_dir():
        return [f"missing MathML golden directory: {GOLDEN_DIRECTORY}"]

    contents = document.read_text(encoding="utf-8")
    golden_names = {path.name for path in golden_directory.glob("*.mathml")}
    return validate_compatibility_document(contents, golden_names, root)


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    errors = validate_mathtype_compatibility(root)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("math-morph MathType compatibility: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
