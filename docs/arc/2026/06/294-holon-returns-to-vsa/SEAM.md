# SEAM — the ONE live breadcrumb. As of 2026-08-21 (`:-` shipped; the codemod is correct). Replaced in place.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `a9168b851`.** Run **`git log --oneline a9168b851..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4855/4855, 0 FAIL, 19 skipped, ~71s   (own invocation, scripts/floor.sh)
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …`. Read exit codes directly, never through a pipe.
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at RUST-compile time).
A rider CANNOT test one. A `wat-scripts/` script IS read from disk — that one a rider can test.

## ★★ THE WORK: ARC 109 — `:-`, THE PARAMETERIZATION OPERATOR

> *"the symbol ` :- ` is declaring **'this thing on the left is parameterized by the thing on the
> right'**… the same as arg-spec and ret-type."*

```clojure
[n :- wat.type/i64]                          arg-spec
:- wat.type/i64                              ret-type
(wat.type/Vector :- [wat.type/i64])          type args        — a REFERENCE, in parens
(wat.type/Vector :- [wat.type/i64] 1 2 3)    constructor
(wat.core/defn ns/f :- [T] [x :- T] :- T x)  declaration      — a BINDER, siblings, NO parens
```

**A binder is the reference form minus its parens.** The param-spec is a vec of TYPE REFS in a
**reserved** position — values are never legal there, which is why nothing sniffs it. `:- []` is the
**assumed default**: no monomorphic-vs-parametric distinction exists, only a param list usually empty.

**Each declaration consumes what it uses.** A surface's params are the UNION of its messages':
`Cache<K,V>` ⇒ `GetRequest<K>` · `GetResult<V>` · `PutRequest<K,V>`. Enforced by the consumption wall.

### Shipped today

```
②-i-b   `:-` accepted + emitted · Tuple brackets · nil stops canonicalizing   c9938cc7b
        ⚠ AMENDED 9741507da — shipped for 3 of 6 constructors and I scored it GREEN
α       8 Rust declarator heads take `:- [T …]`                               c5ac5174c
β-i     both `defrecord` macros                                               26669f8d9
β-ii-a′ `defservice`: binder is the SOURCE OF TRUTH, `<K,V>` a derived shim    bd898a748
β-ii-b  18 generated FUNCTION names drop `{p}`                                8e6e83618
β-ii-c  `type-params-used-in` intrinsic · `lru-svc::Record` monomorphic        8cbd9d4b7
wall    a type decl must CONSUME its param-spec                               1be3f6b5e
codemod THE SLOT RULE — a declaration name is a binder, not a reference        a9168b851
```

## ⛔ NEXT — ②-iii: apply the codemod to `wat/` ALONE

The first real corpus migration. The machine is assembled and proven on dry-runs:
names bare · references wrapped · nesting correct · **idempotent** · the 9,912 arrow sites unmoved
(80 → 80 in `spawn.wat`).

```
printf '["wat/a.wat" …]\n' | ./target/release/wat ./wat-scripts/fixes/parametrics-take-a-type-vector.wat
```

★ **Dry-run on `/tmp` copies and READ THE WHOLE DIFF before writing anything to `wat/`.** This
rewrites the stdlib in place.

## THE STILL-OPEN

- **β-ii-d** — `defservice`'s substring transport test (`contains? fqdn-tp "<T>"` / `"<T,"` / `",T>"`,
  three variants approximating one membership check). Cheap now: `fqdn-tp-syms` exists.
- **γ** — `defn`. TWO capabilities: the declaration binder, and **call-site type application**
  (`(ns/f :- [wat.type/i64] 42)`), which is REJECTED today and whose site count is **UNMEASURED**.
- ⛔ **A user parametric type has NO working FORM spelling.** `(:user::Box :- [i64])` fails where
  `(:wat::core::Vector :- [i64])` works — because `defrecord` mints a *macro* under the record's own
  bare name (`wat/Record.wat:197`), so the list is macro-expanded before the checker sees it.
  Measured: it works when referenced BEFORE the declaration. Blocks any stone that emits a form for
  a user type. Root is 251.8's one-node-two-roles.
- **③** — angle form ILLEGAL · delete `is_type_bracket_candidate` (ONE caller now).
- **`List/of` + `char/of`** retire into `List`/`char` (verb-equals-type; the playbook already ran for
  `vec`→`Vector`, `tuple`→`Tuple`). 72 sites, all tests/probes.
- **296 Stone H** now also closes `Some`/`None`/`Ok`/`Err` — they are enum VARIANTS. Fields stay
  `value`/`value`/`error`.
- **`macro-error` is the way a macro raises a structured failure** — `Option/expect` PANICS. The
  `defrecord` missing-field diagnostic is a recorded, cheap fix.
- Dead bindings `start-name` / `resume-name` in `wat/service.wat` — bound, never read.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **SCOPE THE CHECK FROM THE RULE, NOT THE DIFF.** ②-i-b shipped `:-` for three of six
> constructors and I scored it GREEN: every new-spelling row I ran landed on the half the rider's
> diff had touched, and every row on the other half used the OLD spelling as a control. **A full
> green over that split reads exactly like a full green over everything.**
> `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`
>
> ⚠ **A CENSUS SCOPES WORK IN; IT NEVER SCOPES WORK OUT.** A regex over `.wat` source certified a
> new wall would find zero violations. It found three real defects — one in GENERATED code that
> appears in no file, four hand-written, and one FALSE LAW in a comment claiming "checker-locked"
> for a rule the checker never had.
>
> ⚠ **I OPTIMIZED A STRING AND DESTROYED ITS SHAPE.** Replacing a bad diagnostic with
> `Option/expect` gave a better message and a PANIC instead of an error VALUE. The test that caught
> it was not asserting on the message.
>
> ⚠ **F5: a macro body may not call a user-defined function AT ALL** — refused at DEFINITION, 3029
> tests red. Read `109/NOTE-the-F5-allow-list-and-what-a-macro-body-may-call.md` first.
>
> ⚠ **NEVER `git add -A` WHILE A RIDER IS IN THE FIELD.**
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.`
