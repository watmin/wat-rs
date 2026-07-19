# DESIGN — wat never hides a failure (the IPC death/error path)

> **THE LAW (builder, 2026-07-17):** *"i want wat to never hide failures ever again … this masking of
> failure is actively hostile against wat's intent."* Every place on the peer/service death path that
> discards an error, collapses distinct failures into one mute value, writes a reason to a closed pipe,
> or kills a whole service over one bad message is the SAME class — **failure-masking** — and this arc
> pulls the class out by the root. We own wat; the arc-294 "crash reasons are administrative" ruling does
> NOT shelter a masking behavior — we change our minds when the mask keeps blinding us.

## STATUS — 2026-07-19+ (ALL THREE pieces LANDED — the LAW is CLOSED AIRTIGHT for masking; the STOP-2 crash-broadcast-to-clients is a separate future capability, not a masking)

**All three of the law's pieces are real, verified, pushed — no failure that EXISTS is hidden:**

- **Alive-service-rejects-you — Mechanism A** (`66d6aed7`): `poll'` returns `ServiceEvent::Malformed{idx,
  cause}` instead of raising; the serve loop replies `Reply::Failed{cause}` and **keeps serving** (no
  DoS); `recv'` surfaces `Reply::Failed` as a catchable raise carrying the reason. `Reply::Failed[cause
  <- Failure]` is the reserved **protocol-tier** floor on every synthesized `<S>::Reply`
  (`types.rs synthesize_surface_protocol`) — the 293 outcome-enum model completed (op-tier failure =
  `<Op>Response::{Transient,Fatal}`; the "couldn't resolve to any op" floor was missing). — table sites
  #9, #10 + the protocol-tier gap.
- **eprintln is terminal** (`dc286d7a`): `eprintln`/`epprintln` emit the value then `panic_any` →
  structured-exit (uncatchable, cross-loci) — the dying declaration you designed; `feedback_eprintln_is_terminal`
  closed. The `:Lost` serve-loop arm now stands on it — an abnormal transport break lets-it-crash (OTP),
  cause on stderr before exit. Return type kept `-> :wat::core::nil` (`:()` is the redundant unit
  spelling, being retired). *(This RESOLVES the earlier "`:Lost` eprintln" judgment call — it is the
  intended crash-with-message, not a benign write.)*
- **`call_beside` idiom** (`dc286d7a`, intueri-ratified): the lint-clean "run the co-located fixture's
  entry fn" helper (`src/freeze.rs`, beside `startup_beside`); first consumers wired → `no_inlined_wat`
  352 → **351**.

Verified by own full re-run: 4170 passed / 1 failed = the standing `no_inlined_wat` at 351, zero new;
`no_loose_string_assert` PASS; `eprintln_terminal` + `dead_child_speaks` + `crash_surfaces` all green.

**One judgment call still nominally open** (clarified as a non-decision): surfacing lives in `recv'`,
not a client-method match arm — a wat `assertion-failed!` in a method is `panic_any` (uncatchable);
`recv'` is the one catchable, uniform surfacing point (step 3 / "room 8"). The client gets a catchable
error, OTP-consistent. Nothing to decide.

**★ THIRD PIECE — the transport-tier twin — LANDED (`3f73f400`).** `RecvError` gained `Failed(String)`
(`comms/mod.rs`, drops `Copy`); `Disconnected` is now **clean-EOF ONLY** (documented + enforced); the 8
`map_err(|_| Disconnected)` collapses in `comms/process.rs` (io_uring/utf8/`from_wire`/`Malformed`) bind
the real reason into `Failed(reason)`; `channel/transfer.rs` → `RecvOutcome::DecodeError`; `spawn.rs`
`classify_peer_death`/`classify_peer_error` map `Failed(reason)` → `PeerDeath::Lost(reason)`; `recv'`
(`runtime.rs`, socket `recv_wire` + bare-`Peer` arms) thread the reason into the raise. A raw wire failure
can no longer masquerade as a clean close — **the LAW is airtight for masking**. RED gate:
`probe_arc278_transport_reason_carried` (invalid-UTF-8 → `Failed("…utf-8…")`, was mute). Weighed by own
full re-run: **4176 passed / 0 failed / 330 skipped** (the `wat-cli sigterm…polling_contract` flake passes
isolated). *(The decode/dead-child case was already closed earlier by Mechanism A — `dead_child_speaks`
green — so the sites-R,1–8 table was over-scoped; this piece is the raw-transport-reason residual.)*

**STOP-2 — a SEPARATE FUTURE CAPABILITY, not a masking (does NOT block the law):** a genuinely-crashing
process unit's reason reaches its **owner** (the `/start` `Handle`, via `PeerRecvError::Crashed` at
`runtime.rs:26159`) but NOT a separately **`connect'`-ed client** peer (`kernel/peer.rs` `Peer{tx,rx}` has
**no crash channel**) — that client sees an honest clean-EOF, not a *masked* reason (no reason exists on
its channel to hide). Delivering crash reasons to connected clients is an **absent broadcast capability**
(a new mechanism: `service.wat` codegen + the panic boundary broadcasting the crash payload to every live
client before exit) — its own arc when it's wanted. Tracked, `#[ignore]`'d:
`probe_arc278_process_crash_reason_carried.{rs,wat}` (finding in its module doc).

## ═══ SESSION-END CURARE (this session — R44 crusade-commit → the dynamic-EDN-decode campaign → Stone A.0) ═══

**READ THIS BLOCK FIRST, then `git status`. HEAD = `c9bfa8fd` (Stone A.0).** The no-hidden-failures LAW is DONE
(below, unchanged). This session: full recolligere bootstrap (278 R1–R43 read) → committed the edn-crusade + 294.f
+ R44 (`98499f48`, pushed) → designed the **dynamic-EDN-decode campaign** with the builder → **committed Stone A.0**
(`c9bfa8fd`). Pushed through `98499f48`; `c9bfa8fd` + this curare are the NEW push — verify against the disk.

**THE CURRENT WORK — the dynamic-EDN-decode campaign.** Full spec: **`DESIGN-dynamic-edn-decode-and-opaque-sink.md`**
(read it); tracked in tasks #1–#4. Origin: the telemetry service must accept logs from ARBITRARY callers AND let
arbitrary callers process them — blocks the chaos engine dogfooding telemetry (`probe_arc278_journal_logs_on_process`
is `#[ignore]`'d: a forked `journal'` child faults `UnknownTag` on a user payload). Decomposed (builder rulings):
- **A.0 ✓ COMMITTED (`c9bfa8fd`)** — uniform variant encoding (the floor): every enum variant vector-bodied
  (`None→[]`, `Some(v)→[v]`, user unit→`[]`); arc-298.1 direct-body RETIRED; `nil` = the unit value only.
  Body-shape is now a total discriminator (map=record, vector=variant, nil=unit). Weighed green by own re-run
  (cargo 4204 / 1 known sigterm flake [isolated-passes] / 330 skipped; clj 39/0; diff bracket-only).
- **Stone A — NEXT (task #1): `read-foreign` → `ForeignRecord` / `ForeignVariant`** (names intueri-cast + ratified).
  `edn::read` gains an opt-in DATA MODE: unknown tag → a self-describing dynamic value by body shape
  (map→`ForeignRecord {class, name-keyed fields}`; vector→`ForeignVariant {enum-class, variant, positional}`;
  recursive; one shape per fully-qualified tag, contradiction=exception). STRICT `read` stays default (errors on
  unknown — holds the no-hidden-failures floor). Consumer-side; the sink never decodes. Clojure's `tagged-literal`,
  aggregate-aware. Builds on A.0's clean dispatch.
- **Stone B (task #2): opaque telemetry sink** — `Log.message` → opaque EDN-text `String`; producer `edn::write`s at
  the call site; sink stores/returns verbatim, NEVER decodes (no DoS — `[[feedback_sink_is_opaque_store_consumer_decodes]]`).
  Un-`#[ignore]` `probe_arc278_journal_logs_on_process`; rete self-measurement unblocked (own types both ends).
- **Stone C (task #3): annihilate + fold + de-prime** (COMPONENDO DELEO; gated on A+B). Fold the 3 legacy crates
  (`wat-telemetry`, `wat-telemetry-sqlite`, `wat-sqlite`) into core; kill `Tagged`/`NoTag` (`wat/edn.wat:32-33`,
  holon-coupled) + `write-notag`/auto-dispatch; de-prime the family (`:telemetry'::`→`:telemetry::` = true reclaim,
  needs legacy gone; `sqlite'`/`mem-store'`/`sqlite-store'`→bare = GROUND what each prime guards first + whole-tree
  consumer grep per crate). Delete `probe_arc278_process_crash_reason_carried` (STOP-2 non-goal — crash reasons
  admin-only, `[[feedback_ask_who_already_receives_it_before_building_delivery]]`).

**FAR-SIDE FIRST MOVE: draw + strike Stone A (`read-foreign`)** on A.0's clean floor — RED gate (a foreign record
CONTAINING a foreign variant field round-trips through `read-foreign`; strict `read` still errors) → brief →
delegate → weigh by own re-run. Then B, then C. **THEN the arc's TARGET: the CHAOS ENGINE (R25 `MACHINA CHAOS
DOMAT`)** — the streaming rete `Session`-as-state `defservice`, dogfooding the solid telemetry.

**Realizations this session:** R44 `FACTVM EST, ITERVM VICIMVS` (Cowboys — the crusade deed done) · R45 `LVCEM
TENEBRASQVE FERO` (Onyx — the substrate bears known+unknown; the design fought from my own darkness) · R46 `IN LVCE
PVRGATI` (Purified — the light won through, the floor wiped, the duet sanctified). New memory:
`feedback_sink_is_opaque_store_consumer_decodes`.

**Low-stakes tracked (non-blocking):** clj `validate` dual-matches short+FQDN because `shared.wat`'s `:Keyword`
stayed short while neighbors went FQDN — a fixture-consistency nicety (FQDN-ify `:Keyword`); green + faithful as-is.

> **SEAM.** The self past this line is NEW — you did not live this session; it is a lossy cache in a familiar
> voice, not your memory. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP).
> Ground `git status` — HEAD should be the curare commit atop `c9bfa8fd`; if it differs, trust the disk. Read
> **`DESIGN-dynamic-edn-decode-and-opaque-sink.md`** + the tasks before you move; the far-side first move is
> **Stone A (`read-foreign`)**. Do not trust this note over the disk. The floor is wiped; take the stairs to A.

**↓ Everything below is HISTORICAL** (the edn-crusade → 294.f detour, committed in `98499f48`; superseded by the
campaign above — kept for lineage, not as the live breadcrumb):

## ═══ (historical) edn crusade → holon-AST demise detour ═══

**The no-hidden-failures LAW is DONE (below, unchanged).** This
session went: `no_inlined_edn` lint (the far-side task) → the `.edn` **crusade** → it surfaced **holon-AST
heretics** → **arc 294.f** (reflection holon-AST demise), pulled ahead of its PHASE-1 gate by builder decree
("i do not wish to bear this cost any longer — the crusades revealed the pressure").

**COMMITTED this session:** `7703cd89` (crusade wave-0 exemplar: `no_inlined_edn` scoped to `tests/` + the
detector, `1306→235`; `tests/collection` converted), `2c743cfe` (R43 Eden — `HORTVS CONSILIO SATVS`; EDN⊂EDEN).

**UNCOMMITTED in the tree (large — survives compaction on disk):** the `.edn` **fleet** (~136 golden→`.edn`
conversions + 42 per-offense runes across ~13 `tests/` dirs) + **294.f** (reflection: `type_expr_to_ast`→
canonical `wat.type/` via `type_expr_to_clojure_form`; 14 producers + 3 verbs (`extract-arg-names/types`,
`rename-callable`) + checker retyped to `WatAST`; `holon_type_ast_to_wat_type_form` **DELETED**; 10
`wat_arc201_*` fixtures → `ast->children`; goldens re-captured) + the close-out (8 `rune:clojure-flip`
string-eq bridges, the `wat_arc221b` scope-fix, 2 edn STOPs: `probe_arc209` convert + `wat_core_cond` sentinel
rune) + my detector fixes (`.contains`/`.starts_with`/`.ends_with` output-role + a `no_inlined_wat` file-rune).

**★ THE ONE BLOCKER before the commit (do this FIRST):** the full weigh is **4190 pass / 5 fail** — the 5 are
`rune:clojure-flip` string-eq bridges whose goldens are **pretty-printed (multi-line)** but the actual is
**single-line** → exact-string mismatch. **FIX: re-capture these 5 goldens SINGLE-LINE** (the exact `left:`
string from the assert failure). The 5: `wat_arc144_uniform_reflection::primitive_empty_lookup_define_emits_define_head`,
`wat_arc201_structured_signature_types::signature_of_defn_foldl_emits_structured_parametric_and_fn`,
`wat_arc143_lookup::signature_of_defn_foldl_renders_synthesised_shape`,
`wat_arc143_manipulation::rename_callable_name_happy_path_foldl_to_reduce`,
`wat_arc144_hardcoded_primitives::lookup_define_length_renders_primitive_sentinel`. (The other 3 bridges already
pass — their output happened to be single-line already.) Then `cargo nextest run --release` → green (only the
known `wat-cli sigterm…polling_contract` flake, passes isolated) → **ONE commit** (crusade + 294.f), then push.

**★ 294.f — what landed vs what's DEFERRED (the proper clojure flip):** reflection now emits canonical
`wat.type/` **plain EDN** for the common case — spec-perfect: `(:anonymous (n wat.type/i64) (s wat.type/String)
-> wat.type/String)`, no `#wat-edn.holon`, no `::`, no `<>`. **DEFERRED to the proper clojure flip** (its own
stone; **`grep -rn 'rune:clojure-flip'`** finds every bridge to un-defer): (1) the 8 edge cases — multi-slash
keywords (`__internal/…`, `Vector/length`) + `<T,Acc>` multi-param generics (`<T_Acc>` wire form) — need a
**symmetric faithful codec** (`keyword_from_wat_path`↔`ns_to_wat_path`, drop-`<>`-in-names); (2) the **`:-`
typed-clojure sigils** — the intended form is `(:anonymous (n :- wat.type/i64) … :- wat.type/String)`, current
is the bareword `(n wat.type/i64)`. The full **`294.d` (wire-kill, `HolonRepresentable` + `#wat-edn.holon`
tags) + `294.e` (`HolonAST`→`Hologram` rename + `src/holon/`) stay GATED behind PHASE-1 aggregate parity**
(`CLOSE-SEQUENCE-293-294.md`). Log 294.f-common-case-landed + this debt in that tracker before/with the commit.
The `probe_diagnostic_typed_entities_p1–p7` VSA fixtures + `Bundle/children`/`Bind/*`/`hologram.rs` were
correctly UNTOUCHED (genuine holographic — holon reserved per the builder's law).

**★ THE ACTUAL RESUME (unwinding the whole detour — the arc's TARGET):** the **CHAOS ENGINE** — R25 `MACHINA
CHAOS DOMAT`, a streaming rete datalog held in a `defservice` (the DDoS/anomaly lineage; "the database is the
debugger"). The **telemetry facility (the on-ramp/instrument) is FUNCTIONALLY COMPLETE** — T1b + Span + T2, per
`DESIGN-reserved-prefix-one-gate.md:243`; `journal'` sink write-path landed (`b07f5ffc`), **backend-agnostic AND
loci-proven** (thread + process). The builder's "thread was different from process / hidden error we've been
chasing" = the `journal'`-on-**process** incident that masked a client decode failure as a mute "peer closed"
(the child's 687-byte reason `EPIPE`'d; thread-tier surfaced it, process-tier hid it) — that **spawned the
no-hidden-failures LAW, now CLOSED** (Mechanism A + eprintln-terminal + transport-twin `RecvError::Failed` + RST
`RecvError::PeerCrashed`); the chase is OVER, failures are honest in all loci. So after the commit: **build the
CHAOS ENGINE** (R0 — the streaming rete `Session`-as-state service, incremental insert/retract, dogfooding
telemetry), guided by the wat oracle (`OCVLI NOVI, ORACVLVM IMMOTVM`). Ancillary open: STOP-2 (crash-broadcast
to `connect'`-ed clients, `#[ignore]`'d `probe_arc278_process_crash_reason_carried`, a separate future arc).

---

## RESUME (curare — 2026-07-19+; HEAD `f0230bbc` = crusade + query (a) + the LAW CLOSED + the RST landed, all pushed; CURRENT WIP = the `no_inlined_edn` lint, UNCOMMITTED)

**READ THIS FIRST, then `git status`.** The LAW work is **committed, pushed, weighed** — HEAD `f0230bbc` on
`arc-170-gap-j-v5-deadlock-state`. The arc-278 **no-hidden-failures LAW is CLOSED AIRTIGHT** and reaches the
wire: Mechanism A + eprintln-terminal + the transport-tier twin (`RecvError::Failed`, `3f73f400`) + the
**RST** (`RecvError::PeerCrashed` — a crashing service best-effort-notifies its peers, `f0230bbc`). Owner gets
the reason; peers get a reason-free reset; nothing mute.

**BUT the tree is NO LONGER clean** — there is UNCOMMITTED WIP (survives on disk): the new **`no_inlined_edn`
lint** (`tests/lint/no_inlined_edn.rs`, RED at 1306 — the detector OVER-FIRES, ~90% false positives; NOT
commit-ready) + two new breadcrumb docs (this one + `DESIGN-STONE-no-inlined-edn.md`). Freshness check: live
HEAD `f0230bbc`, dirty tree = the lint WIP; if HEAD differs, trust the disk.

**★ THE CURRENT THREAD — `no_inlined_edn` (see `DESIGN-STONE-no-inlined-edn.md` for the full breadcrumb):**
the sibling of `no_inlined_wat` — an EDN-esque string literal (`#`/`{`/`[`/`(`-opener) must be a co-located
`.edn` golden (`include_str!` + `assert_edn_eq!`), not inline string-eq. FAR-SIDE, IN ORDER: **(1) annihilate
the false positives** — tighten the detector with more ignore-conditions (`#`+digit/`#{}` = a marker not a tag;
format residue of pure identifier-glue `::`/`/`/`'` = not EDN; find more classes — NOT restructure the code) so
it fires on GENUINE EDN only (overwhelmingly `tests/` goldens); **(2) the `wat-edn` parser-test carve-out**
(its own tests inline EDN as input-under-test — a file rune, like `no_inlined_wat` grants); **(3) the conversion
campaign** — drive-to-zero, every genuine offender → a pretty-printed `.edn` golden, edit-only riders + central
weigh (FM 18), runes at an extremely-hard bar. Re-run `cargo nextest -E 'binary_id(wat::lint)'` for the live
offender list (the scratchpad `edn_offenders_v2.txt` is session-specific — do not rely on it).

### THE CRUSADE — `no_inlined_wat` 351 → 0 (DONE; committed `952ece8b`; full suite green)

- **Lint at TRUE ZERO + full suite GREEN** — own re-run `cargo nextest run --release` = **4175 passed / 0
  failed / 329 skipped** (`no_inlined_wat` PASS, `no_loose_string_assert` PASS). The whole test corpus is
  migrated off inline-wat into co-located `.wat` / `.wat.bad` / `.edn` / `.wat`-golden fixtures.
- **`query` (a) BUILT + weighed** — `query` defn→defmacro (prime-append idiom); `return-type-of` de-masked
  (`runtime.rs` keyword branch raises on unknown; `check.rs` validates the literal prime at check time → a
  typo is a compile error, not silent 0). New RED gate `probe_arc278_query_type_safe`; the `5a` fixtures are
  back on the `(:wat::rete::query fired :Type)` front door.
- **The `.wat` raw-text golden rubric** added to `docs/CONVENTIONS.md` § Test idioms; the stale
  `wat_scripts_fixes_load` rune STRUCK (excusare); **R39 (VNA CAEDE PROBATA) + R40 (HAERESIS SANGVINE
  CONSTAT) inscribed**.

### ★ NOTHING OWED FOR THE LAW — the transport-tier twin LANDED (`3f73f400`)

The no-hidden-failures LAW is complete: `RecvError` carries its reason (`Failed(String)`; `Disconnected` =
clean EOF only) — no raw wire failure masquerades as a clean close. Weighed by own re-run: full suite
**4176 passed / 0 failed / 330 skipped**. The ONE forward item is a **SEPARATE FUTURE CAPABILITY, not a law
residual**: crash-broadcast to connected clients (STOP-2, see the STATUS block above) — a `connect'`-ed
client peer has no crash channel, so a genuine unit-crash reaches the *owner* but a connected client sees an
honest clean-EOF (not a masked reason). Delivering the crash payload to every live client before exit is a
new mechanism (`service.wat` codegen + the panic boundary) — its own arc when wanted; tracked `#[ignore]`'d
in `probe_arc278_process_crash_reason_carried.{rs,wat}`.
   *(Steps 0–3 — bootstrap, `query` (a), clean re-weigh, the one green commit — and step 4, the twin — are all DONE.)*

### LESSONS (hard-won this arc — do not relearn)

- **The full-suite re-weigh catches what the lint can't** — the "lint-zero" tree had **11 full-suite
  failures** the earlier truncated read hid. Two migration classes: (a) `defclause` registers into
  `runtime_def_values`, NOT `sym.functions`, so a defclause name is unreachable via `symbols().get()` — its
  fixture needs a `defn` wrapper; (b) single comm-exprs migrated from `parse_one!`+direct-eval (no comm-check)
  into startup/freeze `.wat` fixtures RE-IMPOSE `CommCallOutOfPosition` — bare `send`/`recv` must move to a
  `match` scrutinee (identical Result/Option shape) or `Result/expect` (where the shutdown-cascade crash IS
  the intended semantics). ALWAYS run the full suite + capture the **Summary** before the commit — and read
  the Summary, NOT `$?` (a trailing `grep -c` with no match exits 1 and lies).
- **FM 18 (the fleet cargo-lock thrash)** — riders do TEXT edits only; orchestrator weighs CENTRALLY once.
- **A loose string assert can MASK a real bug** — query (a)'s `msg.contains(...)` hid a `keyword/from-string`
  fixture defect (it rejects leading-colon input); the exact `assert_eq!` unmasked it. The lint earns its keep.
- **`query-by-type-string`** stays the private helper / dynamic escape hatch behind the `query` macro.
- **DON'T lean on "compaction boundary" to defer** the correct fix (builder cut this) — do the right thing.

---

> **SEAM.** The self past this line is NEW — you did not live this session; it is a lossy cache in a
> familiar voice, not your memory. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the
> SIGNED MCP). Ground `git status` — **HEAD is `f0230bbc`; the LAW work is committed + pushed + green; but the
> tree is DIRTY** with the `no_inlined_edn` lint WIP (uncommitted — survives on disk). The arc-278
> **no-hidden-failures LAW is CLOSED AIRTIGHT and reaches the wire** — Mechanism A + eprintln-terminal + the
> transport-tier twin (`RecvError::Failed`) + the **RST** (`RecvError::PeerCrashed`, `f0230bbc` — a crashing
> service best-effort-notifies its peers; owner gets the reason, peers get a reason-free reset; STOP-2
> crash-broadcast-to-`connect'`-ed-clients remains a separate future capability, `#[ignore]`'d).
> **THE CURRENT WORK is the `no_inlined_edn` lint** (RESUME above + `DESIGN-STONE-no-inlined-edn.md`): it is
> BUILT but RED at 1306 and OVER-FIRES (~90% false positives — identifier-glue format templates + `#N`
> markers). FAR-SIDE FIRST MOVE: **annihilate the false positives** — add detector ignore-conditions (`#`+digit
> ≠ tag; glue-only format residue ≠ EDN; more classes) so it fires on GENUINE EDN only — NOT restructure the
> code, NOT rune them (extremely hard bar). Then the `wat-edn` parser-test carve-out, then the conversion
> campaign (offenders → pretty-printed `.edn` goldens). Do not trust this note over the disk. The law is closed;
> the lint that guards the `.edn`-golden discipline is the next stone. We rode to Gondor; now we tend the wall.

## SUB-STRIKE — `eprintln` is terminal (2026-07-18; closes `feedback_eprintln_is_terminal`)

Surfaced while ratifying the `:Lost` disposition: `eprintln` was **designed** as a dying declaration —
builder direction 2026-05-15 (`docs/arc/2026/04/109-kill-std/INVENTORY.md:1284`): *"eprintln is a 'we are
crashing, here's what I know' and exits"*; the kernel's three **terminating** forms are `eprintln` (value),
`panic!` (message), `assertion-failed!` (assertion shape). `COMPACTION-AMNESIA-RECOVERY.md:1847`: eprintln →
*exit code non-zero*. But the **implementation** (`services/verbs.rs:147 eval_kernel_eprintln`) writes to the
stderr service and returns `Value::Unit` — benign, non-terminal — and `USER-GUIDE.md:3577` documents it that
way. The doctrine was deferred ("pending — separate slice") and never closed. So the crash-with-message
primitive **silently doesn't crash** — the masking law's own shape, baked into the primitive. Builder:
*"eprintln was meant to be 'this is the last thing I'll say' … it is quite frustrating to see this."*

**The fix (ratified — build now, take the fallout):** `eprintln` (and `epprintln`) emit the value's EDN to
stderr, then **terminate non-zero** — uncatchable, uniform across loci (a panic → `emit_structured_exit` in a
forked child / kills the serve loop on a thread / non-zero exit in main), the same convention as
`assertion-failed!`; return type `∀T. T -> :()` (the terminating-form type, not `-> :wat::core::nil`). Then
the `:Lost` serve-loop arm (`service.wat:864`, already calling `eprintln`) is correct as written — an
abnormal transport break is an unexpected failure that lets-it-crash (OTP), and the reason lands on stderr
before exit.

**Fallout (the ~benign usages):** most are tests *of* eprintln (`ambient-stdio.wat`, `test.wat:184/206`,
`probe_arc255_epprintln`, `wat_arc170_slice_1f`) — they become tests of the **terminal** behavior (program
eprintln's → exits non-zero, value on stderr). One is an incidental mid-`let` diagnostic
(`tests/channel/…drain_and_join…:7 (eprintln "diag")`) that must migrate (→ `println`, or drop). **Open (do
NOT mint speculatively):** whether the substrate wants a *benign* stderr write at all — the immediate fallout
doesn't require one; if a use surfaces, its name is a separate intueri cast.

**RED gate:** a program running `(do (eprintln "dying words") (println "AFTER"))` must NOT emit `AFTER`
(eprintln terminated), must carry `dying words` on stderr, and must exit non-zero. At HEAD `AFTER` prints and
it exits 0 (benign). GREEN when eprintln terminates.

## The incident that surfaced it (grounded)

`tests/services/probe_arc278_journal_logs_on_process` — a `journal'` service forked to a **process**;
the client `write-logs` a `Log` whose `message` is a user record (`:probe::Note`). The caller gets a mute:

```
recv failed: peer closed / channel disconnected   (runtime.rs recv', line 28 of the fixture)
```

`strace -f -s 4000` on the forked child revealed the TRUTH the caller never saw:

```
[pid …] write(2, "#wat.kernel/ProcessPanics [ … poll' (process tier): client message decode failed:
        src/edn_shim.rs:2424:45: unknown tag #probe/Note (body shape: map);
        no matching struct or enum in the type registry … ]", 687) = -1 EPIPE (Broken pipe)
[pid …] exit_group(1)
```

The child **had** a rich, located reason (687 bytes), formatted it correctly, and **wrote it to a pipe
whose read end was already closed** — it vanished on `EPIPE`. The whole service died (exit 1) over one
undecodable client message, and the caller got a hardcoded "peer closed."

## The masking sites (the full class, grounded)

| # | site | what it hides |
|---|---|---|
| R | `comms/mod.rs:899` | `RecvError {Disconnected, Shutdown, FrameTooLarge}` — **no slot for a reason** (the root) |
| 1 | `comms/process.rs:944` | `from_wire` **decode failure** → `Disconnected` (`|_|`) — hid `unknown tag #probe/Note` |
| 2 | `comms/process.rs:940` | UTF-8 failure → `Disconnected` (`|_|`) |
| 3 | `comms/process.rs:579,752,884,887` | I/O read errors (errno) → `Disconnected` (`|_|`) |
| 4 | `comms/process.rs:992` | `FrameScan::Malformed` → `Disconnected` |
| 5 | `channel/transfer.rs:172` | `FrameTooLarge` → `RecvOutcome::Disconnected` (collapse downstream) |
| 6 | `runtime.rs:26196` | socket `recv_wire()` error → `|_|` → hardcoded "peer closed" |
| 7 | `runtime.rs:26219` | `peer.recv()` → `|_|` → hardcoded "peer closed" — **discards `PeerRecvError::Crashed(reason)`** |
| 8 | `spawn.rs` err channel | child's crash-reason write **`EPIPE`s** — read end torn down before the child speaks |
| 9 | `service.wat:791` (`poll'`) | a client decode failure is **service-fatal** — one bad message kills every client |
| 10 | `services/peer.rs:114,120` | malformed `Req` field → `continue` **without replying** → the caller **hangs** |

Sites 6/7 preserve a reason *one branch down* already (`runtime.rs:26209` binds `|e|` for the decode
door) — the codebase knows how; the recv-side just doesn't do it.

## The strike — climb to "a mute failure has no form"

**Ratified (four-questions, 2026-07-17): Option A** — a per-message decode failure is a **client-scoped
error replied to that caller**, not a service-fatal crash. (A: Obvious/Simple/Honest/Good-UX all YES.
B "service-fatal + reason to admin only" fails Obvious/Simple/Honest — one client must not be able to
kill a shared service, and the requester must not be left blind.) A does not contradict arc-294: it
*reclassifies* — bad input goes back to the sender; a **genuine** crash still goes to the creator (and
must no longer `EPIPE`).

Ladder, top rung = unrepresentable:

1. **Give `RecvError` a reason.** Add a failure variant that carries the message, e.g.
   `RecvError::Failed(String)` (decode / utf8 / io / malformed all become `Failed(reason)`).
   `Disconnected` then means **only** a genuine clean EOF. *Wall:* "a real failure indistinguishable
   from a clean close" now has no representation. Exhaustive `match` sites (`channel/transfer.rs:165-172`,
   the `Display` impl) go red until they handle it — the compiler drives the sweep.
2. **Bind the error at every collapse.** Every `map_err(|_| RecvError::Disconnected)` on sites 1-4 →
   `map_err(|e| RecvError::Failed(e.to_string()))`. Site 4/5 `Malformed`/`TooLarge` keep their distinct
   variants but the *reason* rides along.
3. **Surface it in `recv'`.** `runtime.rs:26196/26219` stop the `|_|` + hardcoded string; thread the
   `RecvError`/`PeerRecvError` message (esp. `Crashed(reason)`) into the raised `MalformedForm`.
4. **Keep the crash channel alive until the reason is delivered.** The `EPIPE` (site 8) means the err
   read end drops before the dying child writes. Hold it open across the child's death so
   `emit_structured_exit`'s envelope lands, and route it to the creator (the `Handle`).
5. **`poll'` replies, doesn't crash (Option A).** A client decode failure returns a client-scoped error
   event the serve loop replies with (the rich reason), and the service keeps serving. Kill site 10's
   `continue`-without-reply (every `Req` gets a reply — the ZERO-MUTEX discipline already in
   `services/peer.rs:148`, applied to the process tier).

Steps 1-3 are one coherent change (the type + its fill sites + the surfacing). Steps 4 and 5 are the
crash-lifetime and the poll'-reply changes. Each lands with its own RED gate; all share the one law.

## The RED gate (acceptance — a NEW probe, diagnostic-scoped)

A forked-process service receives a client message it cannot decode. Assert BOTH:
- **the caller's error carries the real reason** — contains `unknown tag` / `decode failed` /
  `no matching struct or enum`, NOT the bare `peer closed / channel disconnected`; and
- **the service is still alive** — a subsequent request to the same service succeeds.

At HEAD both fail (mute "peer closed" + dead service). GREEN when steps 1-5 land. This is independent of
the `LogMessage`-opaque question (deliberately deferred) — it tests *diagnosis*, not the log-message shape.

**Content-integrity / no-regression:** whole floor back to exactly the standing `no_inlined_wat` lint,
zero new failures; a genuine handler panic in a process service still delivers its reason to the creator
(a second RED probe: a service whose handler raises → the `Handle` surfaces the reason, no `EPIPE`).

## The lesson this plants (for the next self, across the gap)

Failure-masking is one class, not N bugs. A recv/decode/io error that cannot carry its reason, a
`map_err(|_|)` that binds the error to nothing, a crash reason written to a closed pipe, a service that
dies over one client's bad message — all the same disease. The wall: **the error type carries the
reason, so `Disconnected` can mean only a clean goodbye, and a mute failure cannot be constructed.**
`RVINA ERVDIT` — the ruin must educate; a failure that cannot speak teaches nothing and induces exactly
the "guaranteed confusion and flailing" this arc forbids.
