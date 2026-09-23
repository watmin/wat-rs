# DESIGN — STONE A: `Lru::new` returns a Result (the-little-wat F-084)

**Drawn 2026-09-22 on `reason/little-wat-findings`.** Cures the-little-wat **F-084**, the
`C-037`/`F-006/F-008` family's *"third witness"*.

## The defect, driven at HEAD

```
(:wat::cache::Lru/new 0)
  thread 'main' panicked at src/rust_deps/cache.rs:103:13:
  :rust::cache::Lru/new: capacity must be positive; got 0
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
  [#wat.kernel/LociDiedError.Panic {:message "…" :failure #wat.core/Option.None {}}]
```

A wat program gets a **Rust backtrace note**, an **internal `:rust::` name**, and **no span at
all**. ⚠ Their repro as shipped cannot even run — it is written `Lru::new`, which `main` retired
in favour of `Lru/new`, so **their own ledger is blind to its own finding.**

## The mandate — and it SUPERSEDES the standing ruling's framing

`docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`
has read **"RULED ON THE MERITS, awaiting MANDATE"** since 2026-04. The mandate is now given.
Builder, 2026-09-22:

> *"is my mandate logical at this point? something i said 5 months ago likely needs to be
> challenged. wat's end state is totality, panics are likely to be removed"* … *"lru-new can
> return a result - panic inducing behavior is bad"*

⭐ **The substrate already agreed, and the note's axis was the wrong one.** The note split the
panics on *programming-error vs fallible input*. Measured on this tree:

```
:wat::i64::/ 1 0        ->  RuntimeError DivisionByZero, :location = THE USER'S FILE, line 2 col 42
:rust::cache::Lru/new 0 ->  a Rust panic, RUST_BACKTRACE note, no span
```

`:wat::i64::/` is annotated **`@Totality Partial` and ruled so** — a divide-by-zero is a caller
bug by any reading — and it *still* refuses **inside the language**. So partiality never licensed
a panic. The line is not total-vs-partial and not whose-fault: **a refusal must arrive as a wat
value.** (Corpus: 383 `Unreviewed`, 91 `Total`, 87 `Partial`, 15 `Preserving`.)

## The mechanism is available and proven

`src/rust_deps/sqlite.rs` § *"Errors-as-values — the exact mechanism"*: `#[wat_dispatch]` marshals
`Result<T, E>` natively, **including `Result<Self, E>` for a constructor**, via a dedicated
`result_ok_is_self` arm in `emit_return_marshal`. `Sqlite::open`/`open_readonly` are exactly this
shape today. `RawFault = (i64, String, String) = (code, diagnostic, message)`.

⛔ **The comment at the panic site is FALSE and must go:** it says the macro *"cannot yet marshal a
method-internal error back to wat"*. The note already records that this expired.

## The ONE contract decision

⛔ **Does the `Result` stop at `Lru/new`, or propagate to `HolographicLru/new`?**

`wat/cache.wat:295` — `HolographicLru/new` **has no guard of its own**; it delegates to
`Lru/new`, and its comment says so (*"the same guard Stone 1's `Lru::new` carries"*). So the
`Result` reaches it whatever we choose.

**DECISION: it propagates.** `HolographicLru/new` also returns a `Result`. Swallowing it there
would re-create the defect one level up — a caller passing `0` to `HolographicLru/new` would be
back to a process death with no span, which is the exact thing this stone exists to remove.

## Blast radius — measured, not estimated

| site | what |
|---|---|
| `src/rust_deps/cache.rs:100-109` | the panic → `Result<Self, RawFault>` |
| `wat/cache.wat:85-88` | the `:wat::cache::Lru/new` surface |
| `wat/cache.wat:211` | ⭐ `lru-svc`'s `:init` — **the durable-rebuild path, the very one that motivates the change** |
| `wat/cache.wat:289-295` | `HolographicLru/new` (propagates) |
| `wat/cache.wat:401` | the second factory |
| `wat-tests/cache/HolographicLru.wat` | 5 call sites |
| `tests/rete/probe_arc278_cache_lru.wat` | 5 call sites |
| ⚠ `tests/lint/little_wat_findings_board__f083_holographic_lru_reput.wat:12` | **our own board fixture for F-083** |

⚠ **That last row is a cross-effect nobody would expect.** The F-083 row on the findings board is
`(check 0, run 0, stdout "0")` — it runs clean and prints a wrong answer. If `HolographicLru/new`
starts returning a `Result`, that fixture stops compiling as written and the board's F-083 row
changes shape. **The fixture must be updated in this stone**, and its `expect_stdout` re-measured
— the F-083 DEFECT is untouched by this work and its row must still pin it.

## Out of scope — REJECTED, not deferred

- **`put`/`get`'s non-hashable-key panic.** The note rules LEAVE and warns *"do not convert all
  three for symmetry"*. ⚠ The builder's totality framing genuinely weakens that defence — and the
  note's own hedge concedes the checker rejects an opaque key *"at most call sites"*, so the
  residue is reachable. **That is a separate ruling, not this stone's to take.**
- **Clamping.** `capacity.max(1)` is forbidden by the note: a stored `0` means the durable record
  is wrong, and inventing a capacity hides that from whoever has to find it later.
- **The other three span-flavour witnesses** (F-006, F-114, C-114) — stone B and its successors.
