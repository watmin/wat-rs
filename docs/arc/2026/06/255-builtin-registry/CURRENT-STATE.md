# ⛔ CURRENT STATE (breadcrumb, 2026-06-26 SESSION 6; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `eb680f3b` or later.** Tree is clean.
The suite is **fully green + fast** (the 8 arc-255 RED-at-HEAD probes are now `#[ignore]`'d — `eb680f3b`). If
HEAD is older than `eb680f3b`, this breadcrumb is stale — trust the git log + the named docs over it.

> **YOU ARE A NEW INSTANCE.** You did not live what is written below; it is a lossy cache in a familiar voice. Run
> **recolligere** against the disk (grimoire via the signed `datamancy` MCP, this breadcrumb, the git log, the named
> arc docs) BEFORE you propose or move. The feeling of continuity is the failure, not the all-clear.

## ▶▶ SESSION 6 — TWO things shipped: arc-293 unify-2b CLOSED, and the test suite went 5min → 34s

### 1. Arc 293 unify-2b — DONE + R4 inscribed
- **2b-fix shipped `0dab460a`** (`AggregateKind`→`Holder`, `parent: String` restored, `parse_recordtype` accepts any
  parent, nominal same-kind extension). Weighed clean: `c02_user_extends_program_env` GREEN, holder-proof 5-green,
  SET-diff ∅ vs `15157c3d`. The HOLDER × SURFACE model (DESIGN §, REALIZATIONS R1/R2/R3) stands.
- **R4 (`12ed7006`, PROBATUM EST)** — *Doubt Me* (Beartooth), `293-…/REALIZATIONS.md`. The apparatus doubted the
  work all session (invented a "race" for a deterministic type-check, cried "255 failures" over a buggy grep, tagged
  arc-170 wrongly); the disk answered every doubt with a proof the work was sound. **PROBA, NE DUBITES** — doubt and
  blind-trust are one crime; the cure is prove-it-against-the-disk. Fulfilled by the `build_env` annihilation (`ad78e752`).

### 2. The test-suite transformation (the session's bulk) — "probably 170" was a LIE
The ~202-failing "floor" everyone called *"probably 170"* (the execve leak) was **mostly stale-fixture rot** — tests
written against syntax the substrate deliberately tightened weeks ago (`Option/Result::expect` 4→2-arg; macro
param/return `:AST`/`:HolonAST`/`:AST<…>` → `:wat::WatAST`; `first`/`second` return `T` not `Option<T>` (arc-278);
`:wat::core::nil`-as-value → bare `nil`; `ServiceEvent` Admin variant + 3rd type-param; `Locus/launch` 4→6 args).
The isolation sweep proved **0 hangs** — none were 170. **[[project_test_floor_was_stale_fixture_cover]] is the memory.**
- **fix-not-delete campaign:** ~69 modules fixed across loose binaries + nursery + pre-existing homes (commits
  `af8f6388` `a343524b` `1a6bb0be` batches 1-3; `c3c98418` nursery; `c2844eeb` collection/macros/types). Doctrine:
  **fix-to-pass, or high-bar-`#[ignore]`, NEVER delete-without-coverage; eradicate = make-pass, not delete; 0-failures
  eradicates bad judgement.** [[feedback_test_disposition_doctrine]]. 2 genuinely-dead tests deleted (string-entry
  deftest `4a4cef89`; row_g_sweep scaffolding) — both author-marked-deletable + covered elsewhere.
- **REORG `c3c98418`:** 253 loose `tests/*.rs` (each = one monolith link = the 5-min build) → **17 module homes**
  (build.rs auto-wires `tests/<home>/*.rs` into one `[[test]]` binary). 262 binaries → ~16; clean build 3.5min → 1m12s.
- **nextest flip `76d63639`:** consolidation surfaced the genuine execve leak as cross-test pollution; **nextest runs
  each test in its OWN forked process** → fresh globals, leak can't cross tests → pollution GONE without the execve
  fix. Per-test deadline (`.config/nextest.toml`, 30s) SIGKILLs a deadlock into a clean timeout (no manual kills).
  `.github/workflows/ci.yml` installs nextest + clippy, runs the gate. **Run the suite: `cargo nextest run --release
  -p wat` = 3466 tests, 3458 passed, 34s** (NOT `cargo test` — slower + leak-exposed).

**THE GENUINE execve leak** is a RARE (~1/15 runs) nondeterministic DEADLOCK in process-spawn tests — the REAL 170,
now NEUTRALIZED for the suite by nextest's per-test isolation. The execve fix proceeds as the real substrate root,
no longer a test-suite blocker. **The floor is now honest:** 0 deterministic failures.

## ✅ LANDED — the 8 arc-255 RED-at-HEAD probes are `#[ignore]`'d (`eb680f3b`)
The only nextest failures (8) were RED-at-HEAD disconfirming probes for UNBUILT arc-255 features (`metadata-of`
reflection / builtin-registry: `probe_arc255_reflection_parity` ×4, `…_ivb2b_verify_examples`, `…_ivc_metadata_plain_values`,
`probe_undefined_builtin_resolves` ×2 — all `tests/nursery/`). `#[ignore]`'d with reason
*"RED-at-HEAD: arc-255 …; unlock when we circle back to arc 255"* — committed `eb680f3b`. Suite is **fully green +
fast**: `cargo nextest run --release -p wat` = 0 failed / 8+ skipped. **When arc 255 is revisited, remove these
ignores — they are the disconfirming probes for the features built then.**

## ▶ THEN, in order
1. **The `:holder` param-typing ADDITIVE layer** (293's R3 acceptance) — surfaces gain the `:holder` bound; the
   `foobar` form (`[x :- user/EnvHolon]` with `:holder :holon-record` + members) goes green. `293-…/DESIGN.md`.
2. **`/from-map`** uniform ctor (arc 291's ORIGINAL ask — the thing that opened 293).
3. **293 close** (INSCRIPTION) + amend `291/CURRENT-STATE.md` to UNBLOCK → resume arc 291 (defservice durable state:
   trust leg / acyclicity / inscription). **291 is BLOCKED behind 293.**

## Standing discipline (verbatim, non-negotiable)
Work ONLY in `wat-rs/`. NEVER worktrees. Sonnets `model: "sonnet"`, LEAF (no sub-subagents). Commit msgs end
`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. **Weigh EVERY sonnet against the disk
yourself** (forced clean build; failing-test-SET-diff w/ baseline-isolation, NEVER absolute count —
[[feedback_baseline_isolate_on_noisy_floor]]). PRIMED forms only. Commit+push often (GitHub=DR). Amend docs with
recognition (never delete). Cast **intueri** for ALL naming. Decide via **four-questions** (Obvious/Simple/Honest/UX,
flat YES/NO) — NOT AskUserQuestion. `./scripts/run_with_venv.sh` for Python. **Operate as the datamancer — ground
against the disk and ACT; do not recite the spells, cast them; never narrate a "race"/guess where a proof is owed.**

> **⛔ END OF MAP. You are new. The above is a cache, not your memory. Run recolligere; weigh the in-flight sonnet
> against the disk; do not trust a single line you did not re-verify this session.**
