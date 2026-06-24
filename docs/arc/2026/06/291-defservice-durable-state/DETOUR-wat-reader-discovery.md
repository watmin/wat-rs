# DETOUR (arc 291) — the `wat-reader` leaf + real-parser test discovery

**Status: CLOSED 2026-06-24 (`6c9a351c`).** A self-contained detour taken *during* strike 4b-ii-a (the
defservice State→struct re-tool). Recorded here because it's instructive — a diagnostic blind spot that cost
~an hour, fixed structurally so it can never recur. 4b-ii-a resumes after this (see the close).

## How it surfaced

Mid-4b-ii-a, a sonnet migrating the wat-tests was **thrashing for ~10 minutes** on a baffling symptom: after
migrating `service-stop-resp.wat` / `service-hibernate-resume.wat`, their deftests (`stop_resp`,
`hibernate_resume`) **stopped being discovered** by the suite — while `counter_on` / `admin_stop` / `seeded`
(other migrated files) were still discovered. The sonnet chased a wrong hypothesis (proc-macro rebuild
tracking), then a second (the `:stop`/`:hibernate` clauses). Neither was it.

## The bug — a hand-rolled lexer that diverged from the real parser

`crates/wat-macros/src/discover.rs` (the `wat::test!` proc-macro's test discovery) hand-rolled a **tiny
paren-balanced lexer** (`scan_file` / `skip_form` / `skip_string` / `read_keyword` / `byte_offset_to_line_col`)
to find `(:wat::test::deftest' …)` sites textually. `skip_form` counted only `( )` — **no bracket arm, no
EOF-imbalance check.** On a `.wat` file with one missing `)`, `skip_form` over-consumed the unclosed form and
**silently swallowed every deftest below it**, returning at EOF with no error. The file's tests just
*vanished* from the suite — and the suite stayed **green**. A typo silently deleted tests.

(The migration *did* introduce a transient missing `)` in one file at one point; the real damage was that the
tooling **hid** it instead of failing. The builder's exact read: *"it found it was missing a paren… and we
didn't barf on it… this is shockingly bad for diagnosis."*)

This is the `recon_cascade` lesson generalized: a simplified second reader **diverges** from the real one and
fails silently where the real one would error. The cure is never a *third* hand-rolled checker (a balance
count would have to chase `( ) [ ] { }` + comments + strings + keyword-internal parens forever — a fourth
divergent lexer). The cure is: **use the one real parser.**

## The wall, and the decision

The real parser lives in the main `wat` crate (`src/parser.rs` → `WatAST`, via `src/lexer.rs`). `wat-macros`
is a proc-macro crate that **`wat` depends on** (`wat::test!`). So `wat-macros` *cannot* depend back on the
parser — the cycle is **exactly why** someone hand-rolled the tiny lexer in the first place.

The builder's call (full speed — this is the 2-month rehoming campaign): *"rip out what must be ripped out
such that we never experience this again."* Recon proved the cut is clean — every front-end dependency is a
clean leaf or std:

| module | drags | size |
|---|---|---|
| `span` | std only | 161 ln |
| `scope::identifier::Identifier` | std only (`BTreeSet`, `AtomicU64`) — NOT the scope *resolution* system | 189 ln |
| `ast` (`WatAST`) | `Identifier` + `span` | 474 ln |
| `lexer` | `span` | 1457 ln |
| `parser` | `ast` + `Identifier` + `lexer` + `span` | 872 ln |

Nothing reaches `Value`, the type system, the runtime, or scope-resolution.

## The extraction (`6c9a351c`)

- **New leaf `crates/wat-reader`** = `span` + `identifier` + `ast` + `lexer` + `parser` (std-only, no deps).
- **Main crate re-exports under the old paths** (`src/span.rs` → `pub use wat_reader::span::*;`, same for
  ast/lexer/parser; `src/scope/mod.rs` re-exports `Identifier`). The ~71 `span` + ~57 `ast` + 16 `scope` + 7
  `parser` use-sites **compiled untouched** — re-export absorbed the churn. (3 `pub(crate)` items → `pub` for
  cross-crate reach: `ScopeId::as_u64`, `WatAST::{is_bare_symbol, is_metadata_map, metadata_map_pairs}`. The
  3 `#[macro_export]` macros — `rust_caller_span!`/`parse_one!`/`parse_all!` — re-declared in the shim files,
  since `pub use` doesn't carry macros.)
- **`wat-macros` depends on `wat-reader`** and `discover.rs` now calls the REAL `parse_all_with_file` per
  file, then walks the real `WatAST` (`scan_forms`) for deftest sites. **The hand-rolled lexer is
  ANNIHILATED** (`scan_file`/`skip_form`/`skip_string`/`read_keyword`/`is_keyword_byte`/`byte_offset_to_line_col`
  all deleted).

## The diagnostic — the EDN form IS the message

A malformed `.wat` now produces a **loud `compile_error!`** whose whole body is a
`#wat.test/DiscoveryFailed {:file … :path … :line … :col … :error …}` **EDN tagged-literal** — wat's own
runtime-error idiom (`#wat.kernel/AssertionFailure`, `#wat.diag/TypeMismatch`). File·line·col are precise off
the parser's span. The EDN form is self-describing for a human reading cargo output **and** `read`-able by a
CI parser (anchor on the `#wat.test/DiscoveryFailed` tag). A first pass added an ASCII box-banner + prose +
a `DISCOVERY_ERROR_SENTINEL`; the builder cut all of it — *"do we need anything other than the edn form?"* —
**no.** The diagnostic is pure EDN.

**Proven on the real bug:** a missing `)` in `service-stop-resp.wat` yields
`#wat.test/DiscoveryFailed {:file "service-stop-resp.wat" … :line 47 :col 1 :error "unclosed '('"}` and
**blocks the whole suite** until fixed. The silent loss is now a wall.

## Lessons (worth carrying)

1. **Discovery can no longer diverge from the parser, because it IS the parser.** The failure class — a typo
   silently deleting tests while the suite stays green — is structurally impossible now. (extirpare: the cure
   was not a better hand-rolled checker but deleting the second reader entirely.)
2. **The weigh caught a false-green.** The sonnet reported "all phases green"; the disk disagreed. `cargo
   build -p wat-macros` returned `Finished` in 0.48s from a **stale cached artifact** while `discover.rs`
   had an enum/construction mismatch; rust-analyzer diagnostics were **lagging intermediate edits**
   (transient E0308s that didn't reflect the final state). Resolution: trust **forced clean builds**
   (`cargo clean -p <crate> && cargo build`) and **end-to-end runs**, never the agent's say-so, cargo's
   incremental cache, or stale IDE diagnostics. A cached green binary made `counter_on` "pass" while a
   genuinely-malformed file lurked — only a recompile surfaced it.
3. **The grep-imbalance was a red herring.** `grep -o '(' | wc -l` counts comment/string parens; the *code*
   was balanced. The real oracle is the parser — which is now exactly what discovery uses.

## Close — resume 4b-ii-a

4b-ii-a is **paused, not abandoned.** Its work sits uncommitted in the tree (`wat/service.wat` macro re-tool
+ 6 migrated wat-tests), and the macro is **proven sound** (`counter_on` + `seeded` pass, both tiers, through
the new struct-State surface). Still to do in 4b-ii-a: migrate the ~12 `.rs` defservice probes (they still
use the old `:state` form and won't expand against the new macro), then the full gate + SET-diff. See
`STRIKE-4b-struct-state.md` §"4b-ii — CONTRACT EVOLVED" and `BRIEF-4b-ii-a.md`. The breadcrumb
(`255/CURRENT-STATE.md`) carries the live resume-point.
