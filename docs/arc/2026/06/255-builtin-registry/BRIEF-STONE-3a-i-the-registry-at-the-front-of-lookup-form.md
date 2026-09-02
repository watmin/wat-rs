# BRIEF — STONE 3a-i: the registry at the front of `lookup_form`

Make `lookup_form` ask the registry, first among the builtin steps. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-3a-i-the-registry-at-the-front-of-lookup-form.md`.
RULING: `docs/arc/2026/06/255-builtin-registry/RULING-the-registry-is-the-sole-authority.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5119, HEAD `c0132e52c`.

## Read in order

1. The DESIGN, whole — especially § "The `Binding` question", which this brief does not settle for
   you, and § "The ordering question", which it does.
2. **`src/reflect/lookup.rs`** — `enum Binding` and `fn lookup_form`. Read the whole chain.
3. `src/intrinsic/mod.rs`'s `IntrinsicEntry` — what a registry row actually carries.

## The work

### 1 — THE CENSUS FIRST. It is the stone's first act, not its premise.

Before writing a line of the change, enumerate every consumer of `Binding`:

```
46 `Binding::` sites, measured:  reflect/verbs.rs 12 · runtime.rs 11 · intrinsic/reflect.rs 6
                                 special_forms.rs 2 · check.rs 2
 4 callers of lookup_form
```

For each: **what does it do with the variant?** Read the arm, do not infer from the variant's name.
Report the census as a table before you report anything else.

### 2 — pick the shape, against stated criteria

The DESIGN names three and deliberately does not pick:

```
A  a new  Binding::Registered { name, entry: &IntrinsicEntry }  variant
B  map a registry row onto Primitive / SpecialForm by `Kind`
C  populate doc_string from the registry; variants unchanged
```

**The criterion is the acceptance row "a schemeless row resolves":** 89 registry rows carry no
`TypeScheme`, and `Binding::Primitive` requires one. A shape that cannot represent a schemeless
registry row fails, because those rows are exactly what `lookup_form` cannot see today.

⚠ **Apply the criterion to your census and say what it selects, with the evidence.** If the census
shows the selected shape is infeasible — say, an exhaustive `match` on `Binding` in a place a new
variant cannot reach — **STOP and report that instead of forcing it.** The pick is the stone's
finding; a forced pick is worth less than an honest refusal.

### 3 — the registry consult

Insert it into `lookup_form` **after** user defines and macros, **before** the `CheckEnv`
construction and before `special_forms`. Steps 1 and 2 do not move.

★ Carry the registry's data across: a row has `prose`, `args`, `ret_type`, `examples`, `see`,
`added` and five axes. **`doc_string: None` at a branch that could answer is the defect this stone
exists to remove** — at minimum, `prose` reaches `doc_string`.

## Blast radius

`src/reflect/lookup.rs` · whatever the census says must change to accept the shape · **NOT
`src/special_forms.rs`**. No `.wat` corpus change. **No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — DO NOT DELETE ANY `src/special_forms.rs` ROW.** `git diff --stat src/special_forms.rs`
must be empty. Stone 1a-i measured that `and`/`or`'s rows are the ONLY path `lookup_form` has to them
today. After this stone they become dead — but **proven-dead is a floor result, not an argument**,
and their removal is 1a's stone, not yours.

**⛔ STOP-2 — DO NOT REORDER USER DEFINES OR MACROS.** They stay ahead of the registry. The DESIGN
measured that the runtime consults the registry before `sym.has_function`, and that the difference is
unreachable because `:wat::` is a reserved prefix — **that is a reason to leave the ordering alone,
not to change it.**

**⛔ STOP-3 — DO NOT REMOVE THE `CheckEnv` STEP.** It stops being the membership oracle; it does not
die. 89 names in `REGISTRY_MEMBERSHIP_GAP_A` still have a scheme and no registry row, and step 3 is
how `lookup_form` sees them. Removing it drops 89 names off the reflection surface.

**⛔ STOP-4 — A REGISTRY ROW WITH NO SCHEME MUST STILL RESOLVE.** If your shape makes a schemeless
row unrepresentable, the shape is wrong. This is the criterion, not a detail.

**⛔ STOP-5 — DO NOT TOUCH `is_reserved_prefix`.** The DESIGN names it as a live dependency of this
stone's safety argument. It is a later stone's subject. `grep -c "if is_reserved_prefix(head)"
src/resolve/walk.rs` must remain **1**.

**STOP-6 — you cannot compile.** If the census shows a shape needs a change you cannot verify —
an exhaustive match somewhere, a lifetime that will not hold — report it as unverified reasoning and
name the site, as the last five riders correctly did.

## Report

**The census table first** — all 46 sites, what each does with its variant. Then: which shape the
criterion selected and the evidence; the `lookup_form` diff verbatim showing where the consult sits
relative to steps 1–5; what registry data you carried into `Binding` and what you had to drop;
confirmation `special_forms.rs` is untouched and STOP-5's grep is 1. Then: **what surprised you** —
a `Binding` consumer that does something the variant name does not suggest, a site the census found
that `grep Binding::` would have missed, or a place where the registry has less than `CheckEnv` does.
