# BRIEF — 118.3-B · a concrete type satisfies a PARAMETRIC surface (the 2×2's fourth cell)

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming, and a Monitor cannot wake you either. Run every verification in the
**FOREGROUND** and block on it: your turn ends when the numbers are in your hands, not when the
command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## Read first

1. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/MEASURED-118.3-B-is-a-string-compare-not-a-mechanism.md`
   — the diagnosis, with the exact line and the exact string mismatch.
2. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.3-seqable-the-real-fork.md`
   — why B and not the alternatives.
3. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/EXPECTATIONS-STONE-118.3-B.md` — the scorecard.

## The work in one paragraph

`src/check.rs:14858` has an arm for `(Parametric actual, Parametric expected)`. Its guard is right —
`ah != eh` passes for `Vector` vs `Seqable`. Its **comparison** is wrong: it asks
`is_subtype(k, &format_type(&e))`, an **exact string** match, and at a call site `format_type(&e)`
renders `:sq::Seqable<?454>` — the surface's parameter instantiated to a **fresh unification
variable** — while the registered `extend-type` edge is keyed `:sq::Seqable<T>` **verbatim**.
`"<?454>" != "<T>"`, always. Make that comparison **bind** instead of string-match.

## Read in order

1. **`src/check.rs:14858-14869`** — the arm. This is the only place you change behaviour.
2. **`src/check.rs:14822-14835`** — arm 4, `(Path actual, Parametric expected)`. Its comment
   explains the 2×2 and the verbatim-key storage. **The nature-floor call you must preserve is
   modelled here**, including *why* it uses the BARE surface key.
3. **`src/check.rs:14800-14812`** — arm 3, `(Parametric actual, Path expected)`. It resolves the
   surface by `parametric_head_fqdn` — **head only**. That is the lookup you want; it is why the
   bare-surface probe already runs.
4. **`src/types.rs:745`** — `transport_satisfier_heads`, which hardcodes `format!("{fq}<T>")` and
   `format!("{fq}<Xt>")`. **Do not extend this list.** Guessing more letters is the same defect.

## The shape

Inside the existing `ah != eh` branch, before falling through:

1. Resolve the expected head to its **bare** surface key — `parametric_head_fqdn(eh)` — and confirm
   `types.get(&bare)` is a `TypeDef::Surface`. (Guard exactly as arm 4 does, so a parametric
   **non-surface** bound such as `Vector<T>` is untouched and still falls through.)
2. Find the actual's `extend-type` edge to that surface, by the **bare** key.
3. **Unify** the surface's declared type params against the actual's args using the `subst` and
   `unify` already in scope in this function — instead of comparing rendered strings.
4. Preserve `nature_floor_ok(&a, &bare, types)` on success, exactly as both neighbouring arms do.

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY FIRST.** Before touching `src/`, run `./target/release/wat --check` on `wat-scripts/scratch-pad/probe-seqable-parametric-all-four.wat` **with four call sites appended** and capture the 4 RED `TypeMismatch`es **verbatim**. If it is already green, STOP |
| 1 | after: that same file, calls included, **type-checks AND runs**, printing `3,4,5,2` — Vector, PersistentVector, List, Stream |
| 2 | ★ the **bare**-surface probe `probe-seqable-is-spellable-today.wat` still prints `"3,4"` — unchanged. It goes through arm 3; if it moves, you changed the wrong arm |
| 3 | a parametric **non-surface** bound is untouched — name a call site that exercises `Vector<T>` as a bound and show it unchanged |
| 4 | ★ `Dialable` / `TypedCapability` / `Handle` behaviour unmoved — these are the arm's existing tenants. The floor covers them; name which tests |
| 5 | `transport_satisfier_heads` is **unchanged** — no new hardcoded letters |
| 6 | a **kept** test covers rows 1 and 2 — not a scratch probe you delete |
| 7 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

Row 0 is load-bearing: without it rows 1–2 could pass on a stone that changed nothing. Row 4 is the
one most likely to bite — you are editing an arm three prior arcs already built on.

## STOP triggers — ship nothing on that axis; report and stop

- **STOP-1 — binding here turns out to need VARIANCE you must choose.** This arm's own comment says
  args are **INVARIANT** (*"a channel's send/recv types are exact → unify, not
  covariant-assignable"*). If making `Seqable<T>` bind forces a covariance decision, **that is a
  design ruling, not yours.** Name the exact case and stop.
- **STOP-2 — the fix requires touching `transport_edge_keys` / `transport_satisfier_heads`.** Those
  serve `Handle`/`Dialable`/`TypedCapability`. Widening them is a different, larger stone. Report
  what forced it.
- **STOP-3 — row 2 moves.** The bare-surface path must be byte-identical; it is a different arm.
- **STOP-4 — the `#[ignore]` count moves off 13.**
- **STOP-5 — an unintended red. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log at
  `.floor/latest/`. Copy the failing test's **entire** stdout+stderr **verbatim** — never a summary,
  never a `| head`/`| tail` window — and name the exact assertion or match arm that fired. **There
  is no such thing as a known flake.**

⚠ **Goldens:** if an `.edn` golden under `tests/diagnostics/` fails because a line number inside
`src/*.rs` shifted, updating it **is** the work — do it and say which ones moved and by how much.
Anything else red is STOP-5.

## Out of scope — affirmative cuts

- **Minting `:wat::core::Seqable` itself.** This stone makes it *possible*. Declaring it in the
  stdlib, extending the four containers, and pointing `join`/`map` at it is the **next** stone.
- **The seven `-stream` twins** (`wat/seq.wat`) — B's payoff, its own stone.
- **`into`'s missing `(Vector<T>, List)` clause** — found by the probe, sibling of task #45's
  shipped `(PersistentVector, Vector)`. Small, real, independent.
- **Correcting `src/collection/infer.rs:638`** — its three blockers are refuted and it is the most
  expensive stale sentence found this session. It is a comment-only edit; **do it in this stone**
  if and only if row 1 goes green, and say so in your report.
- **Perf.** Already measured: **~795 ns per surface dispatch (upper bound), 1.76×**, and chain-D's
  design dispatches **once per collection**, not per element. `wat-scripts/scratch-pad/bench-surface-dispatch-cost.wat`.
  Nothing here needs re-measuring — **and do not optimize against that number.** Builder, 2026-08-17:
  *"wat will be byte code compiled… the surface will be our expression language for optimized code
  it produces… interpretted wat has a death sentence."* The bench measures the **condemned
  interpreter** (its own DIRECT arm costs ~1.05 µs for a `length` call — that baseline *is*
  interpreter overhead). The surface is the compiler's input, and one polymorphic verb is strictly
  easier to compile than seven hand-rolled twins. **Write the clearest surface; do not hand-roll
  around a cost the compiler deletes.**
