# STONE — the ungated `.wat` corpora: three fossils, and the hole that hid them

DRAWN + BRIEFED 2026-08-25, against `a1aa347e1`.

## Why this stone exists — the builder's ruling

> *"i forgot we had these examples.... we should fix it... we'll delete it later... but it found
> something broken... which is a strong tell for its existence"*

The example's value is **as a detector**, not as a demo. So this stone is not "fix `main.wat`". It is
**make the detector fire without a human**, and fix everything the detector found on its way in.

## What the crawl found — all measured, none guessed

`./target/release/wat --check` over every tracked `.wat` outside `wat/`, `wat-scripts/`, `tests/`,
`wat-tests/`, `docs/` — **11 files, 3 red**:

```
1  examples/console-demo/wat/main.wat            FIVE shape changes + a SEMANTIC rot
1  crates/wat-edn/wat-edn-clj/wat/shared.wat     retired :wat::core::define — and it is LIVE
1  wat-migrate/fix-decl.wat                      8 errors — and it is a DECLARED CORPSE
0  benches/… crates/wat-edn/demo/… examples/with-loader/… (8 files, all green)
```

**The hole:** `every_tracked_wat_parses` only **PARSES**. `every_wat_scripts_file_loads` walks only
`wat-scripts/`. Everything under `tests/` is driven by its own `.rs`. `examples/`, `crates/*/`,
`wat-migrate/`, `benches/` are checked by **nothing**. A `.wat` nobody executes rots silently, and
these three rotted in three unrelated ways across three corpora.

**The natural experiment, n=2, perfect correlation:** `examples/with-loader` has a smoke test that
spawns its binary — it survived two retirement sweeps. `examples/console-demo` has no `tests/` dir at
all — it has been dead since **arc 241**. A binary nothing spawns cannot fail.

---

## ⛔ THE FINDING THAT DECIDES THE WALL'S SHAPE

**Type-checking is NOT enough, and I proved it.** I drove console-demo to `--check EXIT=0` and it
still **died at runtime, exit 2**, after three of five lines.

`:wat::kernel::eprintln` is **wat's PANIC channel** — it emits to stderr and then TERMINATES
non-zero. `wat/grep.wat:421` states the contract outright: *"there is no benign, non-terminating
stderr-write primitive in the substrate."* The demo's entire thesis — routine flow → stdout,
concerning events → stderr — is **unrepresentable in the current language**. It is not a syntax
fossil; it is a semantic one, describing a stdio model that no longer exists.

**So the wall is two walls**, and neither substitutes for the other:
- a **check** wall over the ungated corpora (catches all three fossils, cheap), and
- a **run** gate for the example (catches what a checker cannot).

---

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first. **Ending your turn ENDS you** — nothing
will wake you. Every command **FOREGROUND**, blocking; your turn ends when the numbers are in your
hands, not when a command is launched. **You may not spawn sub-agents.**

Do not commit, push, stash, revert, or `git checkout`. `git stash@{0}` must never be touched.
Use `git rm` for the deletion.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check <f>`, `./target/release/wat <f>`, and **single named tests**.
**Not** the floor, **not** clippy — those are measured centrally after the tree is quiescent.

---

## THE WORK — five parts, each fully determined

### PART 1 — `examples/console-demo/wat/main.wat`

I have already driven this to a **proven** end-state: `--check` exit 0, run exit 0, five lines on
stdout, **empty stderr**. Reproduce it.

Replace the enum declaration (currently lines 29–38) with:

```
(:wat::core::defenum :demo::Event :wat::enum::Pure
  :Buy          [price <- :wat::core::f64  qty <- :wat::core::i64]
  :Sell         [price <- :wat::core::f64  qty <- :wat::core::i64  reason <- :wat::core::String]
  :CircuitBreak [reason <- :wat::core::String])
```

That is **four** changes, and the retirement remedy only names the first: `enum`→`defenum`; a
**mandatory purity marker** (`:wat::enum::Pure` — these variants hold only data); variant names
become **keywords** (`:Buy`); fields become **binder vectors** (`[name <- Type]`). Canonical live
reference: `wat/telemetry.wat:35-37`.

Replace everything from the `;; ─── Wiring` comment to end-of-file with:

```
;; ─── Wiring — five events, every one through ambient `println`.
;;
;; ⚠ THERE IS NO BENIGN STDERR WRITE. `:wat::kernel::eprintln` is
;; wat's PANIC channel (`wat/kernel/diagnostics.wat:52`; registered
;; in `src/check.rs` as a TERMINATING form): it emits to stderr and
;; then TERMINATES the program non-zero. This demo used to route
;; ":warn / :error" events through it as though it were a second
;; ordinary print. It is not, and that version died on its first
;; "concerning" event without ever reaching the last one.
;;
;; What the substrate actually offers is the IPC triangle:
;;   stdout     — complex RETURN values (this demo's five events)
;;   stderr     — complex ERROR values (panic cascades; terminating)
;;   exit code  — a SIGNAL telling the parent which channel to read
;; So a program that has something to SAY says it on stdout. Only a
;; program that is DYING writes to stderr, and it dies as it writes.
;;
;; The ambient ops EDN-encode each value and write one line per
;; call, so every emission round-trips through `:wat::edn::read`.
;; `:user::main` returns bare `nil` (arc 170 slice 1e entry shape).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
      [_a (:wat::kernel::println (:demo::Event::Buy 100.5 7))
       _b (:wat::kernel::println (:demo::Event::Sell 102.25 3 "stop-loss"))
       _c (:wat::kernel::println (:demo::Event::Buy 99.0 12))
       _d (:wat::kernel::println (:demo::Event::CircuitBreak "spike-volume"))
       _e (:wat::kernel::println (:demo::Event::CircuitBreak "exchange-disconnected"))]
      nil))
```

Note the last line: bare `nil`, not `:wat::core::nil` — Doctrine 1 (arc 242), a **type** keyword is
not a value. Keep the file's header comment block (lines 1–25), but **its last paragraph is now a
lie** ("Format selection … no longer applies" is fine; the `2>&1` / `2>err.log` run recipes are not,
because nothing writes to stderr any more). Make those run-recipe lines true.

### PART 2 — `examples/console-demo/src/main.rs`

Its doc says the demo *"renders a tiny domain enum five ways (EDN / NoTagEdn / Json / NoTagJson /
Pretty)"*. That surface was retired by **arc 170 slice 1f-η** — and this file's own sibling
`wat/main.wat` says so in its header, eleven lines away. A **sixth** lying doc. Make it true.

### PART 3 — `examples/console-demo/tests/smoke.rs` — **NEW, and this is the root-pull**

Mirror `examples/with-loader/tests/smoke.rs` exactly in shape: spawn `env!("CARGO_BIN_EXE_…")`,
assert the status is success, assert stdout equals the five lines verbatim. The proven stdout is:

```
#demo.Event/Buy [100.5 7]
#demo.Event/Sell [102.25 3 "stop-loss"]
#demo.Event/Buy [99.0 12]
#demo.Event/CircuitBreak ["spike-volume"]
#demo.Event/CircuitBreak ["exchange-disconnected"]
```

Assert **stderr is empty** too — that is the assertion that would have caught the semantic rot, and
the reason this gate is not redundant with the checker wall.

You will need `[[test]]`/`tests/` wiring in `examples/console-demo/Cargo.toml` if with-loader's has
any; read that Cargo.toml first and mirror it.

### PART 4 — `crates/wat-edn/wat-edn-clj/wat/shared.wat` — LIVE, so FIX it

It uses `:wat::core::define`, retired by **arc 241 Stone 241.11** — *the same stone that retired
`enum`, which also swept this exact file's sibling and missed both.* This file is **live**: Clojure
loads it as a schema via `(wat/load-types! "shared.wat")`, documented at
`crates/wat-edn/docs/IPC-BRIDGE.md:286,357`, `crates/wat-edn/docs/USER-GUIDE.md:551,644`, and
consumed by `crates/wat-edn/interop-tests/`. Bring it to the current language until
`./target/release/wat --check` exits 0. The first error is a `:wat::core::format` program-body eval
failure at line 34.

Its comment says a variant *"should be ignored by the scanner"* — **preserve that intent**; the file
is a fixture whose shape is the point. Change the spelling, never the shape it is demonstrating.

### PART 5 — `wat-migrate/fix-decl.wat` — a DECLARED CORPSE, so DELETE it

Two independent epitaphs, both on disk:

- `docs/arc/2026/06/251-types-as-forms/DESIGN-STONE-251.5-4.1-declaration-migrator.md:4` —
  *"`:migrate::` ns, **non-blessed**, **retires at the hard-cut**."*
- `tests/resolve/probe_arc251_decl_migrator.wat:1` — *"Migrator code **baked in** from
  wat-migrate/fix-decl.wat at migration time (arc 251 **throwaway**…)."*

It was declared disposable, its hard cut has passed, and its code is **already preserved** in the
test that consumed it. `git rm wat-migrate/fix-decl.wat` and remove the now-empty directory.

**One citation must be repointed, not deleted:** `src/types.rs:2770` cites
`wat-migrate/fix-decl.wat:27` for a `[kw <- :wat::WatAST] -> :wat::WatAST` example. Repoint it at
the baked-in copy in `tests/resolve/probe_arc251_decl_migrator.wat` — **verify the cited line
actually holds that shape there** before you write the new citation. (A prose citation names a
SYMBOL, not a LINE — `03585e2be`.)

### PART 6 — THE WALL — `tests/lint/every_ungated_wat_checks.rs`

Mirror `tests/lint/every_tracked_wat_parses.rs` for the walk, but **type-check** instead of parse.

Scope, and derive it in the test rather than hard-coding a file list: every **tracked** `.wat`
whose path does **not** start with `wat/`, `wat-scripts/`, `tests/`, `wat-tests/`, or `docs/`.
Those five are already gated or are history:
`wat/` + `wat-scripts/` by `every_wat_scripts_file_loads`; `tests/` and `wat-tests/` by their own
`.rs` drivers (and several hold **deliberately-bad** fixtures — do not walk them); `docs/` is record.

**⛔ NON-VACUITY IS MANDATORY.** Assert the walked set is **non-empty** and name the count in the
failure message. A wall over a directory that may be deleted later must go **RED** when the corpus
disappears, so that deleting it is a **deliberate** act and not a silent disarming. A gate that
cannot fail is a claim.

Prove the wall works by **breaking a door**: after it passes, temporarily reintroduce one retired
form into one walked file, confirm the wall goes RED and **names that file**, then restore. Report
the red's text. `NISI FRANGAS, NIHIL PROBAS.`

---

## STOP triggers — each rejects; none permits a lesser delivery

1. **STOP-1 — `shared.wat` cannot reach `--check` exit 0 without changing the SHAPE it demonstrates.**
   It is a scanner fixture; its odd shape is its content. Report the conflict; ship nothing for Part 4.
2. **STOP-2 — `src/types.rs:2770`'s cited example is NOT present in the baked-in copy.** Then the
   corpse holds something the record does not, and the delete is not yet grounded. Report; do not
   delete.
3. **STOP-3 — the wall's derived scope pulls in a file that is deliberately bad.** Report the file
   and how you derived it; do not add a name-based exemption. An allowlist of known-bad files rots
   into a permanent excuse.
4. **STOP-4 — a room's line number does not hold what this brief says.** Written against `a1aa347e1`.

---

## Acceptance — every row DERIVES its bar

⚠ **No row here names a magnitude I typed.** Yesterday's brief pinned `docs/arc` at "204" beside a
command that returns 164, and all three of its figures were unreproducible. A bar must be **zero**,
**equality**, or **whatever the command itself returns at the start**. Never a number from prose.

```bash
# 1. the three fossils are green, and the whole ungated set is green.
#    BAR: every line reads 0. Derived, not typed.
for f in $(git ls-files '*.wat' | grep -vE '^(wat|wat-scripts|tests|wat-tests|docs)/'); do
  ./target/release/wat --check "$f" >/dev/null 2>&1; echo "$? $f"; done | sort | uniq -c -w2

# 2. the example RUNS — the claim a checker cannot make.
#    BAR: exit 0, five lines, empty stderr.
cargo build --release && ./target/release/console-demo; echo "EXIT=$?"

# 3. the corpse is gone and nothing dangles.
#    BAR: zero.
git grep -c 'wat-migrate' -- ':!docs/arc' | wc -l

# 4. the wall exists, runs, and is NOT vacuous.
cargo test --release --test lint every_ungated_wat_checks

# 5. the builds that reach macro expansion.
cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each acceptance command's **actual output**, and which command produced each number. If a number
  disagrees with anything in this brief, **the brief is wrong** — say so.
- **The broken-door proof for the wall**: what you reintroduced, the RED's verbatim text, and
  confirmation it named the file.
- The `shared.wat` before/after for every line you changed, and why each change preserves the shape
  the fixture demonstrates.
- The `src/types.rs:2770` citation before/after, with the evidence that the new target holds it.
- Anything the brief got wrong.
- What you did NOT do, and why.
