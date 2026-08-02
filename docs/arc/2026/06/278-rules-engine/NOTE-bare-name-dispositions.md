# The remaining bare top-level names — sorted, with dispositions

**Measured 2026-08-01**, pattern positive-controlled against known ground truth before any count was
trusted (three earlier attempts returned `0`, `0` and `379` — a post-filter eating grep's `file:line`
prefix, a name class missing `:`, and backtracking dodging a lookahead).

**51 real occurrences across 21 files.** stdlib `wat/` = 0 · the `where` corpus = 0.

> The first census said 57/24. Six of those were **prose inside `;;` comments** in
> `wat-scripts/fixes/*.wat` (a codemod's documentation of the form it rewrites). Corrected by stripping
> comments before matching. 24c's recorded trap: *confirm a grep hit is CODE, not a comment.*

---

## ★ Class 1 — the `/` pseudo-namespace (29 of 51, 10 files) — NEEDS A RULING

These are not "forgot a namespace." They use a **different separator**:

| file | names |
|---|---|
| `wat-scripts/scratch-pad/probe-arc278-reap-which-link.wat` | `:rw/try` `:rw/row-tail-bare` `:rw/try-with-handle` … (9) |
| `wat-scripts/scratch-pad/probe-arc278-tco-drops-caller-env.wat` | `:tco/dial` `:tco/service-let-tail` … (6) |
| `wat-tests/counter-actor-proof-process.wat` | `:counter/dispatch` `:counter-proc/get` … (5) |
| `wat-scripts/scratch-pad/probe-arc278-reap-serve-event.wat` | `:se/serve` `:se/row-tail` `:se/try` |
| `wat-scripts/scratch-pad/probe-arc278-wire-type-enforcement.wat` | `:probe-wire/round-trip` `:probe-wire/measure-tier` |
| + 4 more, one name each | `:dos/try` `:tl/try` `:vprobe/render` `:probe-det/round-trip` |

**The tell is inside a single file.** `probe-arc278-reap-which-link.wat` writes its *types* as
`rw::Bag::Op` and its *functions* as `:rw/try` — `::` and `/` for the same intended namespace, in the
same breath.

`/` is the **accessor** form (`Thread/join-result`, `docs/CONVENTIONS.md:45`). So `:rw/try` reads as a
method on a type `rw` that does not exist.

**The fork, and it is the builder's:**

- **(a) `/` is the accessor form, misused.** Then these are real violations and become `:rw::try`. This
  is arc 179's shape exactly — *a second spelling of one thing is a second door around every wall built
  on the first* — and the same-file inconsistency is the evidence.
- **(b) `/` is an accepted informal namespace.** Then `is_namespaced` must accept it
  (`contains("::") || contains('/')`) and the wall being built right now is too strict.

**Leaning (a), and the wall as briefed implements (a).** If (b) is ruled, it is a one-line predicate
change — not a rebuild. Deliberately not interrupting the live rider over it: the stricter wall makes
all 29 self-identify, which is information under either ruling.

---

## Class 2 — genuinely single-word bare names (22 of 51, 11 files)

### 2a — deliberately-bad specimens · **rename, do not delete** (3 files, 6 names)

| file | names | why it must be renamed |
|---|---|---|
| `tests/types/probe_arc237_sA1_assignable_probe03.wat.bad` | `:feed` `:needs-circle` | must keep failing for **assignability** |
| `tests/types/probe_arc237_sA1_assignable_probe06.wat.bad` | `:feed-sq` `:needs-circle` | must keep failing for **assignability** |
| `tests/types/probe_arc237_sC3_macro_split_liskov_base_into_holon.wat.bad` | `:gb` `:wh` | must keep failing for **Liskov** |

These are supposed to fail. If the wall lands and they start failing for *"unnamespaced"* instead, the
property each one exists to pin is **masked** — the specimen stops being a specimen while the gate stays
green. That is 24s's finding (`wat_cli__check_bad.wat` had silently stopped being a bad program, and only
a re-specimen caught it). Renaming preserves the failure reason.

### 2b — subject-is-dead · **retire** (1 file, 1 name)

`wat-scripts/scratch-pad/probe-bare-defrule-name.wat` — its own header: *"probe: is a bare
(non-namespaced) defrule name legal, and does the derived `Rule/name` come out bare?"* Throwaway
reconnaissance for the `defrule` migration, which landed in `b096e779`. The wall reverses its answer.
**Do not rename it** — renaming destroys the specimen and leaves a probe asserting nothing.

### 2c — plain fixtures · **mechanical rename** (7 files, 15 names)

`tests/types/probe_arc237_sC3_macro_split.wat` (`:fb :fh :gh :wb :wh`) ·
`probe_arc237_sA1_assignable_probe01/02/04.wat` (`:force :needs-record :force2 :two :passthru`) ·
`tests/resolve/probe_arc251_stone5a_{read_string,write_forms}.wat` (`:f`) ·
`probe_arc251_read_file_ladder__content.wat` (`:x`) ·
`probe_arc251_decl_migrator.wat` (`:Foo<T>`).

**`:Foo<T>` is worth its own line** — a *parametric* bare name. `contains("::")` rejects it correctly,
and the rename target is `:mig::Foo<T>`; the predicate must not choke on the `<T>` (it does not —
containment is blind to it).

---

## How the fix ships

**Not a hand-edit.** A `.wat` corpus rename is a `wat-fix` codemod (`wat/fix.wat` + `wat-scripts/fixes/`,
CLAUDE.md item 1) — `rename-keyword-exact` per name, expressed as a **`foldl` over a `Vector` of
`(old,new)` tuples, never a nested staircase** (24t's lesson; the staircase's paren count stopped being
eyeballable and was wrong twice). Dry-run onto a `/tmp` copy and `diff` before applying, then prove
idempotence by re-running.

**Extension enumeration matters here:** three of the files are `.wat.bad`. A `-name '*.wat'` glob
silently excludes them — 24t's five-surfaces lesson, where one glob hid 243 files.

**Order is free.** The wall fires only on a *first* registration (`Equivalent → NoOp` short-circuits
ahead of it), and neither the stdlib nor the corpus holds an offender — so arming and clearing can land
in either order.
