# BRIEF — STONE 294.g: the holon record's wire is PLAIN EDN

Make a holon record cross the wire as **the class tag and its fields** — identical to a plain record —
instead of as its serialized hologram. The hologram is a *derived index*; the receiver rebuilds its own.

The committed probe `tests/comms/probe_arc294_holon_wire_is_plain_edn` is the contract. It is RED at
HEAD — **3 passing, 1 failing** — and must be 4/4 green with the three green rows still green.

## Read in order

1. **`tests/comms/probe_arc294_holon_wire_is_plain_edn.{rs,wat}`** — the contract. Read the target off
   this, not off this brief. Row 1 (`control_plain_record_…`) is a plain record and **already produces
   the exact shape row 2 must produce**, modulo the class name.
2. **`src/edn_shim.rs:3785` — the ENCODE arm.** `Value::Aggregate(a)` dispatches
   `match &a.holon { HolonForm::Hologram(h) => Tagged(tag, holon_ast_to_edn(h)), HolonForm::Empty => <named-field map> }`.
   **The two arms collapse into the second.** Read the `Empty` arm carefully — it is the whole target
   implementation, already written, already correct; the field names come from
   `types.get(&type_key)` as an `Aggregate` def.
3. **`src/edn_shim.rs:3177` `reconstruct_holon_record`** — the DECODE side. Today it expects a
   `Bind(_, Bundle(children))` body and projects fields out of the hologram. After this stone the body
   is a **field map**, so this projection dies and the fields come from the map exactly as a base
   record's do.
4. **`src/edn_shim.rs:2960-2975`** — the decode dispatch that routes to `reconstruct_holon_record`.
   **This is the load-bearing seam**: see the contract note below.
5. **`src/runtime.rs:15684` `build_holon_hologram`** — the derive side (`f301a6fc`). The receiver calls
   this to rebuild the index from the fields. It already exists; do not write a second one.

## The ONE contract decision, and where it bites

The encode arm's own comment states today's discriminator:

> *"The body is a `#wat-edn.holon/Bind[...]` value (NOT a map) **so the decode path can distinguish
> holon records from base records (which have Map bodies)**."*

**Body shape is currently how the decoder knows a record is holon-held.** Collapse the encode arms and
that signal is gone — both are maps. The decoder must instead ask the **type registry**: look up the
class and check its `Nature`, the same lookup the `Empty` arm already does for field names
(`types.and_then(|t| t.get(&type_key))` → `TypeDef::Aggregate(def)` → `def.nature`).

That is the stone. Everything else follows from it.

## Implementation sketch

```rust
// ENCODE (edn_shim.rs:3785) — one arm, not two. Delete the HolonForm dispatch.
Value::Aggregate(a) => {
    let type_key = format!(":{}", a.class);
    let tag = tag_from_type_path(&type_key);
    // …the existing HolonForm::Empty body, verbatim, for BOTH natures…
}

// DECODE — distinguish by REGISTRY, not body shape.
// holon-held class  → build fields from the map, then derive: build_holon_hologram(...)
// base record       → build fields from the map (unchanged)
```

## Blast radius

`src/edn_shim.rs` and whatever the decode path needs to reach `build_holon_hologram`. **This changes an
observable wire form**, so expect fallout in tests and `.edn` goldens — that is the work, not a
surprise. Do not chase it into `src/runtime.rs`'s derive logic; that side is correct.

## STOP triggers — surface and ship nothing; never work around

- **STOP-1 — the decoder has no `TypeEnv`.** If the routing site reaches `reconstruct_holon_record`
  with `types == None`, the registry lookup is impossible and the discriminator cannot move there.
  STOP and report the call path — do NOT fall back to sniffing the body, and do NOT invent a marker
  key inside the map. Either would reintroduce the coupling this stone removes.
- **STOP-2 — `holon_ast_to_edn` / `edn_holon_tag_to_ast` still have live callers.** They may: the
  `#wat-edn.holon/*` vocabulary also serves the **arc-093 substrate-internal HolonAST round-trip**
  (`edn_shim.rs:2862`), which is NOT this stone's target and must keep working. Report the surviving
  caller list; do not delete either function on the assumption it is now orphaned.
- **STOP-3 — a `.edn` golden or test pins the hologram wire form.** Expect several. Do **not** decide
  which are defects. Report every file:line with the old and new string. Some encode the thing we are
  deleting (update) and some may encode a real contract (ruling). That call is the orchestrator's —
  the identical STOP fired on stone 279.2 and the answer was "real contract, revert the change."
- **STOP-4 — the probe would need changing to pass.** The probe is the contract. **Do not edit
  `tests/comms/probe_arc294_holon_wire_is_plain_edn.*`.** In particular the two non-vacuity rows must
  keep passing untouched: if `still_measures` or `still_discriminates` goes red you have deleted the
  hologram rather than derived it, which is the opposite of this stone.

## Your gate — THE FULL FLOOR, deliberately

```
scripts/floor.sh
cargo clippy --release --all-targets
```

**Read the Summary line from `scripts/floor.sh`, never a piped exit code.** Baseline is
**4408 run / 4407 passed / 1 failed** — the 1 is this stone's own probe row, red by design.

**Why the whole floor and not a narrow filter:** this stone changes a wire format, so its blast radius
is precisely the thing a narrow gate cannot see. On stone 279.2 the rider's eight-test gate came back
green while the floor was 28 red, and that brief's STOP-2 and STOP-3 — the same two above — could not
fire because the gate's scope could not observe them. **A STOP its own gate cannot see is a STOP that
cannot fire.** Yours can see them. Use it.

**⛔ On any red that is not the known probe row: do NOT re-run.** `scripts/floor.sh` has already kept
the untruncated log. Copy the failing test's whole stdout+stderr block **verbatim** — never a summary,
never a `| head` window — name the exact assertion or match arm that fired, and report it. "Timing",
"pre-existing", "unrelated to my change", "passes in isolation" are descriptions of your search, not
dispositions. A re-run that goes green destroys the only evidence.

## Working rules

Work in `/home/watmin/work/holon/wat-rs`; confirm with `pwd` first. Any path containing
`.claude/worktrees/` is harness state — never operate on one. Use
`git -C /home/watmin/work/holon/wat-rs` for git reads. **Do not commit, push, stash, or revert** —
leave the tree dirty for the orchestrator.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you, and no
notification is coming. Run every command in the FOREGROUND and block on it; your turn ends when the
numbers are in your hands, not when a command is launched.

## Report back

The floor Summary line verbatim · the clippy result · `git diff --stat` · every STOP hit with
file:line and verbatim evidence · any honest delta the brief got wrong.
