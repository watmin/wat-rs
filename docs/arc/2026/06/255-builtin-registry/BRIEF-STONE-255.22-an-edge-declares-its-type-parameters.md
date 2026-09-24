# BRIEF — STONE 255.22: an `extend-type` declares its type parameters

**Drawn 2026-09-24 against `main` @ `a3efefd09`.** Floor 6042/6042, clippy 0, census 215 non-zero,
delta **3 / RECOVERY 0**, ledger 215. **Executor tier: Opus.**

## Ruled 2026-09-24 — the form (B1)

The four questions put B1 against two alternatives (`SCORE-STONE-255.21-…`, and the conversation that
followed). The builder ruled B1: *"these forms feel obvious — i see no reason not to do this."* How the
builder read the hello-world is this stone's reading: **"greet needs to accept an instance of a Greets
who holds some type T."**

```wat
(:wat::core::extend-type :- [T] (:hello::Box :- [T]) (:hello::Greets :- [T])
  (greet [self] -> :T (:hello::Box/v self)))
```

The binder rides the form head. It is the call-position param-spec `(Head :- [T…])` the reader already
has, and it reads like Rust's `impl<T>`: *for any `T`, a `(Box :- [T])` is a `(Greets :- [T])`.*

## Why — measured

`extend-type` has no binder today. `(extend-type :wat::core::Vector (:wat::core::Seqable :- [T]))`
(`wat/seq.wat:83`) and every defservice edge `(Handle :- [T])` use a `T` that **nothing declares**. It is
a type parameter because of its spelling (`is_type_param_letter`). The hello-world measures it on today's
binary (probes in the session scratchpad, `b1/`):

| program (no binder) | `--check` |
|---|---|
| parameter spelled `T` | rc=0, prints `"hello, world!"` |
| same, the greeting added to an i64 | rc=1, `+ … expects i64; got String` |
| **identical, parameter spelled `Elem`** | **rc=1**: `Greets/greet: parameter #1 (receiver) expects :hello::Greets; got (:hello::Box :- [String])` |

**Renaming a type parameter changes the verdict.** This is also the prerequisite for C-b3: a generic edge
can be matched by unification only when its parameters are known (`SCORE-STONE-255.21-…`).

## The work

1. **The form.** `(:wat::core::extend-type :- [P…] <child-type> <target> <methods…>)`. Parse the binder
   wherever `extend-type` is read. Measure every such place: `splice_type_decls` and whatever else reads
   the form. Bodied and bodiless forms both take it.
2. **Well-formedness walls**, refused at registration with a named diagnostic:
   - every binder name **appears in the child type**: an edge cannot invent a parameter the child does
     not carry;
   - every name in the child or target that is **neither declared in the binder nor a known type** is
     refused. **A free letter is an error, not a parameter.**
3. **The binder is what makes a name a parameter here.** Edge registration and edge matching (255.15's
   `register_parametric_extension` / `parametric_extensions_of`, the Gap-1 arm in `assignable`, and any
   site that reads an edge's args) take the edge's parameters **from its binder**, not from
   `is_type_param_letter`. Scope: extend-type edges only. `is_type_param_letter`'s other roles are C-c.
4. **Migrate every generic `extend-type`** to declare its binder, by **wat-fix codemod** (dry-run, diff,
   apply, commit with a replay fixture; run it with `./target/release/wat`, never the stale `cargo wat`),
   including `.wat.bad` fixtures. A non-generic `extend-type` (concrete child and target) takes no binder.
   **Census the population per site first** and report it.
5. **defservice's emitted edges** (`wat/service.wat` ~:2891–:2918: `grantable-extend`, `dialable-extend`,
   `typedcap-extend`, and any other emitted `extend-type`) emit the binder: the service's own params plus
   its transport letter. **The letter's spelling (T/Xt) is unchanged.** Declaring it in the edge's binder is
   exactly this stone. Declaring it in the emitted *defns* is C-b2.
6. **Rows** (`tests/types/probe_arc255_22_*`):
   - the hello-world with `T`: rc=0, runs, prints;
   - ⭐ **the hello-world with `Elem`: rc=0, runs, prints.** Today it is rc=1;
   - the i64 lie, in both spellings: rc=1;
   - a binder name absent from the child: refused;
   - an undeclared free letter in an edge: refused.

   Show each row's pre-stone rc.

## STOP triggers

1. Step 3 needs the satisfier-method path (`satisfier_method_keys`, the letter-rewritten keys) changed to
   pass the Elem row → do what the edge path needs, and **report** the method-path change separately. If
   it grows beyond the edge-matching sites → STOP and report the shape. The coord transport binding is
   C-b3.
2. An `extend-type` in the corpus whose parameters cannot be declared honestly (e.g. a binder name the
   child genuinely does not carry, as with bare `Vector` extending `(Seqable :- [T])`) → **STOP, report
   each site verbatim.** Whether `Vector` becomes `(Vector :- [T])` there is a ruling, not a guess.
3. A legitimate program is refused by the free-letter wall → STOP, report it.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the Elem hello-world | rc=0 and prints; pre-stone rc=1 |
| the T hello-world | rc=0 and prints; pre-stone rc=0 |
| both i64 lies | rc=1 |
| the two walls | each rc=1 with its named diagnostic; pre-stone rc noted |
| census of generic `extend-type` without a binder, after | 0 in live code; survivors named with reasons |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` against a pre-change census · NEW 3 / RECOVERY 0 · ≤ 215 |

Runtime prediction: 3–4 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name the
  arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words on a pre-stone build from `a3efefd09`.
- `defservice`: expand at the form level (`wat-scripts/scratch-pad/255-17a-child-main-transport.wat`).
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents.

## Out of scope

C-b3 (the coord transport binding through the method path), C-b1b, C-b2, C-b4, C-b5, C-c (other uses of
`is_type_param_letter`).
