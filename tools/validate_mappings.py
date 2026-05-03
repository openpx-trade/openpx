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

from mapping_lib import (
    CHANNEL_SECTIONS,
    iter_sources,
    load_schema_definitions,
    load_yaml,
    mapping_kind,
    resolve_ref,
)

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
        kind = mapping_kind(self.mapping)
        if kind == "model":
            return self._run_model()
        if kind == "channel":
            return self._run_channel()
        self.err(f"unknown `mapping_kind`: {kind!r}")
        return False

    def _load_upstream_specs(self) -> tuple[dict[str, str], dict[str, dict[str, Any]]] | None:
        upstream_paths = self.mapping.get("upstream_specs", {})
        if not upstream_paths:
            self.err("missing top-level `upstream_specs:` map")
            return None
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
        for ex in upstream_specs:
            self.coverage[ex] = {"direct": 0, "synthetic": 0, "omitted": 0}
        return upstream_paths, upstream_specs

    def _check_source(
        self,
        section: str,
        name: str,
        ex: str,
        src: dict[str, Any],
        upstream_specs: dict[str, dict[str, Any]],
        upstream_paths: dict[str, str],
    ) -> None:
        t = src.get("type")
        if t not in VALID_TYPES:
            self.err(
                f"{section} `{name}` exchange `{ex}` has type={t!r}; "
                f"must be one of {sorted(VALID_TYPES)}"
            )
            return
        if ex in self.coverage:
            self.coverage[ex][t] += 1
        if t != "direct":
            return
        ref = src.get("ref")
        if not ref:
            self.err(
                f"{section} `{name}` exchange `{ex}` is type=direct but has no `ref:`"
            )
            return
        if is_pointer(ref):
            spec = upstream_specs.get(ex)
            if spec is None:
                return
            if resolve_ref(spec, ref) is None:
                self.err(
                    f"{section} `{name}` exchange `{ex}` ref does not "
                    f"resolve in {upstream_paths[ex]}: {ref}"
                )
        fb = src.get("fallback_ref")
        if fb and is_pointer(fb):
            spec = upstream_specs.get(ex)
            if spec is None:
                return
            if resolve_ref(spec, fb) is None:
                self.err(
                    f"{section} `{name}` exchange `{ex}` fallback_ref does "
                    f"not resolve: {fb}"
                )

    def _run_model(self) -> bool:
        type_name = self.mapping.get("unified_type")
        if not type_name:
            self.err("missing top-level `unified_type:` key")
            return False

        unified = load_openpx_unified(type_name)
        unified_props = unified.get("properties", {})

        loaded = self._load_upstream_specs()
        if loaded is None:
            return False
        upstream_paths, upstream_specs = loaded

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
                self._check_source("field", name, ex, src, upstream_specs, upstream_paths)

        return not self.errors

    def _run_channel(self) -> bool:
        if not self.mapping.get("unified_channel"):
            self.err("missing top-level `unified_channel:` key")
            return False
        loaded = self._load_upstream_specs()
        if loaded is None:
            return False
        upstream_paths, upstream_specs = loaded

        declared_sections = [s for s in CHANNEL_SECTIONS if s in self.mapping]
        if not declared_sections:
            self.err(
                f"channel mapping declares no sections — expected at least one of {CHANNEL_SECTIONS}"
            )
            return False

        for section, name, ex, src in iter_sources(self.mapping):
            if ex not in upstream_specs:
                self.err(
                    f"{section} `{name}` references unknown exchange `{ex}` "
                    f"(not in upstream_specs)"
                )
                continue
            self._check_source(section, name, ex, src, upstream_specs, upstream_paths)

        # Every entry must declare every exchange exactly like model mappings.
        exchanges = set(upstream_specs.keys())
        for section in declared_sections:
            for entry in self.mapping.get(section, []) or []:
                name = entry.get("name") or entry.get("variant")
                if not name:
                    self.err(f"{section} entry missing `name:`/`variant:`")
                    continue
                missing_ex = exchanges - set((entry.get("sources") or {}).keys())
                if missing_ex:
                    self.err(
                        f"{section} `{name}` missing source for exchange(s): "
                        + ", ".join(sorted(missing_ex))
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
