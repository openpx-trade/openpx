#!/usr/bin/env python3
"""Generate docs/setup/<exchange>-credentials.mdx from schema/setup/<exchange>.yaml.

Inputs:
  - schema/setup/*.yaml — one file per exchange, the source of truth

Output:
  - docs/setup/<exchange_id>-credentials.mdx (overwritten)

Each YAML declares the exchange id, page title, frontmatter description, and
an ordered list of sections. Each section is either:

  text_only sections — just a markdown paragraph (no env/code blocks):
    - heading: ...
      text_only: |
        free-form markdown

  configuration sections — rendered as env block + Python/Node/Rust code tabs:
    - heading: ...
      intro: |
        prose before the env block
      env:
        ENV_VAR_1: "comment for the .env line"
      note: |
        prose between env block and code tabs (optional)
      code:
        config_key: ENV_VAR_1   # mapping from constructor key to env var

The generator emits Mintlify <CodeGroup> tabs for Python, Node, and Rust so
all three SDKs stay in lockstep. Re-run via `just setup-docs`.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parent.parent
SETUP_DIR = ROOT / "schema" / "setup"
DOCS_DIR = ROOT / "docs" / "setup"


def render_env(env: dict[str, str]) -> str:
    """Render a `.env` block (no key padding — convention is `KEY=value`)."""
    lines = [f"{k}={v}" for k, v in env.items()]
    return "```env\n" + "\n".join(lines) + "\n```"


def render_python(exchange_id: str, code: dict[str, str]) -> str:
    # Pad after `"key":` so the env lookups line up across rows.
    width = max(len(k) for k in code) + 3  # 2 quotes + colon
    fields = []
    for k, v in code.items():
        prefix = f'"{k}":'
        fields.append(f'    {prefix.ljust(width)} os.environ["{v}"]')
    body = ",\n".join(fields)
    return (
        '```python Python\n'
        'import os\n'
        'from openpx import Exchange\n\n'
        f'exchange = Exchange("{exchange_id}", {{\n'
        f'{body},\n'
        '})\n'
        '```'
    )


def render_node(exchange_id: str, code: dict[str, str]) -> str:
    # Pad after `key:` so the env lookups line up across rows.
    width = max(len(k) for k in code) + 1  # +1 for the trailing colon
    fields = []
    for k, v in code.items():
        prefix = f"{k}:"
        fields.append(f"    {prefix.ljust(width)} process.env.{v}")
    body = ",\n".join(fields)
    return (
        '```javascript Node\n'
        'import { Exchange } from "@openpx/sdk";\n\n'
        f'const exchange = new Exchange("{exchange_id}", {{\n'
        f'{body},\n'
        '});\n'
        '```'
    )


def render_rust(exchange_id: str, code: dict[str, str]) -> str:
    fields = ",\n".join(
        f'    "{k}": std::env::var("{v}").unwrap()' for k, v in code.items()
    )
    return (
        '```rust Rust\n'
        'use openpx::ExchangeInner;\n'
        'use serde_json::json;\n\n'
        f'let exchange = ExchangeInner::new("{exchange_id}", json!({{\n'
        f'{fields},\n'
        '}))?;\n'
        '```'
    )


def render_code_group(exchange_id: str, code: dict[str, str]) -> str:
    return (
        '<CodeGroup>\n'
        + render_python(exchange_id, code) + '\n\n'
        + render_node(exchange_id, code) + '\n\n'
        + render_rust(exchange_id, code) + '\n'
        + '</CodeGroup>'
    )


def render_section(exchange_id: str, section: dict[str, Any]) -> str:
    parts: list[str] = [f"## {section['heading']}", ""]

    if "text_only" in section:
        parts.append(section["text_only"].rstrip())
        return "\n".join(parts)

    if section.get("intro"):
        parts.append(section["intro"].rstrip())
        parts.append("")

    if section.get("env"):
        parts.append(render_env(section["env"]))
        parts.append("")

    if section.get("note"):
        parts.append(section["note"].rstrip())
        parts.append("")

    if section.get("code"):
        parts.append("Then pass them to the constructor:")
        parts.append("")
        parts.append(render_code_group(exchange_id, section["code"]))

    return "\n".join(parts).rstrip()


def render_page(spec: dict[str, Any]) -> str:
    eid = spec["exchange_id"]
    title = spec["title"]
    description = spec.get("description", "")

    body = "\n\n".join(
        render_section(eid, s) for s in spec["sections"]
    )

    return (
        "---\n"
        f'title: "{title}"\n'
        f'sidebarTitle: "{title}"\n'
        f'description: "{description}"\n'
        "---\n"
        "\n"
        + body
        + "\n"
    )


def main() -> int:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    if not SETUP_DIR.exists():
        print(f"error: {SETUP_DIR} does not exist", file=sys.stderr)
        return 1

    yaml_files = sorted(SETUP_DIR.glob("*.yaml"))
    if not yaml_files:
        print(f"error: no YAML files in {SETUP_DIR}", file=sys.stderr)
        return 1

    for yaml_path in yaml_files:
        with yaml_path.open() as f:
            spec = yaml.safe_load(f)
        out_path = DOCS_DIR / f"{spec['exchange_id']}-credentials.mdx"
        out_path.write_text(render_page(spec))
        print(f"wrote {out_path.relative_to(ROOT)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
