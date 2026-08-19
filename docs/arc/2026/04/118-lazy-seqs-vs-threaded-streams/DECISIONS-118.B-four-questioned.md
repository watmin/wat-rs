# DECISIONS — 118.B, every open fork, every option, four-questioned. 2026-08-17, against `a5822547`.

Builder: *"four-questions to inform the debate for any decisions."*

**Every option gets a flat YES or NO on each question — including the options that obviously lose.**
A lean hides which axis decided, and forced enumeration is what surfaces the option that reads best
and fails Honest. `[[feedback_four_questions_for_any_multi_option_decision]]`

Obvious + Simple + Honest must all hold **before** Good UX is weighed; once one is NO the rest are
recorded *unreached*, not guessed.

⚠ **Nothing here is ruled.** These are the grounds, not the verdict.

---

## ⛔ FIRST — A CORRECTION TO MY OWN FRAMING, because it moves the trade

In the previous round I said route 2's cost was *"the 9.1× — population C, the builder's own idiom."*
**That was wrong, and it overstated the case against `Seqable`.**

Population C is a **wat-closure generator** (`:probe::counter`) — that is **user code**. A user writes
their producer in wat under *either* route; no route makes a user's own generator native. And C's
3,124 B/element is **memo retention**, which dies when the memo dies — under **both** routes.

So:

```
C's 3,124 B/element   →  user-generator cost, present under BOTH routes,
                          fixed by DELETING THE MEMO, not by either route
the A-vs-B trade      →  STDLIB VERB throughput only: population B's 343 B/element
                          and the interpreted-vs-native wall clock
```

The memory fix belongs to the memo, not to the route. **The route decides how fast the stdlib's own
sequence verbs run, and who is allowed to write one.** That is a narrower and more honest trade than
I stated.

---

# DECISION 1 — where the sequence walks live

Four options. All four delete the seven `-stream` twins; that much is already ruled
(278, 2026-07-31: *"a workaround for the missing type, not a pattern"*).

## Option A — NATIVE. Verbs become Rust intrinsics; the 8 wat walkers are deleted.

- **Obvious? YES.** Five verbs already work exactly this way (`map`, `take`, `drop`, `filter`,
  `seqable->stream`). A sixth is not a new idea, and no caller's signature moves.
- **Simple? YES.** One body per verb. No new type-system machinery. Deletes the twins and their
  ~29 identical arms and adds no concept.
- **Honest? NO.** It reaches the **check** rung, never `no-form`, and the 278 stone says so in its
  own words: *"nothing stops a new wat-level stage with per-container arms and a `rest`-walk
  tomorrow, and it would be quadratic and green."* The lint it promised to convert that convention
  into a wall **does not exist**. And its own filed risk is unaddressed on disk: *"278 removes the
  pain that would motivate this… degrades to 'improves legibility' — the class of work that never
  gets scheduled."* Choosing it a second time is choosing it knowing that.
  `[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`
- **Good UX?** *Unreached.*

## Option B — SEQABLE, walks stay in wat. Surface + 4 `extend-type`s; one clause per verb, walking with `next`.

- **Obvious? YES.** It is Clojure's `ISeq` — the stated familiarity target — and R28's own model of
  what a surface is. One `keep`, not five arms plus a twin.
- **Simple? YES.** ~20-line `defsurface` + four `extend-type`s, on a mechanism that landed yesterday
  and is green (118.3-B). This was 278's flat NO; the three blockers that produced it are dead.
- **Honest? YES.** Reaches **no-form**: a verb that never names a container cannot hand-roll a
  per-container walk. It also deletes the concept's *duplicate spelling* — today "what is seqable"
  lives in the checker AND implicitly in ~29 `defclause` arms.
- **Good UX? YES.** ★ **And this is the discriminator, in the builder's own words: *"there must not
  be N ways to do a thing."*** Under B the stdlib and the user write **the same thing** — a verb over
  `Seqable<T>` walking with `next`. Under A (and C) the stdlib writes Rust and the user writes wat:
  two ways, permanently, by construction.
- **The cost, stated plainly:** the stdlib's own verbs stay interpreted — population B's 343 B/element
  and ~5.8× wall against native, **until the bytecode compiler lands.**

## Option C — SEQABLE as the TYPE, NATIVE bodies. (Enumerated because the instruction says enumerate; it reads best of all.)

Mint the surface and extend the four containers so wat can *name* the type, but leave the stdlib's
verb implementations in Rust. `extract_lazyable_elem` is replaced by real surface satisfaction at
check time; `StreamContainer` keeps doing value-level dispatch at runtime.

- **Obvious? YES.** The type is nameable, the verbs behave exactly as they do today, and nothing
  regresses. This is why it reads best: it looks like B's win with A's performance.
- **Simple? YES.** Type-level truth in the surface, value-level dispatch in `StreamContainer` —
  genuinely two different questions, not two answers to one.
- **Honest? NO.** ★ **It ships the split brain it claims to close.** The surface would advertise
  "this is what a sequence verb accepts" while every sequence verb in the language is a Rust
  intrinsic no user can imitate. A user who names `Seqable<T>` and writes a lazy stage in wat has
  written a *second-class* verb — slower, differently implemented, and the stdlib will never look
  like it. That is precisely **N ways to do a thing**, with a type on top making it look like one.
- **Good UX?** *Unreached.*

★ **This is the option the forced enumeration exists to catch** — it reads best and fails Honest, and
I had not listed it at all in the previous round.

## Option D — DO NOTHING. Keep the twins.

- **Obvious? NO.** Three ways to write a sequence verb (native · wat+twin · wat armless), and
  `keep`'s five `defclause` arms have byte-identical bodies. That is the missing type rendered as
  code, and a reader cannot tell which shape is correct.
- **Simple?** *Unreached.* · **Honest?** *Unreached.* · **Good UX?** *Unreached.*

## Where decision 1 lands

**A fails Honest. C fails Honest. D fails Obvious. B passes all four.**

B's cost is real and is **not** a four-questions failure — it is a performance number with a known
expiry (the bytecode compiler). The failure modes of A and C do not expire; they are structural.

---

# DECISION 2 — the three doors (`first` / `rest` / `empty?` on a Stream)

Route-independent. This is a **dialect** ruling.

## Option A — close all three on Stream; the three-call walk becomes unrepresentable.

- **Obvious? YES**, provided the diagnostic names `next` as the replacement. One way to walk a stream.
- **Simple? YES** at the surface — one rule, no exceptions. ⚠ The *implementation* is asymmetric and
  must be budgeted: `first` and `rest` are one capability bit each, but **`empty?` has no
  compile-time gate at all** and needs its `∀T. T -> bool` scheme changed. Asymmetric work, not a
  complex result.
- **Honest? YES.** The only option where "user code runs exactly once per element" is **structural**
  rather than a warning in a doc comment.
- **Good UX? YES.** The *cannot* is a gift to the caller: when the only path is the right one, the
  wrong one is not there. The counter — Clojure's `first`/`rest` work on lazy seqs — is answered by
  what the thing IS: wat's Stream is explicitly **not** Clojure's lazy-seq (R1
  `NON BIS IN IDEM FLVMEN`), and the builder frames it as a Ruby Enumerator, which exposes `next`.

## Option B — close none. Keep all three, delete the memo anyway.

- **Obvious? NO.** Two ways to walk, and one of them is **silently wrong for any effectful `f`** the
  moment the memo dies. The builder ruled on exactly this: *"a user's func must never be called 3
  times — we don't know if the func has side effects — that's a massive failure outright."*
- Rest *unreached.*

## Option C — close `rest` only; keep `first` and `empty?`.

- **Obvious? NO.** An arbitrary line: a reader cannot derive why `rest` is illegal and `first` is not.
- **Honest? NO** (recorded even though Obvious already failed, because the mechanism matters):
  it **does not work.** `(first s)` then `(next s)` is still **two forces of the same cell** — user
  code runs twice. Closing the tail does not close the hazard, because the hazard is *any* pair of
  operations that separately force.
- Rest *unreached.*

## Option D — keep all three AND keep the memo forever. Fix nothing.

- **Obvious? YES.** Nothing changes; nothing to learn.
- **Simple? YES.** Zero work.
- **Honest? NO.** It keeps a DoS the builder has already named — and the memo is *why* effectful `f`
  appears to work, so it also keeps a correctness lie standing.
- **Good UX?** *Unreached.*

## Where decision 2 lands

**Only option A survives.** The derivation is not a preference: **a single-pass stream's READ and
ADVANCE are one act; any API that separates them is a lie about what the thing is.** `next` is the
only shape where the two cannot be split — which is exactly why it needs no cache.

---

# DECISION 3 — `stream->pvec` / `stream->vec`

Both say **"internal helper"** in their own doc comments while living in `:wat::core::`, the
user-facing namespace.

- **Option A — delete both; `into` absorbs the drain.** Obvious? **YES** (Clojure has `into` and
  `vec`; no third name). Simple? **YES** (one public verb, one drain). Honest? **YES** (nothing
  claims to be internal while being public). Good UX? **YES**.
- **Option B — move them to a private namespace.** Obvious? **NO** — a private name for a job
  `into` already owns publicly is a second name for one thing, which is the disease, relocated.
- **Option C — keep as-is.** Honest? **NO** — a doc comment saying "internal helper" on a verb any
  user can call is false on its face.

**Option A survives.** Route-independent; true under every branch of decision 1.

---

# DECISION 4 — sequencing: probe the memo-off premise, or draw the stone first?

- **Option A — probe first (population C, memo-off), then draw.** Obvious? **YES.** Simple? **YES** —
  a throwaway build plus a committed four-point series. Honest? **YES** — it validates the premise
  *everything else rests on* before anything is committed to it. Good UX? **YES** — the answer
  arrives before the work that depends on it. **Route-independent and owed either way.**
- **Option B — draw the stone, measure inside it.** Obvious? YES. Simple? YES. Honest? **NO** — the
  stone's acceptance test *is* the premise; if it fails, the whole stone was misconceived, and we
  would find out having already written it. `[[feedback_a_green_test_can_prove_nothing]]`
- **Option C — skip the probe; the population-B result is good enough.** Honest? **NO** —
  population B and population C differ by 9.1× and have never been measured under the same
  condition. Generalizing B's memo-off result to C is exactly a measurement's boundary being
  silently widened. `[[feedback_a_measurements_boundary_is_its_claims_boundary]]`

**Option A survives.**

---

## THE SHARED PREMISE — what none of these four questions can see

The four questions discriminate BETWEEN options; they never validate what the options rest on.
`[[feedback_four_questions_cannot_see_a_shared_premise]]`

Every option in decisions 1 and 2 assumes: **the memos can be deleted once no three-call walker
remains.** For population B that is **measured and controlled** (memo-off equals a program with no
stream in it — see the correction in `DESIGN-118.B-the-route-fork.md`; the record's claim that this
prediction "was wrong" is itself wrong). For **population C it has never been run.** That is what
decision 4 exists to close, and it is why decision 4 should be settled first.

## Summary — one line per decision

| decision | surviving option | what killed the others |
|---|---|---|
| 1 — where the walks live | **B, `Seqable` in wat** | A and C fail **Honest** (a convention where a wall is possible; a type over a stdlib no user can imitate). D fails **Obvious**. |
| 2 — the three doors | **A, close all three** | B and D fail **Honest**; C fails **Obvious** *and* does not actually work. |
| 3 — the `stream->` names | **A, delete both** | B relocates the disease; C is false on its face. |
| 4 — sequencing | **A, probe first** | B and C fail **Honest** on an unvalidated premise. |

⚠ **Surviving the four questions is not being ruled.** Decisions 1 and 2 are the builder's — 1 is an
architecture call with a live performance cost, and 2 is a dialect call. 3 and 4 follow from them.
