# BRIEF — Stone 214.4.6a-i: the typed-peer foundation

> Prereq: Stone 4.6i-lexer (primed generic heads lex). DESIGN:
> `DESIGN-STONE-4.6-POLYMORPHIC-VERBS.md` § "Grounded template + the foundation split".
> Probe (committed, RED): `tests/nursery/probe_arc214_stone46i_typed_peer.rs`.

`spawn-program'` (Stone 4.5) is runtime-complete but CHECK-invisible: it has no
inference, so its result cannot be typed, annotated, or projected from. This
stone gives the peers their types. The verbs (4.6a-ii) project from them next.

## The rooms (read in order)

1. `src/check.rs:10423` — `fn infer_make_channel`: THE TEMPLATE. Reads its
   type-keyword arg, returns `TypeExpr::Tuple([Parametric{head, args:[T]}, …])`.
   Your new inference mirrors this shape exactly (arity check → arg reads →
   `CheckResult` with `Parametric`).
2. `src/check.rs:3666` — `fn infer_list`: the keyword-head inference dispatch.
   The `":wat::kernel::spawn-program'"` arm wires in here, next to its kernel
   siblings.
3. `src/check.rs:~1915-1958` — the kernel parametric-head registry region
   (`LEGACY_KERNEL_QUEUE_NAMES` + where `:wat::kernel::*` parametric heads like
   `QueueSender<i64>` are recognized). Register `:wat::kernel::Thread'` and
   `:wat::kernel::Process'` as valid parametric heads (2 type params each)
   wherever heads are validated — mirror how `:wat::kernel::Thread`/`Process`
   (unprimed, live in `wat/test.wat:486/707`) are recognized.
4. `src/kernel/spawn.rs:123/127` — `THREAD_PEER_TYPE_PATH`/`PROCESS_PEER_TYPE_PATH`
   (`":wat::kernel::Thread'"` / `":wat::kernel::Process'"`) — the runtime
   sentinels; the check-side type heads must match these strings (minus the
   leading `:` per `Parametric.head` convention — confirm against
   `parse_type_expr(":wat::kernel::Thread'<wat::core::i64,wat::core::i64>")`,
   which already returns `Parametric{head:"wat::kernel::Thread'", args:[…]}`).
5. `src/value/value.rs:951/1009` — `RustOpaque.type_path` + `declared_type_name`;
   `src/runtime.rs:5220` — `Value::RustOpaque(_) => ":rust::opaque"`. Route a
   peer opaque (type_path == one of the two sentinels) to report its own
   type_path instead of the generic `:rust::opaque` (the `type` verb + future
   clause dispatch read this).

## The work

**Part 1 — `infer_spawn_program_prime` (NEW, in check.rs near its kernel siblings).**
`(:wat::kernel::spawn-program' :tier env program)` — 3 args:
- args[0]: tier keyword LITERAL (`:thread` | `:process`). Read syntactically
  (it is a `WatAST::Keyword`). Unknown tier → `MalformedForm` (mirror the
  runtime's message at `src/kernel/spawn.rs:226-238`).
- args[1]: the program-env — infer it (recurse) and require it unifies with
  `:wat::program::Env` (the existing alias; see
  `tests/nursery/probe_arc214_slice4_stone1_program_env_typealias.rs` for its
  shape).
- args[2]: the program fn — infer it; from its fn type project the single param
  type `I` and the return type `O`. STOP-1: if the inferred type of args[2] is
  not directly a fn type carrying param/return (e.g. the Fn TypeExpr shape is
  not projectable here), STOP and report the exact TypeExpr you got — do not
  invent a fallback.
- Result: `Parametric { head: "wat::kernel::Thread'", args: [I, O] }` for
  `:thread`; `Process'` for `:process`.
Wire the dispatch arm in `infer_list`.

**Part 2 — head registration.** `:wat::kernel::Thread'<I,O>` / `Process'<I,O>`
accepted wherever parametric heads are validated, so the probe's return-type
annotations check. Mirror the unprimed `Thread`/`Process` handling.

**Part 3 — `declared_type_name` routing.** A `RustOpaque` whose `type_path` is
one of the two peer sentinels reports that type_path (runtime.rs:5220 region +
`value.rs:1009` if both routes exist — keep ONE authority; follow
`Value::declared_type_name`'s existing structure).

## Verify (report exact numbers)

- `cargo test --release --test nursery probe_arc214_stone46i_typed_peer` →
  **5 passed**. Probes 4/5 are the DISCRIMINATORS (measured 2026-06-07: an
  unknown head infers a fresh var that unifies with ANYTHING, so the positive
  probes 2/3 are vacuously green pre-foundation): a wrong scalar return
  annotation and a cross-tier annotation MUST be check errors once the real
  inference exists.
- `cargo test --release --test nursery probe_arc214_lexer_primed_generic_head` →
  still **3 passed** (no lexer regression).
- `cargo test --release --lib -p wat` → green band (~943/0/1).
- `cargo clippy --release` → no new warnings in touched files.

STOP triggers are rejection criteria: if a part cannot be built as specified,
ship nothing for that part and report the gap precisely. Do NOT commit — the
orchestrator scores and commits.

## Expectations (orchestrator scores vs own re-run)

| # | Claim | Check |
|---|---|---|
| 1 | foundation probe 5/5 (incl. negatives 4/5) | orchestrator re-run |
| 2 | lexer probe still 3/3 | orchestrator re-run |
| 3 | lib band green | orchestrator re-run |
| 4 | `:process` tier infers `Process'` (not `Thread'`) | probe 3 distinguishes |
| 5 | declared_type_name: peer reports its sentinel | read the diff; unit reasoning |
| 6 | no new clippy in touched files | clippy grep |
| 7 | tree dirty (no commit) | git status |

Runtime band: 20–35 min (new inference fn + registry + routing).
