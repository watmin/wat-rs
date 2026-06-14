# REALIZATIONS — arc 209 (defservice)

Disciplines and insights named while building defservice. Each entry dated, grounded
against the disk. (Project convention: `REALIZATIONS.md` per arc dir.)

---

## 2026-06-13 — The braid that made defservice trivial (build the floor; keep the bridge; flip once)

**The observation, drawing Stone C.1.** The whole counter service, written as `defservice`
(surface option A, inline bodies), is ~10 lines. The *same* service hand-rolled —
`crates/wat-lru/wat/lru/CacheService.wat` — is **530 lines**: protocol enums, `HandlePool`,
pair-by-index reply routing, `loop-step`, per-op client helpers, the spawn wiring. defservice
doesn't shrink that work; it *generates* it. Why is this suddenly easy?

**The grounded answer — a convergence, not a sequence.** Two honesty-pursuits ran in the same
era and met at defservice. (Chronology matters, and the tidy story gets it backwards: the
concurrency rebuild *predates* the clojure-migration promotion — so this is a braid of two
strands, not a single "we went clojure, which forced a concurrency pause" line.)

- **Strand 1 — honesty of FORM (EDN + faithful Clojure).** Arc 213 (ship a wat program as EDN
  over the wire) surfaced the non-EDN abuses: struct-destructure (odd-arity map), `::`-keyword
  call-heads, `/`-in-keyword. Builder's call (251 DESIGN status header, 2026-06-09): *"the wat
  invented forms… were a bridge to get us here… we go for parity."* → arc 251 (types-as-forms),
  + arc 257 (EDN-native Map/Set). Forms that can't cross the wire as clean EDN, or can't be
  faithful Clojure, get **retired or rebuilt — not migrated**.

- **Strand 2 — honesty of CONCURRENCY (deadlock-free).** defservice was first designed
  (May 2026) against concurrency tooling that was then shelved. The month-long deadlock
  annihilation rebuilt the substrate beneath it:
  - arc **170** (2026-05-09) — program entry points; the annihilation begins.
  - arc **214** (2026-05-18) — `typed_channel` dies; the unified transport-blind `Peer`.
  - arc **249** (2026-06-04) — threading reborn as wat; total-pure macro engine (the tooling
    `defservice` itself is written in).
  - arc **259** (2026-06-11) — `spawn-program'` (the host-type defclause).
  - + arc 209's own C0b campaign — `Peer`/`Listener`/`Address` unified, deadlock-free `poll'`,
    the `SO_PEERCRED` gate.

  The regrounded design says it plainly: *"the rebuild… produced exactly the idealized tooling
  defservice's design assumed it would have to hand-roll."* (`DESIGN-REGROUNDED-2026-06-12.md`.)

**The convergence.** defservice sits at the intersection of the two strands: it can't be written
honestly in the bridge surface, **and** it can't run on the old tooling. So it waited (209
reactivated 2026-06-12). When both strands landed, the 530-line hand-roll collapsed into a
~10-line declaration whose macro *expansion* is those 530 lines — the op enum, the `poll'`
dispatch loop, the client wrappers, the start fn.

**The discipline (the actual lesson).** Arc 251 — the clojure surface cutover — is *deliberately
parked.* We build defservice + the concurrency tooling **first, in current `:wat::` syntax**, and
defer the surface flip to one coordinated cutover (the mechanisms are reader-agnostic; e.g. an
env-fn `(app/beta-fn)` falls out free on cutover, unchanged mechanism). You do not migrate a
surface onto foundations that are not real yet. You build the floor, keep the bridge-forms until
it holds, then flip once. **Pausing to address what can't migrate or can't run yet — defservice,
the concurrency tooling — is *why* the migration will be clean and *why* defservice is trivial.**

**Cross-references:**
- `DESIGN-REGROUNDED-2026-06-12.md` — "the rebuild produced exactly the idealized tooling."
- `docs/arc/2026/06/251-types-as-forms/DESIGN.md` — the parity call + the "bridge" framing.
- `crates/wat-lru/wat/lru/CacheService.wat` — the 530-line hand-rolled reference defservice generates.
- `DESIGN-STONE-C.1-defservice-skeleton-op-enum.md` — the surface (option A) this entry was drawn against.
