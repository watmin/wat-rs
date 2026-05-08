# Arc 163 — INSCRIPTION

## Status

Shipped 2026-05-08. Substrate canonical-form purification complete.
Every wat-internal type representation — container head strings,
primitive path strings, value tags — is FQDN form (`wat::core::Vector`,
`":wat::core::i64"`, etc.). No bridge scaffolding remains. The
substrate's diagnostic stream now covers every user-source bare-form
site (expression-position AND signature-position) per the
substrate-as-teacher Pattern 3 discipline.

`cargo test --release --workspace`: **2041/0** at `6375380`.

Pushed across the slice cycle:

| Slice | Subject | Commit |
|---|---|---|
| 3a | Service path retirements verified hard | `52fb38f` |
| 3b | Unit name + type retirements verified hard | `3a242fe` |
| 3c | Substrate canonicalization recognizes `:wat::core::Vector` | `040e2cc` |
| 3d | `:wat::core::vec` + `:wat::core::list` runtime arms hard-retired | `334f61a` |
| 3e | Substrate container heads → FQDN (waterfall 848 → 0) | `25860be` |
| 3f | Substrate primitive paths → FQDN (waterfall 2041 → 2 → 0) | `7fd753e` |
| 3g | Walker phase A + user-source bare primitive sweep (waterfall 1858 → 0) | `8cf61d7` |
| 3h | Retire canonicalize=true upgrade arms (gate cleared) | `6375380` |

## What this arc adds

A FQDN-pure substrate. Per user direction 2026-05-07:

> *"wat internals are fully qualified - no exceptions... if there's
> a short form - its illegal... if the internal code is mapping to
> a rust primitive then we use the rust form."*

Pre-arc: substrate stored `head: "Vec"`, `Path(":i64")`, value_tags
`"Vec"` / `"Option"` / etc. Post-arc: `head: "wat::core::Vector"`,
`Path(":wat::core::i64")`, value_tags `"wat::core::Vector"` etc.
Source FQDN flows through `parse_type_inner` unchanged; source
bare-form rejected at check time by the BareLegacyPrimitive +
BareLegacyContainerHead walkers.

### Substrate changes

| Layer | What changed |
|---|---|
| `Parametric.head` storage | `"Vec"` → `"wat::core::Vector"` (and Option/Result/HashMap/HashSet) |
| `TypeExpr::Path` storage | `":i64"` → `":wat::core::i64"` (and f64/bool/String/u8) |
| `Value::type_name()` | container arms FQDN; primitive arms stayed Rust form (Rust-side identifier per user rule) |
| `parse_type_inner` canonicalize | Was downgrade-FQDN-to-bare; became identity-passthrough |
| `validate_bare_legacy_primitives` walker | Was wired on `func.body` + post-extraction forms only; now ALSO walks `expanded_user` BEFORE `register_defines` consumes define forms (slice 3g phase A) — covers define-sig type positions |
| Vestigial typealiases for Option / Result / HashMap / HashSet / Vector | Retired (would self-loop in `expand_alias` after head-FQDN) |
| `wat-macros` codegen for `#[wat_dispatch]` | Generated head strings flipped to FQDN |
| `wat-telemetry-sqlite` Cursor + auto-prep | HashMap arm flipped to FQDN |

### Walker coverage extended

Pre-arc the BareLegacyPrimitive walker was wired into `check_program`
but only walked `func.body` and post-extraction top-level forms.
Define-sig type positions (return type + param types) were
SILENTLY ACCEPTED as bare because the parser consumed them into
FnSig structures before the walker ran.

Slice 3g phase A wired a NEW walker call in `freeze.rs` step 4b on
`expanded_user` forms BEFORE `register_defines` extracts. The
diagnostic stream now covers the full user-source surface. The
substrate-as-teacher discipline can drive sweeps from the diagnostic
stream alone.

## What retired

| Pre-arc | Post-arc | Why |
|---|---|---|
| `head: "Vec"` etc. (substrate-internal bare-form storage) | `head: "wat::core::Vector"` (FQDN) | User direction: wat-internals are FQDN, no exceptions |
| `Path(":i64")` etc. (substrate-internal bare-form storage) | `Path(":wat::core::i64")` (FQDN) | Same rule |
| `parse_type_inner` canonicalize-true downgrade arms | Identity passthrough | Internal storage IS FQDN; no rewrite needed |
| `parse_type_inner` canonicalize-true upgrade arms (slice 3e/3f temporary scaffolding) | Identity passthrough | Slice 3g closed the consumer sweep window |
| Walker silent on define-sig types | Walker fires on full user-source surface | Pattern 3 walker coverage gap closed |
| Typealiases for Option / Result / HashMap / HashSet / Vector | Deleted | Identity aliases would self-loop in expand_alias |

## Substrate-as-teacher continuity

Arc 163 is the third worked example of substrate-as-teacher's
Pattern 3 (symbol migration) at substrate-internal scale. Lineage:

- **Arc 109 slice 1c** — first Pattern 3: bare primitives →
  `:wat::core::*` user-source retirement
- **Arc 163 slice 3e + 3f** — Pattern 3 turned inward: substrate-
  internal storage strings flipped to FQDN
- **Arc 163 slice 3g phase A** — Pattern 3 walker coverage
  extended to define-sig type positions (the silent gap closed)

The discipline scales: the diagnostic stream that taught arc 109's
USER-source migration now teaches the orchestrator + sonnet
agents driving SUBSTRATE-side migrations. Same pattern, two
audiences (humans → agents → orchestrators), one stream.

## Realizations

See `REALIZATIONS.md` for the long-form continuation of the arc
111 → 113 → 163 lineage. Highlights:

- **The orchestrator is the third audience** most likely to misread
  the diagnostic stream; "stash + revert + step back" is the failure
  mode the substrate-as-teacher discipline must defend against
- **Slice 3e cost ~1.5 hours of orchestrator-side dodging**;
  slice 3f shipped clean because slice 3e + recovery doc FM 15/16
  codified the discipline; slice 3g/3h carried the discipline
  forward
- **Three NEW recovery doc failure modes surfaced** during the arc:
  - FM 15: treating substrate-as-teacher diagnostics as a crisis
  - FM 16: briefing sonnet with tool-availability preamble
  - FM 17 (memory only): manual sed sweeps as escape hatch from
    sonnet hallucination — never escalate to manual; FM 7 verify +
    re-spawn always

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` — the canonical pattern doc
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 + 16 — orchestrator-
  side disciplines this arc surfaced
- `docs/arc/2026/04/111-result-option-recv/REALIZATIONS.md` — pattern's
  first naming
- `docs/arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md` —
  third application; integ-test verification
- This arc's `REALIZATIONS.md` — orchestrator-side discipline
  named (the third audience for the diagnostic stream)
- This arc's `SURVEY.md` — full surface inventory + cost-ordered
  slice plan
- Memory `feedback_substrate_teacher_failures_are_data.md`
- Memory `feedback_no_tool_preamble_in_briefs.md`
- Memory `feedback_sonnet_hallucination_never_manual.md`
- Memory `feedback_awk_pipe_triggers_sonnet_denial.md`

## Out of arc 163's scope

- **`tuple` → `Tuple` Pascal rename.** Out of arc 163's scope; arc
  165 covers (DESIGN at `docs/arc/2026/05/165-tuple-pascal-rename/`).
- **`:wat::core::List<T>` minted as proper LinkedList.** Out of arc
  163's scope; arc 164 covers (DESIGN at
  `docs/arc/2026/05/164-list-type-mint/`).
- **Charset rule (keywords forbid `_`, allow `,`; symbols inverse;
  EDN-ification swaps `,` ↔ `_`; symbols allow `::` for
  namespacing).** Out of arc 163's scope. Tracked in memory
  `project_keyword_symbol_charset_rule.md`. Arc 163 deliberately
  did not assign an arc number because the rule's enforcement
  timing depends on upstream charset-validation substrate work
  that hasn't been scoped; an arc opens when that scoping happens.
- **User-side def gate (lifting symbol identifiers).** Out of arc
  163's scope; future per memory `project_keyword_symbol_def_gate.md`.

## Coda

Arc 111's REALIZATIONS closed with: *"the user supplied the will.
The substrate supplied the loop. The agent supplied the patience."*

Arc 163 added the orchestrator: *"the orchestrator supplies the
trust."* The loop only works when the orchestrator reads a high
failure count as data, not crisis.

The substrate is now FQDN-pure end-to-end. The next leg of work
opens on a foundation where every wat-internal type expression
matches the user-facing canonical form exactly — no bridges, no
asymmetry, no "internal looks like our form" inconsistency. The
foundation is impeccable. Per user direction 2026-05-03: *"once
109 wraps up - we'll have what we believe to be an incredibly
solid foundation to begin the next leg of work... i cannot begin
any of that work until the foundation is impeccable."*

Arc 163 is one stone in that foundation. The wat machine is what
happens when "every term is honest about what it names" is taken
seriously, in code, with diagnostics that teach.
