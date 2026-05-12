# Arc 170 slice 3 phase D — SCORE (Layer 2 `run-hermetic-with-io` macro)

**Date:** 2026-05-11
**Branch:** arc-170-program-entry-points
**Status:** complete

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `:wat::test::run-hermetic-with-io` macro in `wat/test.wat` | `grep -n "wat::test::run-hermetic-with-io\b" wat/test.wat` shows the defmacro | PASS — line 757 |
| B | `:wat::test::run-hermetic-with-io-driver` helper in `wat/test.wat` | grep | PASS — line 684 |
| C | `:wat::test::RunResultIO<O>` struct registered | grep in src/types.rs + `cargo test` | PASS — line 974 in src/types.rs; struct fields visible via register_struct_methods |
| D | T18 happy-path round-trip + T18b failing-assertion both pass | `cargo test --release --test wat_arc170_program_contracts t18` → 2 passed 0 failed | PASS |
| E | Workspace stays at 0 failed (count rises from 2182 to 2184) | `cargo test --release --workspace --no-fail-fast` → 2184 passed / 0 failed | PASS |
| F | `cargo check --release` green | clean compile | PASS |
| G | SCORE explains Decisions 1/2/3 outcomes + honest deltas | this file | PASS |

**All 7 rows pass.**

## Decision 1 — macro type-param shape (D1)

**Chosen: Option A.1 — full channel-type keywords (4 args total: rx-type, tx-type, inputs, body)**

The aspirational form from the DESIGN:
```
(:run-hermetic-with-io :wat::core::i64 :wat::core::i64 inputs body)
```

requires the macro to CONSTRUCT `Receiver<i64>` and `Sender<i64>` keywords from the inner element types at macro-expand time. This is not possible:

1. **The fn signature requires the full keyword**: `parse_fn_signature` (src/runtime.rs:4259) requires each parameter type to be a `WatAST::Keyword`. The keyword `:wat::kernel::Receiver<wat::core::i64>` is a single keyword string with the parametric type embedded. There is no way to construct this keyword from two pieces (`":wat::kernel::Receiver<"` + `":wat::core::i64"` + `">"`) at macro-expand time.

2. **No `keyword::from-string` verb exists**: The substrate exposes no verb to convert a string to a keyword value at runtime. `value_to_watast` (src/runtime.rs:7884) converts `Value::wat__core__keyword` back to `WatAST::Keyword`, but there is no substrate verb to produce a keyword value from a computed string.

3. **Arc 143 slice 2 computed unquote does not help**: While `~(:verb arg1 arg2)` evaluates a callable expression at expand-time, the result must be a value convertible to `WatAST` via `value_to_watast`. A concatenated string would produce `WatAST::StringLit`, not `WatAST::Keyword`, which fails at the fn parameter type slot.

**What was chosen**: The macro takes the FULL channel-type keywords directly:
```
(:run-hermetic-with-io
  :wat::kernel::Receiver<wat::core::i64>   ;; rx param type (full keyword)
  :wat::kernel::Sender<wat::core::i64>     ;; tx param type (full keyword)
  inputs
  body)
```

The quasiquote substitutes `~rx-type` and `~tx-type` directly into the fn parameter vector positions. Since these are already-complete type keywords, `parse_fn_signature` accepts them correctly.

**No separate `o-type` arg needed**: The driver function is generic `<I,O>`. The type checker infers O from the Process's typed channels (O flows from the Sender<O> embedded in tx-type). The driver body uses `(:wat::core::Vector :O)` for the drain accumulator, which mirrors stream.wat's `(:wat::core::Vector :T)` pattern for generic accumulators. No runtime O value is needed — `:O` is just a placeholder keyword.

**Substrate gap documented**: To support the aspirational inner-type-only form, the substrate needs a `keyword::concat` or `keyword::from-string` verb that produces a `Value::wat__core__keyword` from a computed string. This is a genuine substrate gap, not a workaround.

## Decision 2 — RunResultIO struct registration mechanism (D2)

**Chosen: Rust-side StructDef in `src/types.rs`**

`RunResultIO<O>` is registered via `env.register_builtin(TypeDef::Struct(StructDef { ... }))` in `src/types.rs`, mirroring `RunResult`'s registration. The struct has one type parameter `O` and three fields:
- `outputs :Vector<O>` — typed channel outputs from the child
- `stderr :Vector<String>` — raw stderr lines for diagnostic
- `failure :Option<Failure>` — None on success, Some on child panic

**Why Rust-side over wat-side**:

1. A wat-side `:struct` form (user-space struct declaration) would require load ordering guarantees — `test.wat` would need to declare the struct BEFORE using it, and the stdlib load path must include a struct registration step. The existing pattern (all builtin types in `src/types.rs`, auto-generated accessors via `register_struct_methods` at freeze time) avoids this complexity.

2. Consistency with `RunResult` (Phase C): Phase C used RunResult which is Rust-side. Using the same mechanism for RunResultIO maintains discipline — both test result types are substrate-registered.

3. `register_struct_methods` auto-generates `RunResultIO/new` + per-field accessors (`RunResultIO/outputs`, `RunResultIO/stderr`, `RunResultIO/failure`) without additional code. The wat-side `struct-new :wat::test::RunResultIO` constructor works immediately after registration.

## Decision 3 — send/drain ordering (D3)

**Chosen: Sequential — send all inputs → drain all outputs → join → drain stderr**

The driver implementation:
```
tx   ← Process/tx proc
_    ← run-hermetic-send-inputs tx inputs       ;; 1. send all
rx   ← Process/rx proc
out  ← run-hermetic-drain-outputs rx []         ;; 2. drain all (blocks until child exits)
res  ← Process/join-result proc                 ;; 3. join (child already exited)
err  ← Process/stderr proc
lines← drain-lines err                          ;; 4. drain stderr
```

**Why this ordering is correct for T18**: The child (T18) reads exactly ONE value, sends ONE value, and exits. The flow:
1. Parent sends 21 into the pipe buffer (non-blocking; small value)
2. Parent calls drain-outputs (blocks on recv)
3. Child (in forked process) reads 21, computes 42, sends 42 via tx, exits
4. Child exit drops its tx → parent's rx sees Ok(None) (EOF)
5. Drain-outputs returns [42]
6. Join-result returns immediately (child already exited — waitpid)
7. Stderr drain is safe (child already exited, wrote everything)

**Honest delta — deadlock scenario not covered by sequential pattern**:

If a child body reads rx to EOF (loops until `Ok(None)`), the parent's tx must be dropped (closed) before the child can detect EOF. In wat's function-scope binding model, the Sender Value (parent's tx) lives for the entire driver function scope — there is no explicit `Sender/close` verb. The parent's tx being open means the child's rx never sees `Ok(None)`, and the child loops forever waiting for more input.

For T18, this does NOT apply: the child reads one value and exits without waiting for EOF on rx. The sequential pattern is sufficient.

For child patterns that read to EOF, a threaded drain (concurrent send + recv in parallel threads) or an explicit substrate `Sender/close` verb is needed. This is a known D3 limitation, consistent with Phase C's Delta 3 (which documented the same gap for the IOWriter close).

**T18b outputs are empty**: When the child panics before calling `send tx n`, the parent's drain-outputs gets `Ok(None)` immediately (child exited, tx dropped by panic unwind). The outputs Vec is `[]`. T18b verifies this: outputs.len() == 0, failure == Some(Failure).

## Honest deltas

### Delta 1: D1 substrate gap — `keyword::from-string` absent

The aspirational `(run-hermetic-with-io :i64 :i64 inputs body)` form with inner element types requires constructing parametric type keywords at macro-expand time. No `keyword::from-string`, `keyword::concat`, or equivalent verb exists in the substrate. This is a genuine gap. The chosen Option A.1 (full channel-type keywords) works but is more verbose for the caller. A future substrate arc adding `keyword::concat` would enable the ergonomic form.

### Delta 2: `Option/expect -> :I` as the unwrap mechanism for `first`

`(:wat::core::first vec)` returns `Option<T>` (arc 047 honest absence design — empty/short is a runtime fact; the signature surfaces it honestly). The send-inputs helper must unwrap via `Option/expect -> :I` before passing to `send`. This was discovered during the first compile attempt (type error: `send: parameter #2 expects :I; got :Option<I>`). The fix uses `Option/expect -> :I` with a diagnostic message. The `-> :I` type annotation uses the generic function's own type parameter `I` as the declared type — this compiles correctly because `:I` parses as `TypeExpr::Path(":I")` which unifies with the vector's element type.

### Delta 3: No `Sender/close` verb for EOF signaling

The driver cannot close the parent's tx to signal EOF to the child's rx. This is the same gap Phase C documented in Delta 3 (for IOWriter/close). For Layer 2, the gap matters when child bodies read rx to EOF. For T18's single-recv pattern it does not matter. Future arcs that need EOF signaling require either: (a) a substrate `Sender/close` verb, or (b) a threaded drain pattern where sender/drain run concurrently.

### Delta 4: Two helpers (send-inputs + drain-outputs) not one

The BRIEF sketched a single helper shape. Two separate helpers emerged naturally:
- `run-hermetic-send-inputs<I>` — generic over I, handles the send loop
- `run-hermetic-drain-outputs<O>` — generic over O, handles the recv drain (mirrors stream.wat's `collect-drain<T>`)

This is the correct decomposition per the "one let* per function" feedback principle — each concern gets its own named function. The driver composes them cleanly.

### Delta 5: T18b outputs Vec is empty (child panicked before send)

When the child panics during `assert-eq`, the tx is dropped by the panic unwind before `send tx n` executes. The parent's drain-outputs sees `Ok(None)` immediately. T18b verifies and documents this: `outputs.len() == 0`. This is the honest behavior — structured failure over empty outputs.

## Files modified

| File | Change |
|------|--------|
| `src/types.rs` | Appended `:wat::test::RunResultIO<O>` struct registration (lines 952-990). No existing registration modified. |
| `wat/test.wat` | Appended Layer 2 section: `run-hermetic-send-inputs<I>` helper, `run-hermetic-drain-outputs<O>` helper, `run-hermetic-with-io-driver<I,O>` driver, `run-hermetic-with-io` defmacro (lines 571-769). No existing form modified. |
| `tests/wat_arc170_program_contracts.rs` | Appended T18 (happy-path echo-doubled) + T18b (failing assertion inside body) before T16. |

## Implementation shape (final)

### RunResultIO<O> — `src/types.rs`

```rust
StructDef {
    name: ":wat::test::RunResultIO",
    type_params: vec!["O"],
    fields: [
        ("outputs", Parametric { head: "wat::core::Vector", args: [Path(":O")] }),
        ("stderr",  Parametric { head: "wat::core::Vector", args: [Path(":wat::core::String")] }),
        ("failure", Parametric { head: "wat::core::Option", args: [Path(":wat::kernel::Failure")] }),
    ],
}
```

### Helpers + driver — `wat/test.wat`

Three helpers compose the Layer 2 driver:

1. `run-hermetic-send-inputs<I>` — tail-recursive send loop; unwraps `first` via `Option/expect`.
2. `run-hermetic-drain-outputs<O>` — tail-recursive recv drain; mirrors `stream.collect-drain<T>`.
3. `run-hermetic-with-io-driver<I,O>` — orchestrator: send → drain → join → drain-stderr → build RunResultIO.

### Macro — `wat/test.wat`

```scheme
(:wat::core::defmacro
  (:wat::test::run-hermetic-with-io
    (rx-type :AST<wat::core::nil>)
    (tx-type :AST<wat::core::nil>)
    (inputs  :AST<wat::core::nil>)
    (body    :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-hermetic-with-io-driver
     (:wat::kernel::spawn-process
       (:wat::core::fn
         [rx <- ~rx-type
          tx <- ~tx-type]
         -> :wat::core::nil
         ~body))
     ~inputs))
```

### Canonical test surface form exercised by T18

```scheme
(:wat::core::define (:my::test::echo-doubled -> :wat::test::RunResultIO<wat::core::i64>)
  (:wat::test::run-hermetic-with-io
    :wat::kernel::Receiver<wat::core::i64>
    :wat::kernel::Sender<wat::core::i64>
    (:wat::core::Vector :wat::core::i64 21)
    (:wat::core::let
      [n (:wat::core::Option/expect -> :wat::core::i64
            (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
              (:wat::kernel::recv rx) "recv failed")
            "stream closed")
       _ (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send tx (:wat::core::i64::*'2 n 2)) "send failed")]
      :wat::core::nil)))
```

## What's next — Phase E path (consumer sweep)

Phase E authors the `deftest` / `deftest-hermetic` consumer sweep — migrating existing callers from `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` to the appropriate Layer 1 or Layer 2 entry point. Expected distribution: most → Layer 1 (`run-hermetic`); some → Layer 2 (`run-hermetic-with-io`).

After Phase E completes all migrations, Phase F retires `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` verbs. Slice 4 then retires the BareLegacy* walker variants and the Process<I,O> legacy byte-pipe fields (stdin/stdout/stderr), closing arc 170.

The D1 substrate gap (no `keyword::from-string`) surfaces as an ergonomic improvement candidate: once a `keyword::concat` or similar verb ships, the macro can accept inner element types `(:run-hermetic-with-io :i64 :i64 ...)` instead of full channel type keywords. This does not block Phase E or F.
