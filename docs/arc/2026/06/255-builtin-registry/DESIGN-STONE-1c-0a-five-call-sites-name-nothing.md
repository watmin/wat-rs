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
