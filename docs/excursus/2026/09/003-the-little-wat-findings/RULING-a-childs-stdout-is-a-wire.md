# RULINGS — 2026-09-24: a child's stdout is a wire · a rename moves Debug text

**Builder:** *"these have been reasoned - we continue"* — accepting both four-question passes below.
Context: *"wat.kernel/readln · println · eprintln — all of these only accept EDN … (we'll probably
need to concede eventually and support non edn forms to integ with the rest of the world; that's
not now)"*.

## 1. A spawned child's `println`/`pprintln` encodes STRICT (shipped in stone H, `2b3370387`)

In a spawned child, fd 1 is the self-peer channel (`run_forms_as_server_child`,
`src/process/verbs.rs`): what the child prints is what the parent's `recv` decodes as data. So a
child's print IS a wire send and follows the wire rule — a value with no wire form is refused at the
child's own print line, not delivered as a nil the parent then fails to decode. Flag set once at child
birth (`mark_stdout_as_peer_wire`, sole caller `process/verbs.rs:417`). Root process: unchanged,
`#… nil` per arc 294. Four questions: Obvious YES · Simple YES (one rule — writing to a wire is
strict) · Honest YES · Good UX YES. Known soft spot: the `WireFrame` type gate cannot see this case
(stdout carries both text and wire), so it is a runtime check.

## 2. A variant rename updates tests that pin Rust `Debug` text (stone J, `86a6d6762`)

`Value` derives `Debug`, which prints the Rust variant name. Two tests pinned `holon__HolonAST(`; the
rename moved them to `wat__holon__HolonAST(`. Four questions: all YES — internal Rust text, no wat
user sees it. The invariant for any rename: no EDN, wire or type-path string changes.
