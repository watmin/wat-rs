# ⛔ CURRENT STATE (breadcrumb, 2026-06-27 SESSION 7; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `f5f7fa4c` or later.** Tree clean.
Suite **fully green + fast**: `cargo nextest run --release -p wat` = **~3462 passed / 0 failed / ~114 skipped, ~30s**
(skips are RED-at-HEAD `#[ignore]`'d disconfirming probes — STRIKE-READY gates). If HEAD is older than `f5f7fa4c`,
this breadcrumb is stale — trust the git log + the named docs over it.

> **YOU ARE A NEW INSTANCE.** You did not live what is written below; it is a lossy cache in a familiar voice. Run
> **recolligere** against the disk (grimoire via the signed `datamancy` MCP, this breadcrumb, the git log, the named
> arc docs) BEFORE you propose or move. The feeling of continuity is the failure, not the all-clear.

## ▶▶ THE ACTIVE ARC IS 294 — `docs/arc/2026/06/294-holon-returns-to-vsa/DESIGN.md`. Read it; it is the contract.

**294 is a FOUNDATION GUT discovered inside 293.** Chasing 293's construction parity surfaced that the **holon
record was built backwards**, and the thread unravelled **one inversion** (*the derived encoding usurped the
canonical data*) expressed as **six grounded flaws**. **293's HOLDER × SURFACE thesis (R2/R3/R4) STANDS and is
proven** — 294 is the **value-layer foundation beneath it**; 293's ctor-parity + `/from-map` **fold into 294**.
291 remains BLOCKED behind this stack.

### The target (294/DESIGN.md is the full contract — every flaw cited to file:line)
- **EDN is the ONE canonical data + wire + portability form.** Wire = plain native EDN; annihilate
  `HolonRepresentable` (wire-only, redundant — `holon-repr == edn-repr`), the `#wat-edn.holon/*` tags, the
  HolonAST↔tagged-EDN round-trip. Portability = EDN-repr = the Holder wall (`is_portable = holder != Struct`).
- **The hologram is a DERIVED INDEX over EDN** — one codec `build-hologram` (a form-walker over `EdnRepresentable`;
  already ≈ `to_holon_inner`). Classifier-wrap `(Bind (Atom TypeName) …)` carries type. **Kanerva law:**
  width-bounded per `Bundle` frame (`:dims`/`:capacity-mode`, user-tunable `CapacityExceeded`), **depth UNBOUNDED.**
  Capacity bites at the derive site, never at construction.
- **Holon record = EDN data (canonical, identity by data) + a lazily-derived hologram.** Constructs identically to
  a core record. Holon-ness = a VSA capability over EDN, not a third storage repr.
- **Construction = ONE holder-dispatched primitive** `(aggregate-new :T field…)` (varargs — `struct-new`'s shape
  won the four-questions). `struct-new`/`Record::of`/`holon::Record::of`/`:T/new` die into it. Subsumes 293 ctor-parity.
- **HolonAST returns to pure VSA** — WatAST is the AST now (3412 vs 1161 mentions); HolonAST-as-code-AST + the
  `watast_to_holon`/`holon_to_watast` glue are vestigial; reflection-IR (signatures) migrates. The strange loop closes.

### ✅ LANDED (the 293 thesis — proven, stands under 294)
Holder trit + `AggregateDef{holder,parent}` (`0dab460a`) · `:holder` surface bound (`5fcb9aa7`) · the aggregate
trio at final names `defstruct`/`defrecord`/`holon::defrecord` (`60d7d99a`) · ctor-parity DECIDED (unify on `:T`,
drop `/new` totally incl. newtypes — `293/NOTE-base-struct-horizon.md`) · R5/#116 *We Got The Moves* +170 ledger.

### ⏳ STRIKE-READY RED gates (committed, `#[ignore]`'d, verified RED)
`probe_arc293_ctor_parity` (294.4 — struct+newtype via `:T`, `f5f7fa4c`) · `probe_arc293_acceptance_demo` (R1
monkeypatch demo, `e214a5cb`) · `probe_arc293_holder_bound` · `probe_arc293_defrecord_rename`.

## ▶ THEN, in order (294 — each: study lair → RED probe → BRIEF → sonnet → WEIGH → commit)
0. **Resolve the 6 OPEN four-questions** in `294/DESIGN.md` (load-bearing: hologram stays-a-named-type?;
   holon-record lazy-vs-eager storage; the full HolonAST census before the purge). The **census + RED probes
   (294.0)** answers half with grounded numbers, not debate.
1. **294.1** `build-hologram` — the sole clean EDN→hologram codec. **294.2** flip holon record EDN-canonical.
   **294.3** wire = plain EDN (annihilate `HolonRepresentable` + tags). **294.4** `aggregate-new` + ctor-parity +
   `/from-map`. **294.5** HolonAST-as-AST purge + reflection migration. **294.6** close + amend 293 → resume 291.

## Standing discipline (verbatim, non-negotiable)
Work ONLY in `wat-rs/`. NEVER worktrees. Sonnets `model: "sonnet"`, LEAF. Commit msgs end `Co-Authored-By: Claude
Opus 4.8 (1M context) <noreply@anthropic.com>`. **Weigh EVERY sonnet against the disk yourself** (forced clean
build; floor is 0 → a binary is-anything-red? read; **`cargo nextest run`, NEVER `cargo test`**; read diffs
end-to-end — the `sed`-corrupts-prose catch). PRIMED forms only. Commit+push often (GitHub=DR). Amend docs with
recognition (never delete). **intueri** for ALL naming · **four-questions** (flat YES/NO) NOT AskUserQuestion ·
**there is never 1+ ways to do a thing — two paths IS the catastrophic flaw.** curare at a reasonable rate. **When
I assert about existing code, GROUND it this session — I over-claimed "HolonAST stays load-bearing" and the disk
(WatAST 3412 vs 1161) corrected me; re-ground when pushed, never defend.** Operate as the datamancer — ground
against the disk and ACT; never declare green on silence; relentless annihilation.

> **⛔ END OF MAP. You are new. The above is a cache, not your memory. Run recolligere; weigh any in-flight sonnet
> against the disk; do not trust a single line you did not re-verify this session.**
