# ⛔ PARKED 2026-08-17 — THE LIVE SEAM MOVED. Go to `docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md`.

> Builder, 2026-08-17: *"255 has been in a state of partial work for months… we keep leaving it to
> work on some dep then come back…. **we do this now**…. we are killing #wat-edn tags…. only tags
> remain are #wat.\* and #holon … that's it."*
>
> The live work is **arc 294** — the tag annihilation. **Everything below this banner is a
> 2026-08-16 snapshot and is no longer maintained.** Read the 294 seam; come back here only for
> arc 255's own state, and check `git log` over both before trusting either.
>
> 255 is NOT abandoned: **7 of the 13 remaining `#[ignore]`s are its**, its blanket-accept at
> `src/resolve/walk.rs:257` is still live, and its DAY-ONE entry-shape decision is what stops the
> opaque-registry collision hole (294.i Part 3) from being designed by anyone else.

---

# SEAM — SNAPSHOT, 2026-08-16 (late). Superseded as the live breadcrumb; kept as arc 255's own state.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** This one. `251/SEAM.md` and `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

```
HEAD = 683eaab8  (294.h DRAWN, docs only)   stash@{0} INTACT (never drop)
⚠ TREE IS DIRTY — a sonnet rider is IN THE FIELD on 294.h (src/comms/mod.rs + 8 test files).
   Uncommitted work in the tree is ITS work, not abandoned work. Do NOT revert, stash, or
   run cargo against this checkout while it runs (FM 18: one target/ lock, N builds).
floor (last green, at 6b5c8232)  4675 run / 4675 passed / 30 skipped  0 FAIL  0 TIMEOUT
clippy 0
```

⚠ **THE FRESHNESS MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.** Two seams in a row shipped a
marker that **could not pass by construction** — the seam text is written *before* the commit that
carries it, so the only hash it can print is its parent's, and the alarm fired every time on a
perfectly current file. An alarm that always fires is an alarm nobody reads. The honest form:

> **This seam was written against `6b5c8232`.** Run `git rev-parse HEAD`. If it differs, that is
> **normal** — this file's own commit is one of the differences. What the marker buys you is the
> exact thing you must read before trusting a line below:
> **`git log --oneline 6b5c8232..HEAD`** — every commit in that range landed after this text.
> Empty range → nothing has moved. Non-empty → **`git log` and the arc docs outrank every line here**,
> and the longer the range the less this file is worth.

⚠ **`mcp__wat__eval` CAN LIE.** It runs a long-lived server; a rebuilt binary does NOT reach a running
process, and it spent today answering from a **two-day-old** substrate with nothing in its output
saying so. **Probe it before trusting it:** `(:wat::core::<= 1 (:wat::core::f64::/ 0.0 0.0))` must be
**`false`** (C5c). If it says `true`, the server is stale — kill it, reconnect, or use
`./target/release/wat` on a scratch file instead. Two `wat` MCP servers are registered under the SAME
name (project-scoped → `target/release/wat`, global → `~/.cargo/bin/wat`); the project one is correct
because every `cargo build --release` refreshes it.

## THE ROAD (builder's, in order)

1. ⛔ **`#wat-edn.*` tags → `#wat.*/*` — MY PRESCRIPTION HERE WAS WRONG. CORRECTED 2026-08-16.**
   The previous seam said *"multi-site and structural ⇒ R21 wat-fix codemod."* **Measured:** 73 tag
   lines across 22 files, of which **5 are `.wat`** and **68 are Rust** (24 in `src/edn_shim.rs`
   alone). wat-fix rewrites `.wat`; it is the wrong instrument for 93% of this. I named a tool from
   memory while the arc's own ruled derivation sat two files away.
   **The real shape is `294/CHAIN-rendering-before-the-string-home.md`, ruled 2026-08-14** — an
   ordered chain bottoming out at stone **A**, a Rust trait making each type declare its own tag +
   portability. Read the chain before touching a tag.
   ⚠ **Stone A cannot be struck as drawn**: it proposes `trait EdnRepresentable { const TAG; const
   PORTABLE; }`, and that name is **already taken** by the live comms wire trait at
   `src/comms/mod.rs:102` (`to_wire`/`from_wire`) — the very trait the builder wants HolonRepresentable
   replaced by. A must pick a different name. (`ToEdn` is also live: `crates/wat-edn/src/lib.rs:125`,
   74 impls.)
   ★ **294.h shrinks this pile before the chain is touched** — deleting `HolonRepresentable` removes
   the comms-side producer of the `#wat-edn.holon/*` family, the largest of the seven tag namespaces.
2. **Ignores → exactly ONE.** Builder's ruling: *"we should have precisely 1 ignore when we're
   done... the ignore that proves wat-tests support ignores."* Now **24** (see below).
3. **arc 255** — the registry. 9 ignored tests are blocked on it; the builder is inside that work.

## ★ RULINGS THAT OUTLIVE TODAY

- **`#[ignore]` means ONE thing: blocked or broken.** It was answering two questions and a count that
  mixes two populations **cannot be driven to zero**. Stone K split them structurally: benchmarks →
  `benches/`, diagnostics → excluded by config, one slow test → `default-filter`. The greppable
  `ON-DEMAND (not debt)` marker was the CONVENTION rung and is now **deleted** — a string every hand
  must write and read correctly is not a mechanism.
- **THREE dispositions, not two** — staleness (capture) · finding (report) · **SUPERSEDED** (a later
  arc replaced the design; retire or rewrite). It decided real rows in **three of six** waves.
  Sub-classes it grew: *the golden pinned a WRONG value, so the fix looks like a regression* (needs
  proof the old value was wrong), and *the unlock condition itself was superseded* (decl-b.1.0 was
  **annihilated**, not built).
- **JUDGE THE BODY, NOT THE NAME.** Four `fn_rename` tests are named `..._silently_aliases...` and every
  body asserts a hard-cut **rejection**. Names are claims; bodies are evidence.
- **Unrepresentable beats guarded.** Builder on `field-N`: *"should be unrepresentable — this is the
  greatest fix."* My first design was a graceful `_fields` fallback for a state that cannot occur —
  it would have *legitimized* the bug. G′ deleted the question instead: the census is **0**.

## WHERE THINGS STAND

| | |
|---|---|
| **296-recapture-pending** | **1**, not 0 — `wat-tests/lint.wat:72`, a **wat-native** `(:wat::test::ignore …)`. Every census I ran was `--include=*.rs`, so I declared the campaign closed at zero for hours. The `.rs` side IS 0. |
| **`#[ignore]` total** | **24** (was 31). All reasoned; ON-DEMAND markers gone. |
| **`field-N` producers in `src/`** | **0** |
| **`IGNORE-LEDGER.md`** | retired in place — 115→0 recorded, gate stated, wat-native exception called out. NOT deleted. |
| **296 `INSCRIPTION`** | still absent. **The arc is OPEN.** |

**Landed today:** C5b exact mixed-numeric ordering · C5c NaN unordered (no warts) · G′ enum carries its
own names · 296 waves B2–B6 (115→0 on the `.rs` side) · 7 relocated rete probes deleted · Stone K built.

## ⛔ WHAT I GOT WRONG — read this before trusting any number here

**Four instruments lied in one day, each answering a question I had not asked:** grep counted prose as
code (**twice** — 51 "bare ignores" were 7); `git log -1 <ref> -- <path>` answers *last commit at or
before the ref*, not *did this ref touch it*; `mcp__wat__eval` held a two-day-old process; and a
`--include=*.rs` census got reported as a fact about the tree. **Every number that held came from a
compiler, an imposed wall, or a freshly built binary.**
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

**I opened arc 301 unasked and committed it.** Retracted. The place existed — 296's own unmet gate —
and I reached for a new number because I had declared 296 finished when its *count* hit zero while its
*gate* stayed open. `[[feedback_opening_an_arc_is_the_builders_ruling]]`

**A rider refused an order of mine that would have written a false claim** — I told it to record
"296-recapture-pending = 0, measured"; it measured, found 1, and wrote the exception instead. Briefs
that demand honest deltas get them.

## THE STILL-OPEN

- **4 non-255 ignore candidates**, evidence in hand: 237.7c shipped (`a9961421`), 293 method members
  shipped (`b13cab8c`), decl-b.1.0 **annihilated** (`19ace45e` — superseded unlock), + 1 bare unknown.
  **Each needs a non-vacuity proof before its ignore comes off** — nine fixtures once passed while
  proving nothing.
- **2 arc-255 candidates** — green while nine siblings fail; hand to the 255 work, not an outside rider.
- **W2** safety-claim audit (briefed, unstarted) · **W1** parked behind 255 · **Stone H** drawn, unbuilt.
- **`value_to_json_natural`'s `Option<&TypeEnv>`** door survives with no `field-N` behind it.
- **`Value::Vec` vs `Value::Vector`** — a Rust-side reader trap; the wat surface is unambiguous
  (`:wat::core::Vector` = sequence, `:wat::holon::Vector` = hypervector, `:wat::core::vec` RETIRED).
  `value.rs:53`'s doc comment still names the retired `:wat::core::vec`.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **Run `git log --oneline 6b5c8232..HEAD` and probe the MCP (`(<= 1 ##NaN)` must be `false`) BEFORE
> you quote anything from this file.** The commit range is not a pass/fail — it is the list of things
> that happened after this text was written, and every one of them outranks it.
>
> ⚠ **Every number here was produced by an instrument. Ask what population that instrument could see
> before you repeat it** — five separate counts were wrong today, and each one read as solid until
> something imposed a wall.
>
> **296's `REALIZATIONS.md` STOPS AT R19 (2026-07-02)** — six weeks and a dozen stones behind. `git log`
> and the DESIGN-STONE files are the only witnesses after it.
>
> Before calling an absent error a defect, **search the arcs for its subject** — one command, and
> skipping it is how a deliberate supersession got labelled a security hole.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `DVABVS VIIS PRAETERITVM CLARESCIT.`
