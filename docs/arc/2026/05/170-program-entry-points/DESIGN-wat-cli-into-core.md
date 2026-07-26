# DESIGN — fold `wat-cli` into core, and make wat DISTRIBUTIONS a stated capability

> **Origin (builder, 2026-07-26):** *"i want to kill that crate too"* — and then the constraint that
> reframes the whole thing: *"we must support distributions of wat — others can roll their own wat
> distribution with their own rust deps — the batteries are in wat-core now."*
>
> **This is not a file move. It is a capability commitment written down.** The move is the easy half;
> the load-bearing half is that `Battery` + `run` stop being an accident of crate layout and become
> published surface with a guard on it.

## The capability, stated

**A third party can roll their own wat distribution:** their own Rust crate, their own
`#[wat_dispatch]` extensions, their own binary — composed against wat core, without forking it.

That is what `Battery` and `wat_cli::run(batteries)` are *for*. The reason the tree currently has no
consumer is not that nobody wants it — it is that **we absorbed our own batteries into core**
(cache, sqlite, telemetry). The extension point is supported; its in-tree users were the study
oracles and they are gone.

**⚠ Read this before "cleaning up" the battery API.** The orchestrator argued once (2026-07-26) that
`Battery` should die with the crate, on the reasoning that its only consumers were being deleted.
That reasoning was wrong and the builder corrected it. An extension point with no in-tree consumer
is not vestigial when the whole point is that its consumers are **out of tree**.

## Why core is the better home for it — the distribution story improves

```
today   distributor → wat-cli → wat          two crates, one an indirection
after   distributor → wat, calls wat::cli::run     one crate
```

A distribution author writes a small crate with a `[[bin]]` that calls
`wat::cli::run(&[their_batteries])` — the shape `examples/with-lru` demonstrated, minus a layer.

**And the split is already leaky in the wrong direction.** `src/process/verbs.rs:504-516` holds
`fork_program_from_source`, documented as *"the wat-cli's main program execution path… used
exclusively by wat-cli."* Core already owns the CLI's execution engine; the crate holds argv parsing
and a battery shell. Folding it in **removes an inversion** rather than creating one.

## Grounded scope (measured 2026-07-26)

| | |
|---|---|
| files | 5 |
| source | 1,133 lines (`lib.rs` 764, `staleness.rs` 369, two bins at 20 + 30) |
| tests | 735 lines, **16 tests** |
| new dependencies | **none** — `wat`, `wat-edn`, `libc`; core already has all three |
| library consumers in-tree | **none** — only the workspace member list references it |
| dependency cycle | **none** — `wat-cli` → `wat`, never the reverse; the edge disappears |

```
crates/wat-cli/src/lib.rs           → src/cli.rs        (or src/cli/mod.rs)
crates/wat-cli/src/staleness.rs     → src/cli/staleness.rs
crates/wat-cli/src/bin/wat.rs       → src/bin/wat.rs
crates/wat-cli/src/bin/cargo-wat.rs → src/bin/cargo-wat.rs
crates/wat-cli/tests/wat_cli.rs     → tests/cli/ + a [[test]] target
Cargo.toml                          + two [[bin]] entries, − the member entry
```

Core has `[lib] name = "wat"` and many per-directory `[[test]]` targets; it has **no `[[bin]]` yet**,
so the two bin entries are new but structurally ordinary.

## What must NOT be lost

1. **`wat::cli::run` and `Battery` are PUBLISHED SURFACE.** Doc comments must say so outright —
   this is the extension point third-party distributions build on, not an internal helper. A future
   reader finding it unused in-tree must be told, at the definition site, why that is expected.
2. **The binary names.** `cargo wat` works because the binary is literally named `cargo-wat`. The
   `[[bin]] name =` entries carry that; changing them breaks the subcommand convention.
3. **The 16 tests.** Five of them (`presence_proof_hello_world`, `echo_program_reads_stdin_writes_stdout`,
   `missing_user_main_rejected`, `programs_are_atoms_hello_world`, `sigterm_to_cli_cascades_via_polling_contract`)
   are genuine CLI-startup coverage — they went red this morning when a battery broke, which is how
   we know they bite. `sigterm_to_cli_cascades` is a known timing flake; the other four are not.
4. **The staleness guard** (`staleness.rs`, 369 lines) — it warns when the installed binary is stale
   and suppresses itself under `--check-output edn|json` so machine-readable pipelines stay clean.
   That suppression is load-bearing for tooling; keep it.

## ★ The owed guard — a SYNTHETIC battery fixture

`crates/wat-cli/tests/wat_arc100_public_api.rs` was the only compile-time proof that a
`(register, wat_sources)` pair coerces into `Battery`. It is being deleted with the crates (correctly
— it named them), and its **runtime** assertions were vacuous anyway
(`assert_eq!(slice.len(), 2)` on a two-element literal — it could not fail).

But the compile-time property it carried is real, and it guards a **supported** API. It must come
back **without depending on any real extension crate**: two local functions with the right
signatures, coerced into a `&[Battery]`, so the check is a compile of the shape rather than a
dependency on a battery that happens to exist.

**Without this, `Battery` ships with zero traffic** — and a wall with no traffic stops being a wall
(arc 278 R-series, the day's durable lesson). Track it as owed the moment the crates land deleted.

## Open — decide at the strike, do not assume

- **`wat::cli` vs a flatter path.** `src/cli.rs` with `pub mod cli` is the obvious shape, but the
  public path becomes `wat::cli::run`, changing the documented name from `wat_cli::run`. Since the
  in-tree consumer count is zero and out-of-tree ones are hypothetical-but-intended, this is the
  cheapest moment in the project's life to pick the name. **Pick it deliberately; do not inherit it.**
- **Does the `wat` binary stay batteries-included?** Today `src/bin/wat.rs` hard-codes a battery
  array (it named `wat_lru`/`wat_holon_lru`, both being deleted). Once core owns its own surfaces,
  the canonical `wat` binary may want an EMPTY battery slice — core-only — with batteries becoming
  purely a distributor concern. Ground what the array holds after the cache deletion before ruling.
- **Test-target placement.** `tests/cli/` with its own `[[test]] name = "cli"` mirrors the existing
  per-directory convention (`comms`, `kernel`, `channel`, `process`…). Confirm the harness wiring
  (`wat::test!` / `call_beside`) transfers unchanged.

## Sequencing

**After** cache Stone 5 lands (`crates/wat-lru` + `crates/wat-holon-lru` + `examples/with-lru`
deleted). Stone 5 removes the battery arrays' current contents and the only external-battery
consumers; doing this move first would mean editing the same lines twice.

## Status

**DESIGNED, not built.** Scope measured 2026-07-26; all counts above are reads from that session.
The capability statement at the top is the builder's ruling and is the part that must survive
summarisation — the rest is arithmetic.
