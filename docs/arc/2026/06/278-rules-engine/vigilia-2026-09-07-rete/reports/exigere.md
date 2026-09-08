## Exigere Ward — Cast Report

> Written verbatim as returned.

**Target:** `src/rete/kernel/**` (excluding `tests/`) + `wat/rete/oracle/**` — 22 Rust + 6 wat files, 15,771 lines (`wc -l` verified).

**Patterns grepped** (so a clean result is distinguishable from no cast):
- `TODO|FIXME|XXX|HACK` — **0 hits**, independently confirmed (matches prior ward's report).
- `unimplemented!|todo!|\bstub\b|placeholder` — 0 hits.
- `eventually|someday|later stone|next stone|for now|temporary|temporarily` — 0 hits.
- Full future-arc-deferral / scope-defense / disposition-deferral phrase list from the ward — **3 raw hits**, all investigated below.
- `rune:[a-z-]*` — 54 rune markers found, **0 are `rune:exigere`**.

### Findings

**1. `src/rete/kernel/stratify.rs:332`** — cites `RETE-OPEN-WORK.md` § 4.2 and the builder's eBPF-verifier framing.
Not a finding. Immediately resolved in the same sentence — "It already IS, for everything except termination" — and the ~40 lines following document the verifier's present, shipped state, including a bounded "WHAT IT CANNOT SEE" section and a struck-through dated closure ("~~A GUARDED COUNTER is refused THOUGH IT TERMINATES~~ — **CLOSED 2026-08-29**"). Present-tense documentation plus historical fact — exempt.

**2. `wat/rete/oracle/fire.wat:408`** — "WHY recursive rather than foldl-over-a-PV…"
Not a finding. A WHY-comment justifying the current shipped choice against an alternative — not a deferral. Substantive-annotation exemption.

**3. `src/rete/kernel/stratify.rs:620–632`** (L1 pattern-match, investigated in depth)

> `/// saying so is the fence half of this item, deliberately not attempted.`
> `// up — infeasible by exhaustion, which is why the fence half stays punted.`

Level: **L1 by literal pattern** (`punted`, no inline arc citation at the hit site) — but investigation shows the underlying decision is **already permanently closed**, not open deferred work. Checked `docs/arc/2026/06/278-rules-engine/RETE-OPEN-WORK.md`:
- `:1043` — `8. ~~THE TERMINATION VERIFIER — the fence half~~ · ✅ CLOSED 2026-08-29. BOTH HALVES ARE BUILT.` (a *different* fence half — the monotone-bounded-counter one).
- `:1338` and `:1471` — the *specific* fence-for-i64/f64/String idea this comment describes was investigated and explicitly closed as permanently refused: `★★★★★★★ THE FENCE HALF IS CLOSED — REFUSED, AND THE REPORT'S CENTRAL CLAIM IS WRONG.` — with the reason: population for these types is data-dependent (an `insert`-time seed), not a static type property, so "a static proof cannot reach the quantity that matters, because that quantity is an input." A final, dated, on-disk verdict, not an open TODO.

**Direction:** the wording is stale relative to what the arc doc already knows. "Deliberately not attempted" / "stays punted" reads as an open punt to a fresh reader who hasn't seen the closure; it should read as a **closed, permanent refusal**. A wording fix (option 1, "fix it"), not a tracked-arc gap. Reported because it pattern-matches the L1 list and a reader relying on the in-code comment alone would misread it as unfinished work.

**4. `src/rete/kernel/stratify.rs:605–614`** (adjacent; "What would change it… Raise it once a runtime STATE ceiling exists (item 8's other strike)")
Not a finding. The accepted "affirmative scope-bounding" form: an explicit present-tense decision ("A knob is deliberately NOT added") with a named, verifiable tracker. Confirmed "item 8" resolves on disk at `RETE-OPEN-WORK.md:1003` and `:1109` — real, dated, substantively discussed, not a dangling reference.

### Runes encountered (none are exigere)

54 `rune:` markers across the target, spanning 8 non-exigere categories: `struere` (17), `sequi` (11), `perspicere` (10), `lint` (8), `temperare` (5), `circumspicere` (1), `excusare` (1), `intueri` (1). **Zero** carry the `exigere` name or either of its categories. No verdict attempted on their internal reasoning beyond confirming none is a mislabeled exigere rune.

### Verdict

**CONVERGED.** No TODO-family hits (confirmed independently, not trusted from the prior ward). No genuine L1 or L2 exigere violations survive investigation — #1 and #2 were pattern-match false positives, #4 is a clean verified affirmative scope-bound, and #3 is a wording-staleness issue (a permanently-closed decision described in punt-language) rather than an open untracked deferral. Zero `rune:exigere` markers exist in-target, correctly — nothing here needed one.
