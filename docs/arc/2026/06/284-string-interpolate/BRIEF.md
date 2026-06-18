# BRIEF — Arc 284: `:wat::core::string::interpolate` (pure-total interpolation intrinsic)

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the named tests, report. The orchestrator weighs independently.

## The work (one paragraph)

Add a pure-total Rust intrinsic `:wat::core::string::interpolate` — same `{name}` + trailing `:name val`
kwargs grammar as the `format` macro (same `{{`/`}}` escape, same unquoted render), but a CALL-time
intrinsic, so it is **expand-time-legal** (usable inside defmacro bodies, where `format` is refused).
Five wiring sites; the `format` macro is NOT touched.

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/284-string-interpolate/DESIGN.md` § "The build"** and implement its 5 sites
verbatim: the `eval_string_interpolate` intrinsic (template String + (keyword,value) pairs → unquoted-
rendered, `{name}`/`{{`/`}}`-parsed, strict-matched String), dispatch, a custom infer arm, the allow-list
entry, the doc-comment pair.

## Read in order (the rooms)

1. `docs/arc/2026/06/284-string-interpolate/DESIGN.md` — THE SPEC.
2. `src/runtime.rs:16886` (`eval_str`) — the unquoted render (String→itself, i64→digits, f64/bool/u8→text).
   Factor a `render_unquoted(Value) -> Result<String, …>` helper from its match, or mirror it; interpolate
   renders each kwarg value through it.
3. `wat/core.wat:543-736` (the `format` macro) — the template grammar to MIRROR in Rust: named `{name}`,
   `{{`→`{` / `}}`→`}`, strict (every `{name}` has a `:name`; every `:name` used), out-of-order kwargs.
4. `src/string_ops.rs` (e.g. `eval_string_concat`, `eval_string_split`) — the intrinsic shape
   (eval args, build, return `Value::String`; clean `RuntimeError`s). Add `eval_string_interpolate` here.
5. `src/runtime.rs:4085` (concat dispatch) — add the interpolate dispatch arm beside it.
6. `src/check.rs:4177` (the `infer_string_concat` custom arm) + `infer_string_concat`'s def — model
   `infer_string_interpolate`: arg[0] unifies with String; rest = (keyword, value) pairs (don't reject a
   value's type — it's str-rendered); returns String. Wire it in the infer match beside concat's arm.
7. `src/macros/eval.rs:414` (`is_pure_total`, the `string::concat` entry) — add
   `| ":wat::core::string::interpolate"`. **This is the load-bearing line** — it makes the intrinsic
   expand-time-legal (the whole point; the expand-time probe proves it).
8. `tests/probe_arc284_interpolate.rs` — remove the two `#[ignore = "arc 284 …"]`.

## Implementation notes
- Kwarg key: the keyword's name with leading `:` stripped (`:name` → `"name"`); placeholder `{name}`.
- Even-count guard on the rest (template + N pairs); odd → MalformedForm.
- Strict: missing `{name}`→RuntimeError naming it; unused `:name`→RuntimeError naming it. Repeated `{name}`
  against one `:name` is fine.
- `{{`/`}}` → literal single brace; a lone unpaired `{`/`}` → RuntimeError.

## STOP triggers (halt + report)
1. If the expand-time probe (`interpolate_is_legal_at_expand_time`) fails — the allow-list entry (site 7)
   is missing or the engine still refuses it; STOP, report.
2. If adding the intrinsic moves any existing floor (lib/deftest/nursery counts) — STOP; it must be additive.
3. If the check arm rejects a non-String kwarg VALUE (e.g. an i64) — STOP; values are str-rendered, any
   renderable type is legal.

## Blast radius
`src/string_ops.rs` + `src/runtime.rs` + `src/check.rs` + `src/macros/eval.rs` + un-ignore the probe + a
`wat-tests/` deftest. The `format` macro + `wat/core.wat` are NOT touched. No git.

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc284_interpolate                 # 2/2 GREEN (runtime + expand-time)
cargo test --release -p wat --test probe_arc279_format                       # format macro still GREEN (untouched)
cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"   # 929 passed / 36 failed (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                   # deftest 263 passed / 1 failed (was 262, +1 new)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result        # deporder 1 passed / 0 failed
```
Report: `eval_string_interpolate` + the render helper + the infer arm (paste them), the command outputs
verbatim, any delta. Do not claim green you did not see.
