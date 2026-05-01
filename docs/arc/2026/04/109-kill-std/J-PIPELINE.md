# Arc 109 — Landing Order

Captured 2026-04-30 after arcs 110–113 + 115 + 116 closed. This is
the execution order for everything that remains in arc 109's orbit
plus the downstream arcs § J unblocks.

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
| 115 | no-inner-colon-in-parametric-args (`:Vec<:String>` illegal) | shipped |
| 116 | phenomenal-cargo-debugging (Diagnostic + WAT_TEST_OUTPUT) | shipped |

**Done within arc 109:**

- 1a: parser accepts FQDN primitive types
- 1b: sweep wat sources to FQDN primitive types
- Migration hints retired (arcs 111/112/113 helpers; collect_hints
  scaffold preserved for the next migration arc)

**Remaining — split into two classes:**

### Structural (waits on § J)

These have architectural dependencies; they ride § J's typeclass
infrastructure:

1. **§ J slices 10a–10d** — substrate work. Mint `Program<I,O>`
   supertype; split into `Thread<I,O>` (returned by `spawn-program`)
   and `Process<I,O>` (returned by `fork-program`). Mint
   `ProgramDiedError` as the error supertype satisfied by both
   `ThreadDiedError` and `ProcessDiedError`. Mint typeclass
   dispatch for polymorphic verbs.

2. **§ J slices 10e–10g** — sonnet sweep call sites. Mint typed
   `Thread/send`, `Thread/recv`, `Process/send`, `Process/recv`
   (slice 10f); mint polymorphic bare `:wat::kernel::send` /
   `:wat::kernel::recv` over `Program<I,O>` (slice 10g). Retire
   the `process-send` / `process-recv` arc-112 spelling at this
   point.

3. **Arc 114** — `spawn-as-thread`. Kill spawn's `R` return type.
   Threads stream results out via channels; `Thread<I,O>` is the
   contract. Meta-principle: **hosting is user choice; protocol
   is fixed**. Depends on § J's `Thread<I,O>` type existing.

4. **Arc 113 widen** — `ProcessPanics` / `ThreadPanics` →
   `ProgramPanics`. One-token element-type change in the
   typealias bodies + chain emit + chain parse. Depends on § J's
   `ProgramDiedError` supertype.

### Independent sweeps (do NOT depend on § J)

These are parallel-shippable; sonnet-delegatable with
substrate-as-teacher hints:

5. **1c** — retire bare primitive types in user code (sweep all
   `:Vec<i64>` etc. → `:Vec<wat::core::i64>`).

6. **1d** — mint `:wat::core::unit`; retire `:()` as a type
   keyword. Substrate add + sweep.

7. **9d** — `:wat::std::stream::*` → `:wat::stream::*` (the
   stream stdlib's namespace claims promotion).

8. **9e** — `:wat::std::service::Console::*` →
   `:wat::console::Console::*` (Console gets its own namespace
   path matching its substrate-claim shape).

9. **9f–9i** — file-path moves for already-honest-symbol files
   (the file location catches up with the symbol path).

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

```
                    [opus context → § J substrate]
                              │
                              ▼
              ┌───────────────────────────────────┐
              │  § J slice 10a — Program<I,O>     │
              │  § J slice 10b — Thread/Process   │
              │  § J slice 10c — rename today's   │
              │      unified Process → Program    │
              │  § J slice 10d — typeclass        │
              │      dispatch + ProgramDiedError  │
              └───────────────────────────────────┘
                              │
                ┌─────────────┼─────────────┐
                ▼             ▼             ▼
         [sonnet sweep] [opus arc 114] [opus arc 113w]
              §J 10e        kill R       Panics widen
              §J 10f        threads      element-type
              §J 10g        stream                       
                via channels
                              │
                              ▼
                    [sonnet — independent sweeps]
                       1c   1d   9d   9e   9f-9i
                              │
                              ▼
                       arc 109 INSCRIPTION
                       (the closure that
                        retires the entire
                        arc + 058 row)
```

## Why this order

**§ J first** — it's the substrate gate. Splitting Process<I,O>
back into Thread<I,O> + Process<I,O> with ProgramDiedError
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

- **§ J is not yet started.** All slices 10a–10g pending. Opus
  session needed to land 10a–10d cleanly (the structural piece);
  10e–10g are sonnet sweeps once 10d's typeclass dispatch is in.

- **Arc 114 DESIGN drafted, not implemented.** Captured at
  `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md`. Reads as
  ready-to-implement once § J lands.

- **The "kill std" name retires last.** Arc 109's original framing
  was "FQDN every substrate-provided symbol; flatten std." The
  name became a placeholder for the broader cleanup that emerged
  during arcs 110–116's sequence. INSCRIPTION will rename the
  arc (or document the rename) at closure.
