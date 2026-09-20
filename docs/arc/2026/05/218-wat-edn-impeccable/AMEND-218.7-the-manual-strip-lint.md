# AMEND — STONE 218.7: one clippy error, folds INTO the stone

**Orchestrator's verification row, `49c9900d6` + score `c3d245625` (neither pushed).**

## Everything else PASSED. This is the only thing standing between the stone and a push.

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5921/5921 passed**, 22 skipped, exit 0 — **+1 exactly as you predicted** |
| golden is the oracle's | ✅ I regenerated `golden.txt` myself with `/usr/local/bin/clj` — **byte-identical** |
| corpus generator idempotent | ✅ re-ran `generate_corpus.py` — **byte-identical**, 190 rows |
| corpus is generated, not a hand-list | ✅ all 6 NOT-EDN negative controls present; 21 slash rows; 7 bigint rows (was 0 and 0) |
| the 4 fixes + 7 spec-strict refusals + 6 non-vacuity controls | ✅ **19/19** behaviourally re-verified |
| duplicate rejected BEFORE entering the value | ✅ `duplicate key in map literal`, not a kept pair |
| **the ward actually CATCHES** | ✅ I injected a false golden verdict (`ERR\t42`); ward went **RED** with `"42" clj:ERR wat:OK`; restored → green |
| `census.sh --diff` | (run after the fold) |
| **clippy `-D warnings --all-targets --workspace`** | ⛔ **1 error — below** |

## THE RED, VERBATIM

```
error: stripping a prefix manually
   --> crates/wat-edn/tests/clj_oracle_parity.rs:152:20
    |
152 |         let body = &t[1..];
    |                    ^^^^^^^
    |
note: the prefix was tested here
   --> crates/wat-edn/tests/clj_oracle_parity.rs:151:5
    |
151 |     if t.starts_with(':') {
    |     ^^^^^^^^^^^^^^^^^^^^^^
    = note: `-D clippy::manual-strip` implied by `-D clippy::all`
help: try using the `strip_prefix` method
    |
151 ~     if let Some(body) = t.strip_prefix(':') {
152 ~         // `::foo` is Clojure auto-resolve, not the 219 constituent-char ruling.
    |
```

**Scope, measured — it is exactly one:**
- `cargo clippy --release --all-targets -p wat-edn -- -D warnings` → **1 error**, this one.
- Workspace-wide with `-A clippy::manual_strip` → **clean**. So `manual_strip` is the only blocker
  anywhere; the earlier run merely aborted before reaching the other crates.

## The fold

⛔ **Fold it INTO `49c9900d6`.** Not a repair commit after the stone — the stone must be green at its
own landing. Amending an unpushed commit is safe; nothing here is on `origin`.

⚠ **`#[allow(clippy::manual_strip)]` is NOT the cure.** This is a test that exists to keep a
classification honest; a suppression in it is the same shape as an exemption without a reason.

⚠ **This is the `::foo` arm — the one your SCORE says you tightened** (*"`::foo` is auto-resolve not
constituent"*), and `every_exemption_names_a_reason` asserts `exemption("::foo").is_none()`. **The
rewrite must not change that verdict.** Clippy's suggested shape preserves it, but confirm rather
than assume: after the fold, `exemption("::foo")` must still be `None`, and `exemption("a:b")` must
still be `Some`.

## After the fold

Re-run `cargo test --release -p wat-edn` and state it. The orchestrator re-runs the whole floor,
clippy workspace-wide, and the census uncontended before any push. **Do not push.**

## One thing in your SCORE the orchestrator owes you

> *"The brief asked for 'the three REPL one-liners (5, 6, 7 above)' — those numbered items are not in
> the struck brief (they lived in an earlier draft)."*

**Correct, and it is the orchestrator's defect.** The brief was rewritten after the builder's ruling
and that sentence kept a cross-reference to rows the rewrite had deleted — a dangling pointer inside
a gate row. Your substitution (running the oracle set the brief actually names) was the right call,
and the 18-row REPL table you pasted is a superset of what was intended. Recorded, not held against
the strike.
