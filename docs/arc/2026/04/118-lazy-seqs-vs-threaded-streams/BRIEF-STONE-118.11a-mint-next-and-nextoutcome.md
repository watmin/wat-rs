# BRIEF — 118.11a · mint `:wat::stream::next` + `NextOutcome`. Additive only.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming, and a Monitor cannot wake you either. Run every verification in the
**FOREGROUND** and block on it.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## Read first

1. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.11a-mint-next-and-nextoutcome.md`
2. `…/EXPECTATIONS-STONE-118.11a.md` — the scorecard.
3. `…/DESIGN-118.10-the-pull-primitive-one-force-one-value.md` — why, with the measurements.

## The work in one paragraph

Mint a parametric enum `:wat::stream::NextOutcome<T>` with two variants — `Item [value <- T, rest <-
Stream<T>]` and `Exhausted []` — and a native verb `:wat::stream::next : Stream<T> -> NextOutcome<T>`
that forces **exactly one** cell and returns both halves. **Change nothing else.** No existing verb
moves, the memo stays, no call site is migrated. This stone is purely additive so that stone B's
migration has a proven primitive to migrate onto.

## Read in order

1. **`src/types.rs:1662`** — `name: "Message".into()`, inside `RecvOutcome`'s variant declaration.
   **This is the exemplar for declaring a parametric outcome enum in the Rust registry.** `RecvOutcome<O>`
   is parametric exactly as `NextOutcome<T>` must be. Copy its shape.
2. **`src/runtime.rs:21990`** — `builtin_enum_variant_names(type_path, variant)`. The door for
   building a builtin enum variant from Rust. **Do not hand-roll a variant value.**
3. **`src/runtime.rs:7058`** — a live wrap site (`re-wrap in RecvOutcome::Message`). The shape of
   constructing one.
4. **`src/stream/mod.rs:158`** — `realize`. It already drives a `Stream` to WHNF (`Empty` or `Cons`)
   iteratively. **`next` is `realize` + destructure**: `Empty` → `Exhausted`; `Cons{head, tail}` →
   `Item[head, tail]`. Do not write a second forcing loop.
5. **`src/check.rs`** — register the `TypeScheme`. `type_params: vec!["T".into()]`, param
   `Stream<T>`, return `NextOutcome<T>`. For the parametric-scheme shape see `:wat::eval-ast!` at
   `check.rs:17056` (`TypeExpr::Path("T".into())` inside a `Parametric`'s args) — ⚠ `TypeExpr::Var`
   is a synthetic unification variable and is NOT the constructor for a scheme's own type parameter.

## The one thing this stone is about

**`next` must force EXACTLY ONE cell per call.** `realize` already stops at the first `Empty|Cons`,
so a correct implementation gets this for free — but row 3 measures it rather than assuming it,
because the entire reason this stone exists is that the current three-call protocol forces three
times.

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY FIRST** — before touching `src/`, call `(:wat::stream::next …)` and capture the unknown-verb error **verbatim** |
| 1 | `(next <3-element stream>)` → `Item`, `value` = first element |
| 2 | `(next <exhausted stream>)` → `Exhausted` |
| 3 | ★★ with a **printing** `f`, ONE `next` on `(map f v)` prints **exactly one line** |
| 4 | `next` on row 1's `rest` yields the **second** element |
| 5 | `git diff src/stream/mod.rs` shows **no change to `forced`** — the memo is untouched |
| 6 | `map`/`filter`/`keep`/`into`/`doall` unchanged — the floor proves it |
| 7 | a **kept** test covers rows 1–4 (not a scratch probe you delete) |
| 8 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 9 | `cargo clippy --release --all-targets` → **0** |
| 10 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

## STOP triggers — ship nothing on that axis; report and stop

- **STOP-1 — `next` forces more than one cell** (row 3 prints 2+). Do not "fix" it by adding a
  cache; that is the defect this whole arc exists to remove. Report what it forced and why.
- **STOP-2 — you cannot declare a PARAMETRIC builtin enum.** `RecvOutcome<O>` is the proof it is
  possible; if `NextOutcome<T>` will not register the same way, name the exact difference and stop.
- **STOP-3 — anything existing changes behaviour.** This stone is additive. If minting the verb
  moves a single existing test, that is a finding, not something to absorb.
- **STOP-4 — the `#[ignore]` count moves off 13.**
- **STOP-5 — an unintended red. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log at
  `.floor/latest/`. Copy the failing test's **entire** stdout+stderr **verbatim** — never a summary,
  never a `| head`/`| tail` window — and name the exact assertion or match arm that fired. **There
  is no such thing as a known flake.**

⚠ **Goldens:** an `.edn` golden under `tests/diagnostics/` failing because a **line number in
`src/*.rs` shifted** is yours to update — that IS the work. Say which moved and by how much.
Anything else red is STOP-5.

## Out of scope — affirmative cuts

- **Deleting the memo** (`forced: OnceLock`) — stone B. It cannot die while three-call walkers exist.
- **Migrating the 7 `-stream` twins or any drain verb** — stone B.
- **`dorun`** (builds a Vector and bins it), **`length`** (type-checks then raises), **`first`**
  (returns bare `nil`) — all consequences of B.
- **`Seqable`** — downstream of both.
- **Bikeshedding `Exhausted`.** It matches the substrate's outcome register. Use it.
