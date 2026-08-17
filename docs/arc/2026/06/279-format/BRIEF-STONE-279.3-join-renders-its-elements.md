# ⛔⛔ SUPERSEDED 2026-08-17 — THIS BRIEF SPECIFIES A DESIGN THAT BREAKS STDLIB BOOTSTRAP. DO NOT STRIKE IT.

> A rider built exactly what is written below. It hit **STOP-2**: `wat/core.wat:1885` self-invokes
> `defrecord` while `core.wat` is still loading, `defrecord`'s macro body joins a namespace prefix at
> `wat/Record.wat:172` **at expansion time**, and a wat-defn `join` does not exist yet →
> `UnknownFunction`, before `main`, stdlib dead.
>
> **`core.wat` ↔ `string.wat` is a genuine dependency cycle. The Rust intrinsic is what breaks it.**
>
> ⛔ **THE CORRECTED DESIGN IS `DESIGN-STONE-279.3-…md`'s `⛔ CORRECTION` SECTION** (option A: `join`
> stays an intrinsic and becomes generic — two edits, `TypeScheme` gains `type_params: vec!["T"]`,
> `eval_string_join` renders each element through the total `str`). The four questions were run flat
> on all four options; the builder ruled **"A has been reasoned."**
>
> Everything below is kept as the record of what was tried and why it failed. **The gate rows (1, 2)
> and the contract still stand** — only the *where it lives* is wrong.

---

# BRIEF — 279.3 · `join` renders its elements

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming, and **a Monitor cannot wake you either**. Run every verification in the
**FOREGROUND** and block on it. Three riders on the previous arc ended their turns mid-floor; nothing
was lost, but do not be the fourth.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## Read first

1. `docs/arc/2026/06/279-format/DESIGN-STONE-279.3-join-renders-its-elements.md` — the ruling.
2. **`wat-scripts/scratch-pad/probe-279.3-join-renders-its-elements.wat`** — ★ **the working
   composition, committed, type-checked, green.** You are MOVING this into the stdlib, not inventing
   it. Copy its shape.

## The work

```
(:wat::core::defn :wat::core::string::join<T>
  [sep <- :wat::core::String
   xs  <- :wat::core::Vector<T>] -> :wat::core::String
  (:wat::core::string::join' sep
    (:wat::core::mapv (:wat::core::fn [x <- T] -> :wat::core::String (:wat::core::str x)) xs)))
```

Four moves:

1. **Rename the native.** `:wat::core::string::join` → `:wat::core::string::join'` — the dispatch arm
   (`runtime.rs:5844`), `eval_string_join`'s `OP` const (`string_ops.rs:461`), and its `TypeScheme`
   (`check.rs:16598`). The native keeps its `Vector<String>` signature **unchanged** — it is now the
   primitive, not the surface. House convention: `insert-all` / `insert-all'`.
2. **Add the wat defn** to `wat/string.wat` under the public name, per the probe.
3. **Delete** the public name's hardcoded `Vector<String>` `TypeScheme` — the wat defn's own
   signature replaces it.
4. **A comment on the lambda** pointing at `docs/arc/2026/06/255-builtin-registry/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md`.

## ⚠ THE LAMBDA IS FORCED. Do not simplify it.

`(mapv :wat::core::str xs)` **does not compile** — a bare intrinsic keyword is a
`:wat::core::keyword`, not an `Fn(T)->U`, while a **user** fn keyword IS. Measured both ways. That
asymmetry is arc 255's defect and is filed; it is not yours to fix and not yours to work around
differently. **Keep the lambda, keep the comment.**

## The gate

| # | assertion |
|---|---|
| 1 | `(join "," [1 2 3])` → `"1,2,3"` |
| 2 | ★ `(join "-" ["a" "b"])` → `"a-b"` — **BARE, not `"a"-"b"`.** Load-bearing: it is the Ruby contract AND the proof that per-element `str` did not start re-quoting strings, which would silently corrupt every existing site |
| 3 | all **19** existing call sites green: `wat/core.wat` ×6, `bracket.wat` ×3, `Record.wat` ×2, `string.wat` ×2, `lint.wat`, `service.wat`, `wat-scripts/fixes/` ×4, `wat-tests/service-cache-lru.wat` |
| 4 | `join<T>` is a wat defn in `wat/string.wat`; the native is `join'` |
| 5 | `check.rs` no longer registers a `Vector<String>` scheme for the PUBLIC name |
| 6 | the lambda carries the pointer comment |
| 7 | a **kept test** covers rows 1 and 2 — not a scratch probe you delete afterwards |
| 8 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 9 | `cargo clippy --release --all-targets` → **0** |
| 10 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

## What you report

- the `git diff` of `wat/string.wat`, `check.rs`, `runtime.rs`, `string_ops.rs`
- **measured output, verbatim**, for `(join "," [1 2 3])` and `(join "-" ["a" "b"])`
- how you verified all 19 call sites (name the command)
- the kept test's name
- floor Summary verbatim; clippy count; `#[ignore]` count
- honest deltas

## STOP triggers — ship nothing on that axis; report and stop.

- **STOP-1 — a `.wat` call site does NOT type-check** against `Vector<T>` where it used to against
  `Vector<String>`. It should: `Vector<String>` unifies at `T = String`. If one does not, name it —
  that is a finding about unification, not something to paper over with a second clause.
- **STOP-2 — `join'` collides**, or a caller reaches the native by the old public name that you
  cannot reach. Name the site.
- **STOP-3 — row 2 renders `"a"-"b"`.** Then per-element `str` is applying the nested-string rule and
  the contract is broken. **Do NOT special-case strings to fix it** — that would re-introduce the
  `Vector<String>` asymmetry this stone deletes. Capture it and stop; it means 279.2's top-level rule
  does not compose the way the probe showed, which is a finding about `str`.
- **STOP-4 — the `#[ignore]` count moves off 13.**
- **STOP-5 — an unintended red. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log at
  `.floor/latest/`. Copy the failing test's **entire** stdout+stderr **verbatim** — never a summary,
  never a `| head`/`| tail` window — and name the exact assertion or match arm that fired. There is
  no such thing as a known flake.

## Out of scope — do NOT touch

- **`Seqable`.** `join` stays over `Vector<T>`. `collection/infer.rs:638` records three named
  blockers for naming that type and it is a separate stone.
- **The `wat.string/*` namespace rename** — that is chain-E.
- **Making intrinsics first-class values** — arc 255.
- **`show` / `str` semantics** — 279.2 shipped them; this stone consumes them.
