# Arc 143 Slice 6 — SCORE

**Sweep:** sonnet, agent `aa70d4dc3686d545e`
**Wall clock:** ~6.6 minutes (well under the 30-min time-box cap)
**Output verified:** orchestrator re-ran `cargo test --release --test
wat_arc143_define_alias` + grep'd Gap 2's location.

**Verdict:** **MODE B — clean diagnostic ship.** Two substrate gaps
surfaced with file:line attribution + exact error messages. Sonnet
stopped at first red, did not grind, did not ship workarounds. The
substrate-as-teacher discipline working as designed. **The discipline
is the win, not the slice.**

## Hard scorecard (Mode B variant)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ NEW `wat/runtime.wat` (~25 LOC) + NEW `tests/wat_arc143_define_alias.rs` (3 tests) + MODIFIED `src/stdlib.rs` (1 entry). NO `wat/std/` additions. |
| 2-5 | Macro + registration + tests present | ✅ All verified per sonnet's report. |
| 6 | Test outcomes | Mode B as designed: 1/3 tests pass (the error case via `catch_unwind`), 2/3 fail with the exact Gap 1 / Gap 2 errors. |
| 7 | Workspace stays green | ✅ All other test suites pass; no NEW regressions introduced. The 3 new test failures are the Mode B diagnostic itself. |
| 8 | Honest report | ✅ Detailed diagnosis with file:line + verbatim error messages + remediation pointers. |

## The two gaps surfaced

### Gap 1 — `value_to_watast` can't bridge `Value::holon__HolonAST`

**Error verbatim** (from test 1):
```
Macro(MalformedTemplate {
  reason: "computed unquote value_to_watast failed:
           wat/runtime.wat:23:7: ,(expr): expected primitive
           (i64/f64/bool/String/keyword) or :wat::WatAST, got
           wat::holon::HolonAST",
  span: Span { file: "wat/runtime.wat", line: 23, col: 7 }
})
```

**Diagnosis:** Slice 2's computed unquote uses `value_to_watast` to
convert eval results to WatAST for splicing. `value_to_watast`
handles primitives + `Value::wat__WatAST`, but NOT
`Value::holon__HolonAST`. Slice 3's manipulation primitives
(`rename-callable-name`, `extract-arg-names`) return HolonAST
values. When the macro splices those results, value_to_watast
rejects them.

**Fix scope (slice 5b):** Extend `value_to_watast` in
`src/runtime.rs:5878+` to handle `Value::holon__HolonAST(arc)` by
converting it to `WatAST` (likely via `holon_to_watast` if such a
converter exists, OR by wrapping as a `WatAST::Atom` / `WatAST::Quoted`
form, OR by adding a `WatAST` leaf variant that holds a HolonAST).

The substrate already has `:wat::holon::to-watast` primitive (per
runtime.rs:2675), so the conversion logic exists at the wat-callable
level; just needs to be reachable from the macro splicer.

### Gap 2 — `:wat::core::length` invisible to `signature-of`

**Error verbatim** (from test 2): panic via `Box<dyn Any>` at
`runtime.rs:7480` — the `expect_panic` in `Option/expect` firing
because `signature-of :wat::core::length` returned `:None`.

**Diagnosis:** `:wat::core::length` is implemented via
`infer_length` (hardcoded handler at `check.rs:3080`) instead of a
TypeScheme registration. Per `check.rs:11284-85`: "wat::core::length
scheme retired; polymorphic under infer_length (arc 035). Dispatched
in infer_list."

**Slice 1's `lookup_callable` checks the TypeScheme registry** for
substrate primitives. Hardcoded handlers like `infer_length` bypass
that registry, so `signature-of` returns `:None` for them.

**Scope of this gap:** any substrate primitive that uses a hardcoded
type-checking path instead of a TypeScheme registration is invisible
to `signature-of` (and by extension, all of slice 1's reflection
primitives + slice 4's enumeration when it ships).

**Fix scope (slice 5c):** Two options:
- **Option A (clean):** Register a TypeScheme for hardcoded primitives
  alongside their hardcoded handlers, so `lookup_callable` finds them.
  Specifically, register `:wat::core::length`'s TypeScheme as
  polymorphic `∀T. :T -> :i64` (since the handler accepts any
  container).
- **Option B (extension):** Extend `lookup_callable` in
  `runtime.rs:6090` to ALSO check for hardcoded handlers via a
  parallel registry. More invasive; touches reflection-side discipline.

Option A is cleaner — it makes the registry uniform; future hardcoded
primitives just register their schemes. Option B leaks
"some-primitives-are-special" into reflection code.

**Recommended:** Option A. Audit all hardcoded `infer_*` handlers in
check.rs; register each with an appropriate TypeScheme.

### Architectural-but-not-a-gap surface — user-define visibility at expand-time

Sonnet's report flags: "User defines: expand_all passes
&SymbolTable::default() (empty). User defines are registered at step
6, after expansion at step 4. define-alias can only alias substrate
primitives at expand-time — user-define aliasing is architecturally
impossible with the current sequencing."

**This is NOT a new gap** — the brief mentioned it as possible. The
empirical confirmation matters: the macro CAN alias substrate
primitives (where `signature-of` looks in the TypeScheme registry,
which IS populated at expand-time). It CANNOT alias user defines
(those don't exist at expand-time).

For arc 143's primary goal (`:reduce` → `:foldl`), this is fine —
foldl is a substrate primitive. For broader user-define aliasing,
the substrate's load-order would need restructuring. Out of scope
for arc 143; future arc.

## Calibration record

- **Predicted Mode A (~50%) / Mode B-FQDN (~25%) / Mode B-other (~25%)**: ACTUAL Mode B with TWO different gaps (neither was FQDN — the FQDN concern was unfounded as the prior crawl predicted). The substrate-informed brief discipline correctly anticipated Mode B was likely; the SPECIFIC failure modes were different than predicted.
- **Predicted runtime (10-15 min Mode A, longer Mode B)**: ACTUAL ~6.6 min. Sonnet's STOP-at-first-red discipline shipped clean diagnostic FAST.
- **Time-box (30 min cap)**: not triggered; sweep finished naturally well under cap.

## Path forward

**Slice 5b (NEW, NEXT)**: extend `value_to_watast` to handle
`Value::holon__HolonAST`. ~20-40 LOC. Unblocks slice 6's macro
emission.

**Slice 5c (NEW, parallel)**: register TypeSchemes for hardcoded
primitives (length, etc.) so `signature-of` can find them. Audit
+ register. ~30-100 LOC depending on how many hardcoded handlers
exist.

**Slice 6 RELAND**: after 5b ships, re-spawn slice 6 — the macro
should work for substrate-primitive aliasing. After 5c ships, the
macro additionally works for hardcoded primitives like length.

**Slice 7**: ships when 5b shipped + slice 6 ships clean.

## Discipline lessons

- **The diagnostic IS the win.** Sonnet's STOP-at-first-red shipped
  TWO precisely-named substrate gaps with file:line attribution. The
  previous (killed) slice 6 sweep tried to ship workarounds; this
  one stopped clean. Brief discipline + sonnet discipline both held.
- **Mode B at 6.6 min is the cadence.** Diagnostic value at 1/4 of the
  predicted Mode A time. The discipline accelerates failure.
- **Time-box not triggered** — sonnet's natural completion was way
  under cap. The cap is a safety net, not a normal-operation
  constraint.
