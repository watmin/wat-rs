# DESIGN STONE — 255.1c-taxonomy · the kernel tier EXHAUSTS `Category`. Amend the subject line, mint five.

**Prerequisite to the kernel carve.** Homes #1–#3 (`Bytes` · `time` · `kernel-stdio`) were pure-data
and stdio families, and the ten `Category` variants were derived from them. The next migration is the
**`:wat::kernel::` tier — 44 verbs** — and it is a different kind of thing. Measured this session:
**the taxonomy runs out**, and it runs out in four places plus one the totality campaign will keep
producing.

> **Builder, 2026-08-19:** *"we started the kernel migration.... finish it...."* and, on the fifth
> variant: *"Add CheckGate to the list - we reserve the right to change our mind later - i know for a
> fact we'll have many of these `must-*` forms that exist to ensure a totality expression is handled
> correctly."*

## How this was derived — an `intueri` cast, weighed

`wat/runtime-meta.wat`'s `Category` is the SOURCE OF TRUTH: `wat_enum_derive::wat_enum_from!` reads
it at compile time and the `;;` prose on each variant becomes that variant's Rust `///`. **The prose
IS the deliverable**, not decoration. The domain is closed and append-only.

Two `intueri` casts were spawned against it (the ward embedded verbatim from the signed channel,
never fetched by the worker). Their verdicts were weighed against the orchestrator's own read, and
**every load-bearing claim below was re-verified against the disk before it was written down.**

★ The ward **refuted one of the orchestrator's four proposals** and the refutation is kept because it
is right: a proposed `:Mutate` variant for `allow`/`deny`'s in-place mutation was rejected on the
header's own rule — *whether an effect lands in-place or by building a fresh value is a **HOW**, not
a **WHAT-domain***, the same axis-mix the header already forbids. `allow`/`deny` fold into
`:Resource` instead, broadened to *administer*.

## ⛔ FIRST — THE SUBJECT LINE IS ALREADY WRONG, INDEPENDENT OF ANY NEW VARIANT

`wat/runtime-meta.wat:58` reads:

```
;; Category — what kind of computation an intrinsic or special form performs.
```

**`:Declaration` already isn't a computation.** Registering `def`/`defclause`/`declare-acronyms` is a
splice/resolve/freeze-time act — verified: all three runtime bodies are `Ok(Value::Unit)` no-ops,
each saying so in its own comment (*"edge already registered at splice/pre-check time"* ·
*"registry already populated at freeze time"* · *"resolve-pass declaration"*).

So the crack pre-dates the kernel tier. `require-wire-address` merely makes it impossible to ignore
by having **zero** runtime work rather than merely non-value-shaped work.

**Amend it to name what the axis actually answers:**

```
;; Category — what a verb DOES in the language. Most commonly a runtime computation;
;; sometimes a program-level registration (:Declaration); sometimes a contract
;; discharged entirely at check time (:CheckGate). The axis is the DOING, not the
;; moment it happens.
```

⛔ **Shipping any new variant without this amendment leaves the header's own rule false on its face
for that row.** The amendment goes first.

## The five variants — each with its ONE axis

### `:Resource` — acquires, releases, or ADMINISTERS a handle whose lifetime is tracked outside value scope

Members: `listener` `connect` `accept` `pipe` `spawn-thread` `spawn-process` `after`
`HandlePool::{new,pop,finish}` `close` `drop` — and, by the refutation above, `allow` `deny`
(the allow-set is a property of a resource a handle already names) and `signal`.

**ONE axis:** custody of a handle. NOT what data moves through it (that is `:Message`), NOT where the
handle came from.

★ **The name is ADOPTED, not invented** — this substrate already calls exactly this set resources:
task #68, *"Peers are RESOURCES — the §7 purity wall did not cover Peer/Thread/Process."* The wall
was built around this grouping; the taxonomy simply never got the word.

⚠ **`drop` is a documented NO-OP** (`runtime.rs:26695`): *"exists as a READABILITY MARKER at the call
site… does not force the channel to close while other references remain."* It belongs to the resource
vocabulary, but the variant's prose must NOT imply every member does teardown work.

### `:Message` — moves a payload across a peer/channel boundary to or from another locus

Members: `send` `try-send` `recv` `select` `poll`.

**ONE axis:** payload transfer between loci. NOT the creation or destruction of the peer (`:Resource`).

⛔ **DO NOT OVERLOAD `:Io` FOR THESE, and this is the stone's sharpest finding.** `Io`'s promise is
*"input/output on a **stream**."* Verified verbatim at `runtime.rs:31067`:

```
/// Thread': `peer.send(value)` Value pass-through (crossbeam, no serialisation).
/// Process': encode payload via value_to_edn + wat_edn::write → peer.send(String).
/// Peer' thread-tier: `peer.send(value)` Value pass-through.
/// Peer' socket-tier: encode with sym.types() in eval → `peer.send_wire(String)`.
```

**Four tiers; two are raw `Value` pass-through with NO stream and NO bytes at all.** Filing these
under `Io` makes the variant's promise true for half its members and false for the other half
*depending on which tier a call site happened to choose* — a live violation of "ONE axis throughout."

★ And the risk is FORWARD: `kernel_stdio.rs` already primes `:wat::io::*` as `Io`'s second tenant, so
a carve stone would reach for `Io` here **by resemblance** ("concurrency feels IO-ish") rather than by
re-deriving from the body. That is precisely the accretion pattern — caught before it committed.
`:Io` stays narrowly fd/byte-stream-shaped.

### `:Ambient` — reads or writes process-global state that no value the caller holds addresses

Members: `stopped?` `sigusr1?` `sigusr2?` `sighup?` `reset-sigusr1!` `reset-sigusr2!` `reset-sighup!`.

**ONE axis:** process-global state, both directions.

⛔ **`:Signal` as a sibling to `:Clock` was PROPOSED AND REJECTED.** `Clock`'s framing is *"names WHICH
external source a Nondeterministic verb draws from"* — a **read**. Three of the seven members are
**writes** (`flag.store(false, …)`), which a "which source you sample" axis cannot cover at all. And
`stopped?` is not signal-specific — its promise is *"was the kernel asked to stop"*.

★ This also fixes a MIS-FIT that exists today: the three `sig*?` queries are **nullary** — they read a
global `AtomicBool` and interrogate no value — while `:Probe`'s promise is explicitly *"derives a FACT
about **the input**."* There is no input. Filing them under `Probe` would sort by shape-of-answer
("returns a bool"), the exact axis-mix that sank the proposed `:Predicate`.

### `:Project` — returns a COMPONENT of a compound value

Members: `Failure/message` `Failure/location` `LociDiedError/message` — and every derived
record/struct field accessor.

**ONE axis:** the inverse of `:Combine`. `Combine` *"builds a larger value of the same kind"*; nothing
named taking a part back out. **A mirror gap, not a new domain.**
`[[feedback_the_mirror_is_an_instrument_not_a_fix]]`

Why not `:Probe`: `Probe` *derives a fact about* a value (`empty?`, `length`) — it computes something
new. An accessor **returns a part that was already there**. Different act.
Why not `:Accessor`: every other variant names an action or domain; `Accessor` is an **agent noun**
naming what KIND OF FUNCTION it is — a different axis from all ten.

⚠ **A SECOND POPULATION THIS EXPOSES, and it is not this stone's to fix.** `accessor_meta`
(`rete/purity.rs:959`) DERIVES purity/determinism/totality for record accessors from the frozen
`TypeEnv`, and the completeness gate exempts them by name: *"constructors + field accessors are NOT
here — they DERIVE from the frozen TypeEnv, so they cannot go stale."* **They carry no `Category` at
all**, because they are not registered rows. So the same act — read a field off a compound value — is
metadata-free in one population and needs a name in the other. `:Project` names it for the
hand-written half; whether derived accessors should also carry it is out of scope below.

### `:CheckGate` — refuses a call site at check time; the contract is discharged before evaluation

Member today: `require-wire-address` — `runtime.rs:32214`, *"Runtime identity; the Wire check lives in
`infer_require_wire_address`."* Verified: the runtime body evaluates its argument and returns it
unchanged.

**ONE axis:** constrains which programs compile. The runtime is identity or otherwise incidental.

### ★★ `:CheckGate` IS MINTED OVER THE WARD'S OBJECTION — recorded as an override, with its reason

The `intueri` verdict said **wait**, and its argument was structural rather than statistical: under
the *current* subject line, a verb performing no computation cannot honestly take any variant, **at
any population size**. It also measured the class at **N=1** (correcting the orchestrator's census: it
found four no-op-at-runtime verbs, then ruled `derive`/`declare-acronyms`/`use!` into `:Declaration`,
which names `declare-acronyms` in its own doc — leaving one true orphan).

**The builder overrode it on forward knowledge the ward could not have:** *"i know for a fact we'll
have many of these `must-*` forms that exist to ensure a totality expression is handled correctly"* —
the totality campaign (task #64, *"every core primitive's domain hole becomes a faced outcome"*)
produces exactly this shape: a partial verb gains a check-time gate that refuses the partial case.
`[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]` — the corpus is a record of what
happened to compile, so sizing a closed language axis by grep count is the known error here.

⚠ **The ward's structural objection is NOT dismissed — it is SATISFIED by the subject-line amendment
shipping first.** Once the axis says "what a verb DOES in the language" rather than "what computation
it performs," `:CheckGate` is honest. That ordering is not cosmetic; it is the whole reason the
amendment leads this stone.

⚠ **AND THE NAME IS EXPLICITLY REVISABLE.** Builder: *"we reserve the right to change our mind
later."* One specimen cannot test whether *"identity-at-runtime"* is the class's real shared property
or an accident of this verb — a future gate might NARROW a type rather than pass it through. **Revisit
at the second member.**

## The four questions

- **Obvious? YES.** Each variant names one act, and four of the five are the missing half of something
  already named (`Combine`↔`Project`, `Io` vs a channel that has no stream, `Clock`'s read vs a write,
  a resource the §7 wall already grouped).
- **Simple? YES.** Five appends to a closed enum plus one prose amendment. No mechanism changes.
- **Honest? YES** — and it is the amendment that makes it so. Shipping `:CheckGate` under the old
  subject line would have been a category error on day one, which is what the ward caught.
- **Good UX? YES.** A carve rider stops having to choose between a bad fit and a STOP.

## ⚠ THE TRAPS

**T1 — the prose ships as documentation.** `wat_enum_derive` turns each `;;` block into the Rust
variant's `///`. Sloppy prose here is sloppy API docs everywhere downstream. Every variant's block
must state its ONE axis and at least one thing it is NOT.

**T2 — append-only, order matters.** New variants append at the END of the `defenum`. Inserting one
mid-list renumbers the generated enum.

**T3 — the taxonomy is a wat file read by a Rust macro at COMPILE time.** A malformed `defenum` here
does not fail a test, it fails the build. And `crates/wat-doc`'s `MissingCategory` → `compile_error!`
means any registry row without a `@Category` also fails the build — so this stone cannot be
half-landed.

**T4 — no verb is re-categorized by this stone.** It adds names. Applying them to the 44 kernel verbs
is the CARVE's work, and a carve rider must still re-derive each from its body — the last home's rider
had its `Io`/`Reflection` first pass **overruled and re-derived**, which is the discipline working.

## ACCEPTANCE

| | assertion | instrument |
|---|---|---|
| 1 | ★ the subject line no longer says "computation" | read the diff |
| 2 | five variants appended, each with its ONE axis and a NOT | read the diff |
| 3 | the generated Rust enum carries all fifteen with their prose as `///` | `cargo build --release` |
| 4 | no existing variant's prose is changed except where noted | read the diff |
| 5 | no verb is re-categorized | `git diff --stat` shows no `src/intrinsic/` row edits |
| 6 | floor · clippy · ignores | **orchestrator**: 4819/0, 19 skipped · 0 · 13 |

## Out of scope — affirmative cuts, homes named

- **Applying the variants to the 44 kernel verbs.** That is the carve, and it is the next stone.
- **Whether DERIVED record accessors should carry `:Project`.** They have no `Category` today by
  design (`accessor_meta` derives from the frozen `TypeEnv`). Widening the registry to cover derived
  rows is a registry-shape question, not a taxonomy one. Tracked here as the stone's own named
  residue; it blocks nothing.
- **Renaming `require-wire-address`.** The ward rated it a **Level 2 mumble**: `require-` reads like
  `assert!`/`ensure!` — used for effect — but every call site (`wat/bracket.wat:903`, `:986`) wraps an
  expression and threads the value through, and its neighbours `peer-wire?`/`address-wire?` DO return
  bool. Direction: `as-wire-address`. Out of this stone's scope — it is a corpus rename with its own
  blast radius, and the builder has since named `must-*` as the family's intended prefix, which is a
  ruling this stone does not pre-empt.
- **`serve-dispatch-op`'s home.** The ward found no clean single-axis fit (dispatch + a crash-sentinel
  broadcast). Recommend `:ControlFlow` with the broadcast noted as defensive plumbing — the CARVE
  rules it, not this stone.
- **`:ControlFlow`'s prose for `raise!`/`assertion-failed!`.** They never return; they abandon
  evaluation rather than direct it. The ward accepted the fit and asked the prose be strengthened to
  say so. That is a one-line prose edit the CARVE makes when it files them.
