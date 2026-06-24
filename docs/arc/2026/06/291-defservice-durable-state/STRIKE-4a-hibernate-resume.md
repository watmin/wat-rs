# Arc 291 — Strike 4a: `hibernate` / `resume` — the soul, digitized (PROBANDUM → PROBATUM)

**Status: STRIKE-READY.** THE done-gate of the arc — fulfills R1's prophecy. `hibernate` renders the live
State to a portable Snapshot and terminates; `resume` reanimates a fresh service from that Snapshot, bypassing
`init`. The service cannot tell it was reborn. **Gate-agnostic happy-path** (a portable counter State); the
non-EDN gate is the separate strike 4b (the `is_portable_type` Record short-circuit — DESIGN §4 finding).

## The contract (pinned — builder-confirmed)

`hibernate` is `stop` that returns the **whole State** (the EDN Snapshot, not a projection); `resume` is
`start` that takes a **Snapshot instead of an init-seed**, bypassing `init` (a snapshot is pure data — no
resources to rebuild). `resume : snapshot :: start : init-args`. Both ride the existing lineage protocol —
the same machine as `stop`, extended by two `Admin` variants + one `LineageUp` variant.

## The mechanism (a clean extension of the lineage protocol — mirror of `stop`)

1. **`Admin` defenum** (`service.wat:~250`, `admin-enum-def`): `:Init [seed] :Stop` → add `:Hibernate` (unit)
   + `:Resume [snapshot <- ~state-ty]`. (Admin now = Init | Stop | Hibernate | Resume.)
2. **`LineageUp` defenum** (`~256`, `lineage-up-enum-def`): add `:Hibernated [snapshot <- ~state-ty]`
   (alongside `:Started`/`:Final`).
3. **`init-from-admin`** (`~264`, the startup router): add a `Resume(snapshot) → snapshot` arm (identity —
   the snapshot IS the State, bypass `init`). The match is over Admin → it must stay exhaustive: `Init(seed)
   → (init seed)`, `Resume(snapshot) → snapshot`, `Stop → assertion-failed!`, `Hibernate → assertion-failed!`
   (Stop/Hibernate are not startup messages).
4. **serve `Admin` arm** (`~530`, the dispatch): currently `Stop → Final + terminate`. Add `Hibernate →
   (do (send' self (LineageUp::Hibernated state)) nil)` (return the WHOLE State + terminate). The match over
   the admin-msg must stay exhaustive: `Stop`, `Hibernate`, and `Init`/`Resume` post-startup →
   `assertion-failed!`.
5. **`<fqdn>/hibernate [h <- Handle] -> ~state-ty`** (mirror the `stop` method, `~669`): `send' (Handle/handle
   h) Admin::Hibernate` (bare unit kw) → `recv'` → `match (Hibernated snapshot) → snapshot`; returns the State.
6. **`<fqdn>/resume [locus <- :wat::spawn::Locus  snapshot <- ~state-ty] -> ~handle-name`** (mirror
   `start-body`, `~793`): ships `(Admin::Resume snapshot)` in place of `(Admin::Init seed)` to the SAME
   `launch<…>` + `init-from-admin` machinery (launch is unchanged — `init-from-admin` routes Resume → State).
   Returns the Handle.

**Exhaustiveness cascade (bounded, expected):** adding `:Hibernate`/`:Resume` to `Admin` makes the two
`Admin` matches (init-from-admin + serve) non-exhaustive — add the arms above. This is the only cascade
(mirrors 3a-i's `ServiceEvent::Admin`); STOP if it spreads beyond those two matches.

## RED probe (committed, verify-first)
`wat-tests/service-hibernate-resume.wat` — a counter, both tiers:
`start(0)` → `increment 7` (State count=7) → `snap = (hibernate h)` (service TERMINATES, returns the State
snapshot) → `h2 = (resume locus snap)` (a FRESH service, State = snapshot) → `increment 3` on h2 (count=10,
**proving the resumed State is live and continues**) → `(stop h2)` → **10**. The hibernate-terminate-then-
resume-fresh is the process-death-and-rebirth; the snapshot (EDN on process) is the only bridge.
**RED at HEAD:** `hibernate`/`resume` methods + `Admin::Hibernate`/`Resume` variants don't exist →
unknown-function / not-a-variant. GREEN when the soul survives.

## Blast radius
`wat/service.wat` only (+ the RED probe un-ignore). **Pure wat — ZERO Rust edits** (the lineage protocol +
the portable-State wire already exist; this is two Admin variants + one LineageUp variant + two methods).
Do NOT touch `wat/spawn.wat` (launch is unchanged — resume reuses it via Admin::Resume), any `src/*.rs`, or
any other arc's files.

## STOP triggers
1. STOP if the `Admin` non-exhaustiveness cascade spreads beyond the two Admin matches (init-from-admin +
   serve) — report the extra sites (the design says exactly two).
2. STOP-back-compat: `counter_on` / `seeded` / `admin_stop` / `stop_resp` MUST stay green (the new variants
   are additive; existing matches gain arms but existing behavior is unchanged).
3. STOP if `resume` needs a `launch`/`spawn.wat` change — the design says launch is unchanged (resume ships
   `Admin::Resume`; `init-from-admin` routes it). Surface the gap if not.
4. STOP if the Snapshot/State can't cross the process wire (is_portable_type) for the COUNTER State — the
   counter `:state [count <- i64]` is portable; if it's rejected, that's a real surprise to report.

## Expectations (scorecard)
| what | command | expected |
|---|---|---|
| RED probe green, both tiers | `cargo test -p wat --test test hibernate_resume` | 2 passed (after un-ignore) |
| back-compat | `cargo test -p wat --test test counter_on` + `admin_stop` + `stop_resp` | 4 + 2 + 2 passed |
| pure wat | `git diff --name-only` | only `wat/service.wat` + `wat-tests/service-hibernate-resume.wat` |
| no new regressions | orchestrator: `cargo test -p wat --no-fail-fast`, SET-diff vs HEAD | ∅ (deporder flap aside) |

Runtime: 25–40 min (lineage-protocol extension; the exhaustiveness cascade is the one delicate spot, bounded
to two matches). This is the keystone — on green, R1's PROBANDUM EST becomes **PROBATUM EST**.
