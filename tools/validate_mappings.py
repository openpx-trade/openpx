#!/usr/bin/env python3
"""Validate schema/mappings/*.yaml against canonical sources.

Checks per mapping file:
  1. Every unified field in openpx.schema.json is declared (synthetic or sourced).
  2. Every declared source $ref resolves in the cached upstream spec.
  3. For transform=direct, source type is compatible with unified type.
  4. fallback_ref (if present) also resolves.

Exit code 0 on pass, 1 on validation failure. Intended to run in CI as part of
the sdk-sync gate.

Usage:
    python3 tools/validate_mappings.py                 # all mapping files
    python3 tools/validate_mappings.py market          # single type
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / "schema" / "openpx.schema.json"
MAPPINGS_DIR = ROOT / "schema" / "mappings"

# Type compatibility for transform=direct. LHS is upstream (spec) type as
# normalized via _spec_type(); RHS is unified (openpx) type. A unified field
# typed `string|null` accepts any source typed `string`, etc.
DIRECT_COMPATIBLE: dict[str, set[str]] = {
    "string": {"string"},
    "boolean": {"boolean"},
    "integer": {"integer", "number"},
    "number": {"number", "integer"},
}


def load_yaml(p: Path) -> Any:
    return yaml.safe_load(p.read_text())


def load_openpx_unified(type_name: str) -> dict[str, Any]:
    schema = json.loads(SCHEMA.read_text())
    defs = schema.get("definitions") or schema.get("$defs") or {}
    if type_name not in defs:
        sys.exit(f"openpx.schema.json has no definition for {type_name!r}")
    return defs[type_name]


def resolve_ref(spec: dict[str, Any], ref: str) -> dict[str, Any] | None:
    """Resolve a JSON-pointer-style ref like '#/components/schemas/Market/properties/ticker'."""
    if not ref.startswith("#/"):
        return None
    cur: Any = spec
    for part in ref[2:].split("/"):
        part = part.replace("~1", "/").replace("~0", "~")
        if isinstance(cur, dict) and part in cur:
            cur = cur[part]
        else:
            return None
    return cur if isinstance(cur, dict) else None


def _normalize_type(t: Any) -> set[str]:
    """Return the set of non-null types for an OpenAPI/JSON-Schema type field."""
    if isinstance(t, list):
        return {x for x in t if x and x != "null"}
    if isinstance(t, str):
        return {t} if t != "null" else set()
    return set()


def _spec_type(node: dict[str, Any]) -> set[str]:
    """Best-effort type extraction from a resolved OpenAPI property node."""
    if "type" in node:
        return _normalize_type(node["type"])
    # OpenAPI nullable: true with $ref means the ref's type
    if "$ref" in node:
        return {"$ref"}
    if "allOf" in node:
        for sub in node["allOf"]:
            t = _spec_type(sub) if isinstance(sub, dict) else set()
            if t:
                return t
    return set()


def _unified_type(node: dict[str, Any]) -> set[str]:
    if "type" in node:
        return _normalize_type(node["type"])
    if "allOf" in node:
        for sub in node["allOf"]:
            if isinstance(sub, dict) and "$ref" in sub:
                return {"enum_or_object"}
    if "$ref" in node:
        return {"enum_or_object"}
    return set()


class Validator:
    def __init__(self, mapping_path: Path) -> None:
        self.mapping_path = mapping_path
        self.mapping = load_yaml(mapping_path)
        self.errors: list[str] = []
        self.warnings: list[str] = []
        self.coverage: dict[str, dict[str, int]] = {}

    def err(self, msg: str) -> None:
        self.errors.append(f"[{self.mapping_path.name}] {msg}")

    def warn(self, msg: str) -> None:
        self.warnings.append(f"[{self.mapping_path.name}] {msg}")

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
            self.coverage[ex] = {"sourced": 0, "synthetic": 0, "omitted": 0}

        for field in self.mapping.get("fields", []):
            name = field.get("name")
            if not name:
                self.err("field entry missing `name`")
                continue
            if name not in unified_props:
                continue

            unified_type = _unified_type(unified_props[name])
            sources = field.get("sources", {})

            if not sources and "synthetic" in field:
                for ex in upstream_specs:
                    self.coverage[ex]["synthetic"] += 1
                continue

            for ex in upstream_specs:
                src = sources.get(ex, {})
                if not src:
                    self.warn(
                        f"field `{name}` has no entry for exchange `{ex}` "
                        "(declare `omitted: true` to silence)"
                    )
                    continue
                if src.get("omitted"):
                    self.coverage[ex]["omitted"] += 1
                    continue
                if "synthetic" in src:
                    self.coverage[ex]["synthetic"] += 1
                    continue

                ref = src.get("ref")
                if not ref:
                    self.err(
                        f"field `{name}` exchange `{ex}` has no `ref`, `synthetic`, or `omitted`"
                    )
                    continue

                resolved = resolve_ref(upstream_specs[ex], ref)
                if resolved is None:
                    self.err(
                        f"field `{name}` exchange `{ex}` ref does not resolve "
                        f"in {upstream_paths[ex]}: {ref}"
                    )
                    continue

                fallback_ref = src.get("fallback_ref")
                if fallback_ref:
                    fb = resolve_ref(upstream_specs[ex], fallback_ref)
                    if fb is None:
                        self.err(
                            f"field `{name}` exchange `{ex}` fallback_ref does not resolve: "
                            f"{fallback_ref}"
                        )

                self.coverage[ex]["sourced"] += 1

                transform = src.get("transform", "direct")
                if transform == "direct":
                    spec_t = _spec_type(resolved)
                    if (
                        spec_t
                        and unified_type
                        and "$ref" not in spec_t
                        and "enum_or_object" not in unified_type
                    ):
                        compatible = any(
                            ut in DIRECT_COMPATIBLE.get(st, set())
                            for st in spec_t
                            for ut in unified_type
                        )
                        if not compatible:
                            self.err(
                                f"field `{name}` exchange `{ex}` transform=direct but "
                                f"types incompatible: spec {sorted(spec_t)} vs unified "
                                f"{sorted(unified_type)} (declare an explicit transform)"
                            )

        return not self.errors


def report(validators: list[Validator]) -> int:
    total_err = sum(len(v.errors) for v in validators)
    total_warn = sum(len(v.warnings) for v in validators)

    for v in validators:
        for e in v.errors:
            print(f"ERROR  {e}", file=sys.stderr)
        for w in v.warnings:
            print(f"WARN   {w}", file=sys.stderr)

    print()
    print("=== Coverage ===")
    for v in validators:
        type_name = v.mapping.get("unified_type", "?")
        print(f"\n{v.mapping_path.name} ({type_name}):")
        for ex, c in v.coverage.items():
            total = c["sourced"] + c["synthetic"] + c["omitted"]
            print(
                f"  {ex:12s} sourced={c['sourced']:3d}  "
                f"synthetic={c['synthetic']:3d}  omitted={c['omitted']:3d}  "
                f"(total={total})"
            )

    print()
    if total_err:
        print(f"FAIL: {total_err} error(s), {total_warn} warning(s)")
        return 1
    print(f"PASS: 0 errors, {total_warn} warning(s)")
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
