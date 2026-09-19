# DESIGN — hoist 221 `where` fences into the conditions that bind them

**Status:** drawn 2026-09-09 at the builder's ruling. Supersedes `strike-where-fence-hoistable`'s
"NOT SHIPPED" disposition.

## The ruling that unblocks this

I reverted the check because it refused 221 sites of correct code and reddened 130 tests, calling it
*"a performance preference shipped as a hard error."* **The builder overruled that, and was right:**

> *"we are the only wat users — we have the freedom to work fast and loose here — the validation just
> proved we have bad examples and its named how to correct them."*

**Two things I had collapsed into one.** *Is a hoistable `where` a defect?* — **yes.** In a rete
engine, alpha-vs-beta is the algorithm's central cost distinction: an alpha test discriminates one
fact cheaply, a beta fence forces a join. **143 of the 221 sites are in `wat-scripts/perf/grid/` —
the performance corpus — so some benchmarks measure a slower path than the engine would take.**
*Should a validator refuse it while the corpus predates the rule?* — **no.** The cure is to fix the
corpus, then enable the check.

## What exists already

| | |
|---|---|
| **The check** | `f47a9fccc` — recoverable, intact. `check_where_hoistable`, `hoist_scope` dropping to `None` at `:or`/`:exists`/accumulate, 6 fixtures, mutation-proven |
| **The target form** | ⭐ **the refusal already PRINTS the rewrite** per site — the codemod is handed its output |
| **The census** | 221 sites / 60 files: `wat-scripts/` 143 in 25 files, `tests/` 75 in 32, plus a `docs/arc/` tree |

## ⛔ The stash dance is MANDATORY here — `wat/fix.wat:23-47`

This is the case that note was written for: a codemod shipping **alongside** a Rust change that makes
the old form illegal. Its own header records the cost of not knowing: *"A prior self did exactly that
— abandoned this purpose-built tool because the dance below wasn't written down."*

```
1. git stash push -m "rust change" src/rete/validate/mod.rs src/rete/validate/error.rs
2. cargo build --release                    # OLD checker, so the corpus still loads
3. printf '["pathA" …]\n' | cargo wat ./wat-scripts/fixes/<fix>.wat    # EVERY path
4. git stash pop
5. cargo build --release                    # new checker; corpus now conforms
```

⛔ **Dry-run step 3 on a `/tmp` copy and `diff` it first.** And **no hand-edits, no python, no sed** —
this is what `wat-fix` exists for.

## Phases, and the reason for the order

**Phase 1 — `tests/` (75 sites, 32 files).** ⭐ **No performance baseline depends on these**, so the
codemod proves itself where a mistake is cheap. **STOP after the dry-run diff and report it.**

**Phase 2 — `wat-scripts/` (143 sites, 25 files).** ⚠ **This invalidates the recorded grids.** The
`FLOOR` array in `check-grid-speed.sh` derives from `GRID-native-vs-clara-2026-08-27T07-15-56Z.txt`,
and 29 grids exist. **If the fixtures get faster — which is the entire point — every recorded number
becomes incomparable.** That is not damage; it is the finding landing. But it must be *stated*, not
discovered.

**Phase 3 — restore the check** (`git cherry-pick f47a9fccc` or equivalent) and floor green.

**Phase 4 — re-capture one grid** under the `#grid/Capture` header added in `35f1f4e8b`, so the new
baseline is the first provenance-bearing one.

## The one contract decision, pinned

⛔ **THE CODEMOD MUST BE IDEMPOTENT AND ITS DRY-RUN DIFF MUST BE READ.** Re-running it is 0 changes.
And a `where`→inline hoist is a **structural** rewrite — a predicate leaves the `:when` vector and is
appended to a condition's clause list. **A diff that touches anything else is a bug**, and 60 files
is far past what anyone will eyeball afterwards.

⚠ **Three-way agreement must survive.** These are differential fixtures; after Phase 2, the grid must
still agree across Clara / oracle / native. **If a hoisted form changes a match set, STOP** — that
would mean the two forms are not equivalent after all, which contradicts the compute host's own
proof and is far more important than this migration.

## Out of scope = rejected

- **The 24 `.rs` string-literal sites.** Hand work; separate.
- **`validate/mod.rs:456`.** Different arm, runtime justification, untouched.
- **Re-deriving the FLOOR numbers.** Phase 4 captures a provenance-bearing grid; deciding new floors
  from it is a performance strike with its own scorecard.
