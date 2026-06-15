# BRIEF — Stone (232 follow-on): generic protocol method sigs

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo` PLAINLY (no setsid/timeout). Trust your own clean build over
rust-analyzer. **Do NOT commit — the Inquisitor weighs.** Report your work HONESTLY — describe what you
actually changed (the Inquisitor reads the diff). Full rationale: `DESIGN-STONE-generic-methods.md`.

## Work in one paragraph
A protocol method sig with a `<T>` type param (`make<T> [self x <- :T] -> Vector<T>`) must work like a
generic fn: collect the method's type params at parse, instantiate them to fresh vars at the call site.
Two edits + a struct field, all mirroring the existing generic-fn machinery. NOT parametric protocols.

## Rooms (read the generic-fn precedents first, then mirror)

1. **`src/value/value.rs:426` — `struct ProtocolMethodSig`.** Add `pub type_params: Vec<String>`
   (default empty). Fix every construction site (e.g. check/env.rs:444 test sig — give it `vec![]`).

2. **`src/runtime.rs` — `parse_defprotocol_form`** (the method-sig loop, ~5724, where `method_name`
   is read from `sig_items[0]` as a Symbol). Strip a `<T,…>` suffix off the method name into
   `type_params`, reusing the SAME `<T>`-suffix splitter `defn` uses on its `:name<T>` keyword
   (runtime.rs:2324 — `(name, raw_type_params) = …`; find the helper it calls and apply it to the
   method-name string). Register the method under the BARE name (`make`) with `type_params` populated.
   The `extend-type` impl bodies are UNCHANGED (they bind args positionally — `make [self x]`).

3. **`src/check.rs:5506-5571` — the protocol-method call-site check.** After finding `sig`, if
   `sig.type_params` is non-empty: build a substitution mapping each type-param name → a `fresh.fresh()`
   var, and apply it to `sig.arg_types[1..]` and `sig.ret` BEFORE the existing `assignable` checks and
   the return (mirror `instantiate`, check.rs:13942 — copy its freshening approach). The receiver check
   (arg 0 vs `:P`, line 5536) is unchanged. Empty `type_params` → take the current path verbatim
   (instantiation is a no-op). The returned type must be the INSTANTIATED `sig.ret` (so `T` resolves to
   the caller's type via the unifications the `assignable` calls perform).

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc232_generic_method                    # 1 passed (RED→GREEN: make<T> with T=i64)
cargo test --release -p wat --test probe_arc232_3_protocol_dispatch               # passes (MONOMORPHIC dispatch unbroken)
cargo test --release -p wat --test probe_arc232_2_protocol_assignable             # passes
cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register     # passes
cargo test --release -p wat --test probe_arc267_parametric_extend_type            # passes (267 unbroken)
cargo test --release -p wat --lib -- --test-threads=1                             # zero NEW vs baseline 917/36
cargo test --release -p wat --test nursery -- --test-threads=1                    # zero NEW vs baseline 895/4
cargo test --release --workspace --no-run                                         # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. The `<T>`-suffix splitter can't be reused for the method name (Symbol vs Keyword form mismatch) →
   STOP; report the exact form difference (don't hand-roll a divergent splitter — surface it).
2. Instantiation requires changing `instantiate` itself or the generic-fn path → STOP (this stone only
   ADDS a type_params field + a parse strip + a call-site freshen; it consumes `instantiate`'s approach,
   doesn't modify it).
3. A MONOMORPHIC protocol method (probe_arc232_3 `greet`) regresses → STOP (empty type_params must be
   a verbatim no-op).
4. Runtime dispatch (runtime.rs:4953) needs changes → STOP (type params are check-only; the runtime is
   unaffected).

## Blast radius
`src/value/value.rs` (one field + construction sites), `src/runtime.rs` (`parse_defprotocol_form` name
strip), `src/check.rs` (the call-site instantiation, 5506-5571). NO changes to `instantiate`, the
generic-fn path, `extend-type`, runtime dispatch, or the 267 arm. The probe is already committed.

## Return
Report: the `type_params` field + its construction sites; the parse name-strip (file:line) + which
splitter you reused; the call-site instantiation (file:line) + how you mirrored `instantiate`; every
gate command's counts from YOUR runs; confirm monomorphic dispatch (232_3) + 267 still pass; any honest
delta. If a STOP fires, STOP and report. Do NOT commit.
