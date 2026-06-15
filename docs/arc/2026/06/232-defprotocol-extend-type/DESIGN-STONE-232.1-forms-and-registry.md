# Arc 232 Stone 232.1 — defprotocol / extend-type: the forms + registry

> The foundation stone: parse `defprotocol` + `extend-type`, register them. **Registry only** —
> NO `assignable` satisfaction edge (232.2), NO method dispatch (232.3). After this stone a program
> may DECLARE a protocol and EXTEND a type to it; it cannot yet type a param as `:P` or call a
> protocol method. Grounded against HEAD `ef1c1462`. Single-receiver dispatch (builder-locked).

## Contract decision (the one pinned interface)

**`defprotocol`/`extend-type` are Rust special forms registered exactly like `defclause`** — parsed
by a `parse_*_form` fn → stored as a `Value` in `symbols.runtime_def_values` → mirrored into
`CheckEnv` by `from_symbols`. NOT wat macros: they populate checker/runtime registries the checker
reads, which a macro (expands-to-forms) cannot do. This mirrors the proven defclause mold
(`parse_defclause_form` → `Value::wat__core__clauses` → `register_defclause`/`get_defclause_clauses`).

## Surface (single-receiver; `self` is arg 0, typed `:P`)

```clojure
(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter  loudness <- :wat::core::i64] -> :wat::core::String))

(:wat::core::extend-type :t::Robot :t::Greeter
  (greet [self loudness] (:wat::core::string::concat "beep" "!")))
```

- **`defprotocol :P (m1 [self <- :P …] -> R1) (m2 …)`** — declares protocol `P` + its method
  signatures. Each method: name, arg types (arg 0 = the receiver, typed `:P`), return type.
- **`extend-type :T :P (m1 [self …] body) …)`** — registers that type `T` implements `P`: the
  satisfaction edge `T ⊑ :P` + the per-method impl bodies (parsed like defclause clause bodies).

## The registries (mirror defclause storage)

Check side (`CheckEnv`, alongside `defclause_registrations` at check/env.rs:296):
- `protocol_registrations: HashMap<String, Vec<ProtocolMethodSig>>` — `P → [(method, arg_types, ret)]`.
- `extend_registrations: HashMap<(String,String), Vec<String>>` — `(P, T) → [method names]` (the
  edge + which methods T provides; for 232.2's `assignable` the KEY's existence is the satisfaction).

Runtime side (`SymbolTable.runtime_def_values`): a new `Value` carrier for each (so `from_symbols`
can rebuild the check registries headlessly, the way `wat__core__clauses` does):
- protocol → a protocol-def Value; extend → an extend-def Value carrying the impl Functions
  (keyed by method name) for 232.3's dispatch to consume. Impl bodies parse via the defclause
  argspec/body machinery.

## Rooms (read in order)

1. `src/runtime.rs:1669` (`:wat::core::defclause` head dispatch) + `parse_defclause_form` — the
   parse+register entry to mirror for both new heads.
2. `src/freeze.rs:874-997` (preregister stubs + `register_stdlib_defclauses` + register_runtime_defs)
   — where stdlib/user forms get registered into `runtime_def_values`; add the two new heads.
3. `src/check.rs:8133` `register_defclause_from_form` + `collect_splice_defs_ctx` (8155, head match
   at 8173) — the check-side top-level collection; add `:wat::core::defprotocol` / `:wat::core::extend-type` arms.
4. `src/check/env.rs:279-306` `register_defclause`/`get_defclause_clauses` + `from_symbols`
   (loads defclause from `runtime_def_values`) — add the parallel protocol/extend registries +
   their `from_symbols` load.
5. `src/value/value.rs:342-401` (`Value::wat__core__clauses` + `ClauseSet`) — the model for the new
   Value carrier(s).

## Gate (RED at HEAD → GREEN after)

- **Probe (wat, observable):** `tests/probe_arc232_1_defprotocol_extend_register.rs` — a program
  that `defprotocol`s a protocol + `extend-type`s a record to it, with **no `:P` param and no method
  call**, must `startup_from_source` SUCCESSFULLY. RED at HEAD (`defprotocol` is an unknown call
  head → startup fails). GREEN once the forms parse + register.
- **Anti-fake (Rust registry assertion):** a unit test (mirror env.rs:324
  `from_symbols_loads_defclause_from_runtime_def_values`) builds the symbols, runs `from_symbols`,
  and asserts `protocol_registrations` holds the method sig + `extend_registrations` holds the
  `(P,T)` edge — so the forms can't be faked as parse-but-don't-register no-ops.
- lib 915/36 + nursery 895/4 (zero new) + workspace compiles.

## Scope / out (rejected here)

- **`assignable(T, :P)`** → 232.2. A `:P`-typed param does NOT yet accept an extender.
- **Method dispatch** (calling `(greet r 3)`) → 232.3. The impl bodies are stored, not wired.
- Default methods / protocol inheritance / Parametric protocols → out of arc 232 (DESIGN scope).

## STOP triggers (reject — surface; do not improvise)

- A protocol method's arg 0 is not the receiver typed `:P` → STOP (single-receiver invariant).
- The form needs a NEW `TypeExpr` variant → STOP (DESIGN says Path + registry; if that's false,
  surface it — don't add a variant unbriefed).
- The defclause body-parse machinery can't be reused for extend-type impls → STOP (report the gap).
