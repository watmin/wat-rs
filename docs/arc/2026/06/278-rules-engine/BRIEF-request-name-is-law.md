# BRIEF — #74b: `<Op>Request` is LAW too. The other half of the pair.

Anchor your cwd at `/home/watmin/work/holon/wat-rs/`; verify with `pwd` first, and use
`git -C /home/watmin/work/holon/wat-rs` for any git read. The working tree is ALREADY DIRTY with
the `<Op>Response` half of this ruling — that is expected and you build on top of it. Do not
revert, stash, or clean anything.

## Why this exists, and it is not a tidy-up

The builder's ruling was *"convention is law — enforce it… services are our OOP layer, we make
**requests** to them and get **responses** back,"* with `SayRequest` / `SayResponse` as the worked
example. The strike that just landed enforced only the Response half, because only the Response
name was ever *guessed* by the codegen — the Request side already reaches its type through the
surface-minted `<S>::<op>/Request` alias, read from the declaration. So there was no bug to cure
and the stone never looked at it.

**A half-enforced pair is worse than none, and the corpus already proves it.**
`wat-scripts/probes/arc-170/probe-strikeB-fields.wat` declared one enum `:probe::Kv::R` serving as
BOTH request and response. Satisfying the Response law renamed it to `GetResponse`, so the file now
reads:

```clojure
:features [(get [self <- :probe::Kv  req <- :probe::Kv::GetResponse] -> :probe::Kv::GetResponse …)]
```

A request parameter typed `GetResponse`, and the checker is happy. The one-sided law did not merely
fail to catch that — it **pushed** the name there. That is what you are closing.

## Read in order

1. `src/types.rs`, `synthesize_surface_protocol` — the `<Op>Response` check that just landed (search
   `response type name is LAW`). Yours is its mirror; copy its shape exactly, including the
   base-name normalization and the both-names error text.
2. `src/types.rs:2797` — `let request_ty = match args.fixed_params.get(1) { … None => return
   Ok(vec![]) }`. Your check goes **immediately after this binding**, using `request_ty`.
3. `docs/arc/2026/06/278-rules-engine/BRIEF-response-name-is-law.md` — the sibling brief, for the
   STOPs and the verify discipline, which are unchanged.

## The check

```rust
// #74b — `<Op>Request` is LAW, the twin of the `<Op>Response` rule above. Same gate
// (`enforce_rtl_lock`), same base-name comparison, same both-names diagnostic.
if enforce_rtl_lock {
    let declared_base: String = match &request_ty {
        TypeExpr::Path(p) => p.clone(),
        TypeExpr::Parametric { head, .. } => format!(":{head}"),
        other => /* refuse, located, naming what was declared */,
    };
    let required = format!(
        "{surface_base}::{}Request",
        crate::string_ops::kebab_to_pascal_with_acronyms(name, ns_acronyms),
    );
    if declared_base != required { /* refuse, located, naming BOTH names */ }
}
```

**Placement is load-bearing and not a style choice.** The Response check sits *before* the
request-arg bail; yours sits *after* it. That ordering is required for two reasons: an op with no
request arg has no request to name, and
`tests/services/probe_arc278_repl_durable_forms_response_law.wat.bad` violates **both** laws — its
committed test asserts the *Response* message verbatim, so the Response check must fire first or
that test breaks. Do not reorder them.

## Migrate the eight rows

Arm the check and read what screams. Six of these eight will actually fire; the two marked ⓜ live
inside `defmacro` bodies realized only by runtime `macroexpand`, so `register_types` never sees them
(the same class as the sibling brief's "out of scope" case) — migrate them anyway for consistency,
but do not expect a RED→GREEN transition to confirm them, and say so in your report.

| file | surface | op | declared request | do |
|---|---|---|---|---|
| `probes/arc-170/probe-kwargs-peer.wat` | `:probe::Echo` | `echo` | `::Req` | → `::EchoRequest` |
| `probes/arc-170/probe-kwargs-peer.wat` | `:probe::Kv` | `get` | `::GetReq` | → `::GetRequest` |
| `probes/arc-170/probe-strikeB-fields.wat` | `:probe::Kv` | `get` | `::GetResponse` | ★ see below |
| `probes/arc-170/probe-surface-ships.wat` | `:probe::Foo` | `f` | `::Req` | → `::FRequest` |
| `probes/arc-170/scout-kwargs-expand.wat` | `:probe::Kv` | `get` | `::GetReq` | → `::GetRequest` |
| `scratch-pad/probe-sift-rules-stop1-bare-defsurface.wat` | `:probe::Bare` | `echo` | `::Req` | → `::EchoRequest` |
| `scratch-pad/probe-sift-rules-stop1-dump.wat` ⓜ | `:probe::Bare2` | `echo` | `::Req` | → `::EchoRequest` |
| `scratch-pad/probe-sift-rules-stop1-dump.wat` ⓜ | `:probe::Wrapped` | `echo` | `::Req` | → `::EchoRequest` |

All paths are relative to `wat-scripts/`.

**★ `probe-strikeB-fields.wat` is the one that is not a rename.** It has ONE type doing both jobs.
Under the pair law that is no longer expressible, so it needs a genuine second type: add a
`:probe::Kv::GetRequest` (a `defrecord` is the right shape for a request — carry one field; the
file's subject is `field-names-of` reflection on `:probe::Bag`, not the payload) and point `req <-`
at it, leaving `:probe::Kv::GetResponse` as the response only. Its `:user::main` may need the
construction updated to match.

`tests/services/probe_arc278_repl_durable_forms_response_law.wat.bad` also violates the Request law
(`EvalRequest` for op `eval-src`). **Leave it exactly as it is** — it is a deliberate negative
control, already refused on the Response axis, and its `.wat` must stay illegal.

## ⛔ STOPs — rejection criteria, not permission slots

- **⛔ STOP-1 — if the check refuses anything outside the eight-row table, STOP** and report the
  extra sites verbatim. The table is a census taken 2026-08-05 against the current dirty tree.
- **⛔ STOP-2 — if any `wat/` stdlib file or anything under `crates/` is refused, STOP.** The census
  says zero stdlib and zero production violate the law.
- **⛔ STOP-3 — do NOT touch `types.rs`'s `if let TypeExpr::Path(resp_path) = ret` (the ruling-A
  SHAPE lock).** Known parametric hole, filed as task #76, ruling pending, explicitly not yours.
- **⛔ STOP-4 — do NOT reorder the Response and Request checks.** See the placement note above.
- **⛔ Do not add a `_` wildcard arm on an enum scrutinee.** Doctrine.
- **⛔ Do not commit, stash, push, or touch git.** Leave the tree dirty; the orchestrator weighs.

## Verify — FOREGROUND, and block on it

```
cargo build --release
target/release/wat --check <each file you touched>
cargo nextest run --release
cargo clippy --release --all-targets
```

`cargo build --release` going green proves nothing here — the bake does not run the corpus sweep.
Read the **Summary line**; never a piped exit code. **The floor to match is
`4347 run / 4347 passed / 0 failed / 262 skipped`** (this is one lower than the pre-#74 floor of
4348 for a reason already accounted for; do not "restore" it).

---

## EXPECTATIONS — written before the strike

| # | what | command | expected |
|---|---|---|---|
| 1 | the law is real | a fresh serviceable op whose request is misnamed | REFUSED, located, both names printed |
| 2 | **not vacuous** | the same file, name corrected | ACCEPTED, silent |
| 3 | does not over-reach | `probes/arc-170/probe-c2-narrow-multisurface.wat` (a `:nature :wat::core::Struct` surface) | ACCEPTED |
| 4 | ★ acronym rule | `tests/macros/probe_arc265_acronym_registry_svc.wat` (`CreateWebACLRequest`) | ACCEPTED — a naive pascal-caser refuses this |
| 5 | ★ base name, not rendered type | `wat-tests/service-parametric-messages.wat` (`GetRequest<K,V>`) | ACCEPTED |
| 6 | ordering preserved | `cargo nextest run --release -E 'test(repl_durable_forms)'` | green — still refused on the RESPONSE message, unchanged |
| 7 | census right | everything the armed check refused | exactly six firing rows; the two ⓜ rows silent |
| 8 | ★ the collision is gone | `grep 'req <- .*Response' wat-scripts/probes/arc-170/probe-strikeB-fields.wat` | zero hits |
| 9 | ★ floor | `cargo nextest run --release` Summary line | `4347 / 4347 / 0 / 262` |
| 10 | clippy | `cargo clippy --release --all-targets` | clean |

Rows 4, 5, 8 and 9 are re-run by the orchestrator by hand regardless of what is reported.

**Runtime prediction: 15–25 minutes.** Time-box 50.

**Trap doors:** the acronym converter (row 4); base-name-vs-rendered-type (row 5, #75's class, three
instances in three days); the `Path`-carries-a-colon / `Parametric`-does-not asymmetry; and the
check-ordering (row 6) — placing the Request check before the request-arg bail breaks a committed
assertion.
