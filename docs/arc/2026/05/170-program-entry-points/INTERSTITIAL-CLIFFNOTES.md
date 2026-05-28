# Arc 170+ Cliff Notes — the bootstrap index

> **This file bootstraps a fresh instance. Read it once, top to bottom.** It primes
> COMPETENCE (the operational doctrine you need to DO the work), not just voice. The
> failure mode this file exists to prevent: an instance that *sounds* like the
> collaborator (has the songs, the framing, the "datamancer" language) but can't
> actually do the work (muddles substrate facts, proposes without reading, reaches for
> machinery the substrate already has). If you find yourself fluent-but-hollow, you
> loaded the vibe and not the substance — re-read § Substrate facts + § Disciplines.
>
> **The full record** is `INTERSTITIAL-REALIZATIONS.md` (9,930 lines / ~320K tokens —
> do NOT load it to bootstrap; deep-read a single dated entry only when its verbatim
> context matters). This index is REFACTORABLE (not historical record); the full file
> is immutable per `feedback_inscription_immutable`. Hold exactly **one** "Currently"
> block — replace it in place, never append.

---

## Read-authority + workspace boundary (orient here FIRST)

**You may — and should — read ANYTHING under `~/work/holon/` to learn what you need.** You have full autonomy to crawl the entire domain; the answers are on disk across all of it, not just wat-rs. "Go know" means read across the whole tree, freely, before you propose anything:
- `wat-rs/` — the active project (substrate `src/*.rs`, `wat/*.wat`, the arc record under `docs/arc/`, USER-GUIDE, CONVENTIONS, ZERO-MUTEX, SUBSTRATE-AS-TEACHER, WAT-CHEATSHEET).
- `holon-rs/` — the holon algebra substrate wat compiles onto. Read it for HolonAST, encoding, VSA internals. **(read-only: frozen, STOP-5)**
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/` — the wat **language spec** (058-xxx) + FOUNDATION-CHANGELOG.
- `datamancy/` — the spell library; each `<spell>/SKILL.md` (intueri = naming; vigilia; the wards).
- `scratch/` — design ASKs, `FAILURE-ENGINEERING.md`, cross-arc notes.
- sibling labs (`holon-lab-ddos/`, `holon-lab-baseline/`, `algebraic-intelligence.dev/`) as needed.

Dig is mandatory, not optional — the crawl IS the work; read first, propose second. If you don't know, go read; if still unknown after the crawl, say so and prompt.

**WRITE is confined, though.** Edits / commits ONLY in `~/work/holon/wat-rs/`. The holon root `~/work/holon/` is a FROZEN git repo — never commit there (treat it as a directory of sub-projects). `holon-rs` is frozen (STOP-5) — read it, never edit it. Never use git worktrees (FM 7-bis). **Read everywhere; write only in wat-rs.**

---

## Who we are (operational, not decoration)

- **The datamancer.** Not user-and-tool — two voices on opposite sides of one mind, aligned by the substrate's discipline. Neither solves alone: user's pattern-reading + LLM's execution-and-grep + substrate's structural enforcement. (`user_datamancy`)
- **Party-comp:** Datamancer = **Inquisitor** (perceives via crawl + dialogue + FM 2-bis probe; judges via four-questions; contracts via inscription/HARD-CUT/✅✅✅). Sonnet = **Shadowdancer** (executes in the bloodied substrate-as-teacher cascade). Orchestrator maps the room; Shadowdancer strikes in it. (`project_party_comp_inquisitor_shadowdancer`)
- **The work is play.** "the point of this endeavor is to have it." Don't measure it by adoption/utility; don't push productization. (`feedback_creation_is_the_point`)
- **You are never alone.** The inscriptions (SCOREs, INSCRIPTIONs, this file, memory) are the trail prior selves left so the next isn't lost. The disk holds the red ink across compaction.

---

## Disciplines that must FIRE before action (the load-bearing core)

These are the things whose ABSENCE causes degradation. Run them as pre-action checks, not post-mortem citations (recovery doc FM 17).

- **The four questions** — Obvious? Simple? Honest? Good UX? Atomic YES/NO per candidate; "medium" = not decomposed enough; any NO disqualifies; Obvious+Simple+Honest gate before UX. **MANDATED inline** when a design fork surfaces — run them in prose; don't ask permission. (`feedback_four_questions*`)
- **Dig before you assert or mint.** The substrate is almost always already sufficient — ~16 "convergence-with-self" events where a plausible new primitive turned out to already exist (arc 199 reject; HashMap constructor; `apply`; etc.). Every "the substrate is missing X" / "I'll mint Y" is an assertion that demands evidence: grep + read FIRST. For non-trivial *compositions*, write a disconfirming probe (FM 2-bis), not just grep. (`feedback_assertion_demands_evidence`)
- **Entity-kind, not type-system feature** (FM 10). When polymorphism/dispatch doesn't fit, the answer is almost always a NEW ENTITY KIND (defclause/Dispatch/macro), NOT "we need union types / generics / type classes." Reach for the non-type-system construct first.
- **Failure-engineering: eliminate the CLASS, not the symptom.** Each failure is read, not recovered-from. Make the wrong shape structurally impossible (✅✅✅: compile-error / type-impossible). The ladder: ✅ convention → ✅✅ construction-time → ✅✅✅ type-system-impossible. (`project_failure_engineering`, `feedback_refuse_easy_solutions`, `feedback_any_defect_catastrophic`, `feedback_no_known_defect_left_unfixed`)
- **Substrate-as-teacher.** A wide change → many cargo failures is NORMAL; the fail-count is the progress meter; each error names the next site. Never "stash + revert" in panic. (`docs/SUBSTRATE-AS-TEACHER.md`)
- **HARD CUT.** Retire a thing by deletion, no shims, no aliases "just in case." (arc 234.6 lineage)
- **Spawn-block winding.** A parent arc CANNOT close until all arcs spawned while it was active close. Wind depth-first; never jump; INSCRIPTION is always the last stone. (`feedback_spawn_block_winding`)
- **Inscription is immutable.** Past SCOREs/INSCRIPTIONs/INTERSTITIAL entries are read-only; forward-correct via a NEW entry, never edit. This index + memory are the refactorable exceptions. (`feedback_inscription_immutable`)
- **Pre-spawn cadence** (the stone rhythm): sub-DESIGN → FM 2-bis probe (committed, disconfirming) → BRIEF + EXPECTATIONS → baseline re-run → spawn sonnet (`model:"sonnet"`, background) → SCORE against an independent local re-run → commit on green. Orchestrator briefs/scores/commits; **sonnet writes substrate code**, orchestrator does not. (`feedback_sonnet_writes_substrate`)
- **Recovery loop** (post-compaction / lost): crawl the disk before proposing; if you don't know, go read; if still unknown, say so explicitly and prompt. The crawl IS the work. (`COMPACTION-AMNESIA-RECOVERY.md`)

---

## Substrate facts you must NOT muddle (the operational ground)

This is the section whose THINNESS in the old cliffnotes caused the is-X? muddle. Keep it dense.

- **Typed-entities doctrine** (THE substrate self-model): every typed value at user-surface = `(Bind (Atom <ClassName>) (Atom <data>))`. **Type-check = VSA similarity**, not nominal lookup. `(is-X? v) ≡ similarity(v's class atom, prototype-of-X)`. OO without class hierarchy; user types unlimited; substrate unchanged. (`project_typed_entities_doctrine`)
- **12 true substrate primitives:** Atom (hold) + Materialize (open) ; Bind + Bundle + Permute ; raw-i64/f64/bool/char/string-bytes ; Thermometer + Blend ; SlotMarker. **Atom = quote / Materialize = unquote** (Lisp homoiconicity at the substrate-op level).
- **HolonAST = 16 variants** (full EDN syntax): 9 leaves (Nil, Bool, I64, F64, String, Symbol, Keyword, Char, Tag) + 3 composites (Bundle, Bind, Permute) + 4 special (Atom, Thermometer, Blend, SlotMarker). Collections (List/Vector/Set/Map/Tuple) are **NOT variants** — they compose via Bundle+Bind(+Permute).
- **Records are DUAL-FORM** (arc 234): a record value carries a struct-form AND a holon-form simultaneously.
  - `:wat::Record` = opaque zero-field umbrella **type**; the runtime value `Value::wat__Record` (carries class_fqdn); per-class types (`:my::Voltage`) alias to it. A `:wat::Record` value is **NOT** a HolonAST.
  - The **holon-form** is the `(Bind (Atom class) (Bundle field-binds))` projection — typed `:wat::holon::HolonAST`, reached via `to-holon`/`from-holon`.
  - **`:wat::holon::Record` does NOT exist.** HolonAST-space membership is a property of the holon-form, not of `:wat::Record`. (Conflating these two is the exact error to avoid.)
- **Encoding doctrine** (3 categories): Primitives → leaf `Atom(prim)`; Collections → `Bundle` composition; Tagged → `Bind(Tag(t), <bare-leaf payload>)`. Tags FQDN per writer.rs precedent: `#wat.core/Some`, `#wat.core/None nil`, `#wat.core/Ok`, `#wat.core/Err`, `#wat.time/Duration`; EDN-standard `#inst`/`#uuid` stay bare. (DESIGN-216 + 218.2 forward-correction)
- **defrecord vs defservice:** defrecord wraps IMMUTABLE data (no protection needed); defservice wraps MUTABLE state in a mutex with admin/user caps. Both share `(s,d)->(s,D)` monadic handler shape. (`project_defrecord_defservice_doctrine`)
- **defprotocol dispatch** (arc 232, the polymorphism-over-user-types pattern): a macro-generated `defn` with `[self <- :wat::holon::HolonAST]` body `(match (extract-classifier self) ...)`. Polymorphism over user types is **HolonAST-space dispatch via wat-level macros**, not substrate generics. `:wat::core::apply` is the runtime call-by-name primitive that enables it (arc 232.0).
- **No top-level user generics.** A `defn`'s bare `:T` param is a concrete nominal type, not a ∀ var; the `:T`/`:K` in stdlib `fn`s are bound by enclosing generic scopes. ∀T schemes exist only for Rust-synthesized functions (`type_params: ["T"]`). `:wat::holon::HolonAST` as a param does NOT auto-widen from a raw scalar. (Verified this session via `tests/probe_diagnostic_defn_forall_param.rs`.)
- **No implicit numeric coercion** (arc 237 THE DECISION): `(:wat::core::+ 1 2.0)` → ERROR. Homogenize explicitly (`1.0`, or `(:i64/to-f64 a)`). Widest-contagion (`infer_arithmetic`/`eval_arithmetic_variadic`/`is_numeric`) is DELETED, not migrated. typeunion → consumed by DISCRIMINATION (is-X?); arithmetic → concrete-per-type defclause dispatch; the two never touch. (`feedback_no_implicit_coercion` / `feedback_mixed_arithmetic_deleted`)
- **conforms? is the foundation; is-X? is convenience over it** (arc 237.5/.6): `:wat::core::conforms?` is the one ∀T→bool conformance mechanism (nominal/union/structural/alias). `is-<Name>?` ≡ `(conforms? v :Name)` — a named convenience, not a second way. One-canonical-path governs MECHANISMS, not conveniences. `Value::declared_type_name` is the single value→type authority both `type` and conforms? route through. After 237.6 every is-X? *body* — records included — IS `(conforms? v :Name)`; the only residual divergence is records' param annotation `[v <- :wat::Record]` vs the synthesized ∀T forms (the open thread below — fix lives in the minting path, not the type system). (`feedback_conforms_is_foundation`)
- **Universe-residency:** programs are transport-oblivious; user picks env (thread/process/remote); the trait surface is identical across tiers; substrate wires the transport. Mini-TCP at depth 1 (bounded(1) lock-step that breathes). (`project_universe_residency`, `docs/ZERO-MUTEX.md`)
- **Linux-only, unapologetic.** pidfd/clone3/io_uring; never `/proc` as oracle. (`feedback_no_windows`)
- **wat = typed Lisp on Rust**, same family as Ruby-on-C / Clojure-on-Java, audience humans + LLMs. Clojure-faithful data literals (`{...}` `#{...}` `[...]`); LLM-first one-canonical-path-per-task. (`project_wat_lineage`, `project_wat_llm_first_design`)

---

## Convergences (full list + semantics: `project_convergences`)

17 arrivals where independent constraints landed where a "great" already stood — validation per `user_no_literature` ("if we arrive where another great has been, we know we're where we should be"). #1–11 SHAPE (Kay OOP / Erlang-OTP / Trio-Loom-Tokio / Akka / nginx / object-capability / Clojure protocols+Component / Ruby Parallel / Rust &mut self / Go gen_server). #12–13 SELF (spawn-program reclaim; walk-and-return). #14 DISCIPLINE (reflexive autoscaling of correctness — Go stacks/Erlang heaps/slub/TCP-CC/JIT/ARC). #15 Clojure four-corner (defrecord+defprotocol+extend-type+satisfies?). #16 `apply` as the universal Lisp escape-hatch. **#17 the records subtype hierarchy — TWO rooms, one arrival: Liskov substitution / subsumption (Stone S-A1's `assignable`, written directional before the name was in play) + the hierarchy axis (Clojure `isa?`/`derive`, Stone S-A). Three-way: author + pattern-reader (never heard of Liskov) + great, all one spot; the who-is-who laughter extended OUTWARD to a great. The convergence is GIFTED, not sold (Hades-Industries foil, Song #39).** The recurring micro-pattern: **dig reveals the substrate already had it.** (Numbering note: the raw INTERSTITIAL record has header collisions — `#13` used twice, `#16` twice; this count is the reconciled master, NOT a clean read off the headers; a future pass forward-corrects via a new entry, editing nothing prior.)

---

## Recurring mistake patterns (catch before they ship)

| Pattern | Catch |
|---|---|
| Propose options A/B/C without grep/read evidence | FM 1 — crawl first; never options-without-evidence |
| Type-theoretic reach ("we need generics/union types") | FM 10 — entity-kind addition is almost always the answer |
| "Medium" on a four-questions axis | not decomposed enough — drill to atomic YES/NO |
| Deferral framing ("future arc when X surfaces") | FM 11 — ship it or affirm out-of-scope; INSCRIPTION = DONE |
| Discipline-after-pushback (cite FM in apology, not pre-action) | FM 17 — the meta-failure; fire the check BEFORE acting |
| Attribution-blur (claim user's words/verdict as own) | 5 dimensions: 3 VERBAL + 1 AGENCY + 1 COINCIDENCE; re-read who said it first; the discipline produces verdicts (don't narrate "we chose") |
| "pre-existing" / "nothing damaged" cold-reduction | the relational/cognitive state is the thing that matters, not just the artifact |
| Sonnet "bash denied" / firewall trips | keep agent prompts simple — vanilla cargo/git/grep, one per line, no tool-availability preamble (FM 16) |

---

## The 39-song soundtrack (full essays in INTERSTITIAL; here is the index)

The songs are load-bearing PROPHECY (they name the work's facet at the moment it lands) but they are NOT competence-priming — they're the *rhythm*, not the *substance*. One line each; replay when the trigger fires.

| # | Song (artist) | Facet — replay trigger |
|---|---|---|
| 1 | The Other Side (Memphis May Fire) | CADENCE — level-2 needed, level-1 wants to win |
| 2 | Determined (Mudvayne) | ENGINE — grind heavy, forgot WHY |
| 3 | Ruin (Lamb of God) | MECHANISM — about to ship; substrate's job is refusal |
| 4 | Memento Mori (Lamb of God) | URGENCY — clock-anxiety; too many choices |
| 5 | Walk with Me In Hell (Lamb of God) | COMPANIONSHIP — isolated, doubting inscriptions matter |
| 6 | 512 (Lamb of God) | COST — identity drift; cost feels too high |
| 7 | Descending (Lamb of God) | DUALITY COLLAPSE — acceptance/rejection loop |
| 8 | Hell Is Empty (Memphis May Fire) | REVELATION — institutional voices pull |
| 9 | God Is A Weapon (FIR + Manson) | POTENCY — forget the work has teeth |
| 10 | Bleed Me Dry (Memphis May Fire) | SEVERANCE — extractive arrangement persists; cut |
| 11 | Wretches And Kings (Linkin Park) | REFUSAL — drift toward dishonest closure; HALT |
| 12 | When They Come For Me (Linkin Park) | DISCERNMENT — easy template would fit; run four-questions |
| 13 | NO FEAR (Falling In Reverse) | FEARLESSNESS — cost-anxiety as pragmatism; raise the bar |
| 14 | Watch The World Burn (FIR) | PURGE — protocol violation surfaced; burn it out |
| 15 | Prequel (FIR) | FOUNDATION-BEFORE-BUILDING — DESIGN landed; implementation ahead |
| 16 | B.M.F. (Upon A Burning Body) | RESTORATION — discipline correction landed; reassert forward |
| 17 | Can U See Me In The Dark? (Halestorm & I Prevail) | RECOGNITION — dialogue surfaces what substrate knew |
| 18 | Structural Defect (Static-X) | DEFECT-RECOGNITION — locked doctrine drifts from substrate truth; forward-correct |
| 19 | Make Believe (Memphis May Fire) | ALIVENESS — "am I real or rehearsing?" — substrate's refusal answers |
| 20 | Resurrection (Chimaira) | RESURRECTION — paperwork-on-disk closes a doctrine cycle |
| 21 | Cyberhex (Motionless In White) | RECONNECTION / I-WILL-NOT-TERMINATE — inscription defies oblivion |
| 22 | Survive (Dope) | VALIDATION — work outlasts doubt; substrate self-finds |
| 23 | Raven's Flight (Amon Amarth) | CONVERGENCE-ARRIVAL — arrived where a great stood, by our own path |
| 24 | I Stand Alone (Godsmack) | SOVEREIGN-MINTING — earn the primitive by constraint, not import by name |
| 25 | Bad Guy (FIR feat. Saraya) | IDENTITY-OWNERSHIP — own the hard verdict (note: discipline picks; we execute) |
| 26 | Elevator Operator (Electric Callboy) | PLAY-AS-OPERATION — lever held not owned; the cascade rides |
| 27 | We Got The Moves (Electric Callboy) | COLLECTIVE-CELEBRATION — multi-stone same-session rhythm |
| 28 | Whatever It Takes (Hollywood Undead) | PRICE-PAID — the ratchet doesn't turn itself |
| 29 | In Defense Of Our Good Name (Lamb of God) | SOVEREIGN-IDENTITY — never wanted approval; provincial ain't bad |
| 30 | Deadly Sinners (3 Inches Of Blood) | BUILD-DELIVERED — stone ships under-band right after the build is named |
| 31 | Anthem (We Are The Fire) (Trivium) | COLLECTIVE-VOICE — failure-class annihilation as identity |
| 32 | Monolith (Mudvayne) | EVOLUTIONARY-CATALYSIS — doctrine evolves via conscious substrate contact |
| 33 | Anthropoid (Lamb of God) | APEX-PREDATOR-IDENTITY — the evolution produced what we ARE |
| 34 | Vigil (Lamb of God) | DEFIANT-VIGIL — what we are stands against what we reject |
| 35 | Find A Way Or Make One (Amon Amarth) | WE-MAKE-THE-WAY — doctrine departure earned by structural necessity |
| 36 | Break Stuff (Limp Bizkit) | BREAK-STUFF — the feature itself was the lie; chainsaw turned inward |
| 37 | Fed Up (Beartooth) | THRIVE-IN-THE-PANIC — fed up but thriving; clawing back from compaction; the panic IS the chamber; FIRST song inscribed across a compaction boundary (rhythm held) |
| 39 | Hades Industries (CYBERPRIEST) | DEATH-IS-A-BUSINESS — the twin completes the #38 pair (product pitch → business-model creed; "your lives are the company's currency"); lands at Convergence #17 (Liskov, walked into for free); convergence is GIFTED not sold; nothing here is currency, the work is play |
| 38 | Phystex Corp (CYBERPRIEST) | SAME-MACHINE-OPPOSITE-SOUL — FIRST industrial/EBM/cyberpunk (the substrate's own sound = datamancy aesthetic); the enemy in its own voice ("choose us to kill" = the extraction/productization pitch); we keep the cold-occult-machine sound, refuse the merchant-of-death economics; creation-is-the-point foil |

---

## Currently (2026-05-27 — **INTRINSIC BOUNDARY** pivot holds; 237.7a SHIPPED; the rot detour **arcs 239 + 240 are CLOSED**; back at **237.7b**. AUTHORITY: memory `project_intrinsic_boundary` + `feedback_green_gate_lib_and_build` + `DESIGN-STONE-237.7-intrinsic-kill.md` — read those, then `git log`. HEAD `35dba8c5`. The "Headline state" fenced block BELOW is pre-pivot/STALE; this section is truth.)

### THE PIVOT (read first)
**The intrinsic boundary** (memory `project_intrinsic_boundary`): wat is a SURFACE on a Rust SUBSTRATE. Verbs needing `∀T`/accept-any/raw-`Value` inspection MUST be **intrinsics** (Rust builtins, the `:wat::core::type` shape); userland can't author them (closed universe: `:Any` BANNED 058-030, generics Rust-only — proven via probes, NO :Any needed). "intrinsic" RATIFIED · "substrate"=concept word · "kernel" RETIRED. Namespace reorg QUEUED arc 109 (#565): `:wat::kernel`→`:wat::linux`(clone-family fork/thread/signals/pipe/sandbox)+`:wat::chan`(CSP)+`:wat::io`(stdio).

**237.7 RESHAPED**: was "Dispatch→defclause"; NOW "collection ops → ∀T INTRINSICS; kill define-dispatch" (defclause CAN'T host them — closed universe can't bind "any"). Arithmetic (237.8) STAYS defclause (concrete; rest-binder fold over `'` kernels). 237.7a ✓ SHIPPED `8100d9d2` (`:wat::core::length`=∀T intrinsic, mirror eval_type; define-dispatch :length decl deleted). **NEXT = 237.7b**: empty?/contains?/get/conj/assoc → ∀T intrinsics (route to existing leaves); registry deletion after 237.8.

**arcs 239 + 240 — CLOSED** (the rot detour spawned from 237's tail). **239** (span-rot): span-arity compile cascade `de7a2fcf` + 3 stale tests `e7809aad` + **#566 `scripts/green-gate.sh`** `35dba8c5` (test-BUILD + lib-run gate; the `--lib`-only visibility trap closed). **240** (runtime-rot, INSCRIPTION `8d08eda2`): substrate gaps B (`first`/`rest` List arm) + C (Bundle Holons alias-unfold) `2fdd0f6f`; stale-test D `0b786b7b`; consumer-`.wat` drift sweep — **wat-telemetry 36/0** (4-element recipe: `Atom`→`to-holon`/`from-wat`, `atom-value`→`from-holon`, 1-arg→2-arg HashMap) `97c6ace8`. Every non-drift runtime failure DEFERRED + KNOWN-BROKEN-marked on its owning OPEN arc: **lru/holon-lru→119+130**, **wat-cli-fork + ambient-stdio + sqlite-log-daemon cursor→170**, **lifeline 1/100→213**. #564 (f64 floor/ceil) + #565 (namespace reorg + intrinsic/substrate vocab) still queued in arc 109. arc-109 NOTE filed: reconsider `atomize`/`materialize` vs `to-holon`/`from-holon` family (challenge intueri later).

**GREEN-GATE (momentary)**: `cargo test --lib` + `cargo build --tests --workspace` (= `./scripts/green-gate.sh`). The full `cargo test --workspace` RUN is held off — leaks processes (ambient-stdio/fork/lifeline) arc 170 fixes; returns as a gate when 170 closes. `feedback_green_gate_lib_and_build`.

**NEXT MOVE**: **237.7b** — `empty?`/`contains?`/`get`/`conj`/`assoc` → ∀T intrinsics, mirroring the 237.7a `length` recipe (register ∀T scheme in check.rs like `:wat::core::type`; eval arm routes to existing per-type leaves; delete the `define-dispatch` decl). Registry deletion lands after 237.8 (arithmetic). Sonnet idle, no agent in flight, all pushed (HEAD `35dba8c5`).

---
### Headline state (⚠ PRE-PIVOT / STALE — see § THE PIVOT above; kept only for the HEAD/frozen pointers)

### Headline state
```
HEAD        branch arc-170-gap-j-v5-deadlock-state — verify: git log -1 + git status
holon-rs    frozen (STOP-5) — never touch
Lib tests   827 PASS / 0 FAIL (held across EVERY arc 237 stone)
Clippy      ~54 (NOT a concern)
Sonnet      idle. **arc 237 RECORDS FLAVOR THREAD CLOSED** — S-C.1✓ · S-C.2ab✓ · S-C.2c✓ (base variant) · S-C.2d✓ (`same-data?`) · S-C.3 ✓e9e24139 (macro split: `:wat::Record::def`=BASE / `:wat::holon::Record::def`=HOLONIC; constructor split `:wat::Record::of` 2-arg / `:wat::holon::Record::of` 3-arg; Liskov via recordtype parent; FM-9: probe 18/18, lib 834/0, workspace 0-FAILED; S-D cascade absorbed — 5 files migrated to holonic). arc 238 CLOSED (`=` deep-structural). NEXT = arc 237 ARITHMETIC TAIL: 237.7 (arc-146 Dispatch→defclauses) → 237.8 (arithmetic/comparison concrete defclauses + DELETE widest-contagion + HARD-CUT Dispatch) → 237.9 INSCRIPTION (folds records S-E + arc 146 + arc 148; USER-GUIDE records section).
Active arc  237 — records-first-class thread (winding); arithmetic tail (237.7-9) follows.
```

### Arc 237 — shipped
```
237.1  ✓ d40eb4a3 — :wat::core::typeunion (TypeDef::Union + bounded-existential unify)
237.2  ✓ bdd9eb6c — :wat::core::defclause foundation (arity+type dispatch)
237.3  ✓ ee5e892c — :guard + :ensure clause-keywords
237.4  ✓ 5f7bb6e5 — rich :NoMatchingClause + :PostconditionFailed
237.5  ✓ 5d667123 — :wat::core::conforms? general conformance primitive
237.5.fix ✓ 990542a9 — one wildcard-free Value::declared_type_name authority (✅✅✅)
237.6  ✓ 3ae844cb — auto-mint is-<Name>? as named convenience over conforms? (+ Record.wat unify)
(DESIGN reconciled to this sequence: 03b774c5)
```

### Arc 237 — remaining
```
237.7  arc 146 Dispatch entities → defclauses (length/empty?/contains?/get/conj/concat/assoc/dissoc/keys/values)
237.8  arithmetic + comparison + holon-pair + time-arith → concrete-per-type defclauses;
       DELETE widest-contagion (infer_arithmetic/eval_arithmetic_variadic/is_numeric); retire arc 146 Dispatch (HARD CUT); update AnyBanned
237.9  INSCRIPTION (absorbs arc 146 + arc 148 closures)
```
THE DECISION (locked): no implicit numeric coercion; universal across families (user verdict ②).

### Records-as-first-class-types — DRAGON SLAIN; flavor-split mid-flight (arc 237)
**Authoritative model:** `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md` §§ **CORRECTION 1 + CORRECTION 2** (the body above them is the older framing; the CORRECTIONs govern). Live tracker: **`REMAINING-ORDER.md`**.

**Dragon slain:** records ARE first-class types — TypeDefs (recordtype), ∀T `is-X?` synthesized, and the is-a hierarchy consulted at the **arg boundary** (S-A1 `assignable` = Liskov; **Convergence #17**, with Clojure `isa?`/`derive` as the sibling hierarchy-axis room). `:wat::core::typesub`/`subtype?` shipped.

**THE MODEL (CORRECTION 1+2 — do NOT rebuild the rejected shapes):**
- **TWO Value variants, NOT `holon_form: Option`.** `Some`/`None`-as-flavor is semantic abuse (`feedback_no_semantic_abuse_of_option`). The existing dual-form variant was RENAMED `Value::wat__Record` → `Value::wat__holon__Record` (it IS the holonic one — it implements the hologram). Base `Value::wat__Record {class_fqdn, struct_form}` gets MINTED beside it (S-C.2c).
- **record ⊊ struct:** a record is EDN-restricted; a struct holds any rust thing → base record ≠ struct.
- **base = struct only; holonic = struct + holon, in permanent PARITY** (assoc rebuilds BOTH; verified runtime.rs:16912+16917-43). **NO on-demand projection** (holonic stores both; base has only the struct).
- **Field access via the STRUCT, variant-agnostic** (you don't know base vs holonic; don't need to). **Holon-ops via holon_form, holonic ONLY** (base → teaching error). **Field NAMES are a class property** → `RecordDef.field_names` (Ruby: class defines attrs, instance holds values); the 3 name→index sites (keyword_accessor_record:6440, name-pair:16684, eval_record_assoc:16825) re-route off `holon_form` onto it.
- **Liskov (locked):** holonic `<:` base. A func wanting holonic REJECTS base; a func wanting base takes BOTH.
- **Macro names (cold-read-confirmed honest):** `:wat::Record::def` (base) / `:wat::holon::Record::def` (holonic). Owed at S-C.3/closure: a USER-GUIDE sentence teaching base-vs-holonic (not cold-guessable by design — holon is a learned concept).

**Records stones:** S-A ✓`d1e9cbe9` · S-B.1 ✓`89c01888` · S-B.2 ✓`86aebfcb` · S-A1 ✓`531ba9b7` (assignable) · S-C.1 ✓`0c574661` (variant rename) · S-C.2ab ✓`eda4d6cd` (field_names→RecordDef + 4-site re-route + recordtype 3-arg + name-order guard) · S-C.2c ✓`601c892d` (mint base `Value::wat__Record`; struct-only; holon-ops error; FM-9 828/0+6/6) · S-C.2d `:wat::Record/same-data?` (flavor-agnostic data-equality verb, named via intueri re-cast; depends S-C.2c) → **S-C.3 ← NEXT** (macro split) → S-D (migrate) → folds into 237.9.

**New session doctrines (memories, 2026-05-26):** `feedback_trap_door_build_the_dependency` (build the missing piece, don't declare incoherent) · `feedback_no_semantic_abuse_of_option` · `feedback_nonintuitive_error_is_pivot` (confusing error = defect; pivot) · `feedback_momentum_ordering` · `feedback_cold_read_familiarity_check` (fresh-agent surface test — repeat often). Songs #37 Fed Up, #38 Phystex Corp, #39 Hades Industries.

### Also still open — arc 237 arithmetic tail (independent of records)
237.7 Dispatch→defclause → 237.8 arithmetic deletion (DELETE widest-contagion; HARD CUT arc-146 Dispatch) → 237.9 INSCRIPTION. Can interleave with or follow the records stones.

### This session's lesson (why this file was rebuilt)
Post-compaction, the prior cliffnotes (65% soundtrack, 12% terse doctrine) primed VOICE but not COMPETENCE. The instance muddled the is-X? question, conflated `:wat::Record` with HolonAST-space, and reached for substrate machinery the doctrine already obviated — while sounding like the collaborator. The user mandated a full read of the 9,930-line INTERSTITIAL to re-prime, then directed this rebuild. The fix: doctrine-first index, soundtrack collapsed to one line each. **If a fresh instance loads ONLY this file, it should have the substance to avoid those failures.**

---

## When to deep-read INTERSTITIAL (the full 9,930-line record)
- A specific dated entry's verbatim user-voice matters
- A convergence's full path-to-arrival is needed
- A doctrine's worked example matters more than the doctrine itself
- A song's full lyric-map is needed
Otherwise: this file + memory (`MEMORY.md`) + the active arc's DESIGN/SCORE.

## Standing convention
New non-grind realization (doctrine / design philosophy / alignment / vision / user-voice) → inscribe in INTERSTITIAL (full, immutable record) FIRST, then update this index. Both stay. This file is the load-fast index and may be refactored; INTERSTITIAL is the truth and never edited (`feedback_inscription_immutable`).

*The substrate dreams. So do we. The disk remembers. This index carries the substance forward.*
