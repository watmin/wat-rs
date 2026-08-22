# SEAM — the ONE live breadcrumb. As of 2026-08-21 (②-iii RAN, went RED, and REVERTED). Replaced in place.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.** A hand-typed hash is an instrument that can be
> wrong, and this one WAS: it read `f0d3fb2`, which is **not a valid object in this repository**, so
> the first thing the next self runs could not run. Paste this — it has no hash to mistype:
>
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
>
> **Empty → nothing moved since this was written.** Non-empty → every commit listed outranks every
> line below, and you re-read those before you trust a word of it.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4854/4854, 0 FAIL, 19 skipped, 77.1s  (own invocation, scripts/floor.sh, at 2d25b4790)
                ⚠ 4855 → 4854 is CORRECT and accounted: the --check stone deleted
                `infer_fn_non_vector_args_returns_silent_placeholder` and renamed one
                other (2 removed, 1 added). A count that DROPS must be explained,
                never observed — if you floor and see 4854, that is green.
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

### Shipped

```
②-i-b   `:-` accepted + emitted · Tuple brackets · nil stops canonicalizing   c9938cc7b
        ⚠ AMENDED 9741507da — shipped for 3 of 6 constructors and I scored it GREEN
α       8 Rust declarator heads take `:- [T …]`                               c5ac5174c
β-i     both `defrecord` macros                                               26669f8d9
β-ii-a′ `defservice`: binder is the SOURCE OF TRUTH, `<K,V>` a derived shim    bd898a748
β-ii-b  18 generated FUNCTION names drop `{p}`                                8e6e83618
β-ii-c  `type-params-used-in` intrinsic · `lru-svc::Record` monomorphic        8cbd9d4b7
wall    a type decl must CONSUME its param-spec                               1be3f6b5e
codemod THE SLOT RULE — a declaration name is a binder, not a reference       a9168b851
②-iii   RAN · 3030 RED · REVERTED. Two blockers FIXED, the third documented   169459247
```

## ⛔ ②-iii IS BLOCKED. The codemod is fine; the SUBSTRATE is not ready.

**Read `109/NOTE-2iii-is-blocked-the-angle-string-is-the-type-identity.md` before touching this.**
The codemod ran over all 52 stdlib files, changed 36 (899 lines, 992 tokens), was byte-identically
idempotent, and matched its reviewed dry-run exactly. The stdlib then failed to load — because in
three subsystems the substrate uses the **rendered string `Head<A,B>` as a type's IDENTITY** — and
in a FOURTH place the floor never reached, `defn` simply refuses the binder the codemod writes:

```
⛔ register_subtype     stores the string VERBATIM — the edge key IS ":…::Seqable<T>"     3a OPEN
⛔ transport_satisfier_heads  format!("{fq}<T>") / ("{fq}<Xt>")                           3b OPEN
✅ satisfies_bare_surface     format!("{surface}<") — a PREFIX match → family_extends      3c edb7f66c7
⛔ wat/service.wat         "<K,V>" as a STRING, re-attached as "{b}::Op{p}" — and EMITTED, 3d OPEN
                           so a migrated corpus regrows the angle form at every expansion.
                           ⚠ 2d25b4790 taught it to READ a migrated type; it re-serializes
                           the args straight back. Consumption done, EMISSION untouched.
✅ the :peers check        built "Peer<{r},{o}>" and string-compared it                    3e 2d25b4790
⛔ defrecord/defstruct  mint a COMPANION defmacro under the bare name (wat/Record.wat:197),
                           so `(:wat::cache::Entry :- [K V])` macro-expands before the checker
                           sees it. MEASURED four ways: builtin ✅ typealias ✅ defenum ✅
                           defrecord ⛔ defstruct ⛔ — it is the COMPANION MACRO, not
                           user-vs-stdlib. 6 types / 5 breaking refs in wat/. Root 251.8.
✅ defn / fn            NOW ACCEPT `:- [T …]` — γ-i (c889639aa) covers BOTH heads.
                           Re-measured 2026-08-22 with the angle form as positive control;
                           both exit 0. The line that said "defn alone rejects" is retired.
```

⛔ **Do NOT re-run the codemod on `wat/` until that is fixed.** It is not a partial-migration
hazard — it is a red floor.

## ✅ THE RIDER LANDED — `defservice` compares and destructures types as DATA · `2d25b4790`

Scored: `109/SCORE-STONE-defservice-compares-types-as-data.md`. Floor 4854/4854, clippy 0. It fired
**STOP-3 twice and was right both times** — my classification table was wrong at two of nine rows.

⚠ **It closed blocker 3e, NOT blocker 3.** See the ledger under NEXT.

## ✅ SHIPPED — arc 109

```
γ-i           `fn` AND `defn` take the `:- [T …]` binder (re-measured 08-22)          c889639aa
--check loud  a malformed fn no longer passes --check (an EXEMPTION deleted)       490c3c1e4
identity 1/3  `family_extends` gets its own door — PROVABLY behaviour-neutral      edb7f66c7
identity 2a   six DEAD bindings deleted (one a NAME COLLISION)                     41a3d0dd7
identity 2b   each ROLE gets its own binding — expansion BYTE-IDENTICAL            0366b2f2b
expr-slot     a type reference is not an EXPRESSION (expander + resolver)             b9df7a09a
              ⚠ was labelled "blocker 5" here — a DIFFERENT list from the NOTE's, whose
              blocker 5 is the defrecord COMPANION MACRO and is still OPEN. The collision
              is what let this seam claim ②-iii was re-runnable.
identity 2c   ALL 22 ANNOTATIONs emit the `:-` form                                073dda92c +2
type-equal?   the missing door: types are data everywhere EXCEPT in a macro        c5b9b6552
:peers        `defservice` READS + COMPARES types as data — NOTE blocker 3e only    2d25b4790
```

**RULED:** D1 · G3 · A-i→**S2** · **B-3** · 2a/2b/2c · **F1** · **DECL-NAME emits the new form** ·
**RUNTIME-ARG is NOT a migration** (the runtime already strips the params — `canonical_callable_name`;
drop them, β-ii-b's move, and they stay keywords because they are identity TAGS, not types).

## ⛔ THE SHAPE THAT BIT FOUR TIMES — read before any `:-` work

**A SLOT WITH TWO IMPLEMENTATIONS IS TWO SLOTS.** I verified a form was accepted *somewhere* and
shipped "the slot accepts it":

```
extend-type's surface arg   types.rs ✅ CHECK-time      runtime.rs:8226 ⛔ RUNTIME
(Head :- [args])            expand.rs ✅ EXPANDER       resolve/walk.rs ⛔ RESOLVER
defservice's annotation     the slot ✅ accepts it      the macro READS ITS OWN EMISSION back
extend-type's protocol slot A-i's base_fqdn SURVIVED the S2 revert — two spellings, two KEYS
```

**None surfaced where the defect was.** `[[feedback_a_slot_with_two_implementations_is_two_slots]]`

⚠ **A RIDER'S SCOPED RUN IS NOT THE FLOOR.** `binary_id(wat::services)` 128/128 green, floor red by
six — all six in `binary_id(wat::kernel)`, where the `service-parametric-*` deftests live.

⚠ **AN ACCEPTANCE ROW NAMING AN INSTRUMENT THAT CANNOT RUN IS WORSE THAN NO ROW.** I wrote
`-E 'test(doctest)'` into a brief; it matches ZERO tests, and the runner is `#[ignore]`d. The defect
it existed to catch was in the same diff.

## ⛔ NEXT — and ②-iii is **NOT** re-runnable. Read the ledger, not the old line.

The previous seam said *"Blockers 1·2·4·5 closed; 3 closes with the rider above. Then ②-iii is
re-runnable."* **That was wrong twice**, and both errors were in the record rather than on the disk:
it counted blocker 5 as closed while its own STILL-UNRULED section said *"Needs a DESIGN"*, and it
treated blocker 3 as one thing when the NOTE lists it as **five sub-sites**. The rider closed ONE.

**Measured against the disk 2026-08-22**, `NOTE-2iii-is-blocked-*.md` being the authority:

```
1  wat_source_derive keyword-only parser ........... ✅ CLOSED (shipped)
2  defsurface discriminates on NODE KIND ........... ✅ CLOSED (shipped)
3  the angle string IS the type's identity ......... ⛔ PARTIAL — 2 of 5
   a  register_subtype stores the edge key VERBATIM ...... ⛔ OPEN  (src/types.rs:716)
   b  transport_satisfier_heads format!("{fq}<T>") ....... ⛔ OPEN  (src/types.rs:745-748)
   c  satisfies_bare_surface's `{surface}<` prefix ....... ✅ CLOSED → family_extends, edb7f66c7
   d  defservice EMITS "{b}::Op{p}" with {p} = "<K,V>" ... ⛔ OPEN  (a dozen sites; see below)
   e  the :peers check COMPARES a built angle string ..... ✅ CLOSED 2d25b4790
4  defn / fn REJECT the `:- [T …]` binder .......... ✅ CLOSED (γ-i c889639aa — covers BOTH)
5  defrecord/defstruct companion macro ............. ✅ CLOSED (b9df7a09a) — RE-MEASURED 08-22

⛔⛔ **AND THE LEDGER ABOVE WAS ITSELF STALE WITHIN THE HOUR — blocker 5 is CLOSED.** I wrote it as
OPEN from the NOTE, then read the disk: `b9df7a09a` is titled *"BLOCKER 5 STRUCK"*, postdates the
NOTE, and teaches BOTH slots (`expand.rs:541`, `walk.rs:87`) to decline a form whose element 1 is the
`:-` marker. Re-measured all five heads — builtin · typealias · defenum · **defrecord · defstruct** —
all check clean. Probe kept: `wat-scripts/scratch-pad/arc109-blocker5-parametric-form-reference-by-head.wat`.

⚠ **TWO SHAPES OF THAT PROBE RETURNED FIVE GREENS WHILE MEASURING NOTHING** before one earned the
result. A bare `typealias` file exits 0 even for `(:wat::cache::NoSuchType :- [:i64])`; so does a
`defn` signature naming an unresolvable type — `--check` does not resolve unknown names in that
position. What earned it was a NEGATIVE control failing by the SAME mechanism (a function-type
bracket with no arrow → *"function-type bracket needs a `:->` arrow"*, the NOTE's exact error).
**A green is evidence only while a matching red is available.**

★ **THE STANDING LESSON: `NOTE-2iii-is-blocked-*.md` IS A MEASUREMENT WITH A DATE ON IT.** It was
produced by running the codemod and flooring; four of its five entries have had substrate shipped
under them since, and I have now caught THREE of them stale by reading rather than running. **Do not
write a DESIGN against that list.** The instrument that made it is the only instrument that can
update it, and it is cheap: dry-run the codemod on a `/tmp` copy, diff, apply, floor, read what
breaks NOW, revert. Last run it was one revert. `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
```

⚠ **3d is the one that makes a green corpus regrow the disease.** The `:peers` stone taught
`defservice` to READ a migrated type structurally — and it re-serializes the args straight back into
a `<a,b>` suffix, because every downstream `string::interpolate` in the file still wants one. That
was the brief's scope and it is correct work; it is **not** blocker 3. The NOTE says it plainly:
*"`defservice` EMITS the angle form, so even a fully migrated corpus regrows it at every macro
expansion."* Consumption and emission are two halves and only the first is done.

**Probe that settles blocker 4 in one command each** (both green, so the check is not vacuous):

```bash
printf '(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)\n' > /tmp/b.wat   # binder → exit 0
printf '(:wat::core::defn :user::g<T> [x <- :T] -> :T x)\n'     > /tmp/a.wat   # angle  → exit 0
target/release/wat --check /tmp/b.wat   # read the exit DIRECTLY, never through a pipe
```

### The order

1. **`tests/services/` keeps the `:peers` negative controls** — briefed:
   `109/BRIEF-STONE-the-peers-bijection-keeps-its-negative-controls.md`. This is **debt from
   2d25b4790**, not new capability: that stone's brief named row 4 load-bearing and then ordered its
   evidence deleted, so nothing on disk records that the bijection still REJECTS.
2. **`bracket.wat`** — briefed, unreleased, independent:
   `109/BRIEF-STONE-identity-3-bracket-reads-a-type-node.md`.
3. **Blocker 5 needs a DESIGN** before it can be ruled. It is a substrate strike, not a stone.
4. **Blocker 3's remaining three** (a · b · d) are one strike, not three: *a type's identity is its
   BASE NAME plus a structured param list, never the concatenated `Head<A,B>` string.* The NOTE
   calls this **③'s real prerequisite** — not ②a's 244 bare heads as ②'s DESIGN claimed.
5. **Only then is ②-iii re-runnable.** ⛔ Re-read `109/NOTE-2iii-is-blocked-*.md` first.
## ⛔ STILL UNRULED

- **C** — the codemod also migrates `Fn(args)->ret` (42 in `wat/`) and `:(a,b,c)` (49), which ②'s
  DESIGN scoped out when the `Tuple` renderer was mode-blind. ②-i-b closed that; destinations are
  probe-verified. Gates ②-iii only.
- **Blocker 5** — `defrecord`/`defstruct` mint a companion `defmacro` under the bare name, so a
  parametric FORM reference macro-expands before the checker sees it. **Needs a DESIGN.**
- **The PROBE MATRIX.** Of everything found today, the floor found three; QUESTIONS and CLASSIFICATION
  PASSES found the rest. I still cannot certify the ②-iii blocker list is complete, and **B-4 was
  ruled out precisely because a split drawn from an uncertified list inherits its uncertainty.**

## THE STILL-OPEN

- **β-ii-d** — `defservice`'s substring transport test (`contains? fqdn-tp "<T>"` / `"<T,"` / `",T>"`,
  three variants approximating one membership check). Cheap now: `fqdn-tp-syms` exists.
- **γ** — `defn`. TWO capabilities. (i) the declaration BINDER — MEASURED, RULED FIRST (D1), DESIGN
  written, decision E open; the builder's spec, 2026-08-21: the binder lists every
  type var INCLUDING the return's — `[:-> X]` ⇔ `(wat.core/fn :- [X] [] :- X …)`,
  `[A B :-> X]` ⇔ `(wat.core/fn :- [A B X] [a :- A b :- B] :- X …)`. (ii) **call-site type
  application** (`(ns/f :- [wat.type/i64] 42)`), REJECTED today, site count still **UNMEASURED**.
- ⛔ **`defrecord`/`defstruct` types have no parametric FORM reference** — the companion `defmacro`
  at `wat/Record.wat:197` expands the form before the checker sees it. **Narrower than previously
  recorded**: probed 2026-08-21, builtin ✅ · typealias ✅ · defenum ✅ · defrecord ⛔ · defstruct ⛔.
  It is the COMPANION MACRO, not user-vs-stdlib. ②-iii blocker 5; 6 types / 5 breaking refs in
  `wat/`. Root is 251.8's one-node-two-roles. **Needs a DESIGN before it can be ruled.**
- **③** — angle form ILLEGAL · delete `is_type_bracket_candidate` (ONE caller now). Its REAL
  prerequisite is the identity stone above, NOT ②a's 244 bare heads as the ② DESIGN said.
- **The `Fn`/`Tuple` scope question** — the codemod migrates `Fn(args)->ret` (42 in `wat/`) and
  `:(a,b,c)` (49), which ②'s DESIGN scoped out. Both are `type-shaped-keyword?` by that predicate's
  own definition, so excluding them means ADDING the discriminator the DESIGN forbids; both
  destinations are probe-verified legal (`wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat`).
  The exclusion was written when the `Tuple` renderer was mode-blind; ②-i-b closed that. **Unruled.**
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
> ⚠ **A VERIFICATION APPLIED TO WHAT THE DIFF ADDED IS NOT A VERIFICATION OF THE LIST.**
> `a9168b851`'s SCORE records six declarator heads *"each destination verified against α to accept
> `name :- [T…]` before being listed."* True — and only the six the diff added. `defn` was in the
> ORIGINAL list, never probed, and it REJECTS. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`
> recurring inside the stone that fixed it.
>
> ⚠ **I AM THE INQUISITOR; A SHADOWDANCER EXECUTES.** Builder, 2026-08-21, mid-strike: *"we
> construct the documents for a shadowdancer to execute… we do small, trivial fixes here.. anything
> else requires a doc and a subagent."* I hand-wrote ~180 lines of substrate across four files
> chasing ②-iii's red. It is green and shipped rather than re-derived — but a red floor is a
> DIAGNOSTIC, not a licence to start editing.
>
> ⚠ **A DISCRIMINATOR ON NODE KIND IS A DISCRIMINATOR IN THE WRONG PLACE.** `defsurface` read "a
> `List` is a method member"; a parametric field TYPE is now also a `List`. It reported
> `triple is incomplete` — **naming the field as the defect when the field was fine.** The
> codemod's own slot rule, one level up, found a day later in different code.
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
