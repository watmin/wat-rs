# BRIEF — Stone 232.3: protocol-method dispatch (the keystone)

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo test`/`cargo build` PLAINLY (no setsid/timeout). Trust your
own build over rust-analyzer. **Do NOT commit — the Inquisitor weighs.**

## Work in one paragraph

Make protocol methods callable as `(<P>/<method> receiver args…)` — e.g. `(:t::Greeter/greet g 3)`.
TWO halves, landing together: (1) check-time — the call type-checks via the method's declared sig
(receiver assignable to `:P`, rest args against the sig, return the method's ret); (2) runtime — it
dispatches on the receiver's CONCRETE type via the extend registry, calling that type's impl. This
flips the keystone probe GREEN: a fn typed over `:P` calling a method dispatches on whatever
extender is passed (the host-agnostic-start shape).

## Decided (do not re-open)
- **Call form: `(<P>/<method> receiver args…)`** — namespaced under the protocol, mirroring Record
  accessors (`<fqdn>/<field>`). Split the head on the last `/` → `(protocol_fqdn, method_name)`.
- **Single-receiver:** dispatch on arg 0's concrete type only (builder-locked).

## The two halves

### Check-time — `src/check.rs`, beside the defclause call pre-check (~5458)
- A head `:P/method` where `P ∈ protocol_registrations` (from 232.1, `env.get_protocol_methods`) and
  `method` is one of P's sigs → a protocol-method call. (Disambiguate from Record accessors by
  protocol-registry membership — STOP-1 if they collide.)
- The sig is `[self <- :P, p1 <- :T1, …] -> :R`. Check arg 0 `assignable` to `:P` (REUSE 232.2's
  edge — `assignable`/`is_subtype` UNCHANGED), args 1.. against `T1…`, return `:R`. A `:P`-typed
  receiver is assignable to `:P` reflexively (so the forwarding shape inside `greet-it` type-checks).

### Runtime — `src/runtime.rs`, the keyword-head call dispatch
- Entry: `dispatch_keyword_head_value` (:3644) / the `runtime_def_values.get` resolution (:3443,
  :7366). Add a protocol-method branch BEFORE the generic "unresolved head" failure.
- Recognize `:P/method` (P in the protocol registry). Eval arg 0 (receiver); read its `class_fqdn`
  (`Value::wat__Record { class_fqdn }` / `wat__holon__Record { class_fqdn }`, :4847). `class_fqdn`
  has NO leading colon — reconcile to the keyword form (`format!(":{}", class_fqdn)`).
- Look up the impl: the `Value::wat__core__extend_def` for `(P, concrete_type)` — 232.1 stored it in
  `runtime_def_values` under the `extend:<P>:<T>` key (confirm the exact key format in
  `parse_extend_type_form`). Its `impl_clauses[method_name]` is a `Clause` (body + arg names; binder
  types are placeholder `:nil` — irrelevant at runtime). Eval `Clause.body` in a scope binding the
  clause's arg names to the evaluated args (receiver + rest). Return the result.
- **No impl for the concrete type → a CLEAN runtime error** ("type `:X` does not extend protocol
  `:P`") — never panic, never silent.

## Rooms (read in order)
1. `src/runtime.rs:3644` `dispatch_keyword_head_value` + `:3443`/`:7366` `runtime_def_values.get`.
2. `src/runtime.rs:4847-4851` — reading `class_fqdn` off the receiver.
3. `src/runtime.rs` `parse_extend_type_form` (232.1) — the `extend:<P>:<T>` key format + `ExtendDef`.
4. `src/check.rs:5458` (defclause pre-check) + `:2837` `canonical_callable_name`.
5. `src/check/env.rs` `get_protocol_methods`/`get_extend_methods` (232.1 registries).
6. `src/runtime.rs:906` — the `<fqdn>/<field>` accessor head precedent.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc232_3_protocol_dispatch        # 1 passed (beep + woof — dispatch on concrete type)
cargo test --release -p wat --test probe_arc232_2_protocol_assignable      # 2 passed (232.2 intact)
cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register # 1 passed (232.1 intact)
cargo test --release -p wat --lib -- --test-threads=1                       # 916/36 (zero NEW)
cargo test --release -p wat --test nursery -- --test-threads=1              # 895/4 (zero NEW)
cargo test --release --workspace --no-run                                   # compiles
```
Add a negative assertion (a non-extender receiver where `:P` is required, or a method call on a type
with no impl) → a clean error, not a panic.

## STOP triggers (REJECT — surface; do not improvise)
1. `:P/method` check-time resolution collides with the Record-accessor head path → STOP, surface the
   disambiguation (protocol-registry membership should distinguish).
2. The receiver's concrete type can't be read (a non-Record extender, e.g. a scalar) → STOP, report.
3. You need to change `assignable`/`is_subtype` or the 232.1 registries → STOP (232.3 CONSUMES them).
4. The 232.1 impl `Clause` lacks arg names or body for runtime eval → STOP, report.
5. Tempted to make the probe pass by special-casing the test's type names → STOP (dispatch must be
   general: ANY extender's concrete type selects its impl).

## Blast radius
`src/check.rs` (the method-call pre-check) + `src/runtime.rs` (the method dispatch). NO changes to
`assignable`, `is_subtype`, the 232.1 registries, or the 232.2 edge. The probe is already committed.

## Return
Report: the check-time arm + the runtime dispatch arm (file:line), how the receiver's concrete type
keys the impl lookup, the error path for a missing impl, every gate command's counts from YOUR runs,
and any honest delta. Do NOT commit.
