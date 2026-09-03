# DESIGN — STONE 1c-0a: five call sites name a verb that does not exist

> Ruled by the four questions in-chat, 2026-09-03: this goes FIRST, ahead of any 1c registration.
> Companion crawl: `[[DESIGN-CAMPAIGN-1c-the-lair-study-before-any-strike]]`.

## The measured ground

Five names in the corpus have **no definition anywhere** — no `#[wat_intrinsic]`, no
`#[wat_special_form]`, no dispatch arm, no `CheckEnv` scheme, no `wat/` `defn`, **and no
`RETIREMENT_TABLE` row.** Verified by asking each question of each name.

They pass a live gate today. `tests/lint/wat_scripts_fixes_load.rs` walks **every** `.wat` under
`wat-scripts/` recursively and type-checks it; all six files below are inside that set, and
`target/release/wat --check` was run on three of them by hand and exited **0**. They are green for
exactly one reason: `is_reserved_prefix` blanket-accepts anything under `:wat::`.

```
:wat::core::println          2  scratch-pad/probe-stone-2a-bracket-mechanics.wat:53 · t-bare.wat:1
:wat::core::edn::write       2  probes/arc-170/probe-process-only.wat:6 · probe-edn.wat:2
:wat::core::tuple-get        1  scratch-pad/arc109-2iii-fn-bracket-destinations.wat:55
:wat::core::reduce-walk      1  scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat:30
:wat::spawn::process/grants  1  probes/arc-170/probe-cap2-process-grantpath.wat:10
```

⚠ `[[NOTE-the-registry-is-not-yet-the-largest-membership-set]]` (earlier this arc) lists
`println · None · tuple-get · reduce-walk · edn::write` under the heading **"verbs"** — as
population awaiting registration. **That classification is wrong and this stone corrects it:**
they are not verbs the registry lacks; they are names nothing defines.

## ⛔ THEY ARE NOT ONE KIND, AND THE SPLIT IS THE STONE

**Two are namespace slips with a registered target.** Mechanical, unambiguous:

```
:wat::core::println     →  :wat::kernel::println     (REGISTERED)
:wat::core::edn::write  →  :wat::edn::write          (REGISTERED)
```

★ `probe-edn.wat:2` calls `:wat::kernel::println` and `:wat::core::edn::write` **on the same
line** — the correct spelling and the slipped one, two tokens apart. That is what a laundered
namespace looks like.

**Three are artifacts whose central claim was never true.** Each file documents what it proves;
each names a verb that does not exist at the exact point where the proof happens:

- **`bench-reduce-foldl-vs-seqable-walk.wat`** — a bench comparing native `foldl` against *"what
  a collapsed `reduce` would do"*. **ARM B calls `:wat::core::reduce-walk`, which was never
  built.** The bench's own header warns readers how to cite its number; its comparison arm
  cannot run. ⬜ Its header also carries a real standing ruling that must not be lost with it.
- **`probe-cap2-process-grantpath.wat`** — *"Prove the PROCESS grant path… runs end-to-end."* The
  grant path is `(:wat::spawn::process/grants …)`. **No `:wat::spawn::` name is registered at
  all** (the whole surface is wat-side), and `wat/spawn.wat` defines no `grants` verb. The probe
  proves nothing it claims.
- **`arc109-2iii-fn-bracket-destinations.wat`** — `(:wat::core::tuple-get t 0)`. **The corpus has
  no Tuple accessor** — one call site, one spelling, used nowhere else, and no registered
  `Tuple/*` reader exists in `:wat::core::`.

★★★ **The shape all three share: an artifact that documents itself as proving or measuring
something, whose load-bearing call names a verb that does not exist — so it has never done what
it says, and the whitelist is why nobody found out.** That is a sharper finding than "corpus
rot," and it is `is_reserved_prefix`'s second indictment after the `>X` probe.

---

## ⛔ CORRECTED AFTER THE STONE RAN — I was wrong three times, and the truth is worse

The rider did the git archaeology this design did not, and **all three of the paragraphs above
are wrong in the same direction: these verbs are not names that never existed. They are names
that were BUILT, USED, and then DELIBERATELY RETIRED — and nothing propagated the death.**

**① `reduce-walk` existed and the bench WORKED.** Built `663e5daee` (2026-08-17); the bench was
authored the next day (`4de24007f`) to measure it; **it produced a real number — `5.1×` — quoted
verbatim in `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.B6-native-foldl-over-seqable.md`,
where it is the load-bearing justification for a standing ruling** (verified: that doc names this
bench file as its source). `reduce-walk` was then deleted (`6c84ddf01`, same day, 118.B7) with
`wat/seq.wat:311`'s comment *"A name dies in the stone that removes its last caller."*
★ **But the bench WAS a caller — tracked, gated, type-checked.** The stone removed the last caller
it knew about. The claim was never false; the artifact rotted the day its subject was retired.

**② `process/grants` existed for about ten hours.** Built `36f3acbc8` (2026-07-08 11:03), deleted
`bc472c7ce` (same day, 21:09). ★★★ **The probe was promoted into the tracked corpus the NEXT DAY**
(`661a32216`, 2026-07-09) — already naming a retired verb — and it "froze clean" because the
blanket accept type-checks it. **The promotion gate could not see that the probe's subject was
already dead.** The grant mechanism moved twice more since; no rename can fix this probe, because
its whole shape (grant info riding the locus argument) was superseded by a kwargs tail on
`bracket/map`.

**③ MY OWN CLAIM WAS FLATLY FALSE: the corpus DOES have a Tuple accessor.** `check.rs`'s
`infer_positional_accessor` doc says it outright — *"Polymorphic over (Vector :- [T]) and tuple —
both are index-addressed. Rank-1 HM can't express the union, so this is special-cased."*
`(:wat::core::first t)` reads index 0 of a Tuple, and `wat/rete.wat:300` does exactly that on a
live `(Tuple :- [Record i64])`. I searched for `tuple-*` and `Tuple/*` renderings, found none, and
published **absence** — from a pattern that could never have matched the accessor, because the
accessor is not tuple-shaped. `[[feedback_a_census_of_a_name_must_ask_every_rendering]]`, in a
design written the same day I cited that memory.

## ★★★ THE REAL CLASS, and it is `is_reserved_prefix`'s THIRD and sharpest indictment

**Neither retired name is on the `RETIREMENT_TABLE`** — verified for `reduce-walk`,
`process/grants` and its successor `process/uses`. The substrate has retirement machinery (the
table, `retired_name_justified`, the doctrine that a name dies with its last caller) and **none of
it can fire for a `:wat::*` head, because the checker never validates one.**

So the whitelist does not merely hide typos. **It defeats the retirement machinery**: a verb can
be deleted while gated corpus files still call it, and every gate stays green. Phase 3a is not
tidying a redundant authority — it is restoring the only mechanism that could have caught any of
this.

## THE FOUR QUESTIONS — the disposition of the three, per option

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **rename the two; MEASURE + REPORT the three** | YES | YES | YES | YES | ✅ **PICKED** |
| rename two, delete the three files | YES | YES | **NO** | — | ⛔ |
| rename two, invent the missing verbs | **NO** | NO | **NO** | — | ⛔ |
| do nothing until 3a forces it | YES | YES | **NO** | — | ⛔ |

- **delete — Honest NO.** The bench's header carries a standing ruling about citing benchmark
  numbers; the probes encode real intent about paths worth proving. Deleting the artifact deletes
  the question it was asking, and hides that the question was never answered.
  `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`.
- **invent — Obvious NO, Honest NO.** Building `reduce-walk` or `process/grants` to satisfy a
  dead caller manufactures substrate from corpus rot. It is the exact inversion of the RULING.
- **do nothing — Honest NO.** 3a would then report these five failures mixed with its real ones,
  unable to say which is which, under pressure to reach green.

## Acceptance — DERIVED

```
                     before   after   why
the corpus 68          68      66     ⬅ ONLY the two renames. −2 names, −4 call sites.
                                      The other three names STAY until their disposition
                                      is ruled — a stone that removed them by deleting
                                      artifacts would report the same number dishonestly.
GAP_A                   60      60    none of the five is on it
GAP_B                   68      68    none of the five is on it (they are corpus-only)
DEBT                   106     106    nothing registered
floor            5127/5127  5127/5127
clippy                            0
every_wat_scripts_file_loads      still green — the two rewritten files must still load
```

⚠ **The corpus number is the only thing that moves, and it moves by 2.** This stone's real
deliverable is the report on the three, not the count.

## Out of scope — CUT

- **`:wat::rete::f64::>X`.** Evidence, not rot — a committed negative control whose header is this
  arc's founding indictment. It belongs to **3a**, as a compile-fail expectation.
- **The four `:wat::type::*` rendering rows.** That is `1c-0b`, and it is BLOCKED on proving what
  emits them.
- **Registering anything.** No `:wat::core::` verb is registered by this stone.
