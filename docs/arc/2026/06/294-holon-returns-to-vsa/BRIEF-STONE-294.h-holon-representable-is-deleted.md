# BRIEF — 294.h · delete `HolonRepresentable`; `EdnRepresentable` is the wire trait

**You are a rider, not the orchestrator. Ending your turn ENDS you** — it does not suspend you, and
nothing will wake you. There is no notification coming. Run every verification in the **FOREGROUND**
and block on it: your turn ends when the numbers are in your hands, not when a command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. Use `git -C /home/watmin/work/holon/wat-rs` for any git
read. **Do not commit, push, stash, or revert** — the orchestrator owns integration.

Read `DESIGN-STONE-294.h-holon-representable-is-deleted.md` (this file's sibling) in full first. It
carries the seven measurements this work rests on, the four-questions grid the builder ruled from, and
the exact delete list with line numbers.

## The work, in one paragraph

`HolonRepresentable` has no production consumers: zero bounds in `src/`, zero calls to its methods
outside the file that defines it, and every process-tier type is already bounded on
`EdnRepresentable`. It survives only as the private implementation of eight container types' `to_wire`
— and nothing in production sends a container. Delete the trait, its eight impls, and the eight
delegating `EdnRepresentable` container shims that call into it. `String` and `Value` keep their
`EdnRepresentable` impls untouched; they are already plain EDN and they are the entire production wire
set. Then correct the doc comments in `src/comms/process.rs` that describe a `to_holon_ast` wire chain
which no longer exists — and did not exist before this stone either.

## Rooms — read in this order

1. **`src/comms/mod.rs:88–140`** — the two trait definitions and the doc block explaining the split.
   This is the contract you are collapsing to one trait. `EdnRepresentable` (`:102`) stays exactly as
   it is.
2. **`src/comms/mod.rs:148–159`** — `impl EdnRepresentable for String`. `to_wire` is `self.clone()`.
   **Untouched.** Read it so you can see what "already plain" looks like.
3. **`src/comms/mod.rs:788–815`** — `impl EdnRepresentable for Value`. **Untouched.** This is the
   exemplar: `value_to_edn_string_with(self, None)` / `edn_string_to_value(s)`. Its doc comment
   already states the rule this stone enforces.
4. **`src/comms/mod.rs:161–710`** — the deletion zone. Nine `HolonRepresentable` impls (String,
   HashSet, Vec, HashMap, tuples 2–5) interleaved with seven `EdnRepresentable` container shims and
   one shared tuple helper at `:701`. The design stone lists every line number.
5. **`src/comms/mod.rs:1025–1040`** — `WireError`'s doc names `HolonRepresentable::from_holon_ast` as
   its producer. Keep the type; correct the prose.
6. **`src/comms/process.rs:12, 13, 47, 53, 321, 710, 717, 1125`** — eight doc comments describing
   `T::to_holon_ast → write_holon_ast_tagged` and its inverse. No such call exists in that file.
   Correct them to name `T::to_wire` / `T::from_wire`.
7. **The eight test files** — the design stone's disposition table, and its body-based rule.

## Implementation sketch

```
1. src/comms/mod.rs — delete, in this order (bottom-up keeps line numbers stable longest):
     the tuple from_holon_ast helper (:701)
     impl HolonRepresentable for (T1..T5) / (T1..T4) / (T1..T3) / (T1,T2)
     impl EdnRepresentable  for (T1..T5) / (T1..T4) / (T1..T3) / (T1,T2)
     impl HolonRepresentable for HashMap<K,V> ; impl EdnRepresentable for HashMap<K,V>
     impl HolonRepresentable for Vec<T>       ; impl EdnRepresentable for Vec<T>
     impl HolonRepresentable for HashSet<T>   ; impl EdnRepresentable for HashSet<T>
     impl HolonRepresentable for String        (the EdnRepresentable one at :148 STAYS)
     trait HolonRepresentable (:134) and its doc block
2. Fix the WireError doc (:1030) and the EdnRepresentable doc block (:94-98) — both name the
   supertrait relationship that no longer exists.
3. src/comms/process.rs — correct the eight doc comments.
4. Tests, per the design stone's body-based rule. Grep each file for
   `HolonRepresentable|to_holon_ast|from_holon_ast|pair::<` and let the hits name the removals.
5. cargo build --release, then scripts/floor.sh, then cargo clippy --release --all-targets.
```

`write_holon_ast_tagged` / `read_holon_ast_tagged` (`src/edn_shim.rs:4265`, `:4274`) **stay** — they
are public API in `src/lib.rs:138` and remain the VSA `HolonAST ↔ EDN` round-trip. Likewise
`coerce_to_holon_ast` (`src/runtime.rs:18681`) and `edn_to_holon_ast*` (`src/edn_shim.rs:4050`,
`:4067`) are a different mechanism entirely — the VSA Bind/Bundle path. A name containing
`holon_ast` is not the trait.

## Blast radius

`src/comms/mod.rs` · `src/comms/process.rs` (doc comments only) · the eight test files named in the
design stone. **No other `src/` file.** No new types, no signature changes to `EdnRepresentable`,
`Sender`, `Receiver`, `Process`, `Select`, or `pair`.

## What you report

For each touched test file: the probes **removed** and the probes **KEPT**, by name. The gate's row 8
is that list — a reviewer must be able to see that no `.wat`-driven probe was taken. Plus the floor
Summary line verbatim, the clippy count, and the run/skip delta with the probes that produced it.

## STOP triggers — each is a rejection criterion. Ship nothing and report.

- **STOP-1 — a `HolonRepresentable` bound exists in `src/` that the census missed.** Name the
  `file:line` and stop. The census is the ruling's basis; a miss invalidates it, and the orchestrator
  re-plans rather than you working around it.
- **STOP-2 — a probe file's wat-driven tests share scaffolding with its Rust-cascade tests**, so the
  removal cannot be surgical. Report the file and the exact coupling. Leave the file alone.
- **STOP-3 — deleting the container impls breaks a compile outside `src/comms/` and `tests/`.** That
  is a consumer all seven measurements missed. Capture the compiler error **verbatim** and stop; it
  inverts the builder's ruling.
- **STOP-4 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated,
  ANSI-stripped log. Copy the failing test's entire stdout+stderr block **verbatim** — never a
  summary, never a `| head` or `| tail` window — and name the exact assertion or match arm that
  fired. There is no such thing as a known flake; a red is a red.

## Shape to copy

`BRIEF-STONE-294.g-holon-wire-is-plain-edn.md` and its `EXPECTATIONS` sibling in this directory —
this stone's predecessor, same arc, same flaw. Its SCORE is the format for yours.
