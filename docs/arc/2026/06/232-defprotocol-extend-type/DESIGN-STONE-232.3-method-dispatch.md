# Arc 232 Stone 232.3 — protocol-method dispatch (the keystone)

> 232.1 registered the forms; 232.2 made `:P` a usable bound. 232.3 makes protocol methods
> CALLABLE: `(<P>/<method> receiver args…)` type-checks via the method's declared sig and dispatches
> at RUNTIME on the receiver's concrete type via the extend registry. This flips the forwarding
> shape — a fn typed over `:P` calling a method, dispatching on whatever extender is passed — GREEN.
> That shape is exactly the host-agnostic `start`. Grounded against HEAD `5764cc20`. Single-receiver.

## The call form (DECISION — four-Q; flag for builder override)

**`(<P>/<method> receiver arg1…)`** — namespaced under the protocol, e.g. `(:t::Greeter/greet g 3)`.
Mirrors Record/struct accessors (`<fqdn>/<field>`, runtime.rs:906) — wat heads are namespaced
keywords; a bare `(greet g 3)` symbol head would need symbol-head resolution + a global method
table with cross-protocol collision handling. Four-Q: Obvious (accessor precedent), Simple (split
head on last `/` → (P, method)), Honest (protocol named at the call; no global-name collision),
Good-UX (consistent, unambiguous). **Decided namespaced; surfaced for override** (Clojure uses bare
— if parity demands it, that's a bigger symbol-head-dispatch design; namespaced is the wat-honest
default and all the registries from 232.1 are keyed to support it).

## The two halves (both land together — the gate needs check + runtime)

### Check-time (src/check.rs, beside the defclause pre-check ~5458)
A call head `:P/method` where `P ∈ protocol_registrations` and `method` is one of P's sigs is a
protocol-method call. Resolve:
1. Split the head on the last `/` → `(protocol_fqdn, method_name)`. Look up `protocol_registrations`.
2. The method sig is `[self <- :P, p1 <- :T1, …] -> :R` (from 232.1).
3. Check arg 0 (the receiver) is `assignable` to `:P` — this REUSES 232.2's edge (`is_subtype` over
   the extend graph). Check args 1.. against `T1…`. Return `:R`.
4. (A `:P`-typed receiver — e.g. inside `start [host <- :Host]` — is assignable to `:P` reflexively,
   so the forwarding shape type-checks; a concrete extender is assignable via the edge.)

### Runtime (src/runtime.rs, the keyword-head call dispatch — `dispatch_keyword_head_value` :3644 /
`runtime_def_values` resolution :7366)
1. Recognize a `:P/method` head that is a protocol method (P in the protocol registry).
2. Eval arg 0 (the receiver); read its concrete type — `Value::wat__Record { class_fqdn }` /
   `wat__holon__Record { class_fqdn }` (:4847). `class_fqdn` has NO leading colon; the extend key
   uses the keyword form — reconcile with `format!(":{}", class_fqdn)`.
3. Look up the impl: the `Value::wat__core__extend_def` for `(P, concrete_type)` (232.1 stored it in
   `runtime_def_values` under the `extend:<P>:<T>` key) → its `impl_clauses[method_name]` (a
   `Clause`, body + arg names; the binder TYPES are placeholder `:nil` from 232.1 — irrelevant at
   runtime, only names bind).
4. Eval the impl `Clause.body` in a scope binding the clause's arg names to the evaluated args
   (receiver + rest). Return the result.
5. **No impl for that concrete type → a clean runtime error** (`NoProtocolImpl` or reuse an existing
   shape): "type `:X` does not extend protocol `:P`" — never a panic, never silent.

## Rooms (read in order)

1. `src/runtime.rs:3644` `dispatch_keyword_head_value` + `:3443`/`:7366` `runtime_def_values.get` —
   the call-head resolution entry; add the protocol-method branch BEFORE/beside the generic lookup.
2. `src/runtime.rs:4847-4851` — reading `class_fqdn` off a receiver Value (the dispatch key).
3. `src/check.rs:5458` (defclause call pre-check) + `:2837` `canonical_callable_name` — the model +
   head normalization for the check-time protocol-method pre-check.
4. `src/check/env.rs` `get_protocol_methods` / `get_extend_methods` (232.1) — the registries to read.
5. `src/runtime.rs:906` (accessor `<fqdn>/<field>` synthesis) — the namespaced-head precedent.
6. `src/value/value.rs` `ProtocolMethodSig` / `ExtendDef.impl_clauses` (232.1) — the stored data.

## Gate (RED at HEAD → GREEN after) — the KEYSTONE probe

`tests/probe_arc232_3_protocol_dispatch.rs`: defprotocol `:t::Greeter` with `greet`; extend BOTH
`:t::Robot` ("beep") and `:t::Dog` ("woof"); a fn `greet-it [g <- :t::Greeter] -> :String` that
calls `(:t::Greeter/greet g 3)`; assert `(greet-it (:t::Robot)) == "beep"` AND `(greet-it (:t::Dog))
== "woof"` — proving (a) the `:P`-bound forwarding type-checks and (b) dispatch picks the impl by the
receiver's CONCRETE type. RED at HEAD (`:t::Greeter/greet` unresolved). GREEN after 232.3. Plus: a
non-extender receiver → clean error (negative). + lib 916/36, nursery 895/4 (zero new), compiles.

## Scope / out (rejected here)

- Default methods / multi-arg dispatch → out (single-receiver locked; defaults are a later arc).
- The host consumer (`Host`/`SpawnHandle`/`Endpoint`/agnostic `start`) → arc 209 resumes AFTER this.
- Migrating existing defclause kernel intrinsics to protocols (arc 256, banked) → separate.

## STOP triggers (reject — surface; do not improvise)

1. The check-time `:P/method` resolution collides with the Record-accessor head path (`<fqdn>/<field>`)
   → STOP and surface the disambiguation (protocol-registry membership should distinguish them).
2. The receiver's concrete type can't be read for a non-Record extender (e.g. a scalar extends `:P`)
   → STOP (single-receiver dispatch assumes the receiver carries a class; surface the case).
3. You need to change `assignable`/`is_subtype` or the 232.1 registries → STOP (232.3 consumes them;
   the check side reuses 232.2's edge unchanged).
4. The impl `Clause` from 232.1 lacks what runtime eval needs (arg names / body) → STOP and report
   (it should carry both).
