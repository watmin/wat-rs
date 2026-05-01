# Arc 109 — Landing Order

Captured 2026-04-30 after arcs 110–113 + 115 + 116 closed. **Updated
2026-05-01** after arcs 114 + 117 closed. This is the execution order
for everything that remains in arc 109's orbit plus the downstream
arcs § J unblocks.

Compaction-amnesia-resistant: this file is the durable record. If
the conversation context dies, the next session reads this and the
order is preserved.

## Status snapshot — what's done, what's left

**Done (arcs that spawned from arc 109 blockers):**

| Arc | Subject | Closure |
|---|---|---|
| 110 | kernel-comm-expect (silent disconnect → compile error) | shipped |
| 111 | result-option-recv (Result<Option<T>, ThreadDiedError>) | shipped + closure |
| 112 | inter-process-result-shape (Process<I,O> + process-send/recv) | shipped |
| 113 | cascading-runtime-errors (Vec<*DiedError> chains) | shipped + closure |
| 114 | spawn-as-thread (Thread<I,O> + spawn-thread; R retired) | shipped + closure (2026-05-01) |
| 115 | no-inner-colon-in-parametric-args (`:Vec<:String>` illegal) | shipped |
| 116 | phenomenal-cargo-debugging (Diagnostic + WAT_TEST_OUTPUT) | shipped |
| 117 | scope-deadlock-prevention (compile-time lockstep enforcement) | shipped + closure (2026-05-01) |

**Done within arc 109:**

- 1a: parser accepts FQDN primitive types
- 1b: sweep wat sources to FQDN primitive types
- 1c: retire bare primitive types in user code — shipped 2026-05-01.
  `BareLegacyPrimitive` CheckError variant + data-driven walker via
  `parse_type_expr_audit`; ~1000 rename sites across ~90 files;
  cargo test workspace 1476/0. See `SLICE-1C.md` for the full
  record. Pattern 3 (dedicated variant + walker) proven for the
  bigger arc-109 slices ahead (§ B/C/D/D').
- § J 10a: `:wat::kernel::Program<I,O>` typealias minted (alias for `:Process<I,O>`)
- § J 10b: sonnet sweep — annotations prefer Program (in scope of stdlib boundaries)
- Arc 114 absorbed § J 10c's "Thread as concrete struct"
  prerequisite — `Thread<I,O>` now exists as a concrete struct
  (sibling to `Process<I,O>`), with `spawn-thread` + `Thread/input`
  + `Thread/output` + `Thread/join-result` shipped. The `Program<I,O>`
  typealias still points at `Process<I,O>` per slice 10a; § J 10d's
  typeclass dispatch is what makes Program a real abstract supertype.
- Migration hints retired (arcs 111/112/113 helpers; collect_hints
  scaffold preserved for the next migration arc)

**Remaining — split into two classes:**

### Structural (waits on § J)

These have architectural dependencies; they ride § J's typeclass
infrastructure:

1. **§ J slice 10d** — typeclass dispatch + `ProgramDiedError`
   supertype. Mint `:wat::kernel::ProgramDiedError` enum with the
   same variant shapes both ThreadDiedError and ProcessDiedError
   satisfy. Mint the satisfaction relation (Program<I,O> as a
   structural protocol with Thread<I,O> and Process<I,O> as
   concrete satisfiers). Mint polymorphic `Program/join-result`
   verb that dispatches on the concrete type.

2. **§ J slices 10e–10g** — sonnet sweep call sites. Mint typed
   `Thread/send`, `Thread/recv`, `Process/send`, `Process/recv`
   (slice 10f); mint polymorphic bare `:wat::kernel::send` /
   `:wat::kernel::recv` over `Program<I,O>` (slice 10g). Retire
   the `process-send` / `process-recv` arc-112 spelling at this
   point.

3. **Arc 113 widen** — `ProcessPanics` / `ThreadPanics` →
   `ProgramPanics`. One-token element-type change in the
   typealias bodies + chain emit + chain parse. Depends on § J's
   `ProgramDiedError` supertype.

### Independent sweeps (do NOT depend on § J)

These are parallel-shippable; sonnet-delegatable with the
substrate's diagnostic stream as the brief (Pattern 3 from
SUBSTRATE-AS-TEACHER):

4. **1d** — mint `:wat::core::unit`; retire `:()` as a type
   keyword. Substrate add + sweep.

5. **9d** — `:wat::std::stream::*` → `:wat::stream::*` (the
   stream stdlib's namespace claims promotion).

6. **9e** — `:wat::std::service::Console::*` →
   `:wat::console::Console::*` (Console gets its own namespace
   path matching its substrate-claim shape).

7. **9f–9i** — file-path moves for already-honest-symbol files
   (the file location catches up with the symbol path).

### Discovered-during-sweep follow-ups (lower priority)

Arc 109 is append-only as we find things. Items below were
surfaced during slice work, are real, but rank below the planned
slices.

- **1d post-rename: `:wat::core::unit` → `:wat::core::Unit`.**
  Surfaced during slice 1d via /gaze. Lowercase `unit` mumbles in
  the company of `Vec`/`Option`/`Result`/`HashMap`/`HashSet`
  (substrate-named PascalCase types) and borderline-lies by
  pattern-matching to the lowercase verb-path family
  (`:wat::core::map`, `:wat::core::filter`, etc.). The Rust-
  primitive-lowercase argument doesn't apply: Rust has no `unit`
  keyword, so the wat name is invented and falls under wat's
  nominal-type taxonomy, where typed-things are PascalCase. Cheap
  to flip post-1d-sweep: `s/::unit/::Unit/g` plus walker emit
  string + alias name; substrate-as-teacher mechanism re-runs to
  verify. Do AFTER slice 1d's sweep finishes — small judgment
  call, big-blast-radius rename, but the mechanism is rehearsed.

## The opus / sonnet split

**Opus territory (rich context, architectural decisions):**
- § J slices 10a–10d (the substrate surgery)
- Arc 114 (the protocol-as-contract reframe)
- Arc 113 widen (judgment calls about Vec element-type
  generalization)

**Sonnet territory (structural sweeps with substrate-as-teacher
hints):**
- § J slice 10e (post-substrate-flip call-site sweep)
- 1c, 1d, 9d–9i (namespace moves with `arc_109_slice_N_migration_hint`)

The reflex: every structural slice that ships in this arc gets
an `arc_109_slice_N_migration_hint` in `src/check.rs::collect_hints`
during the wave, retired once the wave clears. Same lifecycle as
arcs 111/112/113's hints (retired 2026-04-30 in commit
`6da1fef`).

## The order

**Dependency correction noted 2026-04-30 mid-pipeline.** § J slice
10c ("split Program back into Thread + Process") requires
`Thread<I,O>` to EXIST as a concrete struct with arc-114's transport
asymmetry (Sender/Receiver fields, not IOWriter/IOReader). Arc 114
slice 1 is what mints Thread. So arc 114 slices 1–3 land BEFORE
§ J 10c, not after. § J 10c ≡ arc 114 slice 4 — the interlock both
slice plans name.

```
[done]  § J 10a — :Program<I,O> typealias for :Process<I,O>
[done]  § J 10b — sonnet sweep: annotations prefer Program

[done]  arc 114 slice 1 — Thread<I,O> + spawn-thread minted
                          (concrete struct sibling to Process<I,O>;
                           Sender/Receiver fields per the transport
                           asymmetry; closes § J 10c's Thread-must-
                           exist prerequisite)
[done]  arc 114 slice 2 — substrate sweep: spawn → spawn-thread
[done]  arc 114 slice 3 — retire bare spawn + ProgramHandle<R>
[done]  arc 114 slice 4 — manual fixes (5 files)
[done]  arc 117 — compile-time scope-deadlock prevention
                  (NEW; surfaced live during arc 114 slice 4;
                  lifts SERVICE-PROGRAMS lockstep into a structural
                  type-check rule; covers Thread/join-result AND
                  Process/join-result, plus future poly verb)
[done]  arc 114 closure — INSCRIPTION + USER-GUIDE + cheatsheet
                  + 058 row
[done]  arc 117 closure — INSCRIPTION + WAT-CHEATSHEET § 10
                  + USER-GUIDE § 14 gotcha + 058 row

[done]  arc 109 slice 1c — BareLegacyPrimitive variant + walker
                  + four-tier sweep; ~1000 rename sites across
                  ~90 files; cargo test workspace 1476/0
                  (commits f2b5dd4 → e0abbfa; SLICE-1C.md)

[next]  § J 10d — typeclass dispatch + ProgramDiedError supertype
                  (mint ProgramDiedError; mint Program<I,O> as
                  abstract protocol; mint poly Program/join-result;
                  Vec<ProgramDiedError> chain for arc 113's
                  deferred widening)

        § J 10e — sonnet sweep: call sites use polymorphic forms
        § J 10f — typed Thread/send, Thread/recv, Process/send,
                  Process/recv (renames arc-112's process-* under
                  § J's naming convention)
        § J 10g — polymorphic :wat::kernel::send / recv over
                  Program<I,O> via the typeclass mechanism

        arc 113 widen — Panics → ProgramPanics (one-token
                  element-type change in the typealiases)

[parallel-shippable, sonnet-delegatable] independent sweeps:
        § J 1c (retire bare primitive types in user code)         ← resuming next
        § J 1d (mint :wat::core::unit; retire :() as type)
        § J 9d (:wat::std::stream::* → :wat::stream::*)
        § J 9e (:wat::std::service::Console::* → :wat::console::Console::*)
        § J 9f-9i (file-path moves for already-honest-symbol files)

        arc 109 INSCRIPTION — closes the entire arc + 058 row
```

## Why this order

**Arc 114 slice 1 unblocks § J 10c** — that's the substrate gate.
Splitting Process<I,O> back into Thread<I,O> + Process<I,O> with
ProgramDiedError
satisfied-by-both is structural; every downstream slice either
exploits the new abstractions or is independent of them. Landing
§ J before 114 means the rename happens once instead of twice
(today's `Process<I,O>` from `spawn-program-ast` becomes
`Thread<I,O>`, AND today's `Process<I,O>` from `fork-program-ast`
stays `Process<I,O>`).

**Arc 114 next** — it depends on `Thread<I,O>` existing and
exploits the typeclass dispatch § J slice 10d minted. It also
unblocks the cleanest framing of the protocol: **hosting is user
choice; protocol is fixed**. No reason to ship 114 before § J;
every reason to ship it second.

**Arc 113 widen alongside 114** — same mechanical change shape
(typealias body update + sonnet sweep). Can ride 114's sweep
momentum.

**Independent sweeps last** — 1c, 1d, 9d–9i don't structurally
depend on anything; they're stylistic / namespace clarity work.
Land them in the closing wave when the substrate is settled.

**Arc 109 INSCRIPTION** — closes the whole arc. Records the
realization that arc 109 birthed seven downstream arcs (110–116)
+ § J's structural reframing + arc 114's protocol-as-contract.
The "kill std" name is a third of what the arc actually
accomplished.

## Cross-references

- `INVENTORY.md` § J — full description of the supertype split,
  ProgramDiedError mirror, typeclass dispatch scheme.
- `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` — the protocol-
  as-contract framing arc 114 ships.
- `docs/arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md` —
  arc 113's known limitation: `ProcessPanics` / `ThreadPanics`
  element type widens to `ProgramPanics` post-§J.
- `src/check.rs::collect_hints` — where each slice's
  `arc_109_slice_N_migration_hint` lands during its sweep wave.

## Honest caveats

- **§ J 10a–10c are effectively done.** 10a (Program typealias)
  shipped early; 10b's sonnet sweep landed in scope; arc 114
  absorbed 10c (Thread<I,O> as concrete sibling to Process<I,O>).
  The remaining § J substrate work is 10d (typeclass dispatch +
  ProgramDiedError) → 10e–g sweeps. Opus session needed for 10d
  (the structural piece); 10e–10g are sonnet sweeps.

- **Arc 114 + arc 117 shipped 2026-05-01.** Arc 114 retired the
  R-via-join asymmetry; arc 117 made the lockstep discipline a
  compile-time rule. Arc 117 was NOT in the original plan — it
  surfaced live during arc 114's HologramCacheService.wat
  migration, when a sibling-scope deadlock hung the runtime
  with no diagnostic. The rule's existence pays forward into
  every future Program/join-result slice (poly verb in 10d,
  typed verbs in 10f, poly bare verbs in 10g) — they all inherit
  the structural deadlock check.

- **The "kill std" name retires last.** Arc 109's original framing
  was "FQDN every substrate-provided symbol; flatten std." The
  name became a placeholder for the broader cleanup that emerged
  during arcs 110–117's sequence. INSCRIPTION will rename the
  arc (or document the rename) at closure.
