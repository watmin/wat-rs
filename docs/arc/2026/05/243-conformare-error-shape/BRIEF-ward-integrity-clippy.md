# BRIEF — Ward-integrity clippy sweep (make the vigilatum stamps TRUE)

**Agent:** sonnet (`model:"sonnet"`). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. `git -C` for git; ignore `.claude/worktrees/`. Do NOT commit. Do NOT touch vigilatum stamps (orchestrator re-stamps).

## Why

Seven `src/` homes carry `//! vigilatum: … L1+L2=0` stamps. Clippy-in-home is an L2. But clippy was NOT an explicit lens in the earlier vigilia casts, so 5 homes drifted — they carry live clippy warnings their stamps claim don't exist. The stamps are currently OVERCLAIMING. This sweep makes them true. (`check/`, `types/`, `argspec/` are already clippy-zero — leave them.)

Scope: ONLY the warded-home files listed below. Do NOT touch flat `src/*.rs` (untrusted-by-design; cleaned when lifted, not now). Do NOT touch `result_large_err` warnings anywhere — that's RuntimeError-by-value design, banked as its own arc-243 question, NOT this sweep.

## MECHANICAL fixes (apply these — all behavior-preserving)

| File:line | Lint | Fix |
|---|---|---|
| `src/rust_deps/marshal.rs` | `needless_borrow` ×9 (51,69,89,109,129,151,176,262,287,378 area) | remove the immediately-dereferenced `&` |
| `src/rust_deps/custodia.rs` | `needless_borrow` (54,144) + check 77,89 | remove redundant `&` |
| `src/comms/process.rs:1060` | `io_other_error` | use `std::io::Error::other(_)` |
| `src/remedy/mod.rs:94` | `collapsible_match` | collapse the nested match per clippy |
| `src/comms/mod.rs:587` | `doc_lazy_continuation` | indent or blank-line the doc list item (same fix as types/defstruct.rs got) |

After each: `cargo build -p wat` clean, behavior identical (these are all syntactic).

## JUDGMENT CALLS — STOP and surface, do NOT auto-fix

These are flagged by clippy but are NOT mechanical — each touches a contract the original ward DELIBERATELY shaped. For EACH, report what you find + your recommendation; do NOT silence or impl blindly:

1. **`len_without_is_empty`** — `src/comms/mod.rs` (CommReceiver trait), `src/comms/thread.rs:163,236` + `src/comms/process.rs:473` (Receiver). The comms 9-spell cast DELIBERATELY narrowed `len()`'s contract (process tier undercounts — kernel pipe bytes invisible). Adding `is_empty()` requires deciding what it HONESTLY returns for a process receiver. Options: (a) add `is_empty()` with a truthful doc + impl consistent with the narrowed `len()`; (b) `#[allow(clippy::len_without_is_empty)]` with a doc comment naming the domain reason. RECOMMEND which + why. Do not just add a naive `self.len() == 0`.

2. **`new_without_default` / `Default for Select`** — `src/comms/thread.rs:236` + `process.rs:819` area (Select<'a,T>). The struere finding established an empty Select is a footgun (hangs forever / panics — requires ≥1 arm). A blind `Default` impl could REINTRODUCE that eliminated footgun. RECOMMEND: likely `#[allow(clippy::new_without_default)]` with a doc reason ("Select requires explicit arm registration; Default would permit the empty-Select footgun the ward eliminated") — but confirm by reading the Select constructor invariant first.

3. **`too_many_arguments`** — `src/function/eval.rs:34` + `parse.rs:188`. Report the arg count + whether a param-struct refactor is clean or whether `#[allow]` with reason is the honest call (don't force a struct if it obscures).

## After

- `cargo clippy -p wat --release 2>&1 | grep -E "src/(comms|function|remedy|rust_deps)/"` → only `result_large_err` lines may remain (those are out of scope); ALL other warded-home lints gone.
- `cargo build -p wat` clean; `cargo test -p wat` green except banked `probe_8_atom_round_trip`.
- No runes, no "deferred"/"TODO". Do NOT commit, do NOT re-stamp vigilatum.

## Return

SCORE-shaped: mechanical fixes applied (file:line, before→after); the 3 judgment calls with your finding + recommendation for each; clippy-on-homes result; test tally. The orchestrator decides the judgment calls, applies any `#[allow]` reasons, re-casts, and re-stamps.
