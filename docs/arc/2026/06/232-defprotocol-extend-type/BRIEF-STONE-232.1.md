# BRIEF — Stone 232.1: defprotocol + extend-type forms + registry

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo test`/`cargo build` PLAINLY (no setsid/timeout). Trust your
own build over rust-analyzer. **Do NOT commit — the Inquisitor weighs.**

## Work in one paragraph

Add two Rust special forms — `:wat::core::defprotocol` and `:wat::core::extend-type` — registered
exactly the way `:wat::core::defclause` is: a `parse_*_form` fn produces a registration, stored as a
`Value` in `symbols.runtime_def_values`, and mirrored into `CheckEnv` by `from_symbols`. This stone
is **registry only**: declare a protocol (its single-receiver method signatures) and extend a type
to it (the satisfaction edge + the impl bodies, stored for later). NO `assignable` edge (232.2), NO
method dispatch (232.3). After it, a program that declares + extends but neither types a param `:P`
nor calls a method must build.

## The model to copy (defclause — study FIRST)

`defclause` is the exact mold. Read, in order:
1. `src/runtime.rs:1669` (`:wat::core::defclause` head dispatch) + `parse_defclause_form` — parse +
   register entry. Mirror it for both new heads.
2. `src/value/value.rs:342-401` — `Value::wat__core__clauses(Arc<ClauseSet>)` + `ClauseSet`/`Clause`.
   Add the parallel carrier(s): a protocol-def Value (name + method sigs) and an extend-def Value
   (protocol, type, impl Functions keyed by method). Seal under `#[wat_value]` like `clauses`.
3. `src/check/env.rs:279-306` — `register_defclause`/`get_defclause_clauses` + the
   `defclause_registrations` field; `from_symbols` (loads defclause from `runtime_def_values`,
   tested at env.rs:324). Add `protocol_registrations: HashMap<String, Vec<ProtocolMethodSig>>` and
   `extend_registrations: HashMap<(String,String), Vec<String>>` (the `(P,T)` edge → method names),
   their register/get accessors, and their `from_symbols` load from the new Values.
4. `src/check.rs:8133` `register_defclause_from_form` + `collect_splice_defs_ctx` (8155; head match
   at 8173) — add `:wat::core::defprotocol` / `:wat::core::extend-type` arms that register check-side.
5. `src/freeze.rs:874-997` — preregister stubs + `register_stdlib_defclauses` + register_runtime_defs.
   Ensure both new heads register into `runtime_def_values` on the user path (same as defclause).

## Surface + shapes

```clojure
(:wat::core::defprotocol :t::Greeter
  (greet [self <- :t::Greeter  loudness <- :wat::core::i64] -> :wat::core::String))
(:wat::core::extend-type :t::Robot :t::Greeter
  (greet [self loudness] (:wat::core::string::concat "beep" "!")))
```

- **defprotocol**: head, protocol name keyword, then N method-sig lists. Each sig:
  `(method-name [self <- :P  arg1 <- :T1 …] -> :Ret)`. Parse the argspec via the SAME canonical
  argspec parser defclause/fn use (`src/macros/parse.rs:160` routes it). Store
  `ProtocolMethodSig { name, arg_types (Vec<TypeExpr>, arg0 = the receiver :P), ret: TypeExpr }`.
  **STOP if arg0 is not the receiver typed `:P`** (single-receiver invariant — builder-locked).
- **extend-type**: head, type-name keyword, protocol-name keyword, then N method impls. Each impl:
  `(method-name [self arg1 …] body)` — parse like a defclause clause (argspec + body → a Function).
  Store keyed by `(protocol, type, method)`. Register the satisfaction edge `(protocol, type)`.

## Gate (run all; report each verbatim)

```
cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register   # 1 passed (startup builds)
cargo test --release -p wat --lib -- --test-threads=1                            # 915/36 (zero NEW; baseline 36)
cargo test --release -p wat --test nursery -- --test-threads=1                   # 895/4 (zero NEW; baseline 4)
cargo test --release --workspace --no-run                                        # compiles
```
PLUS write an **anti-fake Rust unit test** next to `CheckEnv::from_symbols` (mirror env.rs:324
`from_symbols_loads_defclause_from_runtime_def_values`): build a `SymbolTable` with a protocol-def +
extend-def in `runtime_def_values`, run `from_symbols`, assert `protocol_registrations` holds the
`greet` sig and `extend_registrations` holds the `(:t::Greeter, :t::Robot)` edge. This proves the
forms actually populate the registries (not parse-but-drop no-ops).

## STOP triggers (REJECT — surface the gap; do not improvise)

1. A protocol method's arg 0 is not the receiver typed `:P` → STOP.
2. The form needs a NEW `TypeExpr` variant → STOP (DESIGN says Path + registry; surface if false).
3. The defclause argspec/body parser can't be reused for the method sigs/impls → STOP (report it).
4. You're tempted to wire `assignable(T,:P)` or method dispatch to make the probe pass → STOP — that
   is 232.2 / 232.3, OUT of this stone. The probe needs only parse + register.

## Blast radius

`src/runtime.rs`, `src/value/value.rs`, `src/check.rs`, `src/check/env.rs`, `src/freeze.rs` + the
two test files. No changes to `assignable`, the defclause dispatch path (check.rs:5416-5491), or any
existing form. New registries + two new heads only.

## Return

Report: each new Value carrier + registry field, the parse fns, the head-dispatch sites touched, the
two tests' results, every gate command's exact counts from YOUR runs, and any honest delta. Do NOT
commit.
