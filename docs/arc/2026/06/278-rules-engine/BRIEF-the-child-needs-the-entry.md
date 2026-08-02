# BRIEF — the child needs the ENTRY, never the LIBRARY

Design: `DESIGN-STONE-the-child-needs-the-entry-not-the-library.md` (read it first; the mechanism is
grounded there and accounts for every failing name — do not re-derive it).

## The work, in one paragraph

When a `defservice` is started on a **process** locus, the parent ships the child a `Vector<WatAST>`
called `service-forms` containing the service's whole expansion plus its surfaces' forms. For a
**stdlib** service that is pure redundancy — the child bakes the identical source from
`src/stdlib.rs` (96 files, `include_str!`, no load gate) — and it is now also an *error*: in the child
those forms are user residue, so re-declaring `:wat::`-rooted names is refused by
`resolve::gate -> Reserved`. Make each contributor to `service-forms` conditional: ship it only if the
child cannot already have it. The one thing that is **always** shipped is the child's entry point.

## The rule

Every contributor to `service-forms` is included only when the child cannot already have it, and
**`:wat::`-rooted ⇒ the child already has it**. That implication is an invariant, not a convention:
`:wat::` is reserved (`src/resolve/reserved.rs:25-27`) and the gate refuses a user-privilege
registration under it, so a `:wat::`-rooted name can only have come from baked stdlib source.

Applied per contributor — note these are **independent**, because a user service may satisfy a stdlib
surface, or vice versa:

| contributor | site | include when |
|---|---|---|
| the own surface's `<S>::surface-forms` call | `service.wat:1771-1783` | `proto-base` is NOT `:wat::`-rooted |
| each peer surface's `<Si>::surface-forms` call | `service.wat:1778-1784` (`peer-forms-calls` fold) | that peer's fqdn is NOT `:wat::`-rooted |
| the service's own internals — `record-def`, `state-def`, `service-op-def`, the derive items, `serve`, `init`, `stop-project`, `hibernate-project`, `admin-enum-def`, `status-enum-def`, `dispatch-admin-def`, `extract-addr-def` | `service.wat:1749-1762` | `fqdn-base` is NOT `:wat::`-rooted |
| **`child-main-form`** | `service.wat:1763` | **ALWAYS** |

`child-main-form` is the generated agnostic `:user::main` that binds `:user::spawn::service-locus`. It
is produced per service, exists in no bake, and is the child's entry point. Dropping it leaves the
child with nothing to run — this is the refinement a naive "don't ship it" gets wrong.

## Rooms — read in this order

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-child-needs-the-entry-not-the-library.md` — the
   grounded mechanism and the rejected alternative.
2. `wat/service.wat:1741-1787` — `own-forms-call`, `surface-forms-kw`, `peers-forms-node`,
   `service-forms-def`. This is the whole edit site.
3. `wat/service.wat:1660-1670` — where `fqdn-base` and `service-forms-kw` are already in scope, so you
   can see the discriminand is available with no new plumbing.
4. `wat/spawn.wat:488-520` — the consumer: the `ProcessOpts` launch arm concats the locus `def` ahead
   of `service-forms` and spawns. Read it to confirm nothing downstream assumes the vector's length or
   that any particular internal is present.
5. `wat/spawn.wat:449-452` — the thread arm, which **ignores** `service-forms` entirely. This is why
   thread-locus tests pass today and must keep passing.

## Sketch

The two halves already exist as separate expressions. Make each conditional, then keep the existing
`concat`. You will need a string predicate for "is `:wat::`-rooted" — **ground which one the wat
stdlib actually provides** (`string::starts-with?` or equivalent) rather than assuming a name; the
surrounding code uses `string::interpolate` and `string::concat`, so the family is there.

Keep it a fold over data, not a staircase of nested conditionals.

## Blast radius

`wat/service.wat` only. No `src/` Rust. No new types, no signature changes, no change to
`spawn.wat`'s launch arms, no change to any test.

## STOP triggers — rejection criteria. Ship nothing and report.

- **STOP-1 — a `:wat::`-rooted service or surface that the child does NOT bake.**
  `installed_dep_sources` (via `src/stdlib.rs`) admits dependency/battery sources into the stdlib set.
  The exec'd child re-runs the same binary so it should carry the same batteries — but if you find a
  path where a `:wat::`-rooted service can exist in the parent and be absent from the child, STOP and
  report it. The invariant IS the stone; if it is punctured the stone is wrong, not the code.
- **STOP-2 — the child needs more than the entry.** If omitting the internals leaves a child unable to
  resolve something, STOP and report exactly which form and why it is not recoverable from the bake.
  Do NOT restore the full `own-forms-call` to reach green — restoring it is the defect this removes.
- **STOP-3 — a user service's emitted forms change.** The non-`:wat::` path must be byte-identical to
  today. Verify with a user-declared `defservice` (there are many under `wat-tests/` and
  `tests/services/`). If its `service-forms` differ at all, STOP.
- **STOP-4 — the parametric case.** `:wat::cache::lru-svc<K,V>` is `:wat::`-rooted, so the rule says
  ship only its child main. It is genuinely unknown whether the child can monomorphize the generic
  from its own bake. Implement the rule uniformly and let the floor answer. If `lru-svc` is the only
  thing still failing afterwards, that is a FINDING to report, not a thing to special-case — say so
  and stop rather than carving an exception.

## Your gate

`cargo build --release --all-targets` — exit 0, zero warnings. The stdlib must freeze, which is the
real arbiter for a `wat/` change.

Then the load-order gate: a two-line `:user::main` printing `(:wat::deporder::verify-stdlib)` must
return `[]`. This catches stdlib load-order violations `--check` cannot see, and this stone touches a
stdlib macro.

Do NOT run `cargo nextest` or `cargo test` — the floor is measured centrally, once, by me on a
quiescent tree. Run every command in the FOREGROUND and block on it; your turn ends when the numbers
are in your hands, not when a command is launched.

## Report

The exact conditional shape you landed on and the string predicate you used (with its `file:line`);
the diff stat; whether `lru-svc` needed anything the rule did not give it; your build result and the
`verify-stdlib` output; and anything you judged rather than transcribed.
