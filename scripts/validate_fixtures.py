from __future__ import annotations

import json
import re
import sys
from pathlib import Path, PurePosixPath


TAXONOMY = frozenset(
    {
        "xmcd",
        "mcdx",
        "formulas",
        "complex",
        "plots",
        "diagrams",
        "mixed",
        "corrupted",
        "security",
        "compatibility",
    }
)
FORMATS = frozenset({"xmcd", "mcdx", "unknown"})
EXPECTED_STATUSES = frozenset({"accepted", "rejected", "unsupported"})
ROOT_FIELDS = frozenset({"schema_version", "fixtures"})
FIXTURE_FIELDS = frozenset(
    {"id", "path", "format", "version", "features", "expected_status"}
)
ID_PATTERN = re.compile(r"^[a-z0-9][a-z0-9-]*$")
MAX_MANIFEST_BYTES = 1024 * 1024


def _load_manifest(path: Path, errors: list[str]) -> object | None:
    try:
        if path.stat().st_size > MAX_MANIFEST_BYTES:
            errors.append("manifest exceeds 1 MiB")
            return None
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        errors.append(f"cannot read manifest: {exc}")
    except json.JSONDecodeError as exc:
        errors.append(f"invalid fixture manifest JSON: line {exc.lineno}, column {exc.colno}")
    return None


def _validate_relative_path(raw_path: str, errors: list[str]) -> PurePosixPath | None:
    if "\\" in raw_path:
        errors.append(f"fixture path must use forward slashes: {raw_path}")
        return None
    parts = raw_path.split("/")
    candidate = PurePosixPath(raw_path)
    if (
        not raw_path
        or candidate.is_absolute()
        or any(part in {"", ".", ".."} for part in parts)
        or parts[0] not in TAXONOMY
    ):
        errors.append(f"unsafe or unclassified fixture path: {raw_path}")
        return None
    return candidate


def validate_fixtures(project_root: Path) -> list[str]:
    fixture_root = project_root.resolve() / "tests/fixtures"
    manifest_path = fixture_root / "manifest.json"
    errors: list[str] = []

    for category in sorted(TAXONOMY):
        if not (fixture_root / category).is_dir():
            errors.append(f"missing fixture category: {category}")
    if fixture_root.is_dir():
        actual_categories = {
            path.name
            for path in fixture_root.iterdir()
            if path.is_dir() and not path.name.startswith(".")
        }
        for category in sorted(actual_categories - TAXONOMY):
            errors.append(f"unknown fixture category: {category}")

    if not manifest_path.is_file():
        errors.append("missing fixture manifest: tests/fixtures/manifest.json")
        return errors

    manifest = _load_manifest(manifest_path, errors)
    if manifest is None:
        return errors
    if not isinstance(manifest, dict):
        errors.append("fixture manifest root must be an object")
        return errors

    root_fields = frozenset(manifest)
    if root_fields != ROOT_FIELDS:
        errors.append(f"unexpected fixture manifest fields: {sorted(root_fields ^ ROOT_FIELDS)}")
    if type(manifest.get("schema_version")) is not int or manifest.get("schema_version") != 1:
        errors.append("fixture manifest schema_version must equal 1")

    fixtures = manifest.get("fixtures")
    if not isinstance(fixtures, list):
        errors.append("fixture manifest fixtures must be an array")
        return errors

    fixture_ids: set[str] = set()
    fixture_paths: set[str] = set()
    fixture_root_resolved = fixture_root.resolve()

    for index, fixture in enumerate(fixtures):
        label = f"fixture[{index}]"
        if not isinstance(fixture, dict):
            errors.append(f"{label} must be an object")
            continue
        fields = frozenset(fixture)
        if fields != FIXTURE_FIELDS:
            errors.append(f"{label} has unexpected fields: {sorted(fields ^ FIXTURE_FIELDS)}")
            continue

        fixture_id = fixture["id"]
        if not isinstance(fixture_id, str) or not ID_PATTERN.fullmatch(fixture_id):
            errors.append(f"{label} has invalid id")
        elif fixture_id in fixture_ids:
            errors.append(f"duplicate fixture id: {fixture_id}")
        else:
            fixture_ids.add(fixture_id)

        raw_path = fixture["path"]
        if not isinstance(raw_path, str):
            errors.append(f"{label} path must be a string")
            candidate = None
        else:
            candidate = _validate_relative_path(raw_path, errors)
        if candidate is not None:
            if raw_path in fixture_paths:
                errors.append(f"duplicate fixture path: {raw_path}")
            else:
                fixture_paths.add(raw_path)
            target = fixture_root / Path(*candidate.parts)
            try:
                resolved = target.resolve()
                resolved.relative_to(fixture_root_resolved)
            except (OSError, ValueError):
                errors.append(f"fixture path escapes fixture root: {raw_path}")
            else:
                if not target.is_file():
                    errors.append(f"fixture file does not exist: {raw_path}")

        input_format = fixture["format"]
        if not isinstance(input_format, str) or input_format not in FORMATS:
            errors.append(f"{label} has unsupported format: {input_format}")

        version = fixture["version"]
        if not isinstance(version, str) or not version.strip():
            errors.append(f"{label} version must be a non-empty string")

        features = fixture["features"]
        if (
            not isinstance(features, list)
            or any(not isinstance(feature, str) or not feature for feature in features)
            or len(features) != len(set(features))
            or features != sorted(features)
        ):
            errors.append(f"{label} features must be unique sorted non-empty strings")

        expected_status = fixture["expected_status"]
        if not isinstance(expected_status, str) or expected_status not in EXPECTED_STATUSES:
            errors.append(f"{label} has unsupported expected_status")

    actual_paths = set()
    for path in fixture_root.rglob("*"):
        if not path.is_file() or path.name.startswith("."):
            continue
        relative = path.relative_to(fixture_root).as_posix()
        if relative in {"manifest.json", "README.md"}:
            continue
        actual_paths.add(relative)
    for path in sorted(actual_paths - fixture_paths):
        errors.append(f"fixture file is missing from manifest: {path}")

    return errors


def main() -> int:
    project_root = Path(__file__).resolve().parents[1]
    errors = validate_fixtures(project_root)
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("math-morph fixtures: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
