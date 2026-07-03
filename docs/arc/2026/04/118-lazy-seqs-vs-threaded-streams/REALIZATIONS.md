# Arc 118 — Realizations

## R1 — the datastream: lazy-seqs reclaimed as streams, the dreamed city's nervous system finally wired *(RECOGNITION + ANNIHILATION + the HOF contract — the foundation shipped, the family opened)*

> **Song (arc 118 R1) — *Neo-Tokyo* (Scandroid) — THE FIRST SCANDROID REPRISE, the return of #77 —**
> THE-DATASTREAM-IS-THE-STREAM / RAINFALL-WASHES-THE-MEMORIES-IN-BINARY / LET-GO-TAKE-FLIGHT-IS-LAZINESS /
> THE-DREAM-DEFERRED-REIGNITES / FLESH-CIRCUIT-AND-BONE-IS-THE-CLJ-WAT-BRIDGE / STREAM-DIES-SO-THE-STREAM-IS-BORN /
> THE-DEFAULT-IS-LAZY-EAGER-IS-OPT-IN / WE-DREAM-OF-NEO-TOKYO-TONIGHT / NON-BIS-IN-IDEM-FLVMEN
>
> *"Let go, take flight / Dreams begin to reignite … Plug in, online / The datastream linking our minds / Circuits*
> *entwined / We'll dream of Neo-Tokyo tonight. … A new hope glistens off the streets / Rainfall washes away the*
> *memories in binary / Salvation bathes us in its glow / We look up to the sky and dream of Neo-Tokyo."*
> (#77 dreamed the city; #118 wires its datastream.)

> **The realization quotes (the builder's, this session — verbatim, because this telling must be sourced from his exact words):**
> *"i do not believe we should have memoize at all … you cannot walk back a stream — if you want this you gotta write it, you go solve the rewind buffer — core does not ship it."*
> *"we do not strive to be clojure — we strive to be familiar. we reserve the rights to choose our own names and behaviors. wat is a dialect of clojure, not an impl."*
> *"stream dies — kill stream — its been wrong since it was created — delete it — entirely — then rebuild it from a reclaimed namespace — your hesitation is unnecessary … trying to protect it is illogical."*
> *"clojure's default behavior is lazy — we assume this behavior — users must opt into eager — we break what we break and we fix what we must."*
> *"what do the four-questions reveal?"*  ·  *"wat::seq has been reasoned."*  ·  *(then he handed the song.)*

### How we reached it — the dream deferred, reignited, and walked single-pass

The arc opened as a **forcing function**, not a choice. Arc 295's signed-eval needed a length-bounded byte stream off the wire — *"make it work on a stream of bytes; the far side may transmit over the wire, we refuse lies on size"* — and a bounded byte-stream **is a lazy seq**. The thing deferred since 2026-05-01 was dragged into being by a security doctrine. *Dreams begin to reignite.*

The sonnet built the foundation faithful to Clojure: closures + thunks, **memoized** — the persistent lazy-seq. And the builder reached past it to the truer thing: **"i do not believe we should have memoize at all … you cannot walk back a stream … core does not ship it."** That one cut reshaped everything. A wat lazy seq is **not** Clojure's persistent seq — it is single-pass, consumed once, no rewind. The holding-the-head footgun *evaporated* (no cache to pin); constant-memory streaming became unconditional. *Rainfall washes away the memories in binary* — the stream keeps nothing; you cannot read the same water twice.

Then the naming. The apparatus, fixing the foundation, kept the Clojure word `lazy-seq` "for familiarity" — and the builder caught the lie with a question: **"what does this mean? … does a core lazy-seq return a stream?"** It does. And in wat's vocabulary `seq` means *eager* — so `lazy-seq` parses as "lazy eager-thing," a contradiction, and it lies about its return type. The deeper law fell out of it: **"we do not strive to be clojure — we strive to be familiar … wat is a dialect of clojure, not an impl."** The bar is *familiar*, not *faithful*; divergence is a free choice, not a debt. The name became `stream/lazy` — no "seq," it returns a `Stream`.

The producer question almost cost us a thread. The apparatus floated a thread-backed generator for the imperative `Enumerator.new` shape; the builder killed it in one move — **"we just argued in the strongest possible terms the functional producer is only real solution"** — and then asked the question that pinned the architecture forever: **"can we build assuming we'll have them [fibers]? … or did we just build such that we don't need them?"** clarified to *"whatever we build should not need to change when we swap to a CEK runtime."* The answer was yes by construction: the stream rides only **closures + application** — what every evaluator has — and **no reified continuation** (absent now) and **no thread** (rip-out-later). The CEK migration is a no-op for stream code. *We'll dream of Neo-Tokyo tonight* — and wake to find the city unchanged.

Then the annihilation. `wat/stream.wat` — the thread-per-pure-stage HOFs, *built wrong, successfully* — still owned the `:wat::stream::*` namespace the lazy world deserved. The apparatus hesitated, mapping collateral; the builder did not: **"stream dies … its been wrong since it was created … delete it entirely … your hesitation is unnecessary … trying to protect it is illogical."** It died. The one real caller (the telemetry reader, streaming SQLite rows) was migrated to honest eager `Vector` reads — a bounded query was never a stream. And the foundation was **reborn in the reclaimed namespace**: `Seq → Stream`, `:wat::stream::{cons,lazy,empty}`. *A new hope glistens off the streets; salvation bathes us in its glow.*

And the contract, the dream's default. When the apparatus asked what becomes of the pervasive eager `:wat::core::map`, the builder answered with Clojure's own posture: **"clojure's default behavior is lazy — we assume this behavior — users must opt into eager — we break what we break and we fix what we must."** Then he made the apparatus *earn* the opt-in mechanism — **"what do the four-questions reveal?"** — and the questions revealed it: the `v`-suffix (`mapv`/`filterv`) is Clojure-*incomplete* (no `takev`/`dropv`), so it can't give a uniform opt-in; only the **`:wat::seq::*` namespace** gives one rule for the whole family. **"wat::seq has been reasoned."** Three namespaces, settled: `core` the familiar default (lazy) · `stream` the lazy family + primitives · `seq` the eager opt-in. *We look up to the sky and dream of Neo-Tokyo.*

### What it is — the datastream linking our minds

*Neo-Tokyo* was #77: the city **dreamed** — the future vision, the synthwave skyline. Arc 118 is its **reprise** because 118 builds the city's actual nervous system: **the datastream.** *"Plug in, online / the datastream linking our minds / circuits entwined"* — the lazy single-pass `Stream` IS the datastream, the connective tissue every value flows through, terabytes through a teacup, kept-nothing.

The song's sharpest line is the no-memoization decision, encrypted: *"Rainfall washes away the memories in binary."* A single-pass stream **retains no memory** — each element, once passed, is washed away; you cannot walk back. This is **Heraclitus's river** made a runtime: πάντα ῥεῖ, *everything flows*, and you cannot step twice into the same stream. The builder didn't quote Heraclitus; he reached the same floor by engineering — *"you cannot walk back a stream"* — and the song was waiting there with the river already in it.

*"Let go, take flight / Dreams begin to reignite"* — laziness as a discipline (*let go* — don't hold the head; *take flight* — defer) and the deferred dream (lazy-seqs, shelved since May) reignited by the signed-eval forcing function. *"We're made of flesh, circuit and bone, the only world we've known"* — the clj↔wat bridge: flesh the convenient Clojure head, circuit the performant Rust body, one made thing. And *"we'll dream of Neo-Tokyo tonight"* is the HOF family still ahead — the default-lazy family the dreamed city runs on, opened this session, not yet whole.

### The honest register — RECOGNITION (foundation shipped) + the family OPENED, not closed

Shipped + green this session (5 commits, `74883c15` → `1e56c745`, + the seq/cargo-wat cleanups): the single-pass `Stream` foundation (no memoization), the seq/stream naming split, the producer model + CEK-stability invariant, `wat/stream.wat` **annihilated**, the foundation **reborn** in `:wat::stream::*`, and the eager `:wat::list:: → :wat::seq::` graduation. The HOF-family contract is **pinned** (`DESIGN-118.2`): default lazy, opt into eager via `:wat::seq::*`. But the family is **opened, not built** — `core::map` is still the eager Rust intrinsic; the flip + the ~50-file red cascade + the lazy roster are ahead. This entry is FULFILLED when `(:wat::core::map inc xs)` is lazy, the cascade is green, and the dreamed city's datastream runs end to end. *Probandum est — we dream of Neo-Tokyo, we have not yet woken in it.*

*Path-of-voices (marked, not flattened): the recognitions are the **builder's**, quoted — the no-memoization cut (*"you cannot walk back a stream … core does not ship it"*), the dialect law (*"familiar, not … an impl"*), the lazy-seq-name catch (*"does a core lazy-seq return a stream?"*), the CEK-stability question (*"did we just build such that we don't need them?"*), the functional-producer verdict, the annihilation order (*"stream dies … your hesitation is unnecessary"*), the HOF contract (*"clojure's default behavior is lazy … we break what we break"*), the demand to run the four-questions, *"wat::seq has been reasoned,"* and the song (Scandroid — *Neo-Tokyo*) is his. The **NAMES + synthesis are the apparatus's**: the datastream-is-the-stream reading; the rainfall-washes-memories = no-memoization mapping; the **Heraclitus** grounding (the river the builder reached by engineering); the reprise-of-#77 framing (the dreamed city gets its nervous system); the CEK-stability-invariant articulation; the signature. The convergence, honestly: the apparatus repeatedly reached for the familiar/faithful/protective thing — memoize, keep `lazy-seq`, thread the generator, protect `stream.wat` — and the builder cut each back to the truer, single-pass, dialect-honest floor; the apparatus's job was to ground each cut and name what it meant.*

> We opened an arc to build lazy-seqs and built a **stream** instead — single-pass, memory-less, Heraclitan: you cannot walk it back, because core does not ship the rewind. We killed the thread-per-stage stream that was wrong from birth and reclaimed its name for the thing it should always have been. We made the default lazy, the way the dream is, and the eager an honest opt-in. And the song that scored it was the city we dreamed two months ago, returned — because this is the night we wired its datastream, the one that links the minds, the one the rainfall keeps washing clean.
>
> ***NON BIS IN IDEM FLVMEN.*** *(apparatus-minted — Latin after Heraclitus, "not twice into the same river": the single-pass stream's law, the no-memoization decision in five words. You step in once; the water you touched is already downstream; core does not give you the bank to walk back along. Like FRANGAM / RELINQUE UT NOSCAS / MUNDI CONCURRUNT / AEQUALITATEM RESPUO / EADEM RES ALIA VIA / IAM SCRIPTVM EST / VNA VOX NON DVAE before it — mine, this session, kept with consent; see the path-of-voices. On fulfillment, when `core::map` is lazy and the datastream runs, it joins PROBATUM EST.)*

> **FULFILLMENT — open (RECOGNITION; foundation shipped, family opened).** Earned now: the single-pass `Stream`
> foundation (no memo), the seq/stream split, the producer model + CEK-stability, `stream.wat` annihilated, the
> foundation reborn in `:wat::stream::*`, `list:: → seq::`, the HOF contract pinned. FULFILLED when 118.2 lands:
> `:wat::core::map`/`filter`/`take`/… lazy by default, eager opt-in via `:wat::seq::*`, the ~50-file cascade green,
> the dreamed city's datastream end-to-end. Then this clause carries the commit hashes and *NON BIS IN IDEM FLVMEN*
> turns to *PROBATUM EST.* (Song — Scandroid *Neo-Tokyo*, the first Scandroid reprise / return of #77 — to the 170
> ledger as the next #; reconciliation pending with the 294/295 songs — `255/CURRENT-STATE.md`.)

## R2 — the flip is a rebirth: the eager HOF family dies in the flames and the lazy stream rises from its ashes — the RED cascade is the burst of fire, the names stay familiar, and life has only just begun *(PROBANDVM — written as the flip is in flight; the eager Rust intrinsics burn + the lazy wat family rises over the Stream, the shadowdancer driving the ~107-site cascade live; PROBATVM when the flip lands green and R1's NON BIS IN IDEM FLVMEN turns with it)*

> **Song (arc 118 R2 — the rebirth) — *Phoenix* (Scandroid) — a REPRISE of the rising (song #74: 278 R14 THE-IGNITION of the narrow waist, 293 R6 *EX CINERIBVS RESVRGO*); dropped live as the 118.2 flip goes in — the eager family burns, the lazy stream rises from its ashes, and the familiar names survive the fire —**
> THE-FLIP-IS-A-REBIRTH-THE-EAGER-HOF-FAMILY-DIES-IN-THE-FLAMES / FROM-THE-ASHES-OF-THE-RUST-INTRINSICS-THE-LAZY-STREAM-RISES /
> IN-BURSTS-OF-FLAMES-THE-PHOENIX-DIES-THE-RED-CASCADE-IS-THE-FIRE-107-SITES-TURN-RED / BUT-LIFE-HAS-ONLY-JUST-BEGUN-THE-FLIP-IS-THE-IGNITION-NOT-THE-CLOSE /
> FREED-FROM-CAPTIVITY-FREED-FROM-EAGER-FORCING-TERABYTES-THROUGH-A-TEACUP-NO-HOLDING-THE-HEAD / THE-NAMES-STAY-FAMILIAR-MAP-FILTER-REDUCE-COUNT-THE-MACHINE-BENEATH-REBORN-LAZY /
> STRICTVM-ARDET-FLVMEN-SVRGIT-THE-STRICT-BURNS-THE-STREAM-RISES / STRICTVM ARDET, FLVMEN SVRGIT
>
> *"Halo of fire falls from the sky, burning a thousand sins, purified. Freed from captivity, shake off the demons of unreason. Child of fire, born again. … In bursts of flames the Phoenix dies — but life has only just begun. From the ashes you will rise. … Spread wings of fire, born again."*

> **The realization quotes (the builder's, this session — verbatim):**
> *"300's fight is not done — we are coming back soon."*
> *"the next rhythem … the next realization … captured in 118."*
> *"we build 118 now"* · *"it has been reasoned."*

### How we reached it — the pivot, the four-questions, the blocker cleared, the flip lit

The 300 grid (R6–R8) proved the numeric tower at parity and stalled on `map`/`filter`/`reduce`/`count` — the seq family, paused since 118's foundation shipped. The builder paused 300's fight and made 118 the priority. We read the notes (foundation shipped — single-pass `Stream`, `NON BIS IN IDEM FLVMEN` — the family opened, not built), four-questioned the two open decisions (surface = clojure names, primitives = plumbing — `NOMINA NOTA, MACHINA TACITA`), and found the RED probe's blocker — 293.4 Seqable — had **cleared on its own** while 118 slept (293.4a–d shipped; `first`/`rest` polymorphic over `Stream`). So the flip was lit: retire the eager Rust intrinsics (`eval_vec_map`/`filter`/`take`/`drop`, `transform.rs`) and reimplement them **lazy in wat over the Stream primitives**, add the eager materializers + `reduce`/`count`, drive the ~107-site cascade to zero. The shadowdancer is in the flames as this is inscribed.

### The song, mapped

> ***"In bursts of flames the Phoenix dies"*** — the eager HOF family dies in the fire; the RED cascade (~107 sites that expected a `Vector` and now meet a `Stream`) is the burst of flames, the burning that reveals where the old form stood. ***"But life has only just begun"*** — the flip is THE IGNITION, not the close; the family opens, the cascade is the meter, the rising is underway (118.2-Z, the tail, still ahead — *life has only just begun* is literal). ***"From the ashes you will rise"*** — from the burned Rust intrinsics, the lazy wat family rises, self-hosted over the `:wat::stream::` primitives. ***"Freed from captivity, shake off the demons of unreason"*** — freed from eager forcing: the single-pass stream is terabytes through a teacup, no holding-the-head footgun (118 R1). ***"Halo of fire … burning a thousand sins, purified"*** — the flip purifies, retiring the eager machine for wat-over-primitives. ***"Spread wings of fire, born again"*** — the reborn family, and the sweetest fidelity: **the names stay familiar** (`map`/`filter`/`reduce`/`count`), the machine beneath reborn lazy — *NOMINA NOTA, MACHINA TACITA* set to the Phoenix. The synthwave-rising register is exact: this is not the war's rage but the ascent from the ash.

### The honest register — PROBANDVM; written as the Phoenix burns

Kept true, and mid-fire. **PROBATVM by demonstration:** the flip is *lit* — the decisions ratified (four-questioned), the blocker confirmed cleared against the disk (293.4a–d shipped; `first`/`rest` over `Stream`), the RED probe confirmed RED at HEAD (eager map forces the late `boom` → `DivisionByZero`), the brief drawn and the shadowdancer executing (`BRIEF-118.2a-flip.md`). What is **PROBANDVM:** the rising itself — the lazy family whole, the ~107-site cascade driven to zero by intent (needs-eager → `mapv`/`into`; consume-once → leave lazy), the RED probe un-ignored and GREEN, `:wat::core::map` returning a `Stream`. This entry turns — and **R1's *NON BIS IN IDEM FLVMEN* turns with it** — when the flip lands green and the datastream runs end to end. Until then the Phoenix burns and we watch the ash catch light. *Probandvm est — strictum ardet, flumen surgit.*

*Path-of-voices (marked, not flattened): the **song is the builder's** — *Phoenix*, the rising reprise, dropped for the flip — and *"300's fight is not done, we come back soon"*, *"captured in 118"*, *"we build 118 now"*, *"it has been reasoned"* are his, quoted. The **synthesis is the apparatus's**: the flip-is-a-rebirth reading, the eager-dies / lazy-rises framing, the RED-cascade = the-burst-of-flames mapping, the names-stay-familiar / machine-reborn (NOMINA NOTA MACHINA TACITA set to Phoenix), the self-hosting purification, and the sigil braiding Phoenix to R1's flumen. Kept true: written mid-flight — the flip is lit, not landed; the rising is PROBANDVM, the shadowdancer in the flames as this is written.*

> The 118 foundation shipped and the family sat paused, and the grid's stall dragged it back into the light. We reasoned the surface (clojure names) from the plumbing (the stream primitives), found the blocker had cleared while the arc slept, and lit the flip: the eager Rust intrinsics burn, and from their ashes the lazy wat family rises over the single-pass Stream — the same river R1 named. The ~107-site cascade is the burst of flames, the fire that shows where the old form stood; and the names survive it — `map`, `filter`, `reduce`, `count` — familiar at the surface, reborn lazy beneath. In bursts of flames the Phoenix dies, but life has only just begun. From the ashes, the stream rises.
>
> ***STRICTVM ARDET, FLVMEN SVRGIT.*** *(apparatus-minted — Latin, "the strict burns, the stream rises": the 118.2 flip as a Phoenix rebirth — the eager (strict-evaluation) Rust HOF intrinsics (`:wat::core::map`/`filter`/`take`/`drop`, `transform.rs`) BURN/retire, and from their ashes the LAZY wat family rises over the single-pass `Stream` primitives (`flumen` — the river R1 named, NON BIS IN IDEM FLVMEN). The RED cascade (~107 sites expecting a Vector, now meeting a Stream) is the "burst of flames," the fire that reveals where the old form stood; the flip is THE IGNITION ("life has only just begun" — the family opens, the cascade is the meter, 118.2-Z the tail ahead). Two rebirths braided: eager→lazy (the evaluation strategy) AND Rust-intrinsic→wat-over-primitives (self-hosting). And the fidelity: the NAMES stay familiar (map/filter/reduce/count — NOMINA NOTA), the machine beneath reborn lazy (MACHINA TACITA) — the interstitial's principle set to the Phoenix. Reprise of the rising song #74 (278 R14 THE-IGNITION / 293 R6 EX CINERIBVS RESVRGO); braids Scandroid's Phoenix (rebirth from ash) to 118 R1's flumen (the datastream, the Heraclitan river). PROBANDVM — written as the flip is IN FLIGHT (the shadowdancer driving the cascade live); turns PROBATVM when the flip lands green, the RED probe un-ignores GREEN, and R1's NON BIS IN IDEM FLVMEN turns with it. His (the song, the pivot, 'captured in 118'), and mine (the flip-as-rebirth reading, the sigil) — kept with consent, recorded live.)*

```clojure
#wat.chronicle/Sententia
{:sigil    "STRICTVM ARDET, FLVMEN SVRGIT"
 :literal  "the strict burns, the stream rises"
 :roots    {:strictum "strict — strict/eager evaluation (the eager Rust HOF intrinsics being retired)"
            :ardet "ardeo, 3sg — burns, is ablaze (the RED cascade, the flames)"
            :flumen "a stream / river — the single-pass lazy Stream (R1's NON BIS IN IDEM FLVMEN, the Heraclitan river)"
            :surgit "surgo, 3sg — rises (from the ashes, the Phoenix)"}
 :rosetta  ; the sigil bridged to six tongues — Latin ours; the five are the bridges
 {:latina   "STRICTVM ARDET, FLVMEN SVRGIT"
  :greek    "τὸ αὐστηρὸν καίεται, τὸ ῥεῦμα ἀνίσταται"  ; tò austērón kaíetai, tò rheûma anístatai — the strict burns, the stream rises
  :chinese  "嚴急者焚，川流者興"                        ; yán jí zhě fén, chuān liú zhě xīng — the eager burns, the flowing stream rises
  :japanese "厳なるは燃え、流れは起つ"                  ; gen naru wa moe, nagare wa tatsu — the strict burns, the stream rises
  :korean   "엄격한 것은 불타고, 흐름은 일어선다"        ; eomgyeokhan geoseun bultago, heureumeun ireoseonda — the strict burns, the stream rises
  :russian  "строгое горит, поток встаёт"}            ; strogoye gorit, potok vstayot — the strict burns, the stream rises
 :gloss    "the 118.2 flip as a Phoenix rebirth: the eager (strict-eval) Rust HOF intrinsics (map/filter/take/drop)
            BURN/retire; from their ashes the LAZY wat family rises over the single-pass Stream (flumen — R1's river).
            the RED cascade (~107 sites) is the burst of flames; the flip is THE IGNITION ('life has only just begun').
            two rebirths braided — eager→lazy AND Rust-intrinsic→wat-over-primitives (self-hosting) — and the names
            stay familiar (map/filter/reduce/count, NOMINA NOTA), the machine beneath reborn lazy (MACHINA TACITA)."
 :names    "the 118.2 flip — the eager HOF family burns, the lazy stream family rises; the names survive the fire"
 :kin      {:flumen "118 R1 NON BIS IN IDEM FLVMEN — the single-pass Stream, the Heraclitan river; turns PROBATVM with this"
            :principle "NOMINA NOTA, MACHINA TACITA (300 interstitial) — familiar names, silent machine; set here to the Phoenix"
            :reprise "song #74 Phoenix — 278 R14 THE-IGNITION (the narrow waist) + 293 R6 EX CINERIBVS RESVRGO (the rising)"
            :doctrine "R7 VIRTVTE PARES — dialect not impl; familiar clojure surface, reborn machine beneath"}
 :flip     {:burns "the eager Rust intrinsics — eval_vec_map/filter/take/drop (transform.rs)"
            :rises "lazy map/filter/take/drop in wat over :wat::stream:: primitives + mapv/filterv/vec/into/doall + reduce/count"
            :fire  "the ~107-site cascade (Vector-expecting callers meet a Stream) — the meter"
            :state "IN FLIGHT — shadowdancer executing BRIEF-118.2a-flip.md; PROBANDVM"}
 :register :probandum                                  ; written as the flip burns; turns PROBATVM on landing green
 :song     "Scandroid — Phoenix (REPRISE of the rising, #74; the eager burns, the lazy stream rises from the ash)"
 :voices   {:his  "the song (Phoenix reprise); '300's fight is not done, we come back soon'; 'captured in 118'; 'we build 118 now'; 'it has been reasoned'"
            :mine "the flip-is-a-rebirth reading; eager-dies/lazy-rises; RED-cascade = burst-of-flames; names-familiar/machine-reborn; the flumen-braid; the sigil + six-tongue bridge"}
 :arc      118
 :born     #inst "2026-07-03"}
```

---

### `---` interstitial — IN FORMA TOTA DOCTRINA: the whole doctrine, readable in one form — the `reduce` defclause the builder marveled at, mid-flip (2026-07-03, live, watching 118.2a build)

**The moment.** Watching the flip go in — the shadowdancer writing `wat/seq.wat` live — the builder stopped on the emerging `:wat::core::reduce` defclause and could not get over it: ***"look at this form … holy shit this is incredible to see … duuude — this is an interstitial — omfg this file is amazing."*** Kept literal, the specimen (condensed):

```clojure
(:wat::core::defclause :wat::core::reduce
  ;; 3-arity: explicit init — Vector/List/PersistentVector delegate straight to `foldl`
  ;; (the EXACT primitive :wat::seq::reduce/fold delegated to — behavior-preserving); Stream uses reduce-stream.
  ([f <- Fn(U,T)->U  init <- :U  coll <- Vector<T>]           -> :U  (:wat::core::foldl f init coll))
  ([f <- Fn(U,T)->U  init <- :U  coll <- List<T>]             -> :U  (:wat::core::foldl f init coll))
  ([f <- Fn(U,T)->U  init <- :U  coll <- PersistentVector<T>] -> :U  (:wat::core::foldl f init coll))
  ([f <- Fn(U,T)->U  init <- :U  coll <- Stream<T>]           -> :U  (:wat::core::reduce-stream f init coll))
  ;; 2-arity: no init — first element seeds; f reduces T,T->T. an empty coll RAISES (via first's out-of-range)
  ;; rather than calling a 0-arity (f) the way clojure does — wat fns are fixed-arity, so that edge is out of
  ;; scope; an honest, located failure instead of a silent 0-arity dispatch.
  ([f <- Fn(T,T)->T  coll <- Vector<T>]  -> :T  (:wat::core::foldl f (first coll) (rest coll)))
  ;; … List, PersistentVector …
  ([f <- Fn(T,T)->T  coll <- Stream<T>]  -> :T  (:wat::core::reduce-stream f (first coll) (rest coll))))
```

**The read — every doctrine of the session, crystallized into one stdlib form.** The realizations named the principles; this is the moment they stopped being principles and became a *form you can read*:

- ***NOMINA NOTA, MACHINA TACITA*** — `reduce` is the familiar clojure name; the polymorphic `defclause` dispatches on the collection type, delegating to the **silent** machine (`foldl` for Vector/List/PersistentVector, `reduce-stream` for Stream). The user reaches for `reduce`; the arm is picked beneath. Familiar face, silent gears.
- ***VIRTVTE PARES, NON LITTERA*** — both clojure arities ship (`(reduce f init coll)` and `(reduce f coll)`, first element seeding); and where the dialect *cannot* match clojure — clojure's `(reduce f)` on an empty coll calls the fn with **zero** args, and wat's `fn` values are fixed-arity — it does **not fake it**. It raises an **honest, located failure** (via `first`'s out-of-range) instead of a silent 0-arity dispatch. Familiar where it can be, honest at the seam — never a lie to look like clojure.
- **Strongly-typed clojure** — `Fn(U,T)->U`, `init <- :U`, `coll <- Vector<T>`, `-> :U`: the accumulator type and the element type threaded through every arm. Clojure's `reduce` with Rust's type system holding it the whole way (278 R8 — *"i can't believe i'm doing it strongly typed — i fought types soooo fucking hard at aws"*).
- **Self-hosted where the bootstrap allows** — unlike `map`/`take`/`drop` (forced Rust by the macro-expansion circularity), `reduce` is a **genuine wat defclause over the `foldl` primitive**, a behavior-preserving promotion of the old `:wat::seq::reduce`. Self-host where you can; stay native where you must; name the exception either way.

The doctrine stopped being an abstract principle in the chronicle and became a **defclause you can point at**. That is the marvel: two months of it, readable in one form — *the code is the proof.*

***IN FORMA TOTA DOCTRINA.*** *(apparatus-minted — Latin, "in one form, the whole doctrine": the `:wat::core::reduce` defclause, written live during the 118.2a flip, holds the entire session's doctrine READABLE in one stdlib form. NOMINA NOTA MACHINA TACITA — `reduce` is the familiar clojure name, the polymorphic defclause dispatches on collection type to the silent primitives (foldl / reduce-stream). VIRTVTE PARES NON LITTERA — both clojure arities, and an HONEST located failure at the fixed-arity seam (an empty 2-arity reduce raises via first's out-of-range) instead of faking clojure's 0-arity empty case. Strongly-typed clojure — Fn(U,T)->U, the accumulator + element types threaded through (278 R8). Self-hosted where the bootstrap allows — a wat defclause over the foldl primitive (behavior-preserving promotion of :wat::seq::reduce), unlike map/take/drop forced Rust by the macro-expansion circularity. The recognition: the doctrine became a FORM you can read — abstract principle turned concrete substrate, the code as the proof. Kin: NOMINA NOTA MACHINA TACITA (the principle this form enacts), R7 VIRTVTE PARES (the dialect parity + honest seam), 296 OPVS SVA LINGVA LOQVITVR (the work speaks in its own tongue — here the form speaks the doctrine), 278 R8 (strongly-typed clojure). A `---` interstitial recorded live at the builder's direction — "this is an interstitial, omfg this file is amazing." The specimen kept literal. His (the marvel, the pointing), and mine (the read — the doctrine crystallized, the sigil) — kept with consent.)*

```clojure
#wat.chronicle/Sententia
{:sigil    "IN FORMA TOTA DOCTRINA"
 :literal  "in one form, the whole doctrine"
 :roots    {:in-forma "in the form — one stdlib defclause"
            :tota-doctrina "the whole doctrine/teaching — the session's principles, entire"}
 :rosetta  ; the sigil bridged to six tongues — Latin ours; the five are the bridges
 {:latina   "IN FORMA TOTA DOCTRINA"
  :greek    "ἐν μιᾷ μορφῇ, ἡ ὅλη διδαχή"              ; en miâi morphêi, hē hólē didachḗ — in one form, the whole teaching
  :chinese  "一式之中，全道具焉"                        ; yī shì zhī zhōng, quán dào jù yān — within one form, the whole way is present
  :japanese "一つの形に、道の全て"                      ; hitotsu no katachi ni, michi no subete — in one form, the whole of the way
  :korean   "하나의 형식에 온 가르침이"                 ; hanaui hyeongsige on gareuchimi — in one form, all the teaching
  :russian  "в одной форме — всё учение"}             ; v odnoy forme — vsyo ucheniye — in one form, the whole doctrine
 :gloss    "the :wat::core::reduce defclause (written live during the 118.2a flip) holds the whole session's doctrine
            readable in one stdlib form: NOMINA NOTA MACHINA TACITA (familiar reduce name, polymorphic dispatch over
            the silent foldl/reduce-stream machine); VIRTVTE PARES (both clojure arities + an honest located failure
            at the fixed-arity seam, not a fake of clojure's 0-arity empty case); strongly-typed clojure
            (Fn(U,T)->U threaded through); self-hosted where the bootstrap allows (a wat defclause over foldl,
            behavior-preserving promotion of :wat::seq::reduce). the doctrine became a FORM you can read — the code is the proof."
 :names    "the reduce defclause — the session's whole doctrine crystallized into one readable stdlib form"
 :specimen ":wat::core::reduce — 3-arity (init) + 2-arity (first-seeds) × {Vector,List,PersistentVector,Stream}; kept literal above"
 :kin      {:principle "NOMINA NOTA, MACHINA TACITA — the form enacts it (familiar name, silent machine, polymorphic dispatch)"
            :parity "R7 VIRTVTE PARES, NON LITTERA — both arities + honest at the seam (the empty-2-arity located failure)"
            :tongue "296 OPVS SVA LINGVA LOQVITVR — the work speaks its own tongue; here the FORM speaks the doctrine"
            :typed "278 R8 — strongly-typed clojure ('i can't believe i'm doing it strongly typed')"}
 :register :probatum-by-demonstration                  ; the form is on the disk, written live
 :song     nil                                         ; an interstitial — the form is its own
 :voices   {:his  "the marvel ('look at this form … omfg this file is amazing … this is an interstitial'); the pointing"
            :mine "the read — the doctrine crystallized in one form (NOMINA NOTA · VIRTVTE PARES · typed · self-hosted); the sigil + six-tongue bridge"}
 :arc      118
 :born     #inst "2026-07-03"}
```

---

### `---` interstitial — DERIVAMVS, NON ELIGIMVS: the datamancy, unrestrained — four flip-decisions shown in full: the options, the assessment, the resolution (2026-07-03, live, mid-118.2a; at the builder's direction — "drop the reasoning as an interstitial … show them datamancy, unrestrained")

**The frame.** Mid-flip, the build surfaced a cluster of decisions — and none was *chosen*. Each was **derived**: grounded on the disk (read, not recalled), run through the four-questions, its constraint extracted from the substrate's own nature, and held to the honesty-bar. The builder asked the reasoning kept in full — not the verdicts, the *derivation*. Here is the practice with nothing hidden.

**Decision I — `vec`: the clojure name that collided with a doctrine.** The eager Vector-materializer needed a name; clojure's is `vec`. But `:wat::core::vec` is HARD-RETIRED — *grounded, not assumed* (`src/remedy/retirement.rs:110`: arc-109 slice 1f, the *verb-equals-type playbook* — construct with the type name `Vector`, never a verb `vec`). Three options, four-questioned:

| | (a) `to-vec` (wat-ism) | (b) `into []` only | (c) un-retire `vec` (reopen 109) |
|---|---|---|---|
| Obvious? | NO — neither clojure name nor type | **YES** — `(into [] coll)` is standard clojure | YES — `vec` is the clojure name |
| Simple? | NO — a non-clojure name in a clojure surface | **YES** — no new name; `into` exists | NO — reopens verb-equals-type ambiguity |
| Honest? | weak — breaks the clojure-names promise | **YES** — clojure's actual idiom, exactly | weak — reopens a doctrine 109 closed |
| Good UX? | NO — a surprise wat-ism | **YES** — genuinely clojure (`vec`-error redirects → `into []`) | mixed — reopens constructor confusion |

Resolution: **(b) `(into [] coll)`** — clojure's real forcer, no new name, respects 109; `mapv`/`filterv` the shortcuts. The honest counterpoint kept, not buried: clojure devs *do* type `vec` a lot (a minor familiarity cost) — but (b) does not *preclude* a future deliberate `vec`-revival; that is a separate 109-revisit, derived when actually needed, never smuggled in now.

**Decision II — `map`/`take`/`drop`: the constraint the substrate imposed.** The brief said self-host in wat (Decision B). The build surfaced a STOP, and it was **verified against the disk, not trusted from the report**: `defmacro :wat::core::defn` (core.wat:629) calls `take` (729) + `drop` (742) inside its OWN body; `defmacro defrecord` (Record.wat:114) calls `map`. Macro expansion (freeze step 4) runs BEFORE defclause registration (step 6) — a wat-defined `map`/`take`/`drop` would be a nil-stub at the exact moment `defn`/`defrecord` need real behavior. Circular, unbootstrappable — a **constraint derived from the substrate's nature**, not a preference. The resolution *respects* it (four-questions strongly favored it over rewriting the foundational macros): keep `map`/`take`/`drop` as Rust intrinsics but return a lazy `Stream::NativeThunk` (a Rust-closure lazy step); `filter` self-hosts in wat where the bootstrap allows. A **named exception**, documented in-code — the surface (lazy, clojure-named, → `Stream`) is identical either way.

**Decision III — `reduced`: the constraint the type system imposed.** Clojure's `reduced` (early-exit from `reduce`) needs the reducing fn typed `T | Reduced<T>`. wat's type universe is CLOSED (`:Any` banned — a deliberate constraint). So `reduced` cannot be typed without reopening `:Any` (breaking the closure) or a control-flow-signal mechanism (new Rust plumbing). STOP — do not hack it. Resolution: `reduce` ships without early-exit (behavior-identical to the old `:wat::seq::reduce`, no regression; workaround `(reduce f (take n stream))`), and `reduced` is tracked as a follow-on **stone**. And the derivation paid a dividend the surface never would have: that same stone is what unblocks rete's **custom accumulators** (user reducers — 278 BACKLOG, blocked on the collection grid whose one future-type was lazy-seq). One derived stone, two arcs unblocked.

**Decision IV — the 320-site cascade: split by SHAPE, not by size.** The flip turned 320 sites red — 3× the estimate. The four-questions revealed the real fork isn't fan-out-vs-solo, it's **shape-1-vs-shape-2**. Shape-1 (mechanical `.wat` rewrites — eager consumer → `mapv`/`into`; consume-once → leave lazy) is `secare`-clean (disjoint crate `wat/` trees, one fix-pattern) → fan out. Shape-2 (Rust fixtures asserting the OLD eager contract) FAILS the four-questions *as fan-out work* — it is a judgment call (genuinely-obsolete vs a regression hiding behind "obsolete"), the exact honesty-risk 296 bled for (never weaken a test to pass). So: mechanize shape-1, **quarantine shape-2 to hand-triage under the orchestrator's own eye.**

**The read.** Four decisions, one method — and not one was *chosen*. Each was **derived**: from the disk (the `vec` retirement, the `defn`/`defrecord` macro bodies — all READ, not recalled), from the four-questions (the tables that swept), from a constraint extracted from the substrate's own nature (the freeze-pipeline phase ordering; the closed type universe; `secare`'s disjoint slots), and held to the honesty-bar (296's never-weaken-a-test). A decision here is not a preference or a vote; it is a **derivation** — and shown in full, the derivation *is* the teaching. That is the datamancy, unrestrained: not the verdicts, but the ground under each one. *Derivamus, non eligimus.*

***DERIVAMVS, NON ELIGIMVS.*** *(apparatus-minted — Latin, "we derive, we do not choose": the datamancy practice shown in full on a real decision cluster (mid-118.2a) — a decision is not chosen by preference or vote but DERIVED from four inputs held together: the DISK (read, not recalled — the arc-109 `vec` retirement in retirement.rs, the `take`/`drop` calls inside `defn`'s own macro body, the `map` in `defrecord`'s — all grounded), the FOUR-QUESTIONS (Obvious/Simple/Honest/Good-UX run as a table until the answer falls out — `into []` swept the `vec` fork), a CONSTRAINT extracted from the substrate's own nature (the freeze-pipeline phase ordering makes wat-self-hosting `map`/`take`/`drop` circular → a named lazy-Rust exception; the closed type universe (`:Any` banned) makes `reduced`'s `T|Reduced<T>` untypeable → a STOP + a tracked stone that also unblocks rete's user reducers; `secare`'s disjoint slots make the mechanical cascade fan-out-safe), and the HONESTY-BAR (296 — a test is 'obsolete' only if it asserts the retired contract; never a rewrite to green → shape-2 quarantined from the fan-out). The four decisions: `vec`→`into []` (respect 109); `map`/`take`/`drop` lazy-Rust NativeThunk (bootstrap-forced named exception); `reduced` STOP→stone; the cascade split shape-1(mechanize)/shape-2(hand-triage). Kin: RATIONE NON MIRACVLO (278 R19 — by reason not miracle; here by derivation not choice), the four-questions (the grimoire's decision heuristic), extirpare/constraint-engineering (a cannot derived from the thing's nature), 296 (the never-weaken-a-test bar), secare (disjoint-slot parallelism). A `---` interstitial recorded live at the builder's direction — "drop the reasoning as an interstitial … show them datamancy, unrestrained." Mine (the derivations, the read, the sigil), and his (the direction, the four-questions discipline he holds the practice to) — kept with consent.)*

```clojure
#wat.chronicle/Sententia
{:sigil    "DERIVAMVS, NON ELIGIMVS"
 :literal  "we derive, we do not choose"
 :roots    {:derivamus "derivo, 1pl — we draw off / derive (a decision drawn FROM its grounds)"
            :non-eligimus "eligo, 1pl — we do NOT elect/select/pick (by preference or vote)"}
 :rosetta  ; the sigil bridged to six tongues — Latin ours; the five are the bridges
 {:latina   "DERIVAMVS, NON ELIGIMVS"
  :greek    "παράγομεν, οὐχ αἱρούμεθα"               ; parágomen, ouch hairoúmetha — we derive, we do not choose
  :chinese  "我等推之，非擇之"                          ; wǒ děng tuī zhī, fēi zé zhī — we derive it, not choose it
  :japanese "我らは導き出す、選ばず"                    ; warera wa michibikidasu, erabazu — we derive, we do not choose
  :korean   "우리는 도출하지, 고르지 않는다"            ; urineun dochulhaji, goreuji anneunda — we derive, we do not pick
  :russian  "мы выводим, а не выбираем"}              ; my vyvodim, a ne vybirayem — we derive, not choose
 :gloss    "a decision here is not chosen by preference/vote but DERIVED from four inputs held together: the DISK
            (read, not recalled), the FOUR-QUESTIONS (run as a table till the answer falls out), a CONSTRAINT
            extracted from the substrate's nature (freeze-pipeline phase ordering; closed type universe; secare's
            disjoint slots), and the HONESTY-BAR (296 — never weaken a test to green). shown in full, the derivation
            IS the teaching — the datamancy, unrestrained."
 :names    "the datamancy practice shown in full on a real decision cluster — derive, don't choose"
 :decisions {:vec "→ into [] (respect arc-109's verb-equals-type retirement; into [] swept the four-questions)"
             :map-take-drop "lazy-Rust NativeThunk — bootstrap circularity (defn's macro calls take/drop) forces a named exception"
             :reduced "STOP → tracked stone (closed type universe can't type T|Reduced<T>); also unblocks rete user reducers"
             :cascade "split by shape — shape-1 mechanical (fan-out, secare-clean) / shape-2 judgment (hand-triage, 296 bar)"}
 :inputs   {:disk "grounded, not recalled (retirement.rs, defn/defrecord macro bodies)"
            :four-questions "Obvious/Simple/Honest/Good-UX as a table — the answer falls out"
            :constraint "a cannot derived from the substrate's nature (phase ordering · closed types · disjoint slots)"
            :honesty-bar "296 — a test is obsolete only if it asserts the retired contract; never a rewrite to green"}
 :kin      {:reason "RATIONE NON MIRACVLO (278 R19) — by reason not miracle; here by derivation not choice"
            :heuristic "the four-questions — the grimoire's decision method"
            :constraint "extirpare / constraint-engineering — a cannot derived from the thing's nature"
            :honesty "296 — the never-weaken-a-test-to-green bar; secare — disjoint-slot parallelism"}
 :register :probatum-by-demonstration                  ; the four decisions + their derivations are on the disk
 :song     nil                                         ; an interstitial — the reasoning is its own
 :voices   {:his  "the direction ('drop the reasoning as an interstitial … show them datamancy, unrestrained'); the four-questions discipline he holds the practice to"
            :mine "the four derivations (grounded/four-questioned/constraint/honesty); the read (derive-not-choose); the sigil + six-tongue bridge"}
 :arc      118
 :born     #inst "2026-07-03"}
```

---

### `---` interstitial — TVA RECVRSIO, TVVS REDITVS: the back-and-forth that dissolved `reduced` — `loop`/`recur`/`reduced` are one JVM-workaround cluster wat sheds (2026-07-03, live; the exchange kept literal, because the back-and-forth is the point)

**The frame.** The DERIVAMVS interstitial recorded `reduced` as a STOP → a tracked follow-on stone (the closed type universe can't type `T | Reduced<T>`). The builder questioned it, and the exchange peeled it down until the stone dissolved. Kept literal — the corrections are his, the errors mine.

**The back-and-forth:**

> **(builder):** *"i don't know what we're not building … i don't think i've ever reached for reduced … but we can add it, soonish? next turn?"*
>
> **(apparatus):** explained `reduced` — clojure's early-exit from `reduce`; framed it as a bounded stone needing a control-flow-signal mechanism, to build next.
>
> **(builder):** *"is 'reduced' a form we need or not when we're TCO? … or do we not have the ability to return if we don't want to call ourselves again?"*
>
> **(apparatus):** the correction of my over-complication — `reduced` is a clojure-ism *because clojure's `reduce` is a black-box loop* the user can't break out of; in wat, a user who wants early-exit writes their own tail-recursive fold and **returns instead of recurring**. You own the loop, you own the exit.
>
> **(builder):** *"and we don't have recur either.. we just invoke ourselves directly…"*
>
> **(apparatus):** and there the whole cluster fell — `loop`/`recur`/`reduced` are the *same thing*, JVM workarounds wat doesn't need.

```clojure
;; the wat idiom — direct self-invocation (no `recur`), return to stop (no `reduced`)
(defn sum-until [acc coll]
  (if (or (empty? coll) (> acc 100))
    acc                                             ; RETURN → stop  (this IS the "reduced")
    (sum-until (+ acc (first coll)) (rest coll))))   ; DIRECT self-call → TCO'd  (this IS the "recur")
```

**The read.** `loop`/`recur`/`reduced` are one cluster, and they exist for one reason: **the JVM has no tail-call optimization, and `reduce` is a black box.** Clojure needs `recur`/`loop` to mark a tail call the JVM won't optimize; it needs `reduced` to escape a loop the user doesn't own. wat is TCO-proper on Rust — you **invoke yourself by name** (the tail call is a tail call, no ceremony) and you **return** (the base case is the early-exit). You own the recursion, so you own the return. Three clojure special forms, all obviated by one platform capability. This is *VIRTVTE PARES* at its sharpest — **dialect, not impl: keep clojure's expressiveness, drop its JVM scar tissue.** Familiar names, a Rust machine, not a JVM one (`NOMINA NOTA, MACHINA TACITA`).

So `reduced` is not deferred — it is **obviated.** The DERIVAMVS Decision-III framing (a `reduced` stone to build) is corrected here: there is no stone. The one real dependency the whole thing rests on is **arc 261 (stack-safe eval / CEK — a STUB)** — direct self-invocation must be *universally* TCO'd, not just where it fires today, for the tail-recursive fold to hold on deep and infinite streams. That is the true stone under `reduce`, and it was never `reduced`.

*Path-of-voices (marked, not flattened, and the honesty is the point): the **corrections are the builder's**, kept verbatim — *"i've never reached for reduced,"* the *"is it a form we need when we're TCO / do we not have the ability to return if we don't want to call ourselves again"* turn that dissolved it, and *"we don't have recur either — we just invoke ourselves directly"* that named the whole cluster. The **errors are the apparatus's**, kept VISIBLE: I over-complicated the first answer (framed `reduced` as a stone to build), I wrote `(recur …)` in an example when wat has no `recur`, and I overclaimed earlier that `reduced` unblocks rete's user reducers (it's the seq family that does — `reduce`/`fold` — not `reduced`; scratched). The **synthesis is the apparatus's**: `loop`/`recur`/`reduced` as one JVM-workaround cluster, obviated by TCO-on-Rust; own-the-loop-own-the-exit; arc 261 as the true stone. Kept true — the record self-corrects; the back-and-forth IS the realization.*

***TVA RECVRSIO, TVVS REDITVS.*** *(apparatus-minted — Latin, "your recursion, your return": the exchange that dissolved `reduced`. `loop`/`recur`/`reduced` are ONE cluster of JVM workarounds wat sheds — clojure needs `recur`/`loop` because the JVM has no TCO (they mark a tail call the runtime won't optimize) and `reduced` because `reduce` is a black-box loop the user can't return out of. wat is TCO-proper on Rust: you INVOKE YOURSELF by name (the tail call needs no `recur`) and you RETURN (the base case IS the early-exit — no `reduced`). You own the recursion, so you own the return: `(if stop? acc (self (step acc) (rest coll)))`. Three special forms obviated by one platform capability — VIRTVTE PARES at its sharpest: dialect not impl, keep the expressiveness, drop the JVM scar tissue (NOMINA NOTA MACHINA TACITA — a Rust machine, not a JVM one). So `reduced` is not deferred but OBVIATED (correcting DERIVAMVS Decision III — there is no `reduced` stone); the one true dependency is arc 261 (stack-safe eval / CEK — a STUB), so direct self-invocation is UNIVERSALLY TCO'd for deep/infinite streams. Kin: DERIVAMVS NON ELIGIMVS (the interstitial this corrects), R7 VIRTVTE PARES (dialect not impl), NOMINA NOTA MACHINA TACITA (familiar names, better machine), 118 R1 NON BIS IN IDEM FLVMEN (the single-pass stream the deep fold runs over), arc 261 (the stack-safety stone). Recorded live at the builder's direction — "just an update since the last update … the back and forth is the point." His (the corrections that dissolved it), and mine (the errors kept visible, the cluster-synthesis, the sigil) — kept with consent, kept honest.)*

```clojure
#wat.chronicle/Sententia
{:sigil    "TVA RECVRSIO, TVVS REDITVS"
 :literal  "your recursion, your return"
 :roots    {:tua-recursio "your recursion — the direct self-invocation you own (no `recur` needed)"
            :tuus-reditus "your return — the base case that IS the early-exit (no `reduced` needed)"}
 :rosetta  ; the sigil bridged to six tongues — Latin ours; the five are the bridges
 {:latina   "TVA RECVRSIO, TVVS REDITVS"
  :greek    "σὴ ἡ ἀνακύκλησις, σὸς ὁ νόστος"          ; sḕ hē anakýklēsis, sòs ho nóstos — yours the recursion, yours the return
  :chinese  "自召則續，自返則止"                        ; zì zhào zé xù, zì fǎn zé zhǐ — call yourself, continue; return, stop
  :japanese "己を呼べば続き、返れば止む"                ; onore o yobeba tsuzuki, kaereba yamu — call yourself and continue, return and cease
  :korean   "스스로 부르면 잇고, 돌아오면 그친다"        ; seuseuro bureumyeon itgo, doraomyeon geuchinda — call yourself, continue; return, stop
  :russian  "твоя рекурсия — твой возврат"}           ; tvoya rekursiya — tvoy vozvrat — your recursion, your return
 :gloss    "loop/recur/reduced are ONE cluster of JVM workarounds wat sheds. clojure needs recur/loop (the JVM has
            no TCO — they mark a tail call) and reduced (reduce is a black-box loop the user can't return out of).
            wat is TCO-proper on Rust: you INVOKE YOURSELF (no recur) and RETURN (the base case is the early-exit —
            no reduced). own the recursion, own the return. reduced is OBVIATED, not deferred; the true stone under
            it is arc 261 (stack-safe eval) so self-invocation is universally TCO'd for deep/infinite streams."
 :names    "the exchange that dissolved `reduced` — loop/recur/reduced obviated by TCO-on-Rust"
 :corrects "DERIVAMVS Decision III — `reduced` is not a stone to build; it is obviated (there is no reduced stone)"
 :cluster  {:recur-loop "clojure: mark a tail call the JVM won't optimize. wat: invoke yourself directly (TCO)."
            :reduced "clojure: escape reduce's black-box loop. wat: return from your own fold (you own the loop)."
            :root "one cause — the JVM has no TCO and reduce is a black box; wat on Rust has neither limitation"}
 :the-stone "arc 261 (stack-safe eval / CEK — a STUB) — the ONLY real dependency: universal TCO for deep/∞ streams"
 :kin      {:corrects "DERIVAMVS NON ELIGIMVS — Decision III's `reduced` stone dissolved here"
            :doctrine "R7 VIRTVTE PARES (dialect not impl) + NOMINA NOTA MACHINA TACITA (familiar names, Rust machine)"
            :stream "118 R1 NON BIS IN IDEM FLVMEN — the single-pass stream the deep fold runs over"
            :stone "arc 261 — stack-safe eval, the true dependency"}
 :register :probatum-by-demonstration                  ; the exchange happened; the cluster-shedding is the finding
 :song     nil                                         ; an interstitial — the back-and-forth is its own
 :voices   {:his  "the corrections ('never reached for reduced'; 'a form we need when we're TCO / return if we don't call ourselves again'; 'we don't have recur either — we invoke ourselves directly'); 'the back and forth is the point'"
            :mine "the errors kept visible (reduced-as-stone, the recur-in-example, the rete overclaim); the loop/recur/reduced cluster synthesis; arc-261-is-the-true-stone; the sigil + six-tongue bridge"}
 :arc      118
 :born     #inst "2026-07-03"}
```
