# OpenPX docs

This is the OpenPX documentation site, built with [Mintlify](https://mintlify.com).

## Local preview

Install the Mintlify CLI once:

```bash
npm i -g mintlify
```

Then from this directory:

```bash
mintlify dev
```

## Structure

- `docs.json` — Mintlify config: theme, colors, and navigation.
- `index.mdx`, `introduction.mdx`, `quickstart.mdx` — landing pages.
- `exchanges/` — per-exchange config and capability docs.
- `sdks/` — language-binding usage notes (Rust, Python, TypeScript).
- `reference/` — API reference.

## Deploys

Production deploys are handled by Mintlify's hosted git integration — connect this repo at [mintlify.com](https://mintlify.com) and pushes to `main` deploy automatically. There is intentionally no GitHub Actions deploy workflow in-repo.
