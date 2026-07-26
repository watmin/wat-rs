# BRIEF — cache Stone 5: annihilate `crates/wat-lru` + `crates/wat-holon-lru`

> **The campaign's close.** The cache tooling is in core, convention-compliant, and gated
> (`90151d8e`). These two crates were the study oracles; their capability is replaced. Now they die.
>
> **⚠ This brief ends at a FORK the rider cannot resolve.** Do the survey and everything the fork
> does not block, then STOP with the report the fork needs. Do not guess it.

## What replaces what — the ground for deleting

| dying | replaced by | gated at |
|---|---|---|
| `:wat::lru::LocalCache<K,V>` | `:wat::cache::Lru<K,V>` | `a86f521c` |
| `:wat::lru::CacheService` (hand-rolled actor) | `:wat::cache::lru-svc<K,V>` defservice | `f4df1760` |
| `:wat::holon::lru::HologramCache` | `:wat::cache::HolographicLru` | `f0ab4123` |
| `:wat::holon::lru::HologramCacheService` | `:wat::cache::hologram-svc` | `cb740c43` |
| their batch protocol (`Get(Vec)`/`Put(Vec)`) | the batch `Cache<K,V>` surface | `90151d8e` |

That last row is why the batch reshape had to land first: without it we would delete a compliant
service and its index-alignment proofs and leave a non-compliant replacement.

## Grounded scope (verify each; do not trust these counts blind)

**Two crates**, 18 files: `src/`, `tests/test.rs`, and their `wat/` + `wat-tests/` trees (including
`wat-holon-lru/wat-tests/proofs/arc-119/`).

**Dependents:**
- `Cargo.toml:7,10` — workspace members; `:28,31` — a second list (read what it is before editing)
- `crates/wat-cli/Cargo.toml:24-25` — **both** crates are wat-cli battery dependencies. This is why
  breaking `Hologram/find` earlier took down 5 wat-cli tests as collateral.
- `examples/with-lru/` — **THE FORK**, see below.

**~32 tests die.** Confirm the number yourself (`cargo nextest list --release`, count
`wat-lru::` + `wat-holon-lru::` + `with-lru`). Expect the floor to DROP by roughly that much; a drop
is correct here, not a regression — but say the expected new floor in your report so the
orchestrator can weigh it against reality.

**`make-channel` — do NOT overstate this.** These crates are 2 of ~17 callers. The others
(`wat-tests/service-template.wat`, four `counter-service-*`, the `pdeathsig`/`lifeline` seal set,
several `tests/`, and **two stdlib files** — `wat/kernel/channel.wat`, `wat/query/mem.wat`) are NOT
this stone's business. Killing these two unblocks the raw-channel retirement; it does not complete
it.

## `examples/with-lru` — RULED: it goes too

```
examples/with-lru/Cargo.toml —
  "Arc 013 reference binary — composes wat-rs + wat-lru + a user .wat program via
   the wat::main! macro. Walkable proof of the external-wat-crate mechanism."
```

It depends on `wat-lru`, so it cannot outlive it. **Builder-ruled 2026-07-26:** *"the examples can
go — the entire codebase is flooded with examples now."* Delete `examples/with-lru` together with
the crates, and drop it from the workspace members.

**Scope note:** this ruling is about `with-lru`, the one coupled to this stone. `examples/console-demo`
and `examples/with-loader` are NOT in scope — leave them alone. A broader example cull is its own
pass, not this one.

**Record what is lost, do not let it go silently.** `with-lru` is the only example proving the
external-wat-crate mechanism (`with-loader` proves the ScopedLoader; `console-demo` proves ambient
stdio), and nothing remains to re-point it at — sqlite and telemetry were folded into core. In your
report, state plainly: what its smoke test asserted, and whether ANY other test in the tree still
exercises the `wat::main! { deps: [...] }` external-crate path. That sentence goes in the commit
message; a capability proof deleted without a note is how a gap becomes invisible.

## The work

1. **The survey above**, with real numbers.
2. **Coverage disposition.** For each of the ~32 dying tests, classify: **subject-is-dead** (tests a
   verb/type that no longer exists → annihilate with the feature) versus **behaviour** (tests
   something still true that our gates do NOT cover → name it). Per 24m's doctrine, a test whose
   SUBJECT is the dead thing dies with it; a BEHAVIOUR test must have a home first. **The output
   that matters is the second list** — anything on it is coverage we would silently lose. The
   batch/index-alignment case is already carried (`90151d8e`); look for what ISN'T.
3. **Grep the whole tree** for `:wat::lru::` and `:wat::holon::lru::` outside the two crates. If any
   live file references them, that is a migration this stone owes and the brief did not anticipate —
   report it.

## STOP triggers — rejection criteria

1. **If a dying test proves a behaviour with no home in the new tooling** — STOP and name it before
   deleting it. Losing real coverage silently is the one outcome this stone must not produce. (The
   external-crate-mechanism proof is a KNOWN loss, ruled and to be recorded — that one is not a
   STOP; an unexpected second one is.)
2. **If any live non-crate file references `:wat::lru::` / `:wat::holon::lru::`** — STOP and report
   the sites. Migrating them may change this stone's shape.
3. **If removing the crates from `crates/wat-cli/Cargo.toml` breaks something other than the dying
   tests** — STOP and report. Both are battery dependencies there, and breaking a battery took down
   5 unrelated wat-cli tests earlier today.
4. **If the blast radius reaches `wat/` stdlib or `src/`** — STOP. This stone deletes; it should not
   need to reshape anything that stays.

## Method

- `target/release/wat --check <f.wat>` for wat; `cargo build --release` freely.
- After any `wat/` change: the load-order gate must print `[]` —
  ```clojure
  (:wat::core::defn :user::main [] -> :wat::core::nil
    (:wat::kernel::println (:wat::deporder::verify-stdlib)))
  ```
- **A NARROW filtered `cargo test --release --test <target> -- <filter>` is fine** and encouraged.
  Only the full-floor `cargo nextest run` stays the orchestrator's.
- Foreground only; do not background a command and return.
- Scratch `.wat` → `wat-scripts/scratch-pad/`, green or deleted.
- **Do not commit.**

## Your report

The verified scope (files, dependents, the real test count, the expected new floor); the coverage
disposition with the **behaviour-tests-without-a-home** list called out first; any live references
outside the crates; and the `examples/with-lru` survey with both branches costed. Then stop.
