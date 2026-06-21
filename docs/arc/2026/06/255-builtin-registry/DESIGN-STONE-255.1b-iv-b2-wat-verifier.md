# Stone 255.1b-iv-b2 — the wat verifier (R2's self-hosting answer)

**Why this stone.** iv-b1 made `#[wat_intrinsic]` carry the structured doc on the registry entry
(`args`/`examples`/`deprecated`/`see` under a dated `#[allow(dead_code)]`). iv-b2 builds their **reader**
— and per R2, the reader is **wat, not Rust**: the substrate verifies its own examples *in itself*,
`deporder`/`verify-stdlib`-style. `(verify-examples) ≈ (verify (stdlib-sources))`.

Split depth-first: **iv-b2-a** the Rust reflection seam (exposes examples to wat — retires the allows);
**iv-b2-b** the wat verifier + the gate.

## GROUNDING CATCH (recorded, 2026-06-21) — the `is_effectful_op` change is DROPPED

The plan carried an `is_effectful_op` "syscall-honesty fix" (make entropy/clock/time effectful). **It is
wrong and unnecessary; grounding overturned it:**
- `runtime.rs:4023` states the codebase's deliberate model: *"Uuid/v4 IS pure (does no IO)... Uuid/v4
  is NOT [deterministic] (random)."* The substrate models randomness/time as **pure-but-non­deterministic**,
  NOT impure — and the rete `pure?`/`deterministic?` predicates rest on it. Making syscalls "effectful"
  would contradict the model + those predicates.
- It is not needed: the cross-check gate is `pure ∧ deterministic`; the **determinism axis already
  excludes** `uuid/v4`/time. Bytes is genuinely `pure ∧ deterministic`, so the cross-check is correct
  as-is for the current registry.

So iv-b2 uses the EXISTING `pure`/`deterministic` derivation untouched. The legitimate residual — the
`NONDETERMINISTIC` hand-list (`runtime.rs:10108`, `[Uuid/v4]`) is exactly the hand-list 255 exists to
kill — is flagged as **its own future stone** (derive determinism structurally), NOT bundled here.

## Machinery (grounded)
- `:wat::eval-ast!` (`eval_form_ast`, runtime.rs:21930) evals a **quoted form** (`Value::wat__WatAST`)
  against the current world (live usage: rete.wat:1984).
- Examples are stored as source STRINGS (`expr: &'static str`). The seam PARSES them Rust-side
  (`parse_one_with_file`) into `Value::wat__WatAST` quoted forms — the verifier just `eval-ast!`s them.
- Template: `:wat::stdlib::sources` (io.rs:1454, a Rust seam returning plain `Vector`s) → `verify-stdlib`
  (wat wraps into records + verifies). Mirror exactly.

## iv-b2-a — the reflection seam `:wat::intrinsic::examples` (Rust)

A new intrinsic (home: `src/intrinsic/`, exposed via the runtime dispatch like `stdlib::sources`).
Zero args; pure. Walks `registry()` and returns a `Vector` of per-example tuples — mirroring
`stdlib::sources`'s "return plain vectors, let wat wrap" shape (no Rust-built records — keep the seam
dumb):

Each element is a `Vector<Value>` (heterogeneous via `:wat::core::Value`, the universal top — arc 278 R7):
`[fqdn-keyword, expr-quoted-ast, expected-quoted-ast-or-nil, run-bool, pure-bool, det-bool]`
- `fqdn` — the intrinsic's name as a keyword.
- `expr` — `parse_one_with_file(example.expr)` → `Value::wat__WatAST`.
- `expected` — `Some` → parsed quoted-ast; `None` (`@example-norun` w/o marker) → `nil`.
- `run` — the bool.
- `pure` / `deterministic` — derived exactly as `metadata-of` does (`!is_effectful_op` ∧ `∉ NONDETERMINISTIC`).

**This read retires the dated `#[allow(dead_code)]`s** on `IntrinsicEntry.{args,examples,…}` /
`ExampleSubmission` — the seam reads `examples` (and the others follow as readers land). Remove the
`examples` allow + `ExampleSubmission`'s in this stone (its fields are now read); `args`/`see`/
`deprecated` allows come off when their readers land (the wiki/`doc` — keep their dated allows, or expose
them through the seam too if cheap).

Parse-failure of an example string is a loud seam error (acceptable — a malformed example is a real
defect; the macro enforced the doc SHAPE, not that `expr` parses as wat — that parse-check is 255.2's
type-check-no-run). For the current registry (Bytes, known-good) it parses clean.

### iv-b2-a probe (RED at HEAD)
`(:wat::intrinsic::examples)` returns a non-empty vector whose entries include `:wat::core::Bytes::to-hex`
with `run=true`. RED at HEAD (the seam head has no dispatch arm → runtime error). GREEN after b2-a.

## iv-b2-b — `verify-examples` (wat) + the gate

A new stdlib file `wat/doctest.wat` (loaded after its deps; registered in `STDLIB_FILES`; deporder gate
keeps the order honest). The surface mirrors `verify-stdlib`:

```
;; for each example: when run, eval expr + eval expected, assert equal;
;; AND cross-check: a run=true example MUST ride a pure∧deterministic intrinsic.
(:wat::core::defn :wat::doctest::verify-examples [] -> :wat::core::Vector<…Failure>
  ;; foldl over (:wat::intrinsic::examples):
  ;;   run=true:
  ;;     (assert (and pure deterministic))            ; the marker cross-check
  ;;     (= (:wat::eval-ast! expr) (:wat::eval-ast! expected))  ; the doctest
  ;;   run=false: skip (illustrative)
  …)
```
Returns the list of failures (empty = all green), `verify-stdlib`-style. **The gate**: a Rust test (mirror
deporder's gate) evals `(:wat::doctest::verify-examples)` and asserts the failure list is empty.

`(verify-examples)` is the one-liner-over-a-seam that R2 named — the surface that masks the depth.

### iv-b2-b probe (RED until b2-b)
The gate test: `(verify-examples)` returns empty failures (Bytes' `to-hex` `@example` evals to `"ff0010"`,
matches `#=>`; `from-hex` is `@example-norun`, skipped). RED until `wat/doctest.wat` + the seam exist.

## Blast radius
- b2-a: `src/intrinsic/mod.rs` (the seam fn + remove the `examples`/ExampleSubmission allows) + the
  runtime dispatch arm (register `:wat::intrinsic::examples`) + the probe. Bounded.
- b2-b: `wat/doctest.wat` (new) + `src/stdlib.rs` (STDLIB_FILES + load order) + the gate test. Bounded.

## STOP triggers
1. If the seam can't build a heterogeneous `Vector<Value>` cleanly (the universal-top element type),
   STOP and report — don't fall back to stringly-typed.
2. If `eval-ast!` rejects the parsed quoted form (a representation mismatch between `parse_one_with_file`
   output and what `eval-ast!` accepts), STOP and paste the exact error — that's the real integration risk.
3. If wiring `wat/doctest.wat` into the load order fights deporder, STOP and report the violation
   (don't reorder blindly).

## Fulfillment
When `(verify-examples)` runs green, R2's PREQUEL earns its "and it landed" close, and the dated allows
(at least `examples`) come off — satisfied by USE, as the discipline demands.
