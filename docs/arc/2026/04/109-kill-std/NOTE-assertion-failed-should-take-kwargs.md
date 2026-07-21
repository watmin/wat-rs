# NOTE (arc 109 cleanup) — `:wat::kernel::assertion-failed!` should take KWARGS, not 3 bare positionals

**Filed 2026-07-20 (builder catch, mid arc-278 #16 — the RTL migration fleet was spreading the positional form).**
Queued, NOT built — its own stone; a codebase-wide call-site migration, orthogonal to #16's RTL work.

## The finding (grounded)

`:wat::kernel::assertion-failed!` is a **strict arity-3 POSITIONAL** primitive — `(message actual expected)`:
- `src/assertion.rs:107` — `eval_kernel_assertion_failed`; `if args.len() != 3 { ArityMismatch }`.
- `src/assertion.rs:88` — doc: `message: :String`, `actual: :Option<String>`, `expected: :Option<String>`.
- registered in `src/check.rs:16583`.

The common call for a plain "fail with a message" (no equality context) is `(:wat::kernel::assertion-failed! "msg" :wat::core::None :wat::core::None)` — **two bare positional `:None`s**. The builder's catch (2026-07-20): *"what are the last 2 args to assert failure? they need to be kwargs — if not all."* Right: you cannot read which `:None` is `actual` and which is `expected`; it's the exact positional-confusion [[feedback_kwargs_categorically_superior_positional_only_when_necessary]] forbids (kwargs categorically superior; positional only for trivial forms).

**Scale:** **86 calls across 27 files** (positional today) — `wat/test.wat`, `wat/kernel/hermetic.wat`, `wat/bracket.wat`, and many `tests/**/probe_*.wat` (including the arc-278 #16 RTL arms added this session). All matched the established positional idiom.

## The fix (when it comes) — the arc-294 kwargs pattern

Apply the aggregate-kwargs shape ([[project_294_9a_kwargs_flip_blast_radius]], [[feedback_migrations_encourage_good_form_never_spread_escape_hatch]]): the **bare name `:wat::kernel::assertion-failed!` becomes a kwargs macro** (`:message` / `:actual` / `:expected`) that realizes into a **primed positional** primitive (`:wat::kernel::assertion-failed!'`, or a `:rust::`-tier verb). Because the bare name flips from positional to kwargs, **every one of the 86 sites migrates atomically with the rename** (`("msg" :None :None)` → `(:message "msg" :actual :None :expected :None)`) — a mechanical, wat-fix-able sweep. `:actual`/`:expected` default to `:None` so a plain fail is `(assertion-failed! :message "msg")`.

## Why deferred (not #16's scope)

#16 is the RTL / no-hidden-failures migration. Kwargs-ifying a core assert primitive + migrating 86 unrelated call sites is a distinct concern; folding it in would blow #16's blast radius and mix two migrations. #16's new RTL arms use the current positional form (consistent with the 86 existing sites); this stone migrates them **all together** later. Low-priority, mechanical, named-not-lost.
