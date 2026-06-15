# BRIEF — Stone 265: the namespace-scoped acronym registry

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo test`/`cargo build` PLAINLY (no setsid/timeout). Trust your
own build over rust-analyzer (stale mid-edit snapshots — believe your clean build). **Do NOT commit
— the Inquisitor weighs.** Full design: `DESIGN.md` (this dir). Names are DECIDED (intueri) — build them.

## Work in one paragraph
Add a namespace-scoped acronym registry so PascalCase⇄kebab conversion can restore acronym casing
(`WebACL ⇄ web-acl ⇄ WebACL`). Five pieces: (1) an `acronym_registry` field on `SymbolTable`;
(2) `declare-acronyms` (a form populating it, PRE-EXPANSION); (3) `pascal->kebab-in` (Rust intrinsic,
on `is_pure_total`); (4) `kebab->pascal-in` (Rust intrinsic, NOT on `is_pure_total`); (5) thread
`pascal->kebab-in` into `wat/service.wat`'s op-name derivation. Registry keyed by namespace; no entry
→ the plain converters' behavior.

## THE LOAD-BEARING RISK (read first): the ordering
`declare-acronyms` MUST populate the registry BEFORE the defservice macro expands, or defservice's
expand-time `pascal->kebab-in` won't see the acronyms. This is EXACTLY the protocol pre-registration
pattern from 232.3 — **mirror `preregister_protocol_names`** (`src/runtime.rs`, called from
`src/freeze.rs` step 6.95, before macro expansion / resolve). Add `preregister_acronyms` the same way.
The probe's `defservice_consults_its_namespace_acronyms_at_expand_time` gates this exactly. If you
cannot get the registry populated before expansion, STOP and surface — do not fake it.

## The pieces (rooms, read in order)

1. **`SymbolTable.acronym_registry: HashMap<String, Vec<String>>`** — namespace → canonical acronyms
   (e.g. `"ACL"`). Model: the `protocol_registrations` field added in 232.1 (`src/check/env.rs` for
   the check mirror if needed; the runtime/SymbolTable home for the live registry). Reconcile the
   namespace key form consistently (leading-colon handling — see how 232.1/232.3 reconciled
   `class_fqdn` vs `:class_fqdn`).
2. **`declare-acronyms`** — `(:wat::core::string::declare-acronyms :my::ns ["ACL" "HTTP"])`. A
   `parse_declare_acronyms_form` → `(namespace, Vec<String>)`. Registered at: (a) `preregister_acronyms`
   (the PRE-EXPANSION pass, mirroring `preregister_protocol_names`) into `SymbolTable.acronym_registry`;
   (b) a check-side arm so the form type-checks (returns unit) — model the 232.1 `defprotocol`/`extend-type`
   check arms in `src/check.rs` `infer_list` + `collect_splice_defs_ctx`. NO runtime eval effect needed
   beyond registration.
3. **`pascal->kebab-in`** — `(:wat::core::string::pascal->kebab-in :my::ns "CreateWebACL")` → Rust
   intrinsic. Wire the four sites like `pascal->kebab` (string_ops eval fn + check scheme
   `(keyword, String)->String` + runtime dispatch + **`is_pure_total` in `src/macros/eval.rs`** —
   load-bearing, the defservice macro calls it at expand time). The eval fn reads
   `sym.acronym_registry[ns]`; tokenize the PascalCase using the acronym set (a registered acronym is
   one segment; capital-boundary for the rest), downcase, join `-`. **No entry for `ns` → fall back to
   the plain `pascal->kebab` behavior** (capital-boundary).
4. **`kebab->pascal-in`** — Rust intrinsic (reads `sym.acronym_registry` — that's the floor reason
   it's an intrinsic, not a wat helper; **NOT on `is_pure_total`** — no macro needs it). Split on `-`;
   each segment matching a `registry[ns]` acronym (case-insensitive) → the canonical form (`ACL`);
   else capitalize (first char upper via the existing `to-uppercase`, rest as-is). No entry → plain
   `kebab->pascal` behavior.
5. **defservice thread** — `wat/service.wat` op-name derivation (the constructors foldl + methods
   foldl, where it currently calls `:wat::core::string::pascal->kebab op-str`): change to
   `(:wat::core::string::pascal->kebab-in fqdn-str op-str)` — the service's own fqdn (`fqdn-str`,
   already a macro local) is the namespace. Both sites.

The plain `pascal->kebab` / `kebab->pascal` STAY unchanged (namespace-agnostic default).

## Reference precedents
- `preregister_protocol_names` + freeze step 6.95 (the pre-expansion registration — THE model for #2's ordering).
- `eval_string_pascal_to_kebab` / `eval_string_to_uppercase` (`src/string_ops.rs`) — the intrinsic wiring mold for #3/#4.
- 232.1 `defprotocol`/`extend-type` check arms — the model for #2's check-side acceptance.

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc265_acronym_registry                 # 2 passed (both directions + roundtrip + default; defservice expand-time)
cargo test --release -p wat --test probe_arc209_naming_conversion                # 1+ passed (plain converters intact)
cargo test --release -p wat --test probe_arc209_c3_defservice_client_face        # 1 passed (single-word defservice intact)
cargo test --release -p wat --lib -- --test-threads=1                            # zero NEW (baseline 36; +the probes)
cargo test --release -p wat --test nursery -- --test-threads=1                   # zero NEW (baseline 4)
cargo test --release --workspace --no-run                                        # compiles
```
Plus update `docs/PASCAL-KEBAB-CONVERSION.md`'s bijection section: the registry is the escape hatch
from "discipline the namespace" (round-trip total for registered acronyms even on external names).

## STOP triggers (REJECT — surface; do not improvise)
1. `declare-acronyms` can't populate the registry before defservice expands → STOP (mirror
   `preregister_protocol_names`; this is the whole point).
2. `pascal->kebab-in` can't go on `is_pure_total` / the defservice macro can't reach it → STOP.
3. You're changing the plain shipped `pascal->kebab`/`kebab->pascal` contract → STOP (the `-in`
   variants are separate; the plain ones are the namespace-agnostic default).
4. The default fallback (no registry entry → plain behavior) isn't clean → STOP.

## Blast radius
`src/string_ops.rs`, `src/check.rs`, `src/check/env.rs`, `src/runtime.rs`, `src/macros/eval.rs`,
`src/freeze.rs` (the preregister pass), `wat/service.wat`, `docs/PASCAL-KEBAB-CONVERSION.md` + the
probe (already committed). NO changes to defprotocol/the 232 registries/assignable/dispatch.

## Return
Report: the registry field + its home, the `declare-acronyms` parse + pre-registration site
(file:line), the two intrinsics + their wiring (confirm `is_pure_total` got `pascal->kebab-in` ONLY,
not `kebab->pascal-in`), the defservice thread (both sites), every gate command's counts from YOUR
runs, and any honest delta. If a STOP fires, STOP and report. Do NOT commit.
