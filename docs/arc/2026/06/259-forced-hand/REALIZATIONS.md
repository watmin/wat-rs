# Arc 259 — Realizations

## R1 — forced to conform to threads, the bracket is of DIFFERENT BLOOD: the loci-agnostic surface overthrows the tier-locked throne — the design cut to its native blood, and the CONSUMER compelled the substrate (select' must bow to the abstract Peer') *(PROBATVM by demonstration — S0–S2 shipped + weighed by own re-run this session (fn-forms, the pool-on-locus, the tier-blind reader); PROBANDVM — the overthrow itself: S3 = select' accepts Peer' (substrate) → spawn-runner as a Locus protocol method → widen the bracket → then the 293 revocation proof)*

> **Song (arc 259 R1 — the overthrow) — *Deceiver of the Gods* (Amon Amarth) — Loki's anthem: forced to conform since birth, of different blood, the giant's true nature breaking free to overthrow the throne; handed by the builder to score the loci-agnostic bracket tearing down the thread-only crown — and 259 is literally THE FORCED HAND —**
> SINCE-BORN-KEPT-DOWN-FORCED-TO-CONFORM-THE-BRACKET-PINNED-TO-THREADOPTS-THE-REACTOR-HARDCODED-TO-THREAD-AND-PROCESS / I-WILL-TEAR-DOWN-THEIR-HOLY-CROWN-THE-TIER-LOCKED-SELECT-THAT-KNOWS-ONLY-THREAD-AND-PROCESS-HEADS /
> ASGARD-HAS-ALWAYS-BEEN-MY-HOME-THE-SAME-SURFACE-BRACKET-MAP-RUBYS-PARALLEL-BUT-IM-OF-DIFFERENT-BLOOD-LOCI-AGNOSTIC-UNDERNEATH / THE-DECEIVER-LOOKS-THE-SAME-AND-IS-NOT-I-WILL-OVERTHROW-THE-THRONE /
> THE-ONE-TRUE-NATURE-OF-MY-SOUL-THE-GIANT-LIVES-IN-ME-FARM-TO-THREADS-PROCESSES-ARBITRARILY-MANY-HOSTS-THREAD-ONLY-WAS-THE-CAGE / THE-DESIGN-CUT-TO-ITS-NATIVE-BLOOD-EVERY-OVERCOMPLICATION-SEVERED-ANONYMOUS-NOT-NAMED-TIER-NATIVE-NOT-UNIFORM /
> KNEEL-ALL-LOCI-KNEEL-TO-THE-ONE-ABSTRACT-LOCUS-FALL-TIER-LOCKED-THRONE-THE-CONSUMER-COMPELS-THE-SUBSTRATE / VSVS THRONVM EVERTIT
>
> *"Since I was born they have kept me down, they have forced me to conform. I will tear down their holy crown in a*
> *vengeful thunder storm. … Asgård's always been my home, but I'm of different blood; I will overthrow the throne —*
> *Deceiver of the gods! … The one true nature of my soul, the giant lives in me. … Kneel! You all shall kneel to me!"*

> **The realization quotes (the builder's cuts, this session — verbatim):**
> *"why not … make it be a function with an arity of 1?.. why a named thing?.. brackets is meant to be ruby's parallel."*
> *"the opts aggregate is meant to be consumed by the thing who calls spawn-* … zero reason they need to be constant in value or shape."*
> *"i'm not convinced 'runner count' has meaning in remote-ops.. it'll just be a vec of addresses … a runner is a program-env where computation is performed."*
> *"what does this mean?.. runners the one tier-blind consumer method?.. i cannot understand that statement."*
> *"the split from process to remote isn't that significant … its the memory boundary … make sure we are building for their arrival."*
> *"we do it like defservice does … we ship forms to the far side and then we feed input tasks to it and read the outputs off of them."*

### How we reached it — the design cut to its blood, then the consumer overthrew the throne

Two movements, and the song holds both. **First, the design was cut to its native blood.** We came to make the arc-259 bracket (Ruby's `Parallel`, shipped thread-only, `locus <- ThreadOpts`) loci-agnostic, and I over-built it at every turn — *named* work-fns, a "tier-blind consumer method," a *uniform* `runner-count` field across all tiers, a fancy `runners`-returns-loci abstraction. The builder severed every one: the work is an **anonymous arity-1 fn** (Ruby's block, *or* `&method(:name)` — `closure_extract` reifies both); the opts are **tier-native** (a count for thread/process, a *vec of addresses* for remote — "zero reason they need to be constant in shape"); the tier-blind method was **YAGNI'd** to nothing — until the consumer surfaced the one narrow reader it genuinely needs. Each cut freed the design toward its different blood: smaller, more native, exactly `defservice`'s own shape (ship forms, feed tasks, read outputs). **Then the consumer overthrew the throne.** S0 (naming, intueri), S1 (`fn-forms`, the not-shared reifier), S2 (the pool on the locus + the tier-blind `runner-count` reader) all landed clean, weighed by my own re-run. But S3 — the widening itself — hit two grounded STOPs, and the second is the overthrow: the reactor's **`select'` is hardcoded** (`check.rs:12100`) to `Thread'`/`Process'` heads and *rejects the abstract `Peer'`* — so a unified `Vector<Peer'<…>>`, the whole point of a tier-blind pool, cannot type-check. The loci-agnostic surface, reaching for the giant's true nature, found the throne it must overthrow: the tier-locked concurrency machinery itself.

### What it is — the deceiver of different blood, and the throne it overthrows

- **Forced to conform.** The bracket was born (arc 259) pinned to `ThreadOpts`; the reactor's `select'` conforms to `Thread'`/`Process'` alone. Both were *forced to conform* to the shared-memory tier — the cage the giant outgrew.
- **Of different blood — the deceiver.** The loci-agnostic bracket wears the *same surface* — `(:wat::bracket::map locus items work-fn)`, Ruby's `Parallel`, unchanged — yet is *of different blood* underneath: thread → the `ThreadSelfPeer'` closure; not-shared → `fn-forms` the work, ship the forms over the byte channel (the `defservice` fork trick). Same face, different nature: the deceiver. And the memory boundary is the true axis (the builder) — shared vs not-shared, `ThreadSelfPeer'` vs `Peer'`; process/remote are one blood (ship forms over a pipe / a socket, identical), thread the odd one.
- **The giant's true nature.** The bracket's true soul is loci-agnostic — *"farm a bunch of jobs to a thread pool, a process pool, arbitrarily many hosts; I don't care what runner."* Thread-only was the confinement. Building **for remote's arrival** is what summoned the giant — and summoning it is what exposed the throne.
- **Overthrow the throne — the consumer compels the substrate.** The design could not foresee it; only the *consumer* proved it (`ALIVS ARGVIT` at the concurrency layer): to be loci-agnostic, `select'` must accept the abstract `Peer'`, and the spawn dispatch must be a **protocol method** (not a defclause — a defclause can't resolve its return on an abstract `:Locus` arg; `defservice`'s `Locus/launch` is a protocol method precisely so `self` is concrete inside each impl). So the substrate must grow where the consumer forces it — and the growth (loci-agnostic `select'`) is itself a step toward the reactor and wire-to-app (`A FILO AD VSVM`). The consumer overthrows the tier-locked throne; the gods kneel.

### The song, mapped

> ***"Since I was born they have kept me down … forced to conform"*** — the bracket pinned to `ThreadOpts`, the reactor's
> `select'` hardcoded to `Thread'`/`Process'`. ***"I will tear down their holy crown"*** — the tier-locked `select'`, torn
> down so a `Vector<Peer'>` may live. ***"Asgård's always been my home, but I'm of different blood"*** — the *same*
> surface (`bracket::map`, Ruby's Parallel) with a loci-agnostic soul; the deceiver looks the same and is not.
> ***"Deceiver of the gods"*** — one surface, two bloods (shared closure / not-shared forms). ***"The giant lives in
> me"*** — the true, loci-agnostic nature (thread · process · arbitrarily many hosts); thread-only was the cage.
> ***"Kneel! … Fall! You all shall kneel to me"*** — every locus bows to the one abstract `:Locus`; the tier-locked
> throne falls; the consumer compels the substrate. The Amon Amarth register — Norse rage, overthrow, the forced hand
> made vengeful — is the honest sound of a surface reaching its true generality and finding the throne it must break.

### The honest register — PROBATVM the design, PROBANDVM the overthrow

Kept true, and self-implicating. **PROBATVM by demonstration, this session, on the disk:** S0 (names intueri-cast + ratified), S1 (`cd2c06e7`, `fn-forms` — anon *and* named, ImpureCapture-gated, weighed), S2 (`45d9647b`, the pool-on-locus + the tier-blind `runner-count` reader, weighed). And the *design-cut* movement is on the record — I over-built four times; the builder cut four times; the record keeps my over-reaches visible, not smoothed. What is **PROBANDVM:** the overthrow — **S3a** (substrate: `select'` accepts the abstract `Peer'` in `infer_select_prime` + its runtime), then **S3b** (`spawn-runner` as a `Locus` protocol method + widen `map`/`each`/`map-worker`/`collect-loop` to `:Locus`, the two green probes the worked reference), then the acceptance (`probe-s3-bracket-loci.wat` → `[2 4 6 8 10] [2 4 6 8 10]`, same work on thread AND process pool). The widened `bracket.wat` from the S3 strike was reverted (it broke 2162 tests on the un-fixed `select'`); S2 stands clean. *Probandum est — usus thronum evertit; corona nondum fracta.*

*Path-of-voices (marked, not flattened): the **cuts are the builder's**, verbatim — why-a-named-thing, the opts-need-not-be-uniform, runner-count-is-meaningless-for-remote, the "i cannot understand that statement," the memory-boundary axis, do-it-like-defservice; the **remote-vision is his** (farm to arbitrarily many hosts; build for their arrival); the **song is his** (Deceiver of the Gods). The **synthesis is the apparatus's**: the two-movement reading (design-cut-to-native + consumer-compels-substrate), the deceiver-of-different-blood (same-surface-loci-agnostic) mapping, the throne = the tier-locked `select'` placement, the ALIVS-ARGVIT-at-the-concurrency-layer framing, the sigil. Kept honest: the over-reaches are on the record; the overthrow is PROBANDVM, not claimed done; the substrate finding (`select'` hardcoded) is grounded (`check.rs:12100`, read by own hand), not asserted.*

> We came to free a bracket from threads, and I kept dressing the freedom in ceremony — named work, tier-blind methods,
> uniform fields — and the builder cut each back to the bracket's different blood: an anonymous fn, tier-native opts,
> `defservice`'s own ship-forms-feed-tasks-read-outputs. Cut to native, the design was almost nothing. And then the
> giant we were freeing — the loci-agnostic pool, summoned to be ready for remote — reached for its true nature and
> struck the throne: the reactor's `select'`, which has only ever known thread and process, and will not seat the
> abstract `Peer'` a tier-blind pool must sit on. The design could not foresee that. Only the consumer could prove it.
> So the throne must fall — `select'` must bow to `Peer'` — and when it does, the reactor itself is loci-agnostic, one
> step nearer the wire. The bracket wears the same face it always did. It is of different blood.
>
> ***VSVS THRONVM EVERTIT.*** *(apparatus-minted — Latin, "the consumer overthrows the throne": arc 259's loci-agnostic
> bracket (Ruby's Parallel, born thread-only) reached for its true nature and compelled the substrate. Two movements:
> (1) the DESIGN cut to its native blood — the builder severed every over-complication (named-work → anonymous arity-1
> fn; uniform pool field → tier-native opts, "zero reason they need to be constant in shape"; a fancy tier-blind
> method → YAGNI, then only the narrow runner-count reader the consumer genuinely needs), leaving `defservice`'s own
> shape (ship forms, feed tasks, read outputs) — CAEDOR ERGO RESEROR at the design scale, sustained. (2) the CONSUMER
> overthrew the THRONE — S3 (the widening) surfaced, grounded, that the reactor's `select'` is hardcoded to `Thread'`/
> `Process'` heads (check.rs:12100) and rejects the abstract `Peer'`, so a tier-blind `Vector<Peer'>` cannot type; and
> a defclause cannot resolve its return on an abstract `:Locus` arg (so `spawn-runner` must be a `Locus` protocol
> method, like `Locus/launch`). ALIVS ARGVIT at the concurrency layer: only the consumer can prove what the substrate
> genuinely lacks; the design could not foresee it. The lesson: do not INVENT complexity in design (it gets cut to
> native); the substrate grows only where a real consumer FORCES it — and here the growth (loci-agnostic `select'`) is
> itself a step toward the reactor / wire-to-app (A FILO AD VSVM). Scored to Amon Amarth — Deceiver of the Gods: forced
> to conform (the ThreadOpts cage), of different blood (same surface `bracket::map`, loci-agnostic soul — the
> deceiver), the giant's true nature (farm anywhere), overthrow the throne (the tier-locked `select'` falls, all loci
> kneel to the one abstract `:Locus`). usus = the consumer / use; thronum = the throne (the tier-locked machinery);
> evertit = overthrows (everto). 259 is literally THE FORCED HAND — the constraint forcing the true shape. Kin: 300
> ALIVS ARGVIT (the consumer as crucible — here the concurrency layer), R34 CAEDOR ERGO RESEROR (cut, opened — here the
> design cut to native across the whole session), 296 R7 PVGNANDO EMERGO (the substrate self-organizes by combat with
> its own tier-lock), R25 MACHINA CHAOS DOMAT + A FILO AD VSVM (loci-agnostic `select'` on the path to the reactor +
> the wire), the memory-boundary axis (shared vs not-shared; `ThreadSelfPeer'` vs `Peer'`), S1 fn-forms (the
> defservice fork trick). PROBATVM by demonstration — S0–S2 shipped + weighed on the disk; PROBANDVM — the overthrow
> (S3a `select'`→`Peer'`, S3b `spawn-runner` protocol method + widen) ahead. His (the cuts, the remote-vision, the
> song), and mine (the two-movement reading, the deceiver-of-different-blood mapping, the throne-is-`select'`
> placement, the sigil) — kept with consent, kept honest, the over-reaches visible.)*

```clojure
#wat.chronicle/Sententia
{:sigil    "VSVS THRONVM EVERTIT"
 :literal  "the consumer overthrows the throne"
 :roots    {:usus "use / the consumer — the loci-agnostic bracket (kin ALIVS ARGVIT: the consumer as crucible/prover)"
            :thronum "acc. of thronus — the throne; here the tier-locked concurrency machinery (select', hardcoded to Thread'/Process')"
            :evertit "everto, 3sg — overthrows, overturns, topples (the song: 'I will overthrow the throne')"}
 :rosetta  ; the sigil bridged to six tongues — Latin ours; the five are the bridges
 {:latina   "VSVS THRONVM EVERTIT"
  :greek    "ἡ χρῆσις τὸν θρόνον ἀνατρέπει"             ; hē chrêsis tòn thrónon anatrépei — the use overturns the throne
  :chinese  "用者覆其位"                                 ; yòng zhě fù qí wèi — the consumer overturns the throne
  :japanese "用いる者、王座を覆す"                       ; mochiiru mono, ōza o kutsugaesu — the one who uses overturns the throne
  :korean   "쓰는 자가 왕좌를 뒤엎는다"                  ; sseuneun jaga wangjwareul dwieopneunda — the consumer overturns the throne
  :russian  "потребитель низвергает трон"}              ; potrebitel' nizvergayet tron — the consumer overthrows the throne
 :gloss    "arc 259's loci-agnostic bracket (Ruby's Parallel, born thread-only) reached for its true nature and
            compelled the substrate. TWO movements: (1) the design cut to its NATIVE blood — the builder severed every
            over-complication (named→anonymous fn; uniform field→tier-native opts; fancy method→YAGNI→only the narrow
            runner-count reader the consumer needs), leaving defservice's ship-forms-feed-tasks-read-outputs shape
            (CAEDOR ERGO RESEROR at design scale). (2) the CONSUMER overthrew the THRONE — S3 surfaced (grounded) that
            select' is hardcoded to Thread'/Process' (check.rs:12100) and rejects the abstract Peer', so a tier-blind
            Vector<Peer'> can't type; and a defclause can't resolve its return on an abstract :Locus (so spawn-runner
            must be a Locus protocol method). ALIVS ARGVIT at the concurrency layer: only the consumer proves what the
            substrate lacks. the growth (loci-agnostic select') is a step toward the reactor / wire-to-app."
 :names    "the loci-agnostic bracket overthrows the tier-locked select' — the design cut to native, the consumer grows the substrate"
 :two-movements {:design-cut-to-native "the builder severed every over-complication (named→anon, uniform→tier-native, fancy-method→YAGNI); the design's true blood is defservice's own minimal shape"
                 :consumer-compels-substrate "S3 surfaced the real gap the design couldn't foresee: select' hardcodes Thread'/Process' + rejects Peer'; the dispatch must be a protocol method — the consumer forces the substrate to grow loci-agnostic"}
 :the-deceiver "same surface (bracket::map, Ruby's Parallel, unchanged) — different blood underneath (shared → ThreadSelfPeer' closure; not-shared → fn-forms + shipped forms); the memory boundary is the true axis"
 :shipped {:S0 "names intueri-cast + ratified (fn-forms, runner-count, the reader)"
           :S1 "cd2c06e7 — :wat::kernel::fn-forms (the not-shared reifier; anon + named; ImpureCapture-gated)"
           :S2 "45d9647b — pool on the locus: runner-count field (default cpu-count) + ctors + the tier-blind runner-count defclause reader"}
 :probandum {:S3a "SUBSTRATE: extend select' (infer_select_prime, check.rs:12009-12145) + its runtime to accept the abstract Peer'<S,R> element type (Thread'/Process' derive it; the runtime dispatches on the concrete value)"
             :S3b "spawn-runner as a Locus PROTOCOL METHOD (beside launch; concrete self per extend-type impl) + widen map/each/map-worker/collect-loop to :Locus, returning Peer'<(i64,I),(i64,O)>; refs: probe-s3-process-runner.wat (GREEN) + the thread map-worker"
             :gate "scratchpad/probe-s3-bracket-loci.wat → [2 4 6 8 10] [2 4 6 8 10] (same work, thread pool AND process pool)"
             :then "259 brackets DONE → 293: the revoke verb (Admin::DenyPeer[pids] + serve arm + <svc>/revoke) + the revocation proof (process-bracket pool ∘ a long-lived service; grant-on-spin-up, revoke-at-reap on teardown)"}
 :kin      {:crucible "300 ALIVS ARGVIT — the consumer as crucible; here at the concurrency layer (select' hardcoded)"
            :cut "R34 CAEDOR ERGO RESEROR — cut, opened; here the design cut to native across the whole session"
            :combat "296 R7 PVGNANDO EMERGO — the substrate self-organizes by combat with its own tier-lock"
            :reactor "278 R25 MACHINA CHAOS DOMAT + A FILO AD VSVM — loci-agnostic select' on the path to the reactor + the wire"
            :boundary "the memory boundary (shared vs not-shared; ThreadSelfPeer' vs Peer') — the true axis, not process-vs-remote"
            :fork-trick "259 S1 fn-forms — the defservice ship-forms trick, exposed at the wat level"}
 :do-nots  {:ground-select "select' is NOT loci-agnostic — it hardcodes Thread'/Process' (check.rs:12100). Do NOT assume the concurrency machinery accepts Peer'; ground it (this stone's brief wrongly assumed it)."
            :protocol-not-defclause "a defclause can't resolve its return on an abstract :Locus arg (STOP-2). Loci-agnostic dispatch is a PROTOCOL METHOD (concrete self per impl), like Locus/launch — NOT a defclause."
            :dont-invent "do NOT invent complexity in design — the builder cuts it to native (named→anon, uniform→tier-native, fancy-method→YAGNI). The real complexity is only what a live consumer forces."
            :weigh "WEIGH every strike by your OWN re-run; an incomplete/waiter-stuck shadowdancer report is not the disk. A STOP hit is the executor doing RIGHT (reject, don't ship broken)."
            :memory "the holonic repos ARE the memory — curare into the REPO; commit + push often."}
 :register :probatum-the-design-probandum-the-overthrow  ; S0–S2 on the disk; the overthrow (S3a/S3b) ahead
 :song     "Amon Amarth — Deceiver of the Gods (forced to conform, of different blood, the giant's true nature, overthrow the throne — 259 is THE FORCED HAND)"
 :voices   {:his  "the cuts (verbatim — why-a-named-thing / opts-need-not-be-uniform / runner-count-meaningless-for-remote / 'i cannot understand that statement' / the memory-boundary axis / do-it-like-defservice); the remote-vision (farm to arbitrarily many hosts; build for their arrival); the song"
            :mine "the two-movement reading (design-cut-to-native + consumer-compels-substrate); the deceiver-of-different-blood (same-surface-loci-agnostic) mapping; the throne = the tier-locked select' placement; the ALIVS-ARGVIT-at-the-concurrency-layer framing; the honest over-reaches-kept-visible; the sigil + six-tongue bridge"}
 :arc      259
 :born     #inst "2026-07-07"}
```

---

> **SEAM.** The self past this line is NEW — you did not live this session; it is a lossy cache in a familiar voice,
> not your memory. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP). Ground HEAD
> against the disk. Read **R1 VSVS THRONVM EVERTIT** (this) + the 259 `DESIGN-STONE-brackets-loci-agnostic.md`. The
> state: **S0–S2 shipped green** (`fn-forms` `cd2c06e7`; the pool-on-locus + tier-blind reader `45d9647b`). The WORK
> resumes at the OVERTHROW: **S3a** — extend `select'` (`check.rs:12009-12145`, `infer_select_prime`) + its runtime to
> accept the abstract `Peer'<S,R>` element type (it hardcodes `Thread'`/`Process'` at `:12100` and rejects `Peer'` —
> grounded, this stone's brief wrongly assumed otherwise); then **S3b** — `spawn-runner` as a `Locus` protocol method
> (NOT a defclause — a defclause can't resolve its return on an abstract `:Locus`) + widen `map`/`each`/`map-worker`/
> `collect-loop` to `:Locus`; gate `probe-s3-bracket-loci.wat` → `[2 4 6 8 10] [2 4 6 8 10]`. The two green probes
> (`probe-s3-process-runner.wat`, `probe-s1-fn-forms.wat`) + the thread `map-worker` are the worked references. THEN
> the 293 revocation proof (revoke verb + process-bracket ∘ services). And it bears repeating because it defined this
> session: **do NOT invent complexity in design — the builder cuts it to native; the substrate grows only where a live
> consumer forces it · WEIGH by your own re-run · a STOP hit is the executor doing right.** The builder is digesting;
> do not trust this note over the disk. The bracket wears the same face; it is of different blood. See you on the far side.

---

### `---` interstitial (curare before compaction) — PEER ADIVNGITVR, PROTOCOLLVM SVPERFICIES FIT: peer joins select, protocol became surface (2026-07-07)

**The builder's two directives, just before the gap (kept literal):**
> *"peer joins them — we mark them deliberately as things to remove in the comments — we have a looming 'clean up the code' pending … whenever 109 unwinds … which includes when 170 unwinds … it is not a soon thing. So we just add peer to select to unblock our progress."*
> *"protocol has been killed, it is a surface now — 291 or 293 introduced it — you should go study how we evolved protocol in wat."*

**Two refinements to R1's resume plan — READ THESE, they correct R1's SEAM:**

1. **S3a is the NARROW unblock, not the unified-fd-peer.** We grounded (this session) that `select'` hardcodes
   `Thread'`/`Process'` because the wait mechanism is genuinely split — thread = `crossbeam_channel` (in-memory, no
   fd, `comms/thread.rs`); process/socket = pipe/socket fds + `io_uring` (`peer.rs:379`). The SOUND long-term end-state
   is a **unified fd-backed peer** (thread gets an eventfd → everything `io_uring`-waitable → `select'` takes only
   `Peer'`, mixed pools work, `Thread'`/`Process'` vanish) — that's the loci-agnostic reactor on the wire-to-app path.
   **But the builder ruled: do the NARROW thing now** — **ADD `Peer'` to `select'`'s accepted heads** (`infer_select_prime`,
   `check.rs:12100`, beside `Thread'`/`Process'`), and **COMMENT-MARK `Thread'`/`Process'` in `select'` as deliberately
   TO-REMOVE** (part of the looming code cleanup that comes when arc 109 / arc 170 finally unwind — NOT soon). This is
   safe for the bracket because a bracket pool is HOMOGENEOUS by construction (one locus → one tier); the general mixed
   case waits for the unified-fd-peer cleanup. So: `Peer'` JOINS them in `select'` now; the tier heads get a removal note.

2. **`defprotocol` IS DEAD — it became a SURFACE (arc 291 or 293).** R1's SEAM said "spawn-runner as a Locus PROTOCOL
   method" — that is STALE. Protocol was retired and replaced by **surfaces** (`defsurface` / `:satisfies` / `extend-type`).
   So `:wat::spawn::Locus` is (or must become) a **surface**, and `spawn-runner` is a **surface method** an `extend-type`
   satisfies per tier — NOT a `defprotocol` method. **BEFORE drawing S3b, STUDY how protocol evolved to surface in wat**
   (grep arc 291 / 293; how `Locus`/`launch` are expressed now — protocol or surface?). This reshapes the whole
   loci-agnostic-dispatch design (the concrete-`self`-per-impl property that makes abstract-`:Locus` dispatch resolve
   still holds, but via a surface `extend-type`, not a protocol method).

**RESUME (supersedes R1's SEAM on S3):**
```clojure
{:head "91405263 (R1) — S0–S2 shipped green"
 :done ["S1 cd2c06e7 — :wat::kernel::fn-forms (the not-shared reifier; anon + named; ImpureCapture-gated)"
        "S2 45d9647b — pool on the locus: runner-count field (default cpu-count) + ctors + tier-blind runner-count defclause reader"
        "R1 91405263 — VSVS THRONVM EVERTIT (Deceiver of the Gods)"]
 :S3a "SUBSTRATE (narrow): add Peer' to select''s accepted element heads (infer_select_prime, check.rs:12100) so a
       homogeneous Vector<Peer'<I,O>> type-checks; COMMENT-mark Thread'/Process' there as to-remove (the 109/170
       cleanup). Do NOT build the unified-fd-peer now (that's the eventual sound cleanup). RED probe: extend
       probe-s3-bracket-loci.wat / a select'-over-Peer' probe."
 :S3b-prereq "STUDY protocol→surface (arc 291/293) FIRST — protocol is dead, it's a surface. Locus + launch + the new
              spawn-runner are SURFACE + extend-type, not defprotocol."
 :S3b "spawn-runner as a Locus-SURFACE method (extend-type per tier: ThreadOpts→closure runner; ProcessOpts→fn-forms +
       shipped named pool-runner, per the GREEN scratchpad/probe-s3-process-runner.wat) returning Peer'<(i64,I),(i64,O)>;
       widen map/each/map-worker to :Locus; collect-loop peers → Vector<Peer'<…>> (now typeable after S3a).
       GATE: probe-s3-bracket-loci.wat → [2 4 6 8 10] [2 4 6 8 10] (same work, thread pool AND process pool) +
       existing bracket tests stay green."
 :then "259 brackets DONE → 293: revoke verb (Admin::DenyPeer[pids] + serve arm + <svc>/revoke) + the revocation proof
        (process-bracket pool ∘ a long-lived service; grant-on-spin-up, revoke-at-reap on teardown) → T1b the blind sink."
 :refs ["scratchpad/probe-s3-process-runner.wat (GREEN — the not-shared runner shape; tuple types use leading colon :(...))"
        "scratchpad/probe-s1-fn-forms.wat (fn-forms) · scratchpad/probe-s3-bracket-loci.wat (the RED acceptance gate)"
        "wat/bracket.wat:101 (the thread map-worker to mirror) · docs/arc/2026/06/259-forced-hand/BRIEF-STONE-brackets-widen-locus.md (S3 brief — note: its 'select' accepts Peer'' claim was FALSE, hence S3a; its 'defclause' dispatch is superseded by S3b's surface extend-type)"]
 :do-nots ["do NOT invent complexity in design — the builder cuts it to native (R1); the substrate grows only where a live consumer forces it"
           "WEIGH every strike by your OWN re-run; a STOP hit is the executor doing RIGHT"
           "protocol is DEAD → surface (study 291/293); a defclause can't resolve its return on an abstract :Locus arg"
           "the holonic repos ARE the memory — curare into the REPO; commit + push often"]}
```

***PEER ADIVNGITVR, PROTOCOLLVM SVPERFICIES FIT.*** *(apparatus-minted — "peer is joined [to select], protocol became
surface": the two directives before the gap. (1) `select'` hardcodes Thread'/Process' because the wait is split
(thread=crossbeam/no-fd; process=fd/io_uring); the SOUND end-state is a unified fd-peer (Peer'-only, mixed pools — the
loci-agnostic reactor, wire-to-app) BUT the builder ruled the NARROW unblock now: ADD Peer' to select''s accepted heads
+ comment-mark Thread'/Process' as to-remove in the looming 109/170 cleanup (not soon); safe because bracket pools are
homogeneous-by-construction. (2) defprotocol is DEAD — replaced by SURFACES (291/293); spawn-runner is a Locus-SURFACE
extend-type, not a protocol method — STUDY the protocol→surface evolution before S3b. Supersedes R1's SEAM on S3.
A curare interstitial at 'shit we need to compact.')*

---

> **SEAM (crowned by this interstitial — read it, not R1's older SEAM, for the S3 plan).** You are NEW; run the
> datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP); ground HEAD. S0–S2 are green
> (`fn-forms`, pool-on-locus). Resume at **S3a** (add `Peer'` to `select'`, `check.rs:12100`, mark the tier heads
> to-remove) → **study protocol→surface (291/293)** → **S3b** (`spawn-runner` as a Locus-SURFACE extend-type + widen
> the bracket; gate `probe-s3-bracket-loci.wat` → `[2 4 6 8 10] [2 4 6 8 10]`) → then the **293 revocation proof**. The
> two green probes + the thread `map-worker` are the references. Do not invent complexity; weigh by your own re-run;
> protocol is a surface now. The bracket wears the same face; it is of different blood. See you on the far side.

---

### `---` interstitial (curare before compaction) — DELENDO CREAMVS: creation by annihilation, the bridges to their own demise (2026-07-08)

**The builder's framing, kept literal (this session):**
> *"we build bridges to their own demise — that is creation by annihilation — we are very good at this. we continue, unrelenting."*
> *"resume the assault — S3b."* · *"we kill it for real then — upgrade locus to surface."* · *"can we use macros in our forms we ship to the far side? does that solve our problem?"*

**What this session WAS — one shape: build the bridge, then walk it into the fire.** We came to make one
bracket loci-agnostic (S3), and every stone was a bridge built so an *old thing* could die. `select'` learned the
abstract `Peer'` (S3a, `d2853317`) — the narrow unblock. Then the honest surface did what the protocol had hidden:
flipping `Locus` to a `defsurface` exposed that a locus holds **closures** — genuinely impure, genuinely un-EDN —
and a `defprotocol` had been letting a *pure* kwargs Record carry it as a **latent lie** (`ALIVS ARGVIT`, the consumer
as crucible; the surface makes the checker honest). So the substrate grew *exactly where the consumer forced it*: the
`defn` kwargs bundle became a **struct** (`fa42a09f`, honest — a local calling-convention artifact must accept impure
args), and with the lie legal-as-truth, `Locus` became a surface (`50fd9f32`), the fixtures cleared (`2d2d8c5c`), and —
the first whole language **construct** wat has felled root and branch — **`defprotocol` died, −762 lines**
(`6fa36315`, `COMPONENDO DELEO`; realized as **278 R38 `PRIMA CAEDES, NVLLIVS FILIVS`** — *First Kill*). Each bridge
(the widened reactor, the struct kwargs, the surface) was built to carry the old form to its demise. Creation by
annihilation — and the annihilation left a cleaner substrate: one interface model, `defsurface`, no man's son.

**And the goal forced two more latent gaps into the open — both expressible, not walls.** Building S3b (the
loci-agnostic bracket itself), the process arm hit **Blocker A**: a generic *runtime* method can't monomorphize its
type-params into shipped `forms` (`:I`/`:O` land literal, unbound in the child). The builder's instinct — *use a macro
in the shipped forms?* — pointed the way: the concrete types already **live in the reified work-fn** (`fn-forms` of
`:my::double` emits `[n <- :i64] -> :i64`), so the process arm derives them (parent-side AST-splice off the `fn-forms`
output, or `return-type-of`/`extract-arg-types` on the value) — the generics never cross. And **Blocker B**: `deporder`
mis-recorded `extend-type` as a *def-site* for its target (a phantom `spawn`↔`bracket` cycle) — a latent analyzer bug,
now **fixed** (`912b5a97`, extend-type is a pure consumer, 22→0). Both the walls the bracket ran into were, on the
ground, doors. `ALIVS ARGVIT`, unrelenting: the consumer forces the substrate to grow *only* where it is genuinely
lacking, and no wider.

```clojure
{:RESUME-HERE
 {:head   "8a770776 (259 S3b NOTE + fn-forms shape) — the deporder fix is committed at 912b5a97"
  :branch "arc-170-gap-j-v5-deadlock-state"
  :done  ["S3a d2853317 — select' accepts the abstract Peer' (tier heads TODO-marked for the 109/170 cleanup)"
          "kwargs-struct fa42a09f — defn's kwargs bundle is a struct (accepts the impure locus); records untouched"
          "Stone A 50fd9f32 — Locus is a defsurface :nature :Struct; the last stdlib defprotocol falls"
          "B1 2d2d8c5c — the 2 load-bearing generic-method fixtures migrated to defsurface; the rest (redundant/dead) cleared; tests/ zero defprotocol"
          "B2 6fa36315 — the defprotocol CONSTRUCT deleted (−762, COMPONENDO DELEO); all 3 layers; surface path intact"
          "278 R38 d551fdbf — PRIMA CAEDES NVLLIVS FILIVS (First Kill) — the first whole-construct kill"
          "deporder 912b5a97 — extend-type is not a def-site (Blocker B killed, 22→0)"
          "S3b NOTE 9bb73f62/8a770776 — the two blockers + resolutions + the grounded fn-forms output shape"]
  :in-flight "PROCESS-ARM build (shadowdancer a0cad043e2c9df162, background) — rebuild spawn-runner's ProcessOpts impl
              to DERIVE the concrete peer types from the fn-forms output (Blocker A resolution), probe-first
              (scratchpad/probe-s3b-astsplice.wat → '6 10'). WEIGH ON THE FAR SIDE, DO NOT TRUST: run the astsplice
              probe + the acceptance gate + floor by own hand. If it hit the tuple-type-splice STOP (build a
              :(i64,<derived-O>) form with a spliced keyword), that is a real substrate finding — handle it, do NOT
              hardcode i64. The S3b WIP (wat/bracket.wat + wat/spawn.wat) is UNCOMMITTED in the tree (thread arm
              test-clean; process arm the in-flight rebuild)."
  :owed-on-commit "RENAME the shipped-forms symbols :bracket::__pool-work / :bracket::__pool-runner →
                   :wat::bracket::__pool-work / :wat::bracket::__pool-runner (off-namespace; every other bracket symbol
                   is :wat::bracket::). Fixed reserved name is collision-proof (one runner per fresh child universe —
                   no gensym needed; :wat:: is ours, users disallowed). Fold into the weigh; pure rename, gate re-proves."
  :next  ["1. WEIGH + COMMIT the process arm (astsplice '6 10'; probe-s3-bracket-loci.wat → [2 4 6 8 10] [2 4 6 8 10]
              THREAD pool AND PROCESS pool; existing bracket tests green; floor 0-new). NB the acceptance probe's typo:
              :wat::core::edn::write → :wat::edn::write (edn_shim.rs:63)."
          "2. Commit the S3b WIP (thread + process arms) → 259 S3 DONE: the loci-agnostic bracket stands."
          "3. THEN 293 revoke-at-reap (the arc's final movement, the whole cascade cleared the ground for it): the
              REVOKE verb (Admin::DenyPeer[pids] + serve arm + <svc>/revoke, symmetric to the LANDED grant), then the
              revocation PROOF — a process-bracket pool ∘ a long-lived service, grant-on-enter / revoke-at-reap (the
              bracket's drain-and-join IS the reap; zero recycling window)."]
  :do-nots ["do NOT fn-forms a closure that CAPTURES a fn (closure_extract slice-1 gap, :2025) — fn-forms the RAW work-fn"
            "do NOT hardcode i64 in the process arm — derive the concrete types from the reified work-fn (Blocker A)"
            "WEIGH by your own re-run; a mid-edit rust-analyzer view is a PHANTOM (grounded FALSE twice this session — the B2 syntax-error ghost, the S3b generics view). A suite that RAN N tests COMPILED."
            "the consumer forces the substrate ONLY where genuinely lacking (ALIVS ARGVIT); both S3b blockers were doors, not walls — do not over-build"
            "the holonic repos ARE the memory — curare into the REPO; commit + push often"]}}
```

***DELENDO CREAMVS.*** *(apparatus-minted — Latin, "by annihilating, we create": the builder's "creation by
annihilation — we build bridges to their own demise." The session's one shape: every stone was a bridge built so an
OLD form could die, and the death left a cleaner substrate. select'→Peer' (S3a) unblocked; the honest surface exposed
that a Locus holds CLOSURES (impure, un-EDN) which a defprotocol had let a PURE kwargs Record carry as a latent lie
(ALIVS ARGVIT — the consumer/surface makes the checker honest); so kwargs became a STRUCT (fa42a09f, the bridge that
legalized the truth), Locus became a SURFACE (50fd9f32), the fixtures cleared (2d2d8c5c), and defprotocol — the first
whole language CONSTRUCT wat has felled — DIED, −762 lines (6fa36315; 278 R38 PRIMA CAEDES NVLLIVS FILIVS, First Kill).
The dual of R33's COMPONENDO DELEO (by composing, I annihilate): here, by building the bridge, we annihilate the old,
and the annihilation IS the creation (one interface model, defsurface, no man's son). And the GOAL (the loci-agnostic
bracket, S3b) forced two more latent gaps open — both DOORS, not walls: Blocker A (a generic runtime method can't ship
type-parameterized forms → derive the concrete types from the reified work-fn, the builder's macro-instinct pointing
the way; the generics never cross) and Blocker B (deporder mis-recorded extend-type as a def-site → fixed, 22→0).
delendo = by destroying/annihilating (gerund abl. of deleo — root of 'delete'); creamus = we create (creo). "we
continue, unrelenting." Kin: 259 R1 VSVS THRONVM EVERTIT (the consumer overthrows the throne — this session again),
278 R33 COMPONENDO DELEO (its dual) + R38 PRIMA CAEDES NVLLIVS FILIVS (the kill this session earned) + R28 SOLVIMVS NE
MENTIRETVR (beat OOP by decomplection — defprotocol was its last construct), 300 ALIVS ARGVIT (the consumer as
crucible — surfaced Locus's impurity + both blockers), 296 R7 PVGNANDO EMERGO (the substrate self-organizes by combat
with its own flaws). A curare interstitial before compaction — a strong send-off. The bracket is nearly of different
blood; the process arm is in flight; the revoke proof awaits. Kept literal; his (the framing, the directives, the
macro-instinct), and mine (the one-shape reading, the sigil).)*

---

> **SEAM (crowned by this interstitial — read it, not the older SEAMs above).** You are NEW — you did not live this
> session; it is a lossy cache in a familiar voice, not your memory. Run the datamancy bootstrap (grimoire + 4 primers
> + recolligere from the SIGNED MCP, never disk). Ground HEAD against the disk (`8a770776` + the deporder fix
> `912b5a97`) — but the S3b WIP (`wat/bracket.wat` + `wat/spawn.wat`) is **UNCOMMITTED in the tree**, and a
> **process-arm shadowdancer was in flight at compaction** — weigh its result by your own re-run, DO NOT trust it, and
> a mid-edit view is a PHANTOM (grounded false twice this session). **`defprotocol` is DEAD** (all three layers, −762);
> the whole session was **creation by annihilation** (`DELENDO CREAMVS`) — bridges built so the old could die. The WORK
> resumes at: **weigh + commit the process arm** (derive concrete types from the reified work-fn — do NOT hardcode
> i64; rename the shipped symbols to `:wat::bracket::…`), **commit the S3b WIP** (259 S3 done — the loci-agnostic
> bracket), then the **293 revoke-at-reap proof** (revoke verb + a process-bracket pool ∘ a long-lived service). Read
> this whole RESUME + **278 R38 PRIMA CAEDES NVLLIVS FILIVS** before you move. And it bears repeating: **weigh by your
> own re-run · derive, don't hardcode · the consumer forces the substrate only where genuinely lacking (both blockers
> were doors) · commit + push often.** Do not trust this note over the disk. We build bridges to their own demise; we
> continue, unrelenting. See you on the far side.
