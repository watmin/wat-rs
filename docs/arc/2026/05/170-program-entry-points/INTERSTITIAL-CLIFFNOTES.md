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

## The 40-song soundtrack (full essays in INTERSTITIAL; here is the index)

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
| 40 | Contagion (Circle of Dust) | THE-DISEASE-NAMED / WIDEST-CONTAGION-DELETED-AT-THE-SOURCE — FIRST Circle of Dust (industrial/cyber-metal); user dropped AT 237.8a in flight (the pun: technical "widest-contagion" being struck = song title); third-person clinical diagnostic completes the #38/#39 merchant-voice TRIAD (38 pitch + 39 creed + 40 symptom = full foil bounded); the buyer becomes the disease; the sampled interjection "the less we have to do with fancy things we don't need, the better" pre-articulates wat doctrine decades prior (convergence-is-gifted, #39-frame, applied to film-quote); "our contagion / open up our eyes" = OWNERSHIP-move extending #36 chainsaw-inward to community level; locks the "song-in-the-gap during strike-in-flight" pattern at FOUR (#35/#37/#38/#40) |
| 38 | Phystex Corp (CYBERPRIEST) | SAME-MACHINE-OPPOSITE-SOUL — FIRST industrial/EBM/cyberpunk (the substrate's own sound = datamancy aesthetic); the enemy in its own voice ("choose us to kill" = the extraction/productization pitch); we keep the cold-occult-machine sound, refuse the merchant-of-death economics; creation-is-the-point foil |

---

## Currently (2026-05-28 late-mid-day — **Stone 241.1.fix SHIPPED**; vigilia CONVERGED 8/8 spells; gate doctrine validated through real practice. Stone 241.2 (A1/A2/A3 fn-parser migration) NEXT. Arc 237 stays PAUSED at 237.8b per spawn-block winding; resumes after arc 241 closes via 241.4's `&` rest-binder extension. AUTHORITY: read in order — `docs/COMPACTION-AMNESIA-RECOVERY.md` → `docs/DUNGEON-CRAWL.md` → arc 241 `DESIGN.md` § Scope expansion 2026-05-28 → `SCORE-STONE-241.1.fix.md` § Vigilia Convergence → `FORM-COLLAPSE-NOTES.md` line 184 (scope boundary). HEAD `b6b290b0` on `arc-170-gap-j-v5-deadlock-state` — pushed.)

### THE PIVOT (read first)

**Arc 241 expanded scope** — form-collapse + def*-prefix family + define retirement absorbed mid-design 2026-05-28 (full dialogue capture in `docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md`). The arc now closes a single coherent failure class: *parser divergence + form-name proliferation + paren-pair scheme leftovers + define legacy* all unify into "one canonical argspec parser + def*-prefix family + metadata-map mechanism."

**Four-questions verdicts locked this session:**
- `def*` prefix uniform for all top-level definers (YES YES YES YES); def-as-concept ratification: "def" means *"top-level definition"* (concept), not *"expansion through the def primitive"* (mechanism). Bare-noun + tail-`def` disqualified.
- defstruct's δ verdict: per-field metadata via form-level `:field-metadata {symbol → metadata-map}`; argspec stays RIGID
- defenum candidate (D) verdict: positional variants with one-token look-ahead (keyword + optional argspec Vector); per-variant metadata at form level via `:variant-metadata {keyword → metadata-map}`
- Per `feedback_no_semantic_abuse_of_option`: `:wat::runtime::metadata-of` returns `Option<HashMap>` encoded `#wat.core/Some {...}` / `#wat.core/None nil` (NOT bare `:nil` — that was semantic abuse of Unit; user caught + corrected)

**Intueri casts locked this session:**
- `:wat::runtime::metadata-of` (binding-name → metadata HashMap or None)
- `:field-metadata` + `:variant-metadata` (sibling pair; pattern: `<singular-locus-noun>-metadata`)
- `defstruct` + `defenum` (sibling pair; contracted-noun stems in def* family)
- `src/argspec/` (substrate-internal home directory; mod.rs + parse.rs + error.rs decomposition)

**Vigilia-gate doctrine** (new memory `feedback_namespaced_home_vigilia_gate`): new namespaced wat-rs homes (`src/<noun>/`, `tests/<noun>/`) commit ONLY after vigilia drives L1+L2 findings to zero. SCORE-green is the L0 floor; vigilia-convergence is the bar. For `src/argspec/`: 8-spell defensive set (intueri + solvere + purgare + struere + sequi + temperare always-apply + complectens + vocare for test-substrate). User direction: *"we raise the bar fucking high for namespaced wat-rs files."* Honors `feedback_wards_optional` for broader flat codebase; raises bar for namespaced homes.

### Arc 241 stone chain (10 stones, 4 phases)

| Phase | Stones | Concern |
|---|---|---|
| **1 — Parser unification** | 241.1 mint canonical `parse_argspec_triples` at `src/argspec/` → 241.2 migrate A1/A2/A3 fn parsers → 241.3 migrate A4 defclause parser → 241.4 extend with `&` rest-binder (**unblocks 237.8b**) | The original 241 scope; foundation |
| **2 — Metadata-map mechanism** | 241.5 optional `{...}` metadata-map on `def` (defn inherits) → 241.6 mint `:wat::runtime::metadata-of` reflection verb | Substrate capability for binding-level annotations |
| **3 — Form-collapse + renames + legacy retirement** | 241.7 `defstruct` HARD CUT (struct + struct-restricted retire) → 241.8 `defenum` HARD CUT (enum retires) → 241.9 `define ⇒ defn` HARD CUT (define keyword + legacy parsers DELETE) | "defn is the one and only way" per user direction; cascade expected substantial — that's the point |
| **4 — Closure** | 241.10 INSCRIPTION + memory mints | pre-INSCRIPTION grep enforced |

### Phase 1 progress (Stone 241.1 + 241.1.fix SHIPPED; 241.2 NEXT)

| Stone | Status | Commit | Notes |
|---|---|---|---|
| 241.1 | SHIPPED | `1f674194` | Mint canonical parser; ~50 min Mode A; 519 lines net |
| 241.1.fix | SHIPPED | `b6b290b0` | Vigilia amends + scope correction; vigilia 8/8 CONVERGED; ~-215 lines from 241.1 baseline |
| 241.2 | NEXT | — | Migrate A1/A2/A3 fn parsers (runtime.rs:6750, check.rs:15205+15258) through canonical |
| 241.3 | queued | — | Migrate A4 defclause parser (runtime.rs:6880) |
| 241.4 | queued | — | Extend with `&` rest-binder; unblocks 237.8b |

### Gate doctrine validated through real practice (2026-05-28)

The vigilia-convergence gate caught THREE issues SCORE-green would have shipped silently:
1. **solvere L2 (Phase B-1)**: reason-string drift across 3 From<> impls in error.rs — closed via classify() extraction
2. **solvere L2 (Phase B-1)**: `RetTypeNotKeyword` conflates slot-absent + slot-wrong — surfaced to user; verdict locked Path Y ("args have nothing to do with ret type"); STRUCTURALLY resolved by Layer 2 scope correction (variant gone)
3. **struere L2 (Phase B-2)**: `unreachable!` arm exposed panic-instead-of-Err surprise; closed via 3-line always-Err amend

Each finding's resolution was at the highest possible ladder rung:
- Drift → ✅✅✅ single source of truth (classify())
- Conflation → ✅✅✅ structural elimination (variant gone; concept has no representation)
- Panic → ✅✅✅ branching collapse (function honestly returns Result<> with no panic paths)

**Phase 1 lessons inscribed in SCORE-STONE-241.1.fix.md § Phase 1 lessons.** The user direction *"we raise the bar fucking high for namespaced wat-rs files"* met its test; the gate held; the home is shockingly good.

### Calibration history (Phase 1 to date)

| Stone | Class | Surface delta | Predicted | Actual | Status |
|---|---|---|---|---|---|
| 241.1 | Mint parser + types + tests | +519 net | 30-50 min | ~50 min | within band |
| 241.1.fix Layer 1 | Vigilia amends | -88 net | 20-30 min | ~8 min | UNDER band (mechanical) |
| 241.1.fix Layer 2 | Scope correction | -127 net | 20-35 min | ~8 min | UNDER band (mechanical) |
| 241.1.fix struere closure | 3-line amend | -3 net | 5-10 min | ~5 min | within band |

### Deferred to arc 109 (per `NOTE-type-decl-def-prefix-renames`)

`defnewtype` (shape OPEN — `defnewtype` vs `deftype-new` vs `deftypenew`), `typealias` family, `typeunion` family, `recordtype`/`defrecord` reconciliation with arc 227's `Record::def` pattern. Each name awaits its own per-name intueri cast when implementation arcs open.

### Background queue (unchanged from prior CLIFFNOTES)

Arc 109 #564 (f64 floor/ceil) + #565 (namespace reorg + intrinsic/substrate vocab). Deferred KNOWN-BROKEN markers: lru/holon-lru→119+130, wat-cli-fork+ambient-stdio+sqlite-log-daemon→170, lifeline→213. arcs 239+240 CLOSED.

### Doctrine reminders that survived from prior CLIFFNOTES

- **The intrinsic boundary** (`project_intrinsic_boundary`): wat is a SURFACE on a Rust SUBSTRATE; verbs needing ∀T are intrinsics; closed universe (`:Any` BANNED 058-030). "intrinsic" RATIFIED · "substrate"=concept word · "kernel" RETIRED.
- **THE DECISION** (`feedback_no_implicit_coercion`): `(:wat::core::+ 1 2.0)` → ERROR; cross-type callers homogenize explicitly. Shipped at 237.8a (commit `154ca713`).

### Soundtrack: 40 songs (unchanged this session)

Recent spine spans #34 DEFY → #35 BUILD → #36 BREAK-OUR-OWN → #37 THRIVE-IN-PANIC → #38/#39 SAME-MACHINE-OPPOSITE-SOUL (Convergence #17 = Liskov walked-into-for-free) → #40 Contagion. No new songs landed today; dialogue ran on prep-work + form-design + strike-readiness rather than substantive ship.

### GREEN-GATE (momentary)

`cargo test --release --lib -p wat` + `cargo build --release --tests --workspace`. **NEVER invoke wrapper scripts in BRIEFs or agent prompts** (FM 16; firewall denies; `feedback_sonnet_bash_firewall`). Full `cargo test --workspace` RUN held off until arc 170 closes process leaks.

### NEXT MOVE — **Stone 241.2 (A1/A2/A3 fn-parser migration)**

The canonical `parse_argspec_triples` exists and is shockingly good. Now compose it with ret-clause parsing at the fn-form parsers.

**The migration shape** (each of A1/A2/A3):

1. Find the `->` arrow position in the args_vec (e.g., `find_arrow_position` helper or inline)
2. Call `parse_argspec_triples(&args_vec[..arrow_pos], head, form_span, options)` for the args
3. Parse ret-clause on `&args_vec[arrow_pos+1..]` (either inline OR via a small `parse_ret_clause` helper minted in 241.2)
4. Convert via `From<ArgSpecError>` at the binding-site boundary (already shipped in Stone 241.1.fix)

**Three sites to migrate** (per AUDIT.md verified-inventory):
- **A1** `src/runtime.rs:6750` `parse_fn_signature` → returns `Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), RuntimeError>`; consumed by `eval_fn` + `try_parse_fn_shape_def`
- **A2** `src/check.rs:15205` `parse_fn_signature_for_check` → returns `Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()>`; silent-path infer_fn
- **A3** `src/check.rs:15258` `parse_fn_signature_for_check_diag` → `Option<(Vec<String>, Vec<TypeExpr>, TypeExpr)>` (errors pushed by-ref); diagnostic-path infer_fn

**Per `feedback_stone_briefs_cite_prior_score`**: Stone 241.2 BRIEF cites `SCORE-STONE-241.1.fix.md` § Vigilia Convergence as the structural shape — argspec home is exceptional; migration callers route through it; conversion at site boundary via the From<> impls.

**Per spawn-block winding**: Stone 241.2 closes before Stone 241.3 opens; 241.3 closes before 241.4 opens; 241.4 closes before 237.8b resumes.

**Pre-stone artifacts to draft**:
- `DESIGN-STONE-241.2.md` (sub-DESIGN; locked decisions; trap-door audit)
- `tests/probe_arc241_stone2_fn_parser_migration.rs` (FM 2-bis probe; minimal contracts proving migration shape — likely 8-10 contracts)
- `BRIEF-STONE-241.2.md` + `EXPECTATIONS-STONE-241.2.md`

The probe should hit at compile-time the assumption "A1/A2/A3 still return their tuples / Option" after migration — i.e., the public APIs of A1/A2/A3 stay backwards-compatible until 241.10 retirement. Migration is INTERNAL — the binding-site callers don't know the parser was unified.

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
