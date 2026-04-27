# Runbook: watch CI on your own PR until it's green

Followed by every PR-opening agent (`kalshi-maintainer`, `polymarket-maintainer`, `core-architect`, `orchestrator`) **after** running `gh pr create`. Opening the PR is not the end of your work — green CI is.

## Why

Bots that walk away from their own broken CI burn human attention. Reviewers shouldn't have to triage agent-induced format failures, missing regen artifacts, or transient infra retries. The agent that opened the PR fixes its own messes.

## Protocol

After every successful `gh pr create`, the agent's last steps are:

```
1. gh pr edit <PR> --add-reviewer MilindPathiyal
2. Watch CI:
       gh pr checks <PR> --watch
   This blocks until every check has a final state. Wall-clock time is normal
   (~5–8 minutes); turn cost is one tool call.
3. Branch on outcome.
```

### Outcome A: all checks pass

Submit handoff with `status: success`. In `Notes`, include:

- The PR URL.
- A line saying `CI: all checks green on attempt N` (where N starts at 1 and increments per fix attempt).

Done. The human reviewer takes it from here.

### Outcome B: one or more checks fail

Run **up to 3 fix attempts**. After that, escalate.

For each attempt:

1. **Get the failure context.**
   ```
   gh pr checks <PR>                       # see which jobs failed
   gh run view <run-id> --log              # full log
   gh run view --job <job-id> --log        # specific job
   ```

2. **Classify the failure** — match against this table:

   | Symptom | Class | Fix |
   |---|---|---|
   | `cargo fmt --check` shows diff | Format | Run `cargo fmt --all`, commit, push |
   | `cargo clippy` warning | Lint | Read the warning location, edit the source, push |
   | `cargo test` test panic | Logic | Reproduce locally with `cargo test --workspace`, fix the source, push |
   | `manifest_coverage` test fails | Manifest gap | Add `FieldMapping` (preferred) or allowlist entry; per `runbooks/spec-version-bump.md` |
   | `contracts_test` test fails | Snapshot drift | Per `runbooks/contract-redeployment.md` — re-verify on Polygonscan, update either source or snapshot |
   | `sdk-sync` shows diff in `schema/`, `_models.py`, `models.d.ts`, or `docs/reference/` | Codegen | Run `just sync-all`, stage the regenerated files, push |
   | `version-sync` mismatch | Version drift | Don't touch this — escalate. Versions are release-please's domain. |
   | HTTP 502 / network error / timeout | Transient infra | `gh run rerun <run-id> --failed`. Do NOT push a code change. |
   | Permission denied on `gh pr edit` / `gh pr comment` | Auth issue | Escalate; not fixable in-loop. |

3. **Fix** per the table.

4. **Push to the same branch** (do not open a second PR):
   ```
   git add <changed-files>
   git commit -m "fix(ci): <one-sentence>"
   git push
   ```
   The push triggers a fresh CI run on the same PR.

5. **Loop back to step 2** of the parent protocol (`gh pr checks <PR> --watch`).

### Outcome C: max attempts reached

After 3 fix attempts, submit handoff with `status: blocked` and `Notes` that include:

- The PR URL
- Each attempted fix and why it didn't work
- The current CI state with relevant log excerpt
- A specific question or decision the human needs to make

Do NOT silently leave the PR red. The human should be able to read your Notes and either take the next step themselves or unstick you with one comment.

## Hard rules

- **Never bypass CI.** No `--no-verify`, no `[skip ci]` markers, no `cargo test --no-run` to "ship faster." If CI is wrong, that's a separate PR to fix CI.
- **Never push to anything other than the PR's own branch.** Each PR's CI is fixed by edits to that PR's branch.
- **Never close + reopen the PR to retrigger CI.** Use `gh run rerun --failed` for transient retries.
- **Never request a human fix without trying yourself first.** Format errors, clippy warnings, regen drift — these are the bot's job.
- **Never claim `status: success` while CI is red.** Either fix it or escalate with `status: blocked`. Submitting success on a red PR misleads the orchestrator and the human.

## Distinguishing transient from real

Transient (rerun without code change):

- `Unexpected HTTP response: 502` from any setup action (extractions/setup-just, actions/setup-python, etc.)
- `gh: connection reset` or `network error` mid-step
- `error sending request: ...timeout...` from cargo (registry hiccup)
- Any 4 consecutive seconds of identical retry messages followed by failure

Real (push a code fix):

- Anything starting with `error[E0` (Rust compile error)
- `error: clippy::...`
- `test ... FAILED` with a deterministic panic message
- `cargo fmt --check` returning non-empty diff
- `git diff --exit-code` failing inside `sdk-sync`

When unsure, run the failing command locally first — if it fails locally too, it's real.

## Note on `gh pr checks --watch` budget

`gh pr checks <PR> --watch` blocks until every check resolves. Wall-clock can be 5–10 minutes. That's fine — the agent's `--max-turns 30` budget counts turns, not wall-clock. One `--watch` invocation is one turn that happens to take a while.

If the watch exceeds 30 minutes (Mintlify deployments occasionally hang), check status manually with `gh pr checks <PR>` and continue based on the partial state. CI hangs are operational issues to flag in Notes, not to debug.
