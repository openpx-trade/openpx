# Runbook: issue triage

Followed by `orchestrator` on `issues.*` events after the workflow's `if:` condition has confirmed admin-association.

## Pre-condition

The workflow already gated on:

- `github.event.issue.user.login == 'MilindPathiyal'` (admin authored), OR
- `github.event.issue.assignee.login == 'MilindPathiyal'` (admin assigned), OR
- `triage-ready` label present, OR
- A comment by an admin mentioning `@openpx-bot`

Don't second-guess this. Proceed.

## Steps

1. **Read the issue.** `gh issue view <N> --json title,body,labels,user`.

2. **Classify** into exactly one primary type:

   | Type | Signal in body |
   |---|---|
   | `bug` | "doesn't work", "panics", "wrong result", error logs included |
   | `enhancement / parity-gap` | "could you add", "both X and Y support", request for unified method |
   | `enhancement / exchange-specific-feature` | "Polymarket has X but it's not exposed", "support Y on Kalshi" |
   | `new-exchange-request` | "add support for <new exchange name>" |
   | `question` | how-to, clarification, no concrete change requested |
   | `docs-bug` | typo/error in `docs/`, broken link, wrong example |

3. **Apply labels.** Add:
   - `enhancement` or `bug` (the type)
   - `parity-gap` if a parity-gap was identified
   - `area:kalshi`, `area:polymarket`, `area:core`, `area:sdk`, `area:bindings`, `area:onchain`, `area:docs`, or `area:ci` based on what the change would touch
   - `requires-human-careful-review` if the change would touch the trait, models, on-chain code, or release config

4. **Route:**

   - **bug** in a specific exchange's code → dispatch that exchange's maintainer with the issue number. The maintainer investigates and either opens a fix PR (single-purpose) or comments on the issue with reproduction-needed.
   - **parity-gap** or **unified-trait-proposal** → dispatch `parity-analyst`. Analyst posts a technical assessment as a **comment on the existing issue, NOT a new issue.** Filing a new proposal when the user has already filed the source issue produces duplicates. **Do not open a PR.** The human approves the trait shape; only after that does a maintainer implement.
   - **exchange-specific-feature** → dispatch the relevant maintainer. Maintainer either opens a docs PR (if the feature exists in code but isn't documented) or files a sub-issue for implementation work, depending on what's needed.
   - **new-exchange-request** → comment with `cc @MilindPathiyal — adding a new exchange is a human decision (jurisdiction, ToS, scope). The exchange-onboarding agent is deferred until you decide to proceed.` Do not action.
   - **question** → comment with pointers to relevant runbooks/docs. If the question is about a feature, link to the corresponding code or docs page. Close the issue once answered, with `gh issue close <N> --comment "<answer>"`.
   - **docs-bug** → if simple (typo, broken link in `docs/`), dispatch the orchestrator-self to make the doc edit and open a `docs-only` PR. If it's a substantive doc rewrite, route to the relevant maintainer.

5. **Submit handoff.** In `Notes`, list:
   - The classification you chose
   - The labels applied
   - Which subagent (if any) you dispatched and the resulting PR/comment URL

## What you must NEVER do

- Action a public-user issue without admin association (the gate handles this; if you somehow get here without it, refuse and exit).
- Open a code PR yourself. You only label, comment, and dispatch.
- Approve trait additions. Always file as parity-analyst proposals for human review.
- Take a "new exchange" issue as a green light to scaffold. That's a human decision and an `exchange-onboarding` agent (deferred) that doesn't exist yet.
- Close an issue without an explanatory comment.
