# DESIGN — the stdlib vends only `:wat::`

**Drawn 2026-09-18**, builder-ruled, verbatim:

> *"it must be `:wat::repl::*` — we must only vend `:wat::*`"*

**NOT STRUCK.** This is the root fix for the UNSOUND verdict in
`can-a-user-def-change-a-stdlib-verdict/FINDING.md` (`52b5574ac`).

## Why — three names, and they are the entire attack surface

`wat/repl.wat` defines, at top level, in a **non-reserved** namespace:

```
:repl::turn  ·  :repl::eval-form  ·  :repl::eval-and-loop
```

Only `:wat::` is reserved. So these three are **vended into every wat program on earth** — proven, not
inferred: a plain user program calling `(:repl::turn "x")` gets a **TypeMismatch**, not an unresolved
reference. The name resolves. Every program carries it whether it wants it or not.

⛔ **And they are exactly the hole.** The spike instrumented the stdlib check sweep: **30,964 name-probes,
3,060 distinct names, and precisely THREE that a user could legally declare — these three.** A user
`(:wat::core::defclause :repl::turn …)` changes the verdict of five call sites *inside* `wat/repl.wat`
(exit 3, five `NoMatchingClauseAtCallSite`). Same three names also falsified Tier A's tidy argument about
stdlib expansion. **Three appearances, one root.**

⭐ **The prize beyond correctness:** renaming them takes the user-declarable surface **3 → 0**, which
makes Tier B's soundness *structural* rather than argued — the 126 ms becomes reachable by construction.

## The work

1. **Rename** the three to `:wat::repl::turn` / `:wat::repl::eval-form` / `:wat::repl::eval-and-loop`.
2. **The `.wat` side goes through the codemod** — `holon/CLAUDE.md` is explicit; census first, dry-run
   diffed, idempotent, recorded in `wat-scripts/fixes/`. Files: `wat/repl.wat`,
   `wat-scripts/demos/stdio-service/stdio-service.wat`, `crates/wat-edn/demo/repl-daemon.wat`,
   `wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat`.
3. ⛔ **`src/distribution/mod.rs:148` embeds a PROGRAM STRING** that calls `(:repl::turn …)` — that is how
   a distributed binary gets its REPL (`:140`–`:148`). **Miss it and every distributed binary breaks.**
4. **A GATE that keeps the ruling true**: a test asserting the frozen stdlib vends **zero** non-`:wat::`
   top-level names. ⚠ **Measure it from the SYMBOL TABLE, not with grep** — grep already got this wrong
   once: unanchored it reported six offending files (`:myapp::`, `:my::`, `:weather::`, `:usr::`,
   `:user::`) which were doc-example bodies and quasiquotes; anchored it reported the three real ones.
   A gate built on the same instrument would enshrine the same error.

## What this makes obsolete, and it is worth saying

`src/check.rs:848` currently explains that the spike attributes bodies by **source file** rather than by
name prefix, *because* — quote — *"the stdlib is NOT all `:wat::`-prefixed (`wat/repl.wat` defines
`:repl::turn`), so a prefix test would have hidden exactly the three names that answer this spike."*
After this stone **a prefix test becomes valid.** Update that comment rather than leaving a note that
argues from a fact this stone removes.

## Trap-doors

1. ⛔ **The embedded program string in `src/distribution/mod.rs`.** Not a reference — a *generated
   program*. Test that a distributed binary still gets a REPL.
2. **`probe-repl-declaration-refusal.wat` is PRE-EXISTING** (`1eaad2840`, arc 170) and lives under the
   `every_wat_scripts_file_loads` gate. Someone already probed this area; read it before editing it.
3. ⛔ **`:wat::` is RESERVED — the rename may be refused by the very guard it is meant to exploit.**
   `ReservedPrefix` refuses *user* definitions in `:wat::`; `wat/repl.wat` is stdlib, so it should be
   allowed — but verify, and if the guard cannot tell stdlib from user here, **that is the finding** and
   it is a bigger one than the rename.
4. **Do NOT build Tier B.** Its soundness becomes reachable, not proven — the spike must be re-run
   against the renamed tree before anyone touches the sweeps.
5. **Do not "fix" `defclause`.** Its unconditional insert and dispatch precedence
   (`check.rs:800`/`:6042`) remain a real flaw independent of namespaces — a separate ruling, still open.

## Out of scope

Tier B · the `defclause` guard · the queue promotion (unblocked by Tier A, awaiting its own re-weigh).
