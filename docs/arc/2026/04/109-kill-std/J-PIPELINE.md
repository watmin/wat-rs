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
- 1d: mint `:wat::core::unit`; retire `:()` as a type annotation —
  shipped 2026-05-01. `BareLegacyUnitType` variant + Tuple-arm
  extension to slice 1c's walker; 72 files swept across four
  tiers; substrate gap fix in `parse_type_inner` (canonicalize
  `:wat::core::unit` → `Tuple(vec![])` when canonicalize=true so
  raw `==` validators accept the FQDN form); cargo test workspace
  1476/0. See `SLICE-1D.md`. Rename to `Unit` queued as follow-up.
- 1e: FQDN four-of-five parametric type heads (Option/Result/
  HashMap/HashSet) — shipped 2026-05-01.
  `BareLegacyContainerHead` variant + Parametric-head walker arm
  (third TypeExpr shape covered: Path → 1c, Tuple → 1d,
  Parametric.head → 1e); four typealiases minted; 65 files swept
  across four tiers; ~365 rename sites; zero substrate-gap fixes
  required. cargo test workspace 1476/0. See `SLICE-1E.md`.
  Vec<T> deferred to slice 1f (couples with § D verb rename).
- 1f: Vec<T> renames to Vector + vec verb shares the type name —
  shipped 2026-05-01. BARE_CONTAINER_HEADS extended with
  ("Vec", "wat::core::Vector"); Pattern 2 poison on
  `:wat::core::vec` callee with `arc_109_vec_verb_migration_hint`;
  `:wat::core::Vector` dispatch arm added; typealias minted; 73
  files swept across four tiers; **547 bare-Vec sites + 225 vec-
  verb sites = 772 rename sites**. One substrate-gap fix in
  `src/lower.rs::lower_bundle` (Bundle lowering needed both Vec
  and Vector heads). cargo test workspace 1476/0. See
  `SLICE-1F.md`. First slice to bundle Pattern 3 (type) +
  Pattern 2 (verb) cleanly; proves the bundle reusable.
- 1g: list retires (was duplicate of vec; redirects to Vector) +
  tuple → Tuple — shipped 2026-05-01. Pattern 2 only (two more
  poison arms + two more hint helpers + new `Tuple` dispatch
  arm); zero new walker logic. 22 files swept across three
  tiers (tier 4 examples had zero hits); **74 tuple sites + 12
  list sites = 86 rename sites**. Zero substrate-gap fixes.
  cargo test workspace 1476/0. See `SLICE-1G.md`. Pattern 2
  mechanism now well-rehearsed (slices 1f + 1g); reusable for
  any future verb retirement.
- 1h: Option variants FQDN — `Some` → `:wat::core::Some` and
  `:None` → `:wat::core::None` — shipped 2026-05-01. First slice
  to apply Pattern 2 to AST-grammar exceptions (bare Symbol +
  bare Keyword at callable head) rather than substrate-registered
  verbs. 69 files swept across four tiers; **~542 rename sites**
  (249 Some + 293 :None). Two substrate-gap fixes during sweep:
  (1) pattern_coverage / check_subpattern user-enum keyword
  hijack of FQDN builtins; (2) is_match_canonical bare-only
  recognition. cargo test workspace 1476/0. See `SLICE-1H.md`.
  Render_value FQDN flip deferred as task #189 follow-up.
- 1i: Result variants FQDN — `Ok` → `:wat::core::Ok` and `Err` →
  `:wat::core::Err` — shipped 2026-05-01. Mechanical extension
  of slice 1h: both Ok and Err are Symbol-headed-with-payload
  (same shape as Some), so substrate work was just two more
  Pattern 2 poisons + two hint helpers. 39 files swept across
  four tiers; **~337 rename sites** (280 patterns + 57
  constructors). Two substrate-gap fixes mirroring 1h's pattern:
  MatchShape FQDN keyword recognition + try_match_pattern
  FQDN keyword arms. cargo test workspace 1476/0. See
  `SLICE-1I.md`. **§ C structurally complete**: substrate has
  zero bare-symbol-at-callable-head exceptions; the "callable
  heads must be FQDN keywords" rule is universal.
- 1j: § D' Option/Result method forms — Type/verb shape — shipped
  2026-05-01. Three Pattern 2 retirements (`:wat::core::try` →
  `:wat::core::Result/try`; `option::expect` → `Option/expect`;
  `result::expect` → `Result/expect`) PLUS one brand-new
  substrate addition: `:wat::core::Option/try` (mirrors
  Result/try; propagates `:None` via new
  `RuntimeError::OptionPropagate` variant + apply_function
  trampoline arm). 20 files swept (5 stdlib + 15 consumer);
  **197 rename sites total** (15 stdlib + 182 consumer); zero
  substrate-gap fixes; cargo test workspace 1476/0
  (commits ebeb6be → 853fbdc; SLICE-1J.md). **§ D' structurally
  complete**: Option<T> and Result<T,E> have symmetric four-verb
  branching (Option/try, Option/expect, Result/try,
  Result/expect).
- § K (DOCTRINE captured): "/ requires a real Type" — substrate-
  wide rule that the `/` separator earns its place only when the
  LHS is a real Type (struct / parametric kind / substrate
  primitive). Identifies four grouping-noun cleanups (K.console,
  K.telemetry, K.lru, K.holon-lru) for future slices. Includes
  full mental model: Type/method is UFCS, not OOP; stateful
  instances come in pure-value (Stats/Bytes) and handle
  (HandlePool/Sender) flavors; encapsulation is namespace-driven.
  Doctrine-only; no code changes yet (commits bf51fa2 + fef399c).
- 9d: stream namespace promotion + file move — shipped 2026-05-01.
  Pattern 3 (third namespace-prefix application).
  `:wat::std::stream::*` → `:wat::stream::*` (286 rename sites:
  101 in stream.wat itself + 185 across 10 consumer files). File
  moved `wat/std/stream.wat` → `wat/stream.wat` per § G's
  filesystem-path mirror rule; `src/stdlib.rs` `include_str!`
  path updated. New `CheckError::BareLegacyStreamPath` variant +
  `validate_legacy_stream_path` walker (pure keyword-prefix
  detection — no parsed-TypeExpr inspection since this is a
  pure namespace move). Zero substrate-gap fixes; cargo test
  workspace 1476/0 (commits `7837262` substrate + `d22bc4f`
  consumer sweep; SLICE-9D.md). Simplest substrate work in arc
  109's slice catalog so far — no special-case dispatchers, no
  canonicalization map extension; pure file move + sed + walker.
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

### § K cleanups (driven by the "/ requires a real Type" doctrine)

Doctrine captured 2026-05-01 as INVENTORY § K. Four cleanups,
each its own slice; each is sonnet-delegatable via Pattern 2 verb
retirement (the OLD `Type/verb` heads emit synthetic TypeMismatch
poisoning to the NEW namespace-level forms).

4. **Slice K.console** — `:wat::std::service::Console::*` →
   `:wat::console::*` (typealiases at namespace level; verbs lose
   `Console/` prefix and become bare `:wat::console::spawn`,
   `:wat::console::loop`, `:wat::console::out`, etc.). Subsumes
   the original § 9e plan (file-path move
   `wat/std/service/Console.wat` → `wat/console.wat`).

5. **Slice K.telemetry** — `:wat::telemetry::Service::*` (the
   grouping noun's pseudo-children) flattens to
   `:wat::telemetry::*`. `Service/spawn`, `/loop`, `/tick`,
   `/extend`, `/maybe`, `/drain`, `/run`, `/bump`, `/batch`,
   `/null`, `/pair`, `/ack` become bare `:wat::telemetry::<verb>`.
   Real types (Stats, MetricsCadence) keep their /methods because
   they ARE structs.

6. **Slice K.lru** — audit `:wat::lru::CacheService`. Real struct
   → keeps /methods. Grouping noun → flatten verbs.

7. **Slice K.holon-lru** — same audit + treatment for
   `:wat::holon::lru::HologramCacheService`.

### Independent sweeps (do NOT depend on § J or § K)

These are parallel-shippable; sonnet-delegatable with the
substrate's diagnostic stream as the brief (Pattern 3 from
SUBSTRATE-AS-TEACHER):

4. **9d** — `:wat::std::stream::*` → `:wat::stream::*` (the
   stream stdlib's namespace claims promotion).

5. **9e** — `:wat::std::service::Console::*` →
   `:wat::console::Console::*` (Console gets its own namespace
   path matching its substrate-claim shape).

6. **9f–9i** — file-path moves for already-honest-symbol files
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

- **`Queue*` → `Channel` rename across the kernel-channel
  family.** Surfaced during slice 1d via /gaze. The current
  `:wat::kernel::QueueSender<T>` / `QueueReceiver<T>` /
  `QueuePair<T>` aliases mumble in two ways: (1) "Queue" leaks
  the implementation (crossbeam's data-structure name); (2) the
  prose has already drifted — Console.wat comments say "pipe",
  service-template.wat comments say "channel" — three different
  words for one concept and the type spelling matches none.
  /gaze flagged Level 2 mumble + Level 1 lie at the prose/type
  boundary.

  Recommended canonical form (drops the Queue prefix entirely;
  matches Go / Rust crossbeam / Clojure core.async / CSP / Erlang
  vocabulary):
  ```
  :wat::kernel::Channel<T>      ;; the pair (Sender<T>, Receiver<T>)
  :wat::kernel::Sender<T>       ;; input end
  :wat::kernel::Receiver<T>     ;; output end
  ```
  Verb companions: `make-bounded-queue` → `make-bounded-channel`
  (and the unbounded sibling). File rename: `wat/kernel/queue.wat`
  → `wat/kernel/channel.wat`.

  Side benefit: the shorter half-names solve the "half the
  codebase spells `:rust::crossbeam_channel::*` because
  `QueueSender` is too long" leak the user noticed. Today's split
  is 72 raw rust + 50 raw rust + 66 + 66 wat-prefixed; the rename
  collapses both to one canonical name short enough to actually
  use.

  Cost: 243 identifier hits + ~20-30 verb-rename sites + file
  rename + doc pass (USER-GUIDE / CONVENTIONS / WAT-CHEATSHEET /
  SERVICE-PROGRAMS / arc-117 docs). Bigger than the unit→Unit
  rename; same Pattern 3 mechanism (`BareLegacyQueueName` variant
  + walker + four-tier sweep). Could ride as slice 1e or its own
  arc; the substrate-as-teacher rehearsal makes either path
  cheap.

- **`:wat::core::let*` → `:wat::core::let`.** Surfaced during
  slice 1e (post-/gaze conversation). The asterisk is Scheme/Lisp
  tradition for "sequential bindings" *as opposed to a parallel
  `let`*. Wat has only the sequential form, no parallel sibling
  is planned ("i don't think i'll add any more things to core at
  this point"). With no companion to differ from, the `*` is
  vestigial — distinction work without anything to distinguish.

  Through the four questions: `let` is what every non-Scheme
  reader expects (obvious); one name, one form (simple); the `*`
  implies an alternative that doesn't exist (honest); aligns with
  mainstream language vocabulary (good UX).

  Mechanism: not Pattern 3 (it's a callee rename, not a
  TypeExpr-shape detection). Closer to arc 114's poison pattern
  — synthetic `CheckError::TypeMismatch` in the `:wat::core::let*`
  dispatcher with `expected: ":wat::core::let"`, `got:
  ":wat::core::let*"`; an `arc_109_let_migration_hint` in
  `collect_hints` detects the shape pair and prints the rename
  brief.

  Cost: ~10-20 Rust dispatch sites in `src/check.rs` +
  `src/runtime.rs` (recognizer + freeze pass) + several hundred
  wat-source uses. Sweep is mechanical; same four-tier shape via
  sonnet brief.

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

[done]  arc 109 slice 1d — BareLegacyUnitType variant + Tuple-arm
                  walker extension + four-tier sweep; 72 files;
                  substrate gap fix in parse_type_inner;
                  cargo test workspace 1476/0
                  (commits edd6687 → 279277f; SLICE-1D.md)

[done]  arc 109 slice 1e — BareLegacyContainerHead variant +
                  Parametric-head walker arm + 4 typealiases +
                  four-tier sweep; 65 files; ~365 rename sites;
                  zero substrate-gap fixes; cargo test workspace
                  1476/0 (commits f8a82be → 5a96cb0; SLICE-1E.md)

[done]  arc 109 slice 1f — Vec<T> rename + move + vec verb
                  retirement (Pattern 3 + Pattern 2 bundled);
                  73 files; 772 rename sites total (547 type
                  + 225 verb); 1 substrate-gap fix in lower.rs;
                  cargo test workspace 1476/0
                  (commits 61c008d → ad4c54a; SLICE-1F.md)

[done]  arc 109 slice 1g — list retire + tuple → Tuple
                  (Pattern 2 only; two more poison arms +
                  hint helpers + Tuple dispatch arm); 22 files
                  swept; 86 rename sites (74 tuple + 12 list);
                  zero substrate-gap fixes; cargo test workspace
                  1476/0 (commits 1dea484 → e59a077; SLICE-1G.md)

[done]  arc 109 slice 1h — Option variants FQDN
                  (Some → :wat::core::Some; :None → :wat::core::None);
                  first slice to apply Pattern 2 to AST-grammar
                  exceptions (bare Symbol + bare Keyword at callable
                  head). 69 files swept across four tiers; ~542
                  rename sites (249 Some + 293 :None); 2 substrate-
                  gap fixes (pattern_coverage user-enum hijack +
                  is_match_canonical bare-only); cargo test
                  workspace 1476/0 (SLICE-1H.md)

[done]  arc 109 slice 1i — Result variants FQDN
                  (Ok → :wat::core::Ok; Err → :wat::core::Err);
                  mechanical extension of 1h (Ok/Err same shape as
                  Some — Symbol-headed-with-payload). 39 files
                  swept; ~337 rename sites (280 patterns + 57
                  constructors); 2 substrate-gap fixes mirroring
                  1h (MatchShape FQDN + try_match_pattern FQDN);
                  cargo test workspace 1476/0
                  (commits 35e44dc → c7ab499; SLICE-1I.md).
                  § C structurally complete — bare-symbol-at-
                  callable-head exception universally closed.

[done]  arc 109 slice 1j — § D' Option/Result method forms
                  (Type/verb shape; Pattern 2 retirement of three
                  verbs + brand-new Option/try mint). 20 files
                  swept (5 stdlib + 15 consumer); 197 rename sites
                  total (15 stdlib + 182 consumer); 0 substrate-
                  gap fixes; cargo test workspace 1476/0
                  (commits ebeb6be → 853fbdc; SLICE-1J.md). § D'
                  structurally complete — Option<T> and
                  Result<T,E> have symmetric four-verb branching
                  (Option/try, Option/expect, Result/try,
                  Result/expect). Brand-new
                  RuntimeError::OptionPropagate variant + apply_
                  function trampoline arm + eval_option_try +
                  infer_option_try.

[done]  arc 109 § K — DOCTRINE captured (commits bf51fa2 +
                  fef399c). New INVENTORY section codifying the
                  rule "/ requires a real Type"; identifies four
                  grouping-noun cleanups (K.console, K.telemetry,
                  K.lru, K.holon-lru) for future slices. Includes
                  full mental model: Type/method is UFCS, not OOP;
                  stateful instances come in pure-value and
                  handle flavors; encapsulation is namespace-
                  driven. Doctrine-only; no code changes yet.

[done]  arc 109 slice 9d — :wat::std::stream::* → :wat::stream::*
                  + file path move (wat/std/stream.wat →
                  wat/stream.wat). Pattern 3 (third namespace-
                  prefix application after 1c/1d/1e). 11 files
                  swept (1 stdlib + 10 consumer); 286 rename
                  sites total (101 stream.wat self-refs + 185
                  consumer); zero substrate-gap fixes. Walker
                  shape simpler than 1c/1d/1e: pure keyword-
                  prefix detection (no parsed-TypeExpr inspection
                  needed since this is a namespace move, not a
                  type-shape change). cargo test workspace
                  1476/0 (commits 7837262 + d22bc4f; SLICE-9D.md).
                  Stream stdlib's path now mirrors its shipped
                  FQDN per § G's filesystem-path rule.

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
        § 9f-9i (file-path moves for already-honest-symbol files)

        § K.console (Console grouping noun → :wat::console::*
                     flatten; SUBSUMES the original § 9e plan
                     under the / requires a real Type doctrine —
                     see INVENTORY § K)
        § K.telemetry (Service grouping noun → :wat::telemetry::*
                       flatten; real types Stats / MetricsCadence
                       keep their /methods)
        § K.lru (audit CacheService; flatten if grouping)
        § K.holon-lru (audit HologramCacheService; flatten if
                       grouping)

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
