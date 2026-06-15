# DESIGN — Stone (232 follow-on): generic protocol method signatures

> Opened 2026-06-15 as the arc-232-named follow-on — the 267 sibling. 232 shipped protocol method
> dispatch for MONOMORPHIC sigs; a method generic over the implementor's types wasn't built (232
> DESIGN.md scoped parametric protocols out "until a caller surfaces"). The arc-209 host seam is that
> caller: a host-agnostic `Host` launch method must mediate `listener' :Op :Reply` over an abstract
> host — inherently generic over the service's `:Op`/`:Reply`. Grounded against HEAD `2caf01f6`.

## The gap (grounded, two-part)

A protocol method sig with a free type var (`make<T> [self x <- :T] -> Vector<T>`) does not work:
- **PARSE:** `parse_defprotocol_form` (runtime.rs ~5724) reads the method name as a bare Symbol and
  does NOT strip a `<T>` suffix → the method registers under `make<T>` (or the sig breaks) →
  `(:Maker/make …)` is `UnknownCallee`. (Probe RED mode A.)
- **CHECK:** the call-site checker (check.rs:5506-5571) checks args against `sig.arg_types[i]` and
  returns `sig.ret` **directly — no instantiation** → a `:T` arg/ret is a literal `Path(":T")`
  (`expected :T, got :wat::core::i64`). (Probe RED mode B, seen when `<T>` is omitted.)

Generic **fns** already solve both: they collect type params from the `:name<T>` suffix
(`raw_type_params`, runtime.rs:2324 + the free-var union at 2400-2416) into a `TypeScheme.type_params`,
and **instantiate** them to fresh unification vars at every call (`instantiate`, check.rs:13942/5795).
Protocol methods must mirror this.

## The one contract decision

A protocol method may declare type params via `<T,…>` on its **method name** (exactly like a generic
fn's `:name<T>`). At the call site the method's type params instantiate to fresh unification vars,
unify with the arg types, and the **instantiated** return type is produced. Monomorphic methods (no
`<T>`) are unchanged (empty type_params → no-op instantiation). This is NOT parametric *protocols*
(`:P<T>` — still out); it's a generic *method* on a plain protocol — the same distinction 267 drew
for parametric *extenders* vs parametric *protocols*.

## The edits (mirror generic fns)

1. **`ProtocolMethodSig`** (src/value/value.rs:426): add `type_params: Vec<String>` (default empty).
2. **`parse_defprotocol_form`** (src/runtime.rs): strip the method name's `<T,…>` suffix → (name,
   type_params), reusing the SAME splitter `defn` uses on `:name<T>` (runtime.rs:2324). Store
   `type_params` on the `ProtocolMethodSig`. (Optionally union with free bare type-vars in the sig,
   mirroring 2400-2416 — but the explicit `<T>` suffix is the contract; decide at strike.) The
   `extend-type` impl bodies are UNCHANGED — they bind the args positionally (`make [self x]`), no
   type-param decl needed.
3. **protocol-method call check** (src/check.rs:5506-5571): if `sig.type_params` is non-empty, build a
   fresh-var substitution (one `fresh.fresh()` per type param) and apply it to `sig.arg_types[1..]` +
   `sig.ret` BEFORE the `assignable` checks + the return — mirror `instantiate` (check.rs:13942). The
   receiver check (arg 0 vs `:P`) is unchanged. Monomorphic sigs (empty type_params) take the current
   path verbatim.

## Scope / out

- **Parametric protocols** (`:P<T>`) — still out (no caller).
- **Runtime dispatch** — UNCHANGED. Type params are a check-time concern; the runtime extend-registry
  dispatch (runtime.rs:4953, on the receiver's concrete type) is unaffected (the impl body runs the
  same regardless of the static type instantiation).
- **No change** to `extend-type` parsing, `instantiate` itself, or generic-fn handling.

## Probe

`tests/probe_arc232_generic_method.rs` (committed RED) — `(:t::Maker (make<T> [self x <- :T] ->
Vector<T>))`, extend onto `:t::Dup`, call `(:t::Maker/make (:t::Dup) 5)` → expect `T=i64` → `nth 0 =
5`. RED at HEAD (`UnknownCallee` — `<T>` unstripped). GREEN once parse collects + check instantiates.
This unblocks arc-209 stone 4a (the `Host` protocol's generic launch method).
