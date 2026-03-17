#!/usr/bin/env bash
# Generates docs/public/llms.md from all MDX source files.
# Strips frontmatter, imports, and JSX tab components into plain markdown.
# Run automatically before `astro build` and `astro dev`.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCS_DIR="$SCRIPT_DIR/src/content/docs"
OUT="$SCRIPT_DIR/public/llms.md"

mkdir -p "$SCRIPT_DIR/public"

cat \
  "$DOCS_DIR/index.mdx" \
  "$DOCS_DIR/getting-started/installation.mdx" \
  "$DOCS_DIR/getting-started/quickstart.mdx" \
  "$DOCS_DIR/guides/api.mdx" \
  "$DOCS_DIR/guides/websocket.mdx" \
  "$DOCS_DIR/guides/sports-websocket.mdx" \
  "$DOCS_DIR/guides/crypto-websocket.mdx" \
  "$DOCS_DIR/guides/cli.mdx" \
  "$DOCS_DIR/guides/sdks.mdx" \
  "$DOCS_DIR/reference/models.mdx" \
  "$DOCS_DIR/reference/exchanges.mdx" \
  "$DOCS_DIR/reference/errors.mdx" \
| sed '/^---$/,/^---$/d' \
| sed '/^import /d' \
| sed 's/<Tabs[^>]*>//' \
| sed 's/<\/Tabs>//' \
| sed 's/<TabItem label="\([^"]*\)">/\n**\1**\n/' \
| sed 's/<\/TabItem>//' \
> "$OUT"

echo "Generated $OUT ($(wc -l < "$OUT") lines)"
