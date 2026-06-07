# NOTE (arc 109 vocabulary) — EDN-only stdio is a FIRM contract; mechanically enforce it on the Rust side

**Filed 2026-06-07 (builder directive, mid arc-214 Slice 8).** The CONTRACT is
firm and already locked for the wat surface; what this note queues is the
**Rust-side enforcement** — every native `println!` / `eprintln!` in `src/`
must emit an EDN string, and a mechanical gate must make a non-EDN native
print a BUILD FAILURE, not a convention.

## The contract (firm — builder's words, 2026-06-07)

> *"every rust eprintln! needs to be an EDN string — we need a way to
> mechanically enforce this.. maybe a build.rs can do this check for us?
> fail if we find a native eprintln?.. same for println — the contract,
> firm, is EDN in on STDIN and EDN out on STDOUT,ERR"*

**EDN in on STDIN; EDN out on STDOUT and STDERR.** No channel speaks
anything else:

- The wat surface already holds this by construction (arc 170's EDN-only
  stdio contract: `println`/`eprintln` serialize via `value_to_edn_with`;
  `readln -> :T` parses + coerces EDN; the ProcessPanics envelope is tagged
  EDN — recovery doc § 13, the stdout/stderr/exit-code triangle).
- The GAP is the substrate's own Rust-side diagnostics: a native
  `eprintln!("[wat substrate] stdout-peer: …")` puts a bare human string on
  fd 2 — a contract violation from inside the house. Every reader of fd 2
  (a parent process, a test harness, an LLM mid-migration) must be able to
  read EVERY line as EDN. One bare string breaks "the diagnostic surface is
  legible by design" (the self-teaching-substrate realization, 170
  REALIZATIONS 2026-06-07): the garden is only a garden if nothing in it is
  noise.

## Grounded inventory (live sites at HEAD, 2026-06-07)

- `src/freeze.rs:186/195/204` — ProcessRuntime::drop join-error logs.
- `src/services/mod.rs:142/151/182` — service-peer loop diagnostics
  (malformed-Req guards + handle-failure log). NOTE: this file is mid-churn
  under arc 214 Slice 8; re-grep at execution time.
- `src/test_runner.rs` — 12 `println!` sites: the wat test runner's
  libtest-style human output (`running N tests` / `... ok` / `failures:`).
  **Honest tension to decide at execution**: this output deliberately
  mirrors cargo's human format. Either it converts to EDN (a
  `#wat.test/Run{…}` stream a human can still read — EDN is legible by
  design) or the test-runner binary is an affirmatively-cut exception with
  a rune the gate honors. Decide by four-questions then; the note does not
  pre-decide.
- Doc-comment mentions (freeze.rs:102, runtime.rs:25422) — prose, not
  emission; out of scope for the gate (or trivially allowed by matching
  macro-call syntax only).

## Enforcement direction (the extirpare ladder)

Today the contract is a CONVENTION — the bottom rung. Climb:

1. **Check at construction time** (the queued work): a mechanical gate that
   FAILS the build when a native `println!`/`eprintln!` appears in `src/`.
   Two candidate mechanisms — choose by four-questions at execution:
   - **clippy `disallowed-macros`** (`clippy.toml`: ban
     `std::println`/`std::eprintln`) — rides the EXISTING clippy gates the
     wards already enforce; per-site escape is a visible
     `#[allow(clippy::disallowed_macros)]` rune that excusare can audit.
     Likely the cleanest fit.
   - **build.rs source scan** (the builder's sketch) — fail compilation on
     a match; owns its own allowlist format; works even where clippy isn't
     run.
2. **Make the wrong shape unrepresentable** (top rung): mint the sanctioned
   substrate macro — e.g. `edn_diag!(tag, fields…)` emitting a tagged-EDN
   line (`#wat.substrate/Diag{:site … :msg …}`) — and convert the live
   sites to it. Once the gate bans the native macros and the helper is the
   only path, a non-EDN diagnostic cannot be written down.

Scope: `src/` (the substrate). `tests/`, `build.rs` itself, and xtask-style
tooling are not the shipped diagnostic surface — the executing arc bounds
this affirmatively.

## Why this lives in 109's vocabulary

109 is the surface-contract lineage (kill-std; one canonical namespace; the
stdio triangle locked under 170 from 109's slices 1f/1i). This note is that
contract's last unguarded face: the substrate's OWN voice on the channels it
governs. Filed here per builder direction; executes as its own stone/arc
when drawn (natural slot: alongside or just after 214 Slice 8 closes the
service rebirths, while `src/services/` diagnostics are freshly minted).
