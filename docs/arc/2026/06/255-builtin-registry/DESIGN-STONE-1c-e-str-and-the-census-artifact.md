# DESIGN — STONE 1c-e: `str` — the last ordinary `:wat::core::` verb, and the census over-reports

## Two deliverables, and the second is the more valuable

**① Register `:wat::core::str`** — 135 corpus sites, the last ordinary verb in the namespace.
**② Re-derive the corpus census**, which is owed and which we now know **counts names that are
not verb population at all.**

## ① `str` — the simplest registration this campaign has had

```
:wat::core::str   135 sites   runtime.rs:2925  =>  eval_str(args, list_span, env, sym)
```

`eval_str` is **single-use** (that arm is its only caller) and already carries the canonical
`#[wat_intrinsic]` signature — **annotate in place, no wrapper, no extraction**. The first stone
since 1c-a-i where that is true.

⚠ **And it has NO checker knowledge whatsoever.** Measured exactly:

```
check.rs mentions of ":wat::core::str"   0
register_builtins scheme                 0
runtime.rs dispatch arm                  1
```

So there is no checker arm and no `TypeScheme` to mirror `@arg`/`@ret` against — the rider must
ground them in `eval_str`'s own body, and say what happens to a `(str x)` call at check time
today. That absence is itself a finding worth stating.

⛔ **`str` IS A PREFIX OF `struct`.** A census of `str` written without the closing quote matches
`:wat::core::struct`, `struct-new`, `struct->form` and returns 9 where the answer is 0 — this
orchestrator did exactly that while crawling this stone and caught it only by re-running with the
terminator. **Every pattern in this stone's work must be terminated.**

## ★★★ ② The census counts things that are not verbs — three instances now

The corpus experiment flips `is_resolvable_call_head` and collects the names that fail. It
inspects **list heads**, and three kinds of non-verb occupy that position:

```
:wat::type::{Tuple,i64,String,Vector}   4   a TYPE PATH in arc 251's dual-read spelling
                                            (`wat.type/Tuple` in source). Zero corpus text
                                            spells the `::` form; `types.rs:5172` strips it.
:wat::core::None                        1   a declared UNIT VARIANT of Option
                                            (`types.rs:1248`, `EnumVariant::Unit("None")`).
                                            Every corpus site is a match PATTERN — `(:None body)`
                                            — or a value. It is NEVER a call head.
```

Both are already answered by an authority the RULING **explicitly exempts**: *"`constructor_meta`/
`accessor_meta` DERIVE from the frozen `TypeEnv` … Derivation from one source is not
duplication."* Registering either as a `#[wat_intrinsic]` verb would manufacture a verb for a name
that already has an authority — the exact error 1c-0a caught for `println`/`edn::write`.

⚠ **So the number I have been quoting is wrong in a direction I did not check.** "43 names" was
arithmetic on the 2026-09-03 sweep filtered against today's registry — never re-swept, and never
audited for non-verbs. The honest remainder is smaller, and the census needs to say so itself
rather than be corrected by hand each time.

## Acceptance — DERIVED

```
                  before   after   why
registry rows       549     550    +1 attribute site (ANCHORED count)
GAP_A                49      49    `str` is not on it (no scheme exists to be known-about)
GAP_B                45      44    `str` IS on it
DEBT               118     119    +1 — no CheckEnv scheme, so the type gate will skip it
KNOWN_UNREVIEWED     13      13    `str` is not on it — CHECKED against the constant
literal arms deleted  —       1    the no-literal-arm gate will demand `runtime.rs:2925`
floor          5129/5129  5129/5129
the corpus         RE-DERIVED, not predicted — and reported with the non-verb count separated
```

★ The corpus row is deliberately **not** given a number. Every previous prediction of it was
arithmetic; this stone's job is to replace arithmetic with a sweep.

## Out of scope — CUT

- Registering `None` or the four `:wat::type::` rows. They are not verb population; naming them
  as census artifacts is the deliverable, not registering them.
- `=`/`not=` — held at `Partial` on the bounded-generics door.
- Changing the census procedure itself. Report what it over-counts; a fix to
  `is_resolvable_call_head`'s pattern-position blindness is its own question.
