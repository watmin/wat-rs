# DESIGN — the dial declares its peer

Builder, on the four questions: *"you reasoned 4 YES -- it is reasoned."*

**Drawn 2026-09-13. NOT STRUCK.** Closes the hole found by
`FINDING-a-handler-local-dial-is-a-runtime-failure-that-could-be-compile-time.md`.

## ⛔ TWO CORRECTIONS TO WHAT I TOLD THE BUILDER, BOTH MEASURED

**1. It does NOT reach `src/`.** I said *"it reaches `src/check.rs`."* Wrong — both existing checks are
`#wat.macro/MalformedTemplate` raised from **`wat/service.wat`** (`:896`, `:913`) by
`:wat::core::macro-error` inside the `defservice` macro. **This is a wat change.** No Rust, no bootstrap
(`wat/service.wat` is stdlib but no checker changes, so `wat/fix.wat`'s own escape clause applies: no
stash-dance).

**2. The "silent crash" was my misreading**, corrected twice in the FINDING. The substrate delivers the exact
cause on the **lineage's** crash channel — `RuntimeError ["unknown function: :mid::Mid/ping" … line 66 col 44]`
— while a **dialed peer** honestly reports only `Disconnected []` (`src/kernel/spawn.rs:181`/`:202`). **So this
stone is not about legibility.** It is about moving a runtime death to a compile error.

## What the two existing checks are, and why a third is needed

`wat/service.wat` enforces a **bijection**:

```
:896  every :peers surface            ⇒ an :ephemeral field typed Peer<S::Op,S::Reply>
:913  every :ephemeral dialed Peer    ⇒ that surface declared in :peers
```

Together they force the **init-dial** shape, which is why `sqs.wat` dials the store inside `:init` and threads
the peer through state. ⛔ **A `connect` inside an `:impls` body participates in neither**: it puts nothing in
`:ephemeral` and declares nothing in `:peers`, so the bijection has nothing to check — and the forked child,
launched without that surface's spliced forms, dies on the dial.

## ⭑⭑ THE CONTRACT DECISION — and the strict version is REFUTED BY MEASUREMENT

I first wanted the simple rule: *"an `Address<S>`-typed field ⇒ `S ∈ :peers`."* **Measured against the corpus,
it produces false positives on correct code:**

| file | `Address<S>` field surfaces | `:peers` declares |
|---|---|---|
| `sqs.wat` | `queue::Queue`, `wat::query::Store` | `[:wat::query::Store]` |
| `sns-fanout.wat` | `demo::Topic`, `demo::TopicWorker`, `queue::Queue` | `[:queue::Queue]` |
| `circuit.wat` | 5 surfaces | subsets |

★ **Holding an address without dialing it is common and legitimate** — a service holds its *own* address to hand
out, or another's to pass to a child. The strict rule would demand `:peers` for those, and check `:896` would
then demand a spurious `:ephemeral` peer, forcing a dial nobody wants. So:

> **The rule is: a `connect` INSIDE an `:impls` body, whose argument resolves to a `:durable`/`:ephemeral`
> field of this service declared `(Address :- [S::Op S::Reply])`, requires `S ∈ :peers`.**
> The connect-site walk is not an optimisation — **it is what distinguishes *holds* from *dials*.**

## It does not forbid the shape the corpus relies on

`:fanout::worker` (`circuit.wat:370`) redials **inside a handler**: `(connect (Record/seen-addr rec))` in
`-disrupt`, with `:peers [:queue::Queue :fanout::Seen]` and both peers in `:ephemeral`. ⭑ **That passes the new
check** — the surface is declared. The legitimate redial keeps working; only the undeclared dial is refused.

⚠ And `:init` dials are untouched: `:init` is its own clause, not `:impls`, and they are already covered by the
`:913` check via their `:ephemeral` peer.

## The four questions

**Obvious?** YES — the same sentence the checker already says twice, said a third time. **Simple?** YES — one
more `foldl` beside the two, reusing `peers-surfaces` and `macro-error`. **Honest?** YES — it makes the mistake
**unwritable**, which is where the other two rules already are, and it refuses nothing the corpus does.
**Good UX?** YES — a named compile error instead of a dead child whose reason you must know to go fetch from
the lineage.

## Scope

**IN:** a third bijection check in `wat/service.wat` · the `:impls` connect-site walk · a message mirroring the
existing two · a probe proving the refusal fires, and a probe proving the worker's declared redial still
compiles.

**OUT = REJECTED:**
- ⛔ **The strict `Address`-field rule.** Refuted by the table above.
- ⛔ **Giving `Disconnected` a cause.** The FINDING's second correction shows it is honest as it stands and
  `LociDiedError` already carries `Panic`/`RuntimeError`/`StartupError` for the cases that have a cause.
- ⛔ **Anything in `src/`.** This is a macro-time check.
- **A connect on an address that arrived in a MESSAGE.** ⚠ Named residual: it cannot be resolved
  syntactically, so it stays unchecked. This stone covers the field case — which is the corpus's shape and the
  one that bit. Say so in the SCORE rather than implying total coverage.

## Trap-doors

1. ⛔ **Do not make it the strict rule** — the table shows three files it would falsely refuse.
2. ⛔ **`:init` is not `:impls`.** Walking the wrong clause would either miss the bug or double-refuse the
   init-dial shape the whole corpus uses.
3. **`:fanout::worker` is the must-still-compile control.** A handler redial of a *declared* surface.
4. **`wat/service.wat` is stdlib and frozen at build time** — but no Rust change, so no stash-dance. If the
   floor's `.wat` corpus trips the new check anywhere, that is a **finding**, not a reason to weaken it.
5. **Mirror the existing message wording.** Two excellent diagnostics already exist; a third that reads
   differently makes the class look like three unrelated rules.
