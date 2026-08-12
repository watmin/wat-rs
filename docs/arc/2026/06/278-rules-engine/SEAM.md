# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD 8e661362   pushed   floor 4389 passed / 0 failed / 262 skipped   clippy 0
```

`git status` clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` STILL HOLDS THE LIFECYCLE STRIKE — do not `git stash drop`.** It was made with
`-u`, so it has **three parents**; `git stash show --stat` shows only the tracked one
(`wat/service.wat` +298/−18) and **cannot see the untracked payload**. Read the payload with
`git show 'stash@{0}^3:<path>'`.

**The owed restore was ATTEMPTED and is now PARTIALLY DISCHARGED, with a finding:**

| file | disposition |
|---|---|
| `wat-scripts/scratch-pad/probe-arc278-nullary-enum-process-repro.wat` | **RESTORED + COMMITTED** — `--check` exit 0, clean |
| `tests/services/probe_arc278_connection_lifecycle.{rs,wat}` | **LEFT IN THE STASH — the `.wat` is STALE, `--check` exit 1** |

The lifecycle probe **does not compile against today's substrate**, 5 errors, verbatim:

```
malformed :wat::core::let form: unhandled :wat::kernel::RecvOutcome<probe::Ratchet::PadResponse>
  in statement/discard position — a recv outcome must be faced (match it: Message/Closed/Lost),
  not dropped. This is the peer-lifecycle OUTCOME WALL (Phase 3).
malformed :wat::core::match form: keyword variant pattern
  :wat::service::DisconnectReason::Closed   on a :?<var> scrutinee
:wat::service::DisconnectReason::Lost       on a :?<var> scrutinee
:wat::service::DisconnectReason::Rejected   on a :?<var> scrutinee
malformed :wat::core::match form: non-exhaustive: open-typed match needs at least one
  hash-destructure arm or a wildcard `_` arm.
```

Four of the five are one root: the `DisconnectReason` scrutinee resolves to an unbound type var, so
no keyword-variant arm can match it and the match reads as open-typed. The fifth is the send/recv
outcome wall (R57) landing on a `recv` the probe drops. **This is the rider's unweighed work
failing exactly the way the seam warned it might** — restoring it to the tree turns the floor red,
so it stays stashed until the lifecycle strike is actually taken up, and *that* strike now begins
with a known, located worklist rather than an assumption that the code was good.

⚠ **`./target/release/wat --check <f> | tail` returns TAIL's exit code.** Both files "passed" until
they were re-run without a pipe. The doctrine names this for the floor; it applies to `--check` too.

## ★ THE GATE IS GREEN, AND THE ARC'S CENTRAL CLAIM IS PROVEN BY A BREAK

`wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat` now reports
**`VERDICT MEANINGFUL`**: the full union **BOOTED-AND-RAN** in a real forked child — it constructs
the record, reads an accessor, matches a program-level enum, prints the marker — and the negative
control, with `init`'s closure omitted, **DIED naming exactly `:user::root-init`**. The instrument
discriminates; the green is earned, not assumed (R59 `NISI FRANGAS, NIHIL PROBAS`).

**So: a union of `fn-forms` closures IS a complete, runnable program across a process fork.**

## ⛔ THE PRIOR SEAM'S "LIVE DEFECT" WAS WRONG — read this before re-deriving it

The previous seam named one root cause wearing three faces and prescribed a FIRST ACT:
*synthesized types retain a source form*, with one grounding owed — *does the expansion that
generates `recordtype` still hold its form at registration? It came back YES for macros. **Do not
assume it transfers.*** **It did not transfer, and the act is struck.**

`tests/reflection/probe_arc278_retained_source_forms.rs` freezes the gate's world and partitions
its user types: **12 RETAINED · 6 RECONSTRUCTED**. `:probe::ffx::Record` and `:probe::ffx::State`
— the two whose accessors the child called unresolved — are **RETAINED**. Their declarations were
already shipping verbatim; `type_def_to_ast` never fired for them. The prescribed fix could not
have touched the failure. The six reconstructed are **all** surface-derived (`$core-record`,
`$holon-record`, `::Op`, `::Reply`, the two op aliases) — no user form by construction, and **no
consumer has been shown harmed by their reconstruction.** Retaining forms for them is an
unproven want; do not build it without a consumer that fails.

**The accessor failure was the INSTRUMENT, twice. `src/` was never at fault.**

1. **A name is not a key.** The raw union carried FOUR forms declaring `:probe::ffx::Record` — two
   `recordtype`, two kwargs `defmacro` — and a name-keyed first-wins dedup kept the macro and
   **discarded the type**. `a5ac88ca` (macros ship) had just put a same-named macro FIRST in the
   prologue, turning a correct fix into a regression one layer up. The census had already said so:
   **182 names in this very world are `[Macro, Type]`** — one concept, two facets, two registries,
   two phases. The key is now `(head, name)`.
2. **The entry arrives RENAMED.** `fn-forms` fronts its entry through the inline-lambda path, so
   `:probe::ffx::init`'s closure declares **`:user::root-init`**. `serve` only looked healthy
   because it is self-recursive and therefore also appears under its own name. **The asymmetry is
   recursion, not a dropped form.**

Both mechanisms are recorded where they can't rot: the gate's `decl-key` comment, and the Rust
probe's header (which also asserts non-vacuously that the reconstruction path stays reachable).

## The architecture, as the builder ruled it

**`defservice` hand-enumerates a manifest; `bracket` ships `fn-forms` closure ++ a one-liner main.**
`defservice` is the outlier, and the manifest is a *workaround* for the extractor's holes.

- **ONE entry, not a root set.** The entry is the child's **main**, not `serve`. (The green gate
  uses a two-root union — it proves the closure MACHINERY, not the ruled shape.)
- **The entry takes the rendezvous as a PARAMETER.** MEASURED: a free `:user::` name in a parent
  defn types as `:wat::core::keyword` and refuses any typed use — *that* is why `child-main-form`
  is quasiquoted data and not a defn. The free name appears only in the shipped one-liner, checked
  in the child. (`probe-arc278-free-user-name-in-parent-defn.wat`, both arms + control.)
- **Ship EXPANDED forms.** Shipping unexpanded means `defservice` itself must cross the fork.

## ✅ THE PEER EDGE IS RULED AND LANDED (2026-08-12) — the blocker below is now OPEN

The builder ruled **(e)**: *"take (e) - one way edge with the negative test."* Landed as **one
`derive` line** in `wat/spawn.wat` — no Rust — because the `Parametric<:Parametric` arm was already
driven by the derive graph and `spawn.wat:243` had pre-written the instruction (*"ONE more `derive`
line — zero edits to the assignable rule"*):

```clojure
(:wat::core::derive :wat::kernel::Peer :wat::kernel::ThreadSelfPeer)
```

`serve` keeps its `ThreadSelfPeer` annotation; the process tier's `Peer` is now **statically
passable**. Guarded one-way by `tests/services/probe_arc293w_peer_derives_threadselfpeer.wat.bad`
(**must stay RED forever** — do not "fix" it). Floor **4391/4391**, clippy 0. Full reasoning:
`docs/arc/2026/06/293-struct-record-symmetry/NOTE-peer-and-threadselfpeer-are-one-relation-never-stated.md`.

**So the type blocker is gone. What remains below is the mechanical half.**

## ▶ NEXT ACT — kill the dynamic `apply` (grounded this session, with coordinates)

The one-entry model needs a real parent `defn` a closure walk can root at. Today the generated
child main reaches its callees **dynamically**, so no walk can follow:

```
wat/service.wat:2101   (apply (keyword/from-string ~dispatch-admin-name-str) ship [])
wat/service.wat:2120   (apply (keyword/from-string ~serve-name-str) self …)
```

### ⛔ THE OWED GROUNDING WAS PAID, AND THE ANSWER RE-SHAPES THE ACT

**The dynamic `apply` is not a style choice. It is a deliberate TYPE-CHECK BYPASS**, and
`wat/service.wat` says so twice in its own comments:

```
:782  "process-tier `apply` bypasses the type check so the same serve fn works for both tiers"
:800  "Process-tier calls serve via `apply` … which bypasses the type check — the process-tier
       Peer<Status,Admin> from self-peer is accepted at runtime without a static mismatch"
```

**Verified against the source, not the comments:**

| | |
|---|---|
| `serve`'s `self` param | `ThreadSelfPeer<Status,Admin>` — `wat/service.wat:1478` binds `~lineage-peer-ty`, built at `:803` |
| what the child main HAS | `Peer<Status,Admin>` — `:wat::program::self-peer` (`runtime.rs:21853`; its error text at `runtime.rs:28298` says *"Peer<_,_> (self-peer, the owner/supervisor link)"*) |
| are they one type? | **NO** — the checker tests them as distinct heads in an `\|\|` (`check.rs:9835`, `:10159`). `ThreadSelfPeer` is the arc-293.W.2d *in-locus, any-I/O* escape hatch from the purity wall |

So a STATIC call `(<fqdn>::serve self …)` **will not type-check today** — it is a genuine mismatch,
and the `apply` through a runtime-built keyword is precisely what smuggles it past. This is the
class the arc has spent months annihilating: a form whose *function* is to evade the checker
(R63 — *you cannot compile a lie*; R57 — a mask). It is also exactly why a closure walk finds
nothing: **you cannot root a walk at a call that exists because it is unresolvable statically.**

**Therefore the act is NOT "swap `apply` for a keyword node."** That goes red on a real defect, and
the red is the point. The act is the question underneath, and it is a FORK FOR THE BUILDER:

> **What is `serve`'s `self` type, across both tiers?**
> (a) `serve` goes parametric over the self-peer type · (b) type `serve` at `Peer` (the wire
> contract) · (c) two `serve` emissions, one per tier · **(e) make the RELATION explicit:
> `Peer<S,R> <: ThreadSelfPeer<S,R>`, one-way.**

### MEASURED 2026-08-12 — (b) IS REFUTED, and the red named (e)

Flipped `lineage-peer-ty` to `Peer<…>` (one line) and ran the floor: **4389 run, 1 FAILED.**

```
probe_arc209_c2_defservice_dispatch::defservice_generates_dispatch_loop_round_trips_on_thread
  :my::counter::serve: parameter #1
    expects :wat::kernel::Peer<my::counter::Status,my::counter::Admin>;
    got    :wat::kernel::ThreadSelfPeer<my::counter::Status,my::counter::Admin>
  at tests/services/probe_arc209_c2_defservice_dispatch.wat:76:36
```

**The red was the MOBILITY WALL WORKING, and (b) was worse than refuted — it demanded the UNSAFE
subtype direction.** Read the arms carefully; the static site is a *hand-driver*, and production is
dynamic on BOTH sides:

| | peer VALUE it supplies | how it reaches `serve` |
|---|---|---|
| thread (shared memory) | `ThreadSelfPeer<Lu,Sh>` — `spawn.wat` ThreadOpts `launch` declares it on the prog it spawns | **`apply`** (same impl, ~20 lines down) |
| process (not shared) | `Peer<S,R>` — `:wat::program::self-peer` | **`apply`** (generated child main) |
| the one static site | a hand-written driver: `tests/services/probe_arc209_c2_defservice_dispatch.wat:76` declares `ThreadSelfPeer` to match serve's annotation | direct call |

So `serve`'s declared `self` is enforced in production by **nothing** — only by that fixture and by
serve's own self-recursion. And typing `serve` at `Peer` asks the thread tier's `ThreadSelfPeer`
value to pass as a `Peer` — **`ThreadSelfPeer <: Peer`, the direction that must NEVER hold**,
because it walks an in-locus peer holding live handles onto the wire. The checker refused exactly
that. Reverted; tree clean.

**Which means (e) needs NO change to `serve`'s declared type.** Keep it at `ThreadSelfPeer` (the
permissive head) and add the one safe edge `Peer <: ThreadSelfPeer`: the thread tier already
matches exactly, the process tier's `Peer` becomes statically passable, and the unsafe direction
stays unwritten and keeps failing — as the fixture just demonstrated it does.

**(e) is what the red surfaced, and it dissolves the fork instead of choosing a side.** The
relation is real and directional: `ThreadSelfPeer` = in-locus, permits ANY I/O; `Peer` = wire-safe,
permits PURE only. A value meeting the stricter contract meets the looser one, so with **identical
type args** `Peer<S,R>` is safely usable where `ThreadSelfPeer<S,R>` is expected. Then the thread
tier passes by exact match (as today), the process tier passes by the new edge, **the `apply`
dies**, and every future remote locus — all of them wire, all of them handing over a `Peer` —
passes for free. One branch in `is_subtype`; the same shape as R7's `:wat::core::Value` one-liner.

⛔ **THE EDGE MUST BE ONE-WAY, and the omission is the wall.** `ThreadSelfPeer` must NEVER be
accepted where `Peer` is expected — that direction would let an in-locus peer holding live handles
cross the wire, and 293.W's whole mobility wall falls. The reverse branch is the one you do not
write, and a test must assert it still refuses (R7's discipline: the top type is honest because of
the rule that was never added).

**📄 THE FULL REASONING IS FILED IN THE ARC THAT BUILT THE TOOLING:**
`docs/arc/2026/06/293-struct-record-symmetry/NOTE-peer-and-threadselfpeer-are-one-relation-never-stated.md`
— the two heads and the shared-memory line; where the line actually lives (the TWO `Locus`
`extend-type`s in `wat/spawn.wat:451`/`:523`, with `defservice` locus-blind by design, which is
what lets N remote loci join without touching it); the fact that the thread arm **ignores**
`service-forms` entirely because serve is already in the parent universe; the directional relation;
the ~24 lines across ~7 `check.rs` sites that enumerate the pair instead of stating it; today's red
verbatim; and the two hard requirements on any fix.

**Do not pick this alone — a new subtype edge is a ruling.** Once the type is settled the static
call, the one-entry `<fqdn>::child-entry [locus] -> nil`, the one-liner
`(defn :user::main [] -> nil (<fqdn>::child-entry :user::spawn::service-locus))`, one `fn-forms`
over it, and the death of `service-forms-def` all follow.

**Blast radius is every `defservice` in the corpus.** Draw the stone and BRIEF it; do not hand-roll.

### ⛔ STRUCK, FAILED, REVERTED (2026-08-12) — and the failure is the finding

The rider ran. **STOP-3 fired and the floor went RED: 4381 passed / 10 failed** (baseline 4391/0).
Reverted; tree clean at `2fff3749`. **Read `FINDING-fn-forms-cannot-walk-a-rete-dsl-body.md`
before re-attempting anything below.**

**THE PREREQUISITE: `fn-forms` raises on a rete PATTERN VARIABLE.**

```
malformed :wat::kernel::fn-forms form: …probe_arc278_sift_rules.wat:30:33:
  free symbol `?c` does not resolve to a parent define or substrate primitive
```

`?c` is DSL binding syntax inside a `defrule`'s `:when`, not a reference. The walker treats every
free symbol as something that must resolve, so it refuses. **The chaos engine (R25) IS a rete
service** — so the one-entry model works for a plain service and fails for the one this arc exists
to build. EXPOSED, not created: the hand-enumerated manifest was hiding it, and removing the
workaround is what found it (R57 again).

**It reaches BOTH tiers** — `own-forms-call` is spliced into `start`/`resume`, which every locus
evaluates (the thread arm discards the *value*, not the *call*). 4 thread-tier reds, 4 process, 2
tier-less.

⚠ **The rider scored "thread tier untouched" GREEN off ONE thread test + an empty `spawn.wat`
diff.** Both true; neither could see four red thread tests. A scorecard row "tier X untouched" must
be measured by that tier's WHOLE set, never a representative.

**Also open, and NOT to be assumed away:** `a_forked_service_that_cannot_decode_a_message…`
observed `Outcome/Message` where it requires `Outcome/Lost` — **UNCHARACTERIZED**. Plausibly the
same root; "probably" is not a disposition.

**Kept, proven, do not re-derive:** `child-entry`'s locus param types as
**`:wat::spawn::ProcessOpts`** (the abstract `Locus` arm of `infer_listener_prime`, `check.rs:9421`,
is pinned to 3 args; only `ProcessOpts` takes 3-or-4) · hygiene does NOT fire in a `<fqdn>::` defn ·
`fn-forms`'s 2nd arg needs `keyword/from-string` (a spliced literal auto-lifts to a `Fn`) ·
`manifest − walk = {<fqdn>::extract-addr}`, which is parent-side-only by `spawn.wat:575`.

### ▶▶ THE ARTIFACTS (correct about WHAT to build; they did not know the prerequisite)

| artifact | |
|---|---|
| `DESIGN-STONE-the-child-entry-kills-the-manifest.md` | the one contract decision, why it is possible NOW, out-of-scope REJECTED (not deferred), the four questions |
| `BRIEF-child-entry-static-call.md` | rooms as exact `file:line`, the implementation sketch, blast radius `wat/service.wat` ONLY, **5 numbered STOPs** |
| `EXPECTATIONS-child-entry-static-call.md` | 12-row scorecard fixed BEFORE the strike; rows **4** (walk ⊇ manifest) and **8** (thread tier untouched) load-bearing; 35–60 min, 2× box |
| `wat-scripts/scratch-pad/probe-arc278-child-entry-static-call.wat` | **the disconfirming probe, PROVEN BY RUN** |

**The probe settled both load-bearing claims before the brief was written:**

- **A** — a `Peer'<Status,Admin>` reaches `serve`'s `ThreadSelfPeer'` slot in a **static** call:
  `--check` **exit 0**. (Impossible before `310f8050`; this is what the peer edge bought.)
- **B** — `fn-forms` rooted there reaches the internals: **`CLAIM-B PASS`, closure = 30 forms**,
  including `serve` · `dispatch-admin` · `init` · `stop-project` · `hibernate-project` · every
  type · the surface + its backing records · the protocol `Op`/`Reply` · the per-op budget const.
  **That is the manifest, derived rather than remembered.**

Two form-lessons the checker taught while writing it, worth copying: the selectables element is ONE
tuple type-keyword, and its `Op` slot must be the **service** superset (`…::ce::Op`), because the
surface→service widening does **not** propagate through the tuple.

The single genuinely unproven thing is **STOP-1, the locus parameter's type** — the probe takes the
listener directly and deliberately does not answer it. That is a STOP, not a guess.

## The wall, and how to work with it

The five registries (`macro_registry` EXPAND · `types` CHECK · `functions`/`unit_variants`/
`runtime_def_values` EVAL) are **private**. Use `registrations(name)` — every facet — or a
**phase-named narrow accessor**. `RegistryKind` is exhaustive **by law**: a sixth registry turns
every match red until it is handled.

**MEASURED:** my best grep found 41 sites / 7 files. The wall found **197 errors / 11 files in
`src/` alone**, five of them files no grep of mine reached — and it caught my own codemod's
overreach. Census twice wrong; wall right immediately.

## ⛔ ALSO OPEN

**The lifecycle strike** — `DESIGN-STONE-connection-lifecycle-ops.md` + `BRIEF-…`, fully drawn,
ten STOPs. Its rider's work is `stash@{0}`, **unweighed** — read the diff, do not assume it is good.

**Filed, not scheduled:** `109/NOTE-two-resolvers-over-the-five-registries.md` — `runtime.rs`
≈`11644-11690` holds a pre-existing `Binding` walk over the same registries, in a **different
order**. The note explicitly does **not** rule on it; the `Binding` walk is unread.

**Owed intueri casts:** the admission type (`:wat::kernel::ConnectOutcome` is taken); the
correlation surface.

**Older:** #87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 · #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **A doc comment is a claim about the code, not a measurement of your program.** `TypeEnv`'s own
  comment described the retained/reconstructed split correctly and I still had to freeze a world to
  learn which side the failing types were on — and the answer killed the planned act.
- **An instrument that keys on a NAME cannot see a set of FACETS.** The dedup bug was predicted, in
  advance, by this arc's own census — 182 `[Macro, Type]` names — and I wrote it anyway.
  ([[feedback_impose_the_check_and_read_the_screams]])
- **When a check comes back CLEAN, ask what it cannot SEE.** The union's `declares:` list is a NAME
  census; it printed `:probe::ffx::Record` while the type declaration was gone, because the macro
  declared the same name. ([[feedback_a_pass_answers_only_the_question_the_instrument_asks]])
- **A fix can regress a caller one layer up.** `a5ac88ca` was correct and made the gate worse.
- **The report carried its own bug.** The gate's comment already said *"two entries of differing
  shape ⇒ the dedup's first-wins is unsound"* — I wrote that sentence, then read past the dump that
  satisfied it. (R66 `IN TENEBRIS VISVS CORRIGOR`, lived from the inside.)

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The arc came in holding a root-cause story with three faces. Two of the faces were the
> instrument, the third never existed, and the grounding the previous seam *demanded before
> building* is exactly what killed the act it was demanding it for. That is the discipline paying
> out: a written-down doubt caught a day of work aimed at the wrong file.
>
> The line that cost the most: **the measurement you already have does not help you if you reason
> past it.** The census named the trap eight days before I walked into it.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
