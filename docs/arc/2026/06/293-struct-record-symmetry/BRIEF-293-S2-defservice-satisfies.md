# BRIEF — 293 S2: `defservice … :satisfies <surface>` (the service wears the surface's protocol)

> **Executor: one sonnet SHADOWDANCER.** A **wat-macro** strike (extend `wat/service.wat`). S1 is DONE + weighed
> green (`b13cab8c`): a surface with pure method members already synthesizes `<S>::Op`/`<S>::Reply` (enums) +
> references the user-declared request/response records. S2 makes a `defservice` **use** that synthesized protocol
> instead of minting its own. Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/`
> illegal). `cargo build`; `cargo nextest run --release` (NEVER `cargo test`); `./target/release/cargo-wat <f>`.
> **Commit NOTHING.** Full design: `DESIGN-293-services-as-surfaces.md` (read § "The mechanism" + § "S1 — DRAWN").

## The work (one paragraph)

Add a `:satisfies <surface>` clause to the `defservice` macro. When present, the service **does NOT mint** its own
`{fqdn}::Op`/`{fqdn}::Reply`/request/response — it **references the surface's synthesized `<S>::Op`/`<S>::Reply`** (S1)
and takes **`:impls` (bodies only)** in place of `:ops` (whose signatures now come from the surface). The macro
generates the serve-op-arms (dispatch), the per-op client methods, and the `Handle`/`Address'`/`Peer'` types over the
surface's `<S>::Op`/`<S>::Reply`. **Result:** every service that `:satisfies` the same surface speaks the *identical*
protocol and shares one `Address'<S::Op, S::Reply>` type — the thing that lets a client (later, S3) dial any of them blind.

## Why the validation is FREE (the load-bearing insight — no Rust, no separate checker)

The serve loop is a `match` over the `Op` enum (`wat/service.wat:600`, `serve-op-arms`). Point it at `<S>::Op` and read
the arms from `:impls`; then:
- **impls-cover-all-ops** is enforced by **match exhaustiveness** — a missing `:impl` leaves a `<S>::Op` variant
  unhandled → non-exhaustive match → compile error (R29). No coverage check to write.
- **each impl's request/response types match the surface** is enforced by the **variant field types** — the serve-arm
  binds `req` from `<S>::Op::<Op>`'s `req` field (typed as the surface's request record, from S1), and the reply is
  wrapped in `<S>::Reply::<Op>` (which demands the surface's response record). A wrong-typed impl fails to type-check.

So S2 is a wat-macro change alone; the type system is the validator.

## The map (`:impls` → dispatch, per impl)

For each `:impl` `(<op> [s req] …body… (:wat::service::Outcome::Reply <new-state> (<S>::<Op>Response …)))`:
- **`s`** binds the `:State` (the server's "self" — the surface method's `self` maps to the service State). **`req`**
  binds the request record (from `<S>::Op::<PascalCase(op)>`'s `req` field).
- serve-op-arm: `match` `<S>::Op::<PascalCase(op)>` → run the body → the body's `Outcome::Reply new-state <Response>` →
  reply `<S>::Reply::<PascalCase(op)> <Response>` (the macro wraps the Response record in the Reply variant, exactly as
  `:ops` does today — see mem.wat's `Outcome::Reply … (…PutResponse true)`).
- client method: `<S>/<op> peer (<S>::<Op>Request …)` → sends `<S>::Op::<Op>` → recvs `<S>::Reply::<Op>` → returns the
  Response record.
- **PascalCase(op)** via the existing conversion (S1 used `kebab_to_pascal_with_acronyms`); `<Op>Request`/`<Op>Response`
  are the surface's user-declared records (S1 wires them into the `Op`/`Reply` variants).

## Read the rooms, in order
1. **`DESIGN-293-services-as-surfaces.md`** § The mechanism + § S1 — DRAWN (the contract, the two-selves, the reference target).
2. **`wat/service.wat`** — the macro. The clause-folding + `known-clauses` (~80; ADD `:satisfies`, and allow `:impls`
   as an alias/mode of the op-bodies when `:satisfies` present); the `enum-name`/`reply-name`/`peer-ty`/`addr-ty`
   generation (~321-367 — when `:satisfies`, these become `<S>::Op`/`<S>::Reply` instead of `{fqdn}::Op`); the per-op
   request/response record generation (~490-538 — SKIP when `:satisfies`, the records are the surface's); the
   `serve-op-arms` (~600 — the dispatch, re-pointed at `<S>::Op` variants, arms from `:impls`); the `op-methods`
   (~856 — client fns over `<S>::Op`/`<S>::Reply`).
3. **`wat/query/mem.wat`** — a real defservice (the `:ops` shape you're offering an alternative to; the
   `Outcome::Reply new-state (Response …)` body shape the `:impls` mirror).
4. **`wat/query.wat`** — the `:wat::query::Store` surface (methods) — but for the S2 GATE, author a small fresh surface
   + service (a `Kv` toy), don't wire the real Store yet (that's S4).
5. **`src/types.rs`** `synthesize_surface_protocol` (S1) — how `<S>::Op`/`<S>::Reply` are shaped (variant `<Op>` with
   field `req`/`resp`), so your serve-arms + client-methods match S1's variant/field names exactly.

## The GATE probe you author (a fresh Kv surface + service, end-to-end)
Author `Kv` request/response records + a `defsurface :Kv` (pure sigs → S1 synthesizes `Kv::Op`/`Kv::Reply`) + a
`defservice :satisfies :Kv` with `:impls` for every method + a `deftest'`/`main` that starts it and round-trips via the
generated client fns (`Kv/put`, `Kv/get`). Then prove: (a) it round-trips; (b) DELETING one `:impl` → a non-exhaustive
`match` compile error (the free coverage check); (c) a service `:satisfies :Kv` and the client both type against
`Kv::Op`/`Kv::Reply` (one shared protocol).

## STOP triggers (halt + report, don't hack)
1. **STOP-SATISFIES-MINTS:** when `:satisfies` is present, the macro must NOT emit `{fqdn}::Op`/`{fqdn}::Reply`/the
   request/response records — it references the surface's. If you find the macro's structure fights this (e.g. the
   Op/Reply gen is deeply entangled), STOP and report the entanglement — do NOT emit BOTH (a `{fqdn}::Op` AND use
   `<S>::Op`).
2. **STOP-EXHAUSTIVE:** the coverage guarantee is the exhaustive `match` over `<S>::Op`. If the serve-arm match is NOT
   exhaustive-checked against `<S>::Op` (so a missing impl slips through), STOP and report — that's the whole safety.
3. **STOP-NOCP:** do NOT change S1 (`src/types.rs` synthesis), `defenum`/`defrecord`, or the `:ops` path (a non-`:satisfies`
   defservice must behave EXACTLY as before — this is purely additive).

## The gate (EXPECTATIONS — I re-run these myself)
| what | command | expected |
|---|---|---|
| a `:satisfies` service round-trips over the surface's protocol | `cargo wat` on your Kv gate file | round-trips (prints the expected values) |
| a missing `:impl` is a compile error (free coverage) | `cargo wat` on a copy with one `:impl` deleted | non-exhaustive-match error naming the uncovered `Kv::Op` variant |
| the `:ops` path is UNCHANGED | `cargo nextest run --release -E 'test(smem_roundtrip) or test(sqlite_store_differential) or test(counter)'` | passed (existing defservices unaffected) |
| whole floor | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder |

Runtime ~40-60 min (a baked-macro change forces a rebuild + the full suite).

## Final report (structured): files changed · how `:satisfies` folds into the clause map + how `:impls` is read · where
the Op/Reply/Handle/Address' gen is re-pointed to `<S>::Op`/`<S>::Reply` (and where per-service minting is SKIPPED) · the
serve-op-arms + client-methods re-pointing · the Kv gate (paste it) + the verbatim round-trip + the deleted-impl
compile error + the `:ops`-unchanged result + the whole-floor Summary · STOP triggers hit or "none" · surprises.

## Prior comparable: S1 (`b13cab8c`, the synthesis) + the existing `:ops` codegen in `wat/service.wat` (the machinery
you re-point). The `:impls` body shape mirrors `wat/query/mem.wat`'s op bodies (`Outcome::Reply new-state (Response …)`).
