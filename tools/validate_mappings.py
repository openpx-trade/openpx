#!/usr/bin/env python3
"""Validate schema/mappings/*.yaml against canonical sources.

Each field source must declare exactly one of three entry types:
    type: direct     — value is taken from upstream at the given $ref.
    type: synthetic  — value is computed by OpenPX, not present upstream.
    type: omitted    — upstream does not expose this concept.

Checks per mapping file:
  1. Every unified field in openpx.schema.json is declared (per exchange).
  2. Every `type: direct` declares a `ref:`; refs that look like JSON pointers
     (`#/...`) must resolve in the cached upstream spec. Bare-name refs
     document a spec gap and are recorded for the daily refresh check.
  3. `fallback_ref` (if present) must resolve when it's a JSON pointer.

Exit code 0 on pass, 1 on validation failure.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from mapping_lib import load_schema_definitions, load_yaml, resolve_ref

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / "schema" / "openpx.schema.json"
MAPPINGS_DIR = ROOT / "schema" / "mappings"

VALID_TYPES = {"direct", "synthetic", "omitted"}


def load_openpx_unified(type_name: str) -> dict[str, Any]:
    defs = load_schema_definitions(SCHEMA)
    if type_name not in defs:
        sys.exit(f"openpx.schema.json has no definition for {type_name!r}")
    return defs[type_name]


def is_pointer(ref: str) -> bool:
    return isinstance(ref, str) and ref.startswith("#/")


class Validator:
    def __init__(self, mapping_path: Path) -> None:
        self.mapping_path = mapping_path
        self.mapping = load_yaml(mapping_path)
        self.errors: list[str] = []
        self.coverage: dict[str, dict[str, int]] = {}

    def err(self, msg: str) -> None:
        self.errors.append(f"[{self.mapping_path.name}] {msg}")

    def run(self) -> bool:
        type_name = self.mapping.get("unified_type")
        if not type_name:
            self.err("missing top-level `unified_type:` key")
            return False

        unified = load_openpx_unified(type_name)
        unified_props = unified.get("properties", {})

        upstream_paths = self.mapping.get("upstream_specs", {})
        if not upstream_paths:
            self.err("missing top-level `upstream_specs:` map")
            return False
        upstream_specs: dict[str, dict[str, Any]] = {}
        for ex, rel in upstream_paths.items():
            p = ROOT / rel
            if not p.is_file():
                self.err(f"upstream_specs.{ex} points at missing file: {rel}")
                continue
            try:
                upstream_specs[ex] = load_yaml(p)
            except Exception as e:
                self.err(f"upstream_specs.{ex} failed to parse: {e}")

        declared_fields = {f["name"] for f in self.mapping.get("fields", []) if "name" in f}
        unified_field_names = set(unified_props.keys())

        missing = unified_field_names - declared_fields
        extra = declared_fields - unified_field_names
        if missing:
            self.err(
                f"{len(missing)} unified field(s) on {type_name} have no mapping declaration: "
                + ", ".join(sorted(missing))
            )
        if extra:
            self.err(
                f"{len(extra)} mapping declaration(s) reference fields not in openpx schema: "
                + ", ".join(sorted(extra))
            )

        for ex in upstream_specs:
            self.coverage[ex] = {"direct": 0, "synthetic": 0, "omitted": 0}

        for field in self.mapping.get("fields", []):
            name = field.get("name")
            if not name or name not in unified_props:
                continue

            sources = field.get("sources", {}) or {}

            for ex in upstream_specs:
                src = sources.get(ex)
                if not src:
                    self.err(
                        f"field `{name}` exchange `{ex}` has no entry "
                        "(declare `type: direct|synthetic|omitted`)"
                    )
                    continue

                t = src.get("type")
                if t not in VALID_TYPES:
                    self.err(
                        f"field `{name}` exchange `{ex}` has type={t!r}; "
                        f"must be one of {sorted(VALID_TYPES)}"
                    )
                    continue

                self.coverage[ex][t] += 1

                if t != "direct":
                    continue

                ref = src.get("ref")
                if not ref:
                    self.err(
                        f"field `{name}` exchange `{ex}` is type=direct "
                        "but has no `ref:`"
                    )
                    continue

                if is_pointer(ref):
                    resolved = resolve_ref(upstream_specs[ex], ref)
                    if resolved is None:
                        self.err(
                            f"field `{name}` exchange `{ex}` ref does not "
                            f"resolve in {upstream_paths[ex]}: {ref}"
                        )

                fb = src.get("fallback_ref")
                if fb and is_pointer(fb):
                    resolved = resolve_ref(upstream_specs[ex], fb)
                    if resolved is None:
                        self.err(
                            f"field `{name}` exchange `{ex}` fallback_ref does "
                            f"not resolve: {fb}"
                        )

        return not self.errors


def report(validators: list[Validator]) -> int:
    total_err = sum(len(v.errors) for v in validators)

    for v in validators:
        for e in v.errors:
            print(f"ERROR  {e}", file=sys.stderr)

    print()
    print("=== Coverage ===")
    for v in validators:
        type_name = v.mapping.get("unified_type", "?")
        print(f"\n{v.mapping_path.name} ({type_name}):")
        for ex, c in v.coverage.items():
            total = c["direct"] + c["synthetic"] + c["omitted"]
            print(
                f"  {ex:12s} direct={c['direct']:3d}  "
                f"synthetic={c['synthetic']:3d}  omitted={c['omitted']:3d}  "
                f"(total={total})"
            )

    print()
    if total_err:
        print(f"FAIL: {total_err} error(s)")
        return 1
    print("PASS: 0 errors")
    return 0


def main() -> int:
    args = sys.argv[1:]
    if args:
        files = [MAPPINGS_DIR / f"{name}.yaml" for name in args]
    else:
        files = sorted(MAPPINGS_DIR.glob("*.yaml"))
    if not files:
        sys.exit(f"no mapping files in {MAPPINGS_DIR}")
    missing = [p for p in files if not p.is_file()]
    if missing:
        sys.exit(f"missing mapping file(s): {[p.name for p in missing]}")

    validators = [Validator(p) for p in files]
    for v in validators:
        v.run()
    return report(validators)


if __name__ == "__main__":
    sys.exit(main())
