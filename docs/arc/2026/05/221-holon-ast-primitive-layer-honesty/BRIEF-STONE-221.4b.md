# BRIEF — Arc 221 Stone 221.4b — Finish keyword→Symbol substrate-doctrine class in wat-rs

**Stone scope (sonnet portion):** Stone 221.4 closed `value_to_atom` at `src/runtime.rs:~13800`. Post-flight audit surfaced **5 more illegal substrate paths** still emitting `HolonAST::symbol(k.as_str())` for keyword content — plus 1 in `edn_shim.rs`. Stone 221.4b finishes the doctrine class. **Wat-rs ONLY — holon-rs untouched.**
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 120 min STOP.
**Depends on:** Stones 221.1-221.4 (shipped). Stone 221.3 (holon-rs `fa48b39`) provides `HolonAST::keyword()` constructor that strips leading colon.
**Calibration:** Per `feedback_stone_briefs_cite_prior_score`, read **SCORE-STONE-221.4.md** (the closest precedent; same kind of dispatch-arm work + cascade test fixes). Stone 221.4b is ~6 fix sites + Value::Unit audit + likely cascade test fixes.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`.
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs files.
- DO NOT modify Stone 221.5's scope (Symbol/String canonical-bytes seed in holon-rs — separate stone).

## Pre-flight verified (orchestrator-grep'd 2026-05-22 late)

### 6 illegal substrate sites (full audit complete)

| Site | Function context | Current line |
|---|---|---|
| `src/runtime.rs:13959` | `watast_to_holon` (WatAST → HolonAST lowering) | `WatAST::Keyword(k, _) => HolonAST::symbol(k.as_str()),` |
| `src/runtime.rs:14018` | Value→HolonAST dispatcher (rejects non-primitives) | `Value::wat__core__keyword(k) => HolonAST::symbol(k.as_str()),` |
| `src/runtime.rs:20938` | `:wat::holon::leaf` verb dispatch | `Value::wat__core__keyword(k) => HolonAST::symbol(k.as_str()),` |
| `src/runtime.rs:21273` | `eval-step!` Terminal stepping (Keyword form) | `WatAST::Keyword(k, _) => Ok(StepValue::Terminal(HolonAST::symbol(k.as_str()))),` |
| `src/runtime.rs:21322` | Step-form converter (sibling of 21273) | `WatAST::Keyword(k, _) => Some(HolonAST::symbol(k.as_str())),` |
| `src/edn_shim.rs:1899` | EDN keyword reader | builds `":foo::bar"` then `HolonAST::Symbol(Arc::from(s))` |

### Sites that ARE legal (do NOT touch)

| Site | Why legal |
|---|---|
| `src/special_forms.rs:77` | Slot placeholders `"<cond>"` etc. — substrate-internal pattern markers; bare identifiers; correctly Symbol |
| `src/runtime.rs:11588` | `new_name` is a renamed identifier; bare; correctly Symbol |
| `src/runtime.rs:11653` | `arg_name` is argument identifier; bare; correctly Symbol |
| `src/runtime.rs:13960` | `WatAST::Symbol(ident, _) => HolonAST::symbol(...)` — Symbol passthrough; no convention to strip |
| `src/runtime.rs:13830` | Comment inside Stone 221.4's keyword arm (historical reference) |

### Value::Unit consistency audit (sonnet judgment call)

Stone 221.4 mapped `Value::Unit => HolonAST::Nil` in `value_to_atom` at `~13800`. The other two dispatchers (`runtime.rs:14018` + `runtime.rs:20938`) currently REJECT `Value::Unit` as `RuntimeError::TypeMismatch` (their `other => Err(...)` catch-all).

Decision tree:
- **Option A:** Add `Value::Unit => HolonAST::Nil` to both dispatchers — consistent with `value_to_atom`. Nil is atomizable per Stone 221.3 doctrine.
- **Option B:** Keep them strict (Unit → TypeMismatch) — these dispatchers have different semantic contracts than value_to_atom; the 14018 dispatcher returns TypeMismatch for non-primitives, and the 20938 dispatcher is `:wat::holon::leaf` which may have stricter type expectations.

Per the four-questions: ALL THREE Value→HolonAST dispatchers being consistent (Option A) is more obvious + more honest + better UX. **Recommendation: Option A.** Sonnet adopts unless inspection of function contracts surfaces a clear reason for asymmetry; documents the choice in SCORE.

## Your scope (sonnet)

### 1. Fix 5 runtime.rs illegal sites

For each line, replace `HolonAST::symbol(k.as_str())` with `HolonAST::keyword(k.as_str())`:

```
runtime.rs:13959  WatAST::Keyword(k, _) => HolonAST::keyword(k.as_str()),
runtime.rs:14018  Value::wat__core__keyword(k) => HolonAST::keyword(k.as_str()),
runtime.rs:20938  Value::wat__core__keyword(k) => HolonAST::keyword(k.as_str()),
runtime.rs:21273  WatAST::Keyword(k, _) => Ok(StepValue::Terminal(HolonAST::keyword(k.as_str()))),
runtime.rs:21322  WatAST::Keyword(k, _) => Some(HolonAST::keyword(k.as_str())),
```

Update each site's nearby doc comment to cite Stone 221.4b + the post-arc-221 doctrine.

### 2. Fix edn_shim.rs:1899 illegal site

Current:
```rust
OwnedValue::Keyword(k) => {
    let s = match k.namespace() {
        Some(ns) => format!(":{}::{}", ns.replace('.', "::"), k.name()),
        None => format!(":{}", k.name()),
    };
    Ok(Arc::new(HolonAST::Symbol(Arc::from(s))))
}
```

New (drop the manual leading colon; emit Keyword):
```rust
OwnedValue::Keyword(k) => {
    let s = match k.namespace() {
        Some(ns) => format!("{}::{}", ns.replace('.', "::"), k.name()),
        None => k.name().to_string(),
    };
    Ok(Arc::new(HolonAST::Keyword(Arc::from(s))))
}
```

Doc comment updated to cite Stone 221.4b doctrine.

### 3. Value::Unit consistency audit + alignment

Recommended Option A: add `Value::Unit => HolonAST::Nil` arms to `runtime.rs:14018` and `runtime.rs:20938`. If function contracts of either dispatcher contradict this (read the surrounding function signature + doc), pick honestly and document in SCORE.

### 4. Cascade test fixes (per Stone 221.3 Delta 1a discipline)

After steps 1-3, run `cargo test --release --lib -p wat`. Some tests assert on the old Symbol-shape output of these dispatchers (e.g., `eval-step!`'s Terminal HolonAST, `:wat::holon::leaf`'s output, watast_to_holon's quoted-form lowering). They will fail.

**These are tests-broken-by-this-stone**, NOT pre-existing failures (per Stone 221.3 Delta 1a). Frame them honestly in your SCORE Delta sections:

> *"Stone 221.4b doctrine sweep broke N tests in <files>; they passed on baseline (post-Stone-221.4); they failed BECAUSE OF this stone's intentional substrate change."*

Fix each mechanically — flip `as_symbol() == Some(":foo")` to `as_keyword() == Some("foo")`. If a test's purpose is to verify the OLD Symbol convention (e.g., a regression test FOR the old convention), invert it like Stone 221.3 did for `keyword_distinct_from_symbol_at_type_level`.

### 5. New probes — `tests/wat_arc221b_keyword_dispatcher_completeness.rs`

5+ probes verifying each illegal-site fix produces Keyword (not Symbol):

1. **watast_to_holon Keyword arm:** quote a wat keyword form via `(:wat::core::quote :foo)` then convert via `:wat::holon::Atom`; result HolonAST is `Keyword("foo")` (no leading colon in content; not Symbol)
2. **Value→HolonAST second dispatcher (14018):** identify the function (likely `:wat::holon::leaf` or similar verb) and verify a keyword Value lowers to Keyword leaf
3. **`:wat::holon::leaf` keyword path (20938):** verify keyword → Keyword leaf via this verb
4. **eval-step! Terminal Keyword:** verify a Keyword form steps to a Terminal HolonAST::Keyword
5. **EDN keyword reader (edn_shim):** verify `#wat-edn.holon/...` or bare `:foo` EDN form parses to HolonAST::Keyword (not Symbol with leading colon)
6. **Value::Unit consistency (if Option A taken):** verify Unit lowers to Nil via both 14018 + 20938 dispatchers

If any probe is non-trivial to set up (e.g., the eval-step! Terminal needs a specific entry point), STOP and surface as a question.

### 6. Verification

From wat-rs/:

```
cargo build --release -p wat
cargo test --release --lib -p wat
cargo test --release --test wat_arc220_char
cargo test --release --test wat_arc221_char_atomization
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc221b_keyword_dispatcher_completeness
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All clean. Pre-existing wat-clippy backlog stays gated.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.4b.md`** mirroring SCORE-STONE-221.4.md shape.

## STOP triggers

- **STOP-1 (test regression beyond planned + DISHONESTLY framed):** if you find yourself writing "pre-existing failure" for a test broken AFTER Stone 221.4 baseline by THIS stone's changes — STOP. Apply Stone 221.3 Delta 1a discipline. Frame as "Stone 221.4b doctrine sweep broke this; mechanical consequence."
- **STOP-2 (load-bearing probe fails):** any of the 5-6 new probes fails its load-bearing assertion → STOP + diagnostic + report.
- **STOP-3 (120 min elapsed):** wall-clock STOP.
- **STOP-4 (holon-rs touched accidentally):** STOP and report.
- **STOP-5 (additional illegal substrate sites discovered beyond the 6 enumerated):** surface as Delta + ask before fixing — scope may need to grow but the orchestrator decides whether to extend this stone or open 221.4c.
- **STOP-6 (Value::Unit consistency decision unclear):** if you can't honestly pick Option A vs Option B from reading the function contracts, STOP and surface as a question.

## Out-of-scope

- holon-rs changes (Stone 221.3 + previous)
- Stone 221.5 — Symbol/String canonical-bytes seed distinction
- Stone 221.6 — INSCRIPTION (blocked on arc 223 + 222 per spawn-block)
- Arc 222 + 223 work
- Wat-edn wire format changes (edn_shim consumer side only)
- BOOK / USER-GUIDE updates
- Pre-existing wat-clippy backlog (115 warnings) — gated separately per arc 218 discipline
- New HolonAST variants — these are settled at the 16-variant count per arc 221 DESIGN
