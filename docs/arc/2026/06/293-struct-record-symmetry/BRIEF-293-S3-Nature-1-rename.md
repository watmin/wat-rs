# BRIEF — 293 S3-Nature-1: rename `Holder` → `Nature` (behavior-preserving sweep)

> **Executor: one sonnet SHADOWDANCER.** A **wide mechanical rename** (substrate-as-teacher: rename the def, let
> `cargo` name every broken reference, fix each, floor stays 100% green). NO new capability — `:Peer` is the NEXT stone,
> NOT this one. Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/` illegal). `cargo build`;
> `cargo nextest run --release` (NEVER `cargo test`); `./target/release/cargo-wat <f>`. **Commit NOTHING.**
> Motivation: 278 R32 — "holder" LIES once a `:Peer` nature joins (a peer holds nothing); the axis is the satisfier's
> `Nature`, not what it holds. This stone does the honest rename FIRST, so the tree never ships the lie; the `:Peer`
> variant lands on the renamed foundation next.

## The work (one paragraph)

Rename the aggregate-nature concept from `Holder` to `Nature` EVERYWHERE — the Rust enum, the struct field, the
user-facing `:holder` clause keyword, the wat sources + fixtures, the error strings, and `AGGREGATE-AUDIT.md`. It is
**behavior-preserving**: the three variants (`Struct`/`Record`/`HolonRecord`), the methods (`is_pure`/`rank`/
`root_keyword`/`from_root_keyword`), the rank ladder, the satisfaction logic — all UNCHANGED in meaning, only renamed.
`:holder` is **hard-retired** (a `defsurface` using `:holder` becomes a `MalformedDecl`; every use migrates to
`:nature`) — the same clean cut K0a made for the bare-`:features` form. Do NOT add a `:Peer` variant, do NOT touch the
rank/floor semantics — those are S3-Nature-2.

## The exact renames

| from | to | notes |
|---|---|---|
| `pub enum Holder` (`src/types.rs:130`) | `pub enum Nature` | 3 variants unchanged: `Struct`/`Record`/`HolonRecord` |
| `Holder::Struct` / `::Record` / `::HolonRecord` (everywhere) | `Nature::Struct` / … | mechanical; `cargo` names each site |
| `Holder::is_pure` / `rank` / `root_keyword` / `from_root_keyword` | `Nature::…` (method NAMES unchanged) | only the type qualifier changes |
| the `holder:` FIELD on `SurfaceDef` (and any aggregate def) | `nature:` | grep `holder:` / `.holder` field accesses |
| the `:holder` CLAUSE keyword parse (`src/types/surface.rs` ~308-390) | `:nature` | HARD-retire `:holder` → the old keyword is now unrecognized (`MalformedDecl`) |
| error/doc strings containing "holder" (parser messages, `:holder value must be…`) | "nature" | e.g. `":holder value must be a holder-root symbol…"` → `":nature value must be a nature-root symbol…"` |
| wat sources + fixtures using `:holder :<kw>` | `:nature :<kw>` | grep `--include='*.wat'` and the `.edn` goldens |
| `AGGREGATE-AUDIT.md` (+ any 293 doc) "holder" as the live concept | "nature" | the doc's live concept-word; keep historical/quote text intact |

**Word-boundary discipline:** rename the identifier `Holder` and the whole word "holder"/"Holder" — do NOT touch the
substring inside unrelated words (there is none for "Holder"; and NEVER touch "Signature"). Prefer a targeted
read→replace per file over a blind global sed; read the diff of each file.

## The method (substrate-as-teacher)
1. Rename the `enum Holder` → `Nature` at `types.rs:130` (+ its `impl` block).
2. `cargo build` → it names every broken `Holder::…` / `holder:` reference. Fix each mechanically (they're all the same
   rename). Watch the error count waterfall to zero.
3. Rename the `:holder` clause parse → `:nature` (hard-retire), update the error strings.
4. Grep the wat sources + `.edn`/`.wat` fixtures for `:holder`; migrate to `:nature`.
5. Update `AGGREGATE-AUDIT.md`'s live "holder" concept-word to "nature".
6. `cargo nextest run --release` → the floor must be **byte-identical green** (this is a rename; every test that passed
   still passes). Any test that changed BEHAVIOR (not just a `:holder`→`:nature` fixture text) is a BUG in the rename —
   STOP and report it.

## Read the rooms, in order
1. `src/types.rs:125-174` — the `Holder` enum + its 4 methods (the def to rename).
2. `src/types/surface.rs:~308-390` — the `:holder` clause parse (→ `:nature`, hard-retire; the error strings).
3. The blast-radius files (by hit count): `runtime.rs` (39), `rete/kernel.rs` (19), `edn_shim.rs` (16), `check.rs` (15),
   `value/value.rs` (11), `test_runner.rs` (7), `collection/eval.rs` / `closure_extract.rs` (6 each) — mechanical `Holder::`→`Nature::`.
4. `docs/arc/2026/06/293-struct-record-symmetry/AGGREGATE-AUDIT.md` — the doc concept-word.

## STOP triggers (halt + report, do NOT hack)
1. **STOP-BEHAVIOR:** the floor must stay byte-identical green. If a test fails for any reason OTHER than a
   `:holder`→`:nature` fixture-text migration you already made, STOP — the rename changed behavior; report the site.
2. **STOP-NO-PEER:** do NOT add a `:Peer`/`Peer` variant, do NOT touch `rank()`/the floor/the satisfaction logic. This
   stone is a pure rename; the `:Peer` nature + its satisfaction are the NEXT stone (S3-Nature-2).
3. **STOP-NOCP:** do NOT change `Purity`/`:wat::enum::Pure|Impure` (the enum's parallel axis — it is NOT renamed), or
   any behavior. Rename only.

## The gate (EXPECTATIONS — the orchestrator re-runs these)
| what | command | expected |
|---|---|---|
| whole floor byte-identical | `cargo nextest run --release` | verbatim Summary; `0 failed` modulo the known `no_inlined_wat_in_tests` reminder — SAME pass count as before the rename |
| `:nature` works in a surface | `cargo wat` on a small `(:defsurface :S :nature :wat::core::Struct :features […])` probe | type-checks |
| `:holder` is retired | `cargo wat` on a copy using `:holder` | `MalformedDecl` (the old keyword is unrecognized) |
| no `Holder`/`:holder` residue | `grep -rn "Holder\|:holder" src/ wat/ tests/ crates/` | only historical/quote text in comments (no live `Holder::` / `:holder` clause) |

Runtime ~40-60 min (a wide rename + a full rebuild + the full suite).

## Final report (structured): the files touched + hit counts · the `enum`/field/clause/error/fixture/doc renames done ·
the verbatim whole-floor Summary (pass count SAME as baseline) · the `:nature`-works + `:holder`-retired probe results ·
the residue grep (empty of live refs) · STOP triggers hit or "none" · any site where the "rename" was NOT purely
mechanical (a behavior subtlety) — surfaced, not smoothed.

## Prior comparable: the substrate-as-teacher renames (arc 162 lambda→fn internal sweep; `docs/SUBSTRATE-AS-TEACHER.md`)
— rename the def, let the cascade name the sites, floor stays green. This is that pattern, on `Holder`→`Nature`.
