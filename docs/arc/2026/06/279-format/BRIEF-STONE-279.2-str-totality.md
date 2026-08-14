# BRIEF — STONE 279.2: `str` becomes TOTAL

Make `:wat::core::str` render **any** value, and make `:wat::core::show` render the same way with
top-level strings quoted. Both route to the EDN encoder that already renders every value correctly;
neither keeps its own rendering logic. The committed probe
`tests/value/probe_arc279_str_totality` is the contract — it is RED at HEAD (3 controls pass,
5 rows fail) and must be GREEN when you are done.

## Read in order

1. **`tests/value/probe_arc279_str_totality.rs` + `.wat`** — the contract. Eight rows; the three
   `control_*` ones already pass and must keep passing. Read the target rendering off these, not off
   this brief.
2. **`src/runtime.rs:20213` `eval_str`** — the five-arm match to replace. Its body is
   `match v { String|i64|f64|bool|u8 => …, other => Err(TypeMismatch{expected: "String | i64 | f64 | bool | u8"}) }`.
3. **`src/runtime.rs:20175` `eval_show`** — one line of body:
   `Ok(Value::String(Arc::new(crate::value::observe::render_value(&v, 0))))`. This is the call that
   goes; `render_value` itself stays (see blast radius).
4. **`src/edn_shim.rs:3423` `value_to_edn_string(v: &Value) -> String`** — the door. It is
   `wat_edn::write(&value_to_edn_with(v, None))`. The sibling
   `value_to_edn_string_with(v, types: Option<&TypeEnv>)` at `:3433` takes a registry; `None` is a
   supported call (`panic_hook.rs:191` uses it) — see STOP-1.
5. **`src/value/observe.rs:96-135`** — `ValueSnapshot::of` also calls `render_value`. Read this so you
   can see why `render_value` is NOT being deleted.

## Implementation sketch

```rust
// eval_show — the readable form, verbatim from the encoder.
let v = eval_inner(&args[0], env, sym)?.value_owned();
Ok(Value::String(Arc::new(crate::edn_shim::value_to_edn_string(&v))))

// eval_str — identical, EXCEPT a top-level String renders bare.
let v = eval_inner(&args[0], env, sym)?.value_owned();
let s = match &v {
    Value::String(s) => (**s).clone(),                      // bare, top level only
    other            => crate::edn_shim::value_to_edn_string(other),
};
Ok(Value::String(Arc::new(s)))
```

That is the whole shape. A string nested inside a collection goes through the encoder and stays
quoted, which is what `str_keeps_nested_strings_quoted` checks.

## Blast radius

`src/runtime.rs` (two function bodies) and nothing else in `src/`. **`src/value/observe.rs` is NOT
edited and `render_value` is NOT deleted** — `ValueSnapshot::of` still uses it for the `:rendered`
field in diagnostics, which is a separate consumer with a depth cap and an unbounded-List guard.
Expect the arity/error-shape checks in both functions to stay exactly as they are.

Existing tests may move: `format` expands to `str` calls, so any test asserting that `format` or
`str` REFUSES a non-scalar is asserting the defect. If you find one, see STOP-3.

## STOP triggers — surface and ship nothing, do not work around

- **STOP-1 — the encoder needs a `TypeEnv` you cannot supply.** If `value_to_edn_string`'s `None`
  path renders a **record or enum** differently from the `Some(types)` path (`capability/registry.rs:361`
  passes `Some`), STOP and report BOTH renderings for the same value. Do not thread a `TypeEnv` into
  `eval_str` on your own judgement — that is a signature change this stone did not authorize.
- **STOP-2 — a golden pins the old `show` output.** If any test or golden asserts `[1, 2, 3]`,
  `{:a: 1}`, `()` for nil, or `(Some 5)`, do **not** edit it to match the new rendering. Report the
  file and line. Some are the defect and some may be a real contract, and that call is not yours.
- **STOP-3 — a test asserts that `str` REFUSES something.** Same rule: report it, do not delete or
  rewrite it. A test that pins the five-arm domain is measuring the thing this stone removes, and it
  needs a ruling, not an edit.
- **STOP-0 — the `#wat-edn.*` tag namespace is NOT yours.** Routing `str` through the encoder means
  an opaque or holon value will render with a `#wat-edn.opaque/…` / `#wat-edn.holon/…` tag. Those tags
  are **arc 294's**, condemned there and unstruck (builder ruling 2026-08-14,
  `294/RULING-holonast-and-hologram-are-both-correctly-named.md`). Do **not** rename, prettify, or
  special-case them. Emit whatever the encoder emits. If a probe row seems to need a tag changed, that
  is STOP-4, not a licence.
- **STOP-4 — the probe would need changing to pass.** The probe is the contract. If your
  implementation produces a different string for any of the eight rows, the implementation is wrong
  or the encoder disagrees with what was measured — either way report it. **Do not edit
  `tests/value/probe_arc279_str_totality.*`.**

## How you are weighed

`EXPECTATIONS-STONE-279.2-str-totality.md`, written before this brief. Row 1 is the probe going
8/8 green with the three controls still passing. Row 2 is `render_value` still existing and
`ValueSnapshot` still using it. The orchestrator runs the full floor centrally.

## Working rules

Work in `/home/watmin/work/holon/wat-rs` and confirm with `pwd` first; any path containing
`.claude/worktrees/` is not yours and must not be operated on. Use `git -C /home/watmin/work/holon/wat-rs`
for any git read. **Do not commit, push, stash, or revert** — leave the tree dirty for the
orchestrator. Run every command in the FOREGROUND and block on it: **ending your turn ends you**,
nothing will wake you, and a verification you launched but did not read is a verification that did
not happen. Your turn ends when the numbers are in your hands. Your gate is
`cargo nextest run --release -E 'binary_id(wat::value) and test(probe_arc279)'` plus
`cargo clippy --release --all-targets` — not the full suite; the orchestrator measures that centrally.
