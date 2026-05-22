# BRIEF — Arc 221 Stone 221.4b — Finish keyword→Symbol substrate-doctrine class in wat-rs

> **SCOPE EXPANSION (mid-flight 2026-05-22 very-late):** First sonnet attempt landed the 6 dispatcher fixes (already on disk, uncommitted) but aborted before SCORE/verification due to piped-bash firewall trip. Post-flight verification surfaced that the **macro-support family in runtime.rs is the second half of the doctrine class** — `eval_rename_callable_name` + `eval_extract_arg_names` (+ audit `eval_signature_of_defn` / `eval_body_of` / `eval_lookup_define`) all assert + emit `HolonAST::Symbol(":foo")` (the retired pre-arc-221 convention). They tolerated the substrate's dishonest emission; Stone 221.4b's honest `watast_to_holon` change exposed them. 7 cargo lib tests fail through this single root cause. **Scope expanded within 221.4b** — same stone, expanded surface; not 221.4c per spawn-block discipline (221.4b never declared done). See Section "Phase 2 (NEW): Macro-support family cleanup" below.

**Stone scope (sonnet portion):** Stone 221.4 closed `value_to_atom` at `src/runtime.rs:~13800`. Post-flight audit surfaced **5 more illegal substrate paths** in dispatchers + 1 in `edn_shim.rs` (Phase 1, already done on disk) + the macro-support family (Phase 2, this re-spawn). Stone 221.4b finishes the doctrine class. **Wat-rs ONLY — holon-rs untouched.**
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

## Phase 2 (NEW — re-spawn focus): Macro-support family cleanup

Stone 221.4b Phase 1 (6 dispatcher fixes) is on disk in working tree. **Do NOT re-touch the Phase 1 edits** — `git diff` will show them already applied. Phase 2 = the macro-support family that ALSO lies about function/arg names.

### Phase 2 illegal sites (audit + fix)

| Site | Function | Current behavior | Honest fix |
|---|---|---|---|
| `runtime.rs:11560` | `eval_rename_callable_name` — assertion | `match &children[0] { HolonAST::Symbol(s) => ... }` rejects non-Symbol | Accept `HolonAST::Keyword(s)` (function names in macro-support context are keyword-shaped per arc 221 doctrine; the macro signature at `wat/runtime.wat:19-20` types them `:AST<wat::core::keyword>`) |
| `runtime.rs:11588` | `eval_rename_callable_name` — writer | `let new_first = HolonAST::symbol(new_name.as_str())` writes Symbol-with-leading-colon (`to_str` includes `:`) | Use `HolonAST::keyword(new_name.as_str())` (Stone 221.3 constructor strips leading colon) |
| `runtime.rs:11644` | `eval_extract_arg_names` | `match &children[0] { HolonAST::Symbol(s) if s.as_ref() == "->" => break }` — checks for `->` Symbol return-type sentinel | `->` is a bare symbol (no colon), so Symbol stays HONEST here. **Verify in context.** |
| `runtime.rs:11647` | `eval_extract_arg_names` | `if let HolonAST::Symbol(arg_name) = &pair[0]` — extracts arg name | Arg names are keyword-shaped per macro context; flip to `HolonAST::Keyword`. **Verify by tracing what produces `pair[0]`.** |
| `runtime.rs:11653` | `eval_extract_arg_names` — writer | `HolonAST::symbol(arg_name.as_ref())` produces output | If arg names are returned as keywords, flip to `HolonAST::keyword()`. **Verify caller expectations.** |
| `runtime.rs:11719` | similar pattern | `HolonAST::Symbol(s) if s.as_ref() == "->"` | Same as 11644 — `->` is a bare symbol; HONEST. |
| Doc comments at `runtime.rs:10485` / `10490` / `10494` | stale doc text | references `WatAST::Keyword(":Foo") → HolonAST::Symbol(":Foo")` | Refresh to cite arc 221 / Stone 221.4b doctrine — `WatAST::Keyword → HolonAST::Keyword` now |

### Audit-only sites (may not need changes — confirm honesty in context)

- `eval_signature_of_defn` at `runtime.rs:11228` — does it produce Bundle with Symbol or Keyword leaves? Trace through to see post-Stone-221.4b shape.
- `eval_body_of` — similar audit
- `eval_lookup_define` — similar audit

**Decision rule per site:** if the function processes content extracted FROM a quoted-keyword form, that content should be `HolonAST::Keyword(...)` per arc 221 doctrine. If the function processes substrate-internal markers (`->`, slot placeholders), those stay `HolonAST::Symbol(...)` because they're bare identifiers in the substrate's grammar, not user keywords.

### The shape distinction matters

```
User wrote     wat AST                       After Stone 221.4b watast_to_holon
─────────      ─────────────────────────     ───────────────────────────
my-fn          WatAST::Symbol("my-fn")        HolonAST::Symbol("my-fn")     ← bare identifier (HONEST)
:my-fn         WatAST::Keyword(":my-fn")      HolonAST::Keyword("my-fn")    ← keyword leaf (HONEST per arc 221)
->             WatAST::Symbol("->")           HolonAST::Symbol("->")        ← substrate sentinel (HONEST, bare)
```

The lying code was the macro-support family assuming "keyword-shaped function names emit as `Symbol(":name")`" — that was the retired convention. Now they must accept `Keyword("name")` (no colon) per arc 221.

## Your scope (sonnet)

### Phase 1 (already done on disk — VERIFY only, do NOT re-touch)

Confirm via `git diff src/runtime.rs src/edn_shim.rs` that these 6 fixes from prior sonnet flight are present:

1. `runtime.rs:13959` — `WatAST::Keyword(k, _) => HolonAST::keyword(k.as_str())`
2. `runtime.rs:14018` — `Value::wat__core__keyword(k) => HolonAST::keyword(k.as_str())` + `Value::Unit => HolonAST::Nil`
3. `runtime.rs:20938` — same as #2
4. `runtime.rs:21273` — `WatAST::Keyword(k, _) => Ok(StepValue::Terminal(HolonAST::keyword(k.as_str())))`
5. `runtime.rs:21322` — `WatAST::Keyword(k, _) => Some(HolonAST::keyword(k.as_str()))`
6. `edn_shim.rs:1899` — drops leading colon in `s = format!("{}::{}", ns, name)`; emits `HolonAST::Keyword(Arc::from(s))`

If `git diff` shows these — proceed to Phase 2. If not — STOP and report.

### Phase 2 (this re-spawn — execute)

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

### 6. Phase 2 specific Rust edits

After Phase 1 verification + diagnosing each Phase 2 site:

**A. `eval_rename_callable_name` (runtime.rs:11491-11600 area):**
- Line ~11560 assertion: flip `HolonAST::Symbol(s) => s.as_ref()` to `HolonAST::Keyword(s) => s.as_ref()`. The keyword content has NO leading colon per Stone 221.3 — adjust `first_str` comparison against `from_str` (which DOES have leading colon from Value::wat__core__keyword); strip the colon from `from_str` OR add it back to `first_str` for the comparison. Choose the path that keeps the error messages clear.
- Line ~11588 writer: `HolonAST::symbol(new_name.as_str())` → `HolonAST::keyword(new_name.as_str())`. The constructor strips any leading colon, so passing `to_str` (with or without colon) produces correct Keyword content.
- Update the error message at the assertion: `"Symbol as first Bundle child"` → `"Keyword as first Bundle child (function name)"`.

**B. `eval_extract_arg_names` (runtime.rs:11611-11700 area):**
- Lines 11644 + 11719 (`->` Symbol check): VERIFY these stay correct — `->` is a bare-symbol return-type sentinel in the substrate's grammar, NOT a user keyword. Keep as `HolonAST::Symbol`.
- Line 11647 (arg name extraction from pair): TRACE what produces `pair[0]`. If it's from a quoted-keyword form (likely yes per macro context), flip to `HolonAST::Keyword(arg_name) => ...`. Update line 11653 writer accordingly (`HolonAST::keyword(arg_name.as_ref())`).
- **STOP and surface as a question if the trace is ambiguous** — don't guess.

**C. Audit-only (no edits unless lying surfaces):**
- `eval_signature_of_defn` at runtime.rs:11228 — read the body, trace the Bundle it constructs. Does the first child come from a quoted-keyword form? If yes, it's now `HolonAST::Keyword` post-Stone-221.4b. If consumers expect Symbol, that's a lie to fix. If consumers correctly accept Keyword, no change needed.
- `eval_body_of` + `eval_lookup_define` — same audit pattern.

**D. Refresh stale doc comments (3 sites):**
- `runtime.rs:10485` / `10490` / `10494` — replace `WatAST::Keyword(":Foo") → HolonAST::Symbol(":Foo")` text with the post-Stone-221.4b reality (`WatAST::Keyword → HolonAST::Keyword` no colon in stored content).

### 7. Cascade test fixes (per Stone 221.3 Delta 1a discipline)

The 7 known failures all share root cause (rename-callable-name macro asserts Symbol). After Phase 2 fixes, these should pass:
- `runtime::tests::try_recv_on_ready_queue_returns_some`
- `runtime::tests::walk_w3_skip_short_circuits`
- `runtime::tests::values_sum_matches_map_values`
- `runtime::tests::walk_w2_already_terminal_input`
- `runtime::tests::zip_empty_with_nonempty_is_empty`
- `runtime::tests::zip_pairs_shorter_length`
- `runtime::tests::dissoc_removes_existing_key`

Other tests may surface as cascade once these compile + run. Tests asserting on `as_symbol() == Some(":foo")` for function/arg names need flipping to `as_keyword() == Some("foo")`. **NOT pre-existing — Stone 221.4b cascade.**

### 8. New probe — `tests/wat_arc221b_macro_support_keyword_shape.rs`

3+ probes verifying the macro-support family handles Keyword-shaped function names:

1. **rename-callable-name accepts Keyword first child:** construct a Bundle with `HolonAST::Keyword("foo")` as first child + dummy rest; call `(:wat::runtime::rename-callable-name <bundle> :foo :bar)`; verify result Bundle has `HolonAST::Keyword("bar")` as first child (with colon stripped).
2. **extract-arg-names extracts keywords:** construct a signature Bundle with Keyword-shaped arg names; verify extract-arg-names returns them as keywords.
3. **define-alias end-to-end:** define a function `foo`, then `(:wat::runtime::define-alias :bar :foo)`, verify `(bar args...)` dispatches to `foo`'s body.

### 9. Verification — TARGETED ONLY, NO `--lib` full sweep

The full `--lib` sweep includes 5+ pre-existing signal-handler test hangs (tracked in task #413). Running them costs ~9 minutes wall-clock. **AVOID.** Use targeted invocations.

Run each command DIRECTLY (no pipes, no `| grep`, no `| tail`):

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2 walk_w2_already_terminal_input walk_w3_skip_short_circuits try_recv_on_ready_queue_returns_some values_sum_matches_map_values zip_empty_with_nonempty_is_empty zip_pairs_shorter_length dissoc_removes_existing_key
cargo test --release --test wat_arc143_manipulation
cargo test --release --test wat_arc220_char
cargo test --release --test wat_arc221_char_atomization
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc221b_keyword_dispatcher_completeness
cargo test --release --test wat_arc221b_macro_support_keyword_shape
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

**Each command runs ALONE.** No backgrounding. No piping output to grep/tail (the pipe buffers everything until process exit, fooling sonnet into thinking the command hung). No concurrent runs (the prior sonnet attempt launched 3 background cargo tests simultaneously — that's what caused the "no output" panic; vanilla foreground commands produce streaming output normally).

If any verification command fails — read its stderr in the cargo output directly, identify the failure category, fix at root.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.4b.md`** mirroring SCORE-STONE-221.4.md shape. Must include:
- Phase 1 verification (6 dispatcher fixes confirmed on disk)
- Phase 2 deltas (macro-support family fixes with honest "what was lying" framing)
- Cascade test fixes per Stone 221.3 Delta 1a discipline
- Verification summary (targeted suites only; --lib full sweep explicitly skipped per task #413)
- Calibration record + substrate state + unblocks

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
