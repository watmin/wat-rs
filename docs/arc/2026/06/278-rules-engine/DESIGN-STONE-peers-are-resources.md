# DESIGN-STONE — peers are RESOURCES, and the §7 purity wall did not know it

> **Status: RULED + STRUCK 2026-08-03.** Builder's ruling, verbatim: *"they are resources - they
> are not pure."* Found while weighing A1 (`peer-process`) of
> `DESIGN-STONE-a-service-that-measures-itself.md`. Board: **#68**.
>
> This stone is small in edit and large in consequence: **two tokens of substrate, and every
> service `Handle` in the substrate changes nature.**

## The hole

`is_pure_type` (`src/check.rs`, the §7 purity predicate) carries a list of parametric heads that
are impure **regardless of their type arguments**:

```rust
"rust::crossbeam_channel::Sender" | "rust::crossbeam_channel::Receiver"
| "wat::kernel::Sender" | "wat::kernel::Receiver"
| "wat::kernel::ProgramHandle" | "wat::kernel::HandlePool"
| "wat::kernel::ThreadSelfPeer" => false,
_ => args.iter().all(|a| is_pure_type(a, types)),      // <- everything else
```

`ThreadSelfPeer` is on it, with a comment stating exactly the right reason — *"even if its I/O are
pure scalars, the peer itself is an in-locus opaque (crossbeam channel) that cannot cross a comms
boundary."* **Its three siblings — `Peer`, `Thread`, `Process` — were absent.** So each fell through
to *pure iff its type args are pure*, and `Peer<i64,String>` was judged **PURE**.

Downstream, `validate_aggregate_containment` (called at freeze) enforces *a pure aggregate may only
hold pure fields*. With a peer reading pure, that pass **admitted a live peer as a field of a
record** — which is to say into a `defservice`'s `:durable`, and onto the wire.

Two standing doctrines say otherwise, and they were already written down:

- **293.W** — only **addresses** cross. A peer is crossbeam tx/rx or an fd pair: process-local, and
  a process must *dial* its peer. A peer is never shipped.
- **The aggregate law** — `struct` = may hold resources; `record` = **guaranteed** pure data
  (`[[reference_struct_holds_resources_record_is_pure_data]]`).

The wall was not wrong in design. Three names were missing from one list.

## How it was found — a probe with a positive control

Not a grep, and not a reading. A four-row probe, deliberately kept OUT of `wat-scripts/`
(the loader gate type-checks everything under it, and this file is *supposed* to fail):

| row | before | after |
|---|---|---|
| `HoldsPeer <- Peer<i64,String>` | **accepted** | refused |
| `HoldsProcess <- Process<i64,String>` | **accepted** | refused |
| `HoldsThread <- Thread<i64,String>` | **accepted** | refused |
| `HoldsThreadSelfPeer` — **positive control** | refused | refused |

The control is the load-bearing row: `ThreadSelfPeer` was *already* on the impure list, so it had to
be refused both times. It was — which proves the probe reaches the wall, and that the other three
rows' acceptance was a real verdict rather than a probe that could not fire
(`[[feedback_a_grep_that_cannot_reach_is_not_evidence]]`, applied to a probe).

## ★ The cascade — one root, 26 Handles, 2697 tests

Method: **impose the check and read the screams** — never survey for the worklist
(`[[feedback_impose_the_check_and_read_the_screams]]`, R52 `QVOD LEX ACCENDIT`). Arming the three
names lit **2697 of 4343 tests across 26 distinct `<fqdn>::Handle` types**, including the stdlib's
own `stdout-svc` / `stdin-svc` / `stderr-svc`.

All of it one root — `wat/service.wat`, the `defservice` macro's C.3:

```clojure
handle-fields  `[handle <- ~handle-peer-ty     ;; Peer<Admin,Status>   — a live peer
                 addr   <- ~addr-ty]           ;; Address<Op,Reply>    — a RustOpaque
handle-record  `(:wat::core::defrecord ~handle-name ~handle-fields)   ;; <- a RECORD
```

**Both fields are resources, and it was declared a record.**

The fix is one token: `defrecord` → `defstruct`. And the argument for it was already on disk, in the
comment of the Handle's own parent — `Launched` (`wat/spawn.wat:265`) holds these *same two fields*
and has always been a `defstruct`:

> *"A STRUCT, not a record (address is an Address' RustOpaque; handle is :Spawned)."*

The hand-written parent knew. The macro-generated child did not. **A `Handle` is an owner-side
CAPABILITY, never data** — which is exactly why `stop` can `send`/`recv` on it, and exactly why it
must not cross.

### The cascade collapsed 2697 → 1 → 0, and the corpus needed no codemod

| step | floor |
|---|---|
| wall armed, nothing else | `4343 run: 1646 passed, 2697 failed` — 26 Handles |
| `Handle` → `defstruct` (one token) | `4343 run: 4342 passed, 1 failed` |
| the one file fixed | *(see the strike's Score)* |

The last failure was the loader gate (`every_wat_scripts_file_loads_on_the_current_runtime`), which
walks all 260 `.wat` under `wat-scripts/` and names its own worklist. **It named exactly one file** —
`wat-scripts/probes/arc-170/probe-strikeB-fields.wat`, whose `:probe::Bag` held a `Peer`.

That file is a fix, not a migration: **a codemod is the instrument for a structural rewrite across
MANY `.wat`; this was one site.** Reaching for `wat-fix` here would have been ceremony. And the file
was already telling us — its own header reads *"STRIKE B RED PROBE — struct-field reflection"* while
the declaration underneath said `defrecord`. The one token made the declaration match the name, and
the probe's actual subject still works: `field-names-of` → `[:kv :n]`, `field-types-of` →
`[(wat.kernel/Peer probe.Kv/Op probe.Kv/Reply) wat.type/i64]`. Reflection over a struct is fine; it
always was.

**The 2697 was never 2697 problems.** It was one macro and one probe, and the gate that found the
probe is the same discipline that found the macro — impose, then read.

## ★★ The method finding — the build is NOT the freeze arbiter

Kept visible because it is a live trap and the orchestrator walked into it this session.

`cargo build --release` returned **exit 0 in 35s** with the wall armed. On the strength of that the
orchestrator wrote *"the build IS the freeze arbiter, so the baked stdlib froze clean."* **It had
not.** 2697 tests were about to fail. The claim came from a seam note (a 24c far-side update said
the build was the freeze arbiter for a *different* change) rather than from a check
(`[[feedback_ground_the_substrate_not_just_the_chronicle]]`).

**THE FLOOR IS THE ARBITER.** A green build says the Rust compiled. It does not say the corpus
freezes.

## What this does NOT claim

- **A1 (`peer-process`) did not create this.** `Handle/handle` already returned a `Peer<Sh,Lu>`, so
  a peer-in-a-record was reachable before it existed. A1 only made the shape easy to *produce*
  (`Option<Process<I,O>>`). The hole predates it and is independent of it.
- **No wire-crossing bug is asserted.** The wall now forbids a shape that was *representable*; it is
  a separate question, unmeasured, whether any live code ever actually shipped a peer. Nothing in
  the corpus was found putting a `Handle` in a `:durable` — `wat/cache.wat:129` shows the discipline
  being held **by hand** for the `Lru` handle, which is precisely the convention this wall replaces
  with a structure.

## The strike

| | |
|---|---|
| **Substrate** | `check.rs` — three names added to `is_pure_type`'s impure-head list. |
| **Substrate** | `wat/service.wat` — the generated `Handle` becomes a `defstruct`. One token. |
| **Corpus** | Whatever survives the two above. If it is a structural rewrite across many `.wat`, it is a **wat-fix codemod** (`wat/fix.wat` + a recorded `wat-scripts/fixes/<migration>.wat`), dry-run on a `/tmp` copy and diffed before it touches the tree — NEVER hand-edits, never sed. Nearest shapes on the shelf: `response-record-to-enum.wat`, `struct-new-failure-to-message-only-failure.wat`. |

## STOPs

- **⛔ Do not widen `is_pure_type` back to make a red go green.** A red here is the wall working. If
  something legitimately needs to hold a peer, it becomes a **struct** — that is the whole ruling.
- **⛔ Do not hand-edit the corpus.** The codemod is the instrument; the shadowdancers strike.
- **⛔ Do not read a green `cargo build` as a freeze.** Weigh the floor, by your own `--release`
  re-run, and read the Summary line.
- **⛔ Do not delete the probe's positive control** if the probe is ever given a durable home. Without
  `ThreadSelfPeer`'s row, three accepted rows prove nothing — the probe might simply not reach the
  wall.
