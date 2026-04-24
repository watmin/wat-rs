# Arc 038 — USER-GUIDE.md recovery + forward sync

**Opened:** 2026-04-24.
**Status:** notes shipping; slices ready in obvious-in-shape order.

## What happened

Commit `5b5fad8` (arc 028 slices 5+6 — INSCRIPTION + full doc sweep, 2026-04-22) introduced content into `docs/USER-GUIDE.md` that crashes input processing on read. The doc became unreadable to the assistant; the failure mode is the same neighborhood as the perl-regex-with-pipes-in-pipe-dense-targets crash named in BOOK Chapter 32 — wholesale mechanical sweeps over markdown-heavy targets are a poison vector.

`cdec632` reverted the file to commit `467a3d4` (2026-04-22 21:12, "USER-GUIDE — sync with arc 021 + 022 shipped state"). Working tree clean. File readable.

The revert leaves the doc ~2 days behind shipped state, missing arcs 023-037 plus the wat-lru namespace promotion. Arc 038 closes the gap via **targeted edits per arc/topic** — never a wholesale sweep.

## The discipline

- **No mechanical sweeps.** Each slice edits a specific section against a specific arc's surface change. The corruption originated in `s/old-path/new-path/` -style edits across thousands of lines; we don't repeat that pattern.
- **No file rewrites.** Every change is an `Edit` against a known small region.
- **One arc per slice (or two if the section is small).** Easier to revert, easier to audit.
- **Verify after each slice** — `wc -l`, header grep, spot-read the changed region. If anything reads weird, stop and report.

## Gap list — arcs to fold in

| Arc | User-facing change | Target section |
|---|---|---|
| 023 | `:wat::holon::coincident?` measurement | §6 measurements |
| 024 | `presence-sigma` / `coincident-sigma` config knobs | §1 overrides |
| 025 | Polymorphic `get`/`assoc`/`conj`/`contains?` over HashMap/HashSet/Vec | new container subsection |
| 026 | `eval-coincident?` family (4 forms) | §6 measurements |
| 027 | `loader:` option on `wat::main!` / `wat::test!` | §1 setup |
| 028 | load/eval hoisted to `:wat::*` root (`load-file!`, `eval-edn!`, etc.) | §1 setup |
| 029 | Nested quasiquote `,,` + `make-deftest` factory | §4 macros + §13 testing |
| 030 | `macroexpand` / `macroexpand-1` primitives | §4 macros |
| 031 | Sandbox inherits Config — drop mode/dims from test macros | §13 testing |
| 032 | `:wat::holon::BundleResult` typealias | §6 algebra + §12 errors |
| 033 | `:wat::holon::Holons` typealias | §6 algebra |
| 034 | `:wat::holon::ReciprocalLog` stdlib macro | §6 idioms |
| 035 | `:wat::core::length` polymorphism | §4 / container subsection |
| 036 | wat-lru → `:wat::lru::*` | §10 caching paths |
| 037 | Multi-tier dim-router; `set-dims!` retired; cross-dim cosine | §1 setup overhaul |

## Why we do this now (not a future arc)

Project discipline: **we do not leave broken things when we find them.** USER-GUIDE.md is the primary onboarding doc for wat consumers; an out-of-date guide misleads every new reader and every cold-boot recovery. The arc 028 doc-sweep instinct was right — *keep the user-facing surface honest*. The execution mode was wrong (wholesale sweep), not the goal. Arc 038 is the same goal under the right execution mode.

## Out of scope

- Other docs in `5b5fad8` (README.md, CONVENTIONS.md, INVENTORY.md, wat-tests/README.md). Builder named only USER-GUIDE.md as poisoned. If a future read of those flags similar issues, they get their own targeted-edit arc.
- Restructuring USER-GUIDE.md sections. The 467a3d4 structure is sound; we extend it, we don't reshape it.
- Re-deriving any arc's claims. Each gap-fill cites the arc INSCRIPTION as source of truth.
