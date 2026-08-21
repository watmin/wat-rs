# SEAM — the ONE live breadcrumb. As of 2026-08-21 (`:-`, the parameterization operator). Replaced in place.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `a39eb99aa`.** Run **`git log --oneline a39eb99aa..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4855/4855, 0 FAIL, 19 skipped, ~72s   (own invocation, scripts/floor.sh)
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …`. Read exit codes directly, never through a pipe.
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild.** `wat/*.wat` is baked in by `include_str!`
at RUST-compile time; `--check` reflects the last build and prints a staleness warning. A rider
CANNOT test one.

## ★★ THE WORK: ARC 109 — `:-` IS THE PARAMETERIZATION OPERATOR

> Builder: *"the symbol ` :- ` is declaring **'this thing on the left is parameterized by the thing on
> the right'**"* … *"this is the same as arg-spec and ret-type in my mind — they declare what they are
> explicitly."*

```clojure
[n :- wat.type/i64]                        arg-spec
:- wat.type/i64                            ret-type
(wat.type/Vector :- [wat.type/i64])        type args
(wat.type/Vector :- [wat.type/i64] 1 2 3)  constructor
(wat.core/defn ns/f :- [T] [x :- T] :- T x)   declaration binder
```

**The param-spec is a vec of TYPE REFS.** Values are never legal there — that position is
**reserved**, which is why nothing sniffs it. `:- []` is the **assumed default state**: there is no
monomorphic-vs-parametric distinction, only a param list that is usually empty.

### Where it stands

```
②-i-b  `:-` accepted + emitted; Tuple arm brackets; nil stops canonicalizing   ✅ c9938cc7b
       ⚠ AMENDED 9741507da — it shipped for 3 of 6 constructors and I scored it GREEN
α      8 Rust declarator heads take `:- [T …]`                                 ✅ c5ac5174c
β-i    both `defrecord` macros                                                 ✅ 26669f8d9
β-ii-a′ `defservice`: the binder is the SOURCE OF TRUTH, `<K,V>` a derived shim ✅ bd898a748
⛔ NEXT — β-ii-b: the ~40 `fqdn-tp` emissions become FORMS
β-ii-c  the 10 `proto-tp` sites  ·  β-ii-d `:741`'s substring transport test
γ      `defn` — TWO capabilities: the declaration binder AND call-site type application
③      angle form ILLEGAL · `is_type_bracket_candidate` DELETED (it has ONE caller now)
②-iii  apply the codemod to wat/ — BLOCKED, see below
```

## ⛔ THE CODEMOD TURNS A BINDER INTO AN APPLICATION — 84 SITES

`wat-scripts/fixes/parametrics-take-a-type-vector.wat` rewrites **arg 1 of a `def*` head** as if it
were a type reference. Measured on a copy of `wat/spawn.wat`:

```
(:wat::core::defn :wat::kernel::recv-all-loop<I,O>       ← the BINDER
(:wat::core::defn (:wat::kernel::recv-all-loop [I O])    ← rewritten as an APPLICATION
```

Four `<…>` in four lines of that one form: one binds `I`/`O`, three reference them, and one rule hit
all four. **The slot rule:** arg 1 of a `def*` head is a binder; every other `<…>` is a reference.
Total, no sniffing — a name slot can never hold a reference to something that already exists.

## THE STILL-OPEN

- **γ's second half is UNMEASURED.** Builder: *"if a defn declares a parametric, the caller must
  declare it too"* — `(ns/hash-set-of-xs :- [wat.type/i64] 1 2 3)`. Call-site type application is
  REJECTED today (measured). Nobody has counted the call sites.
- **`defn`'s `<T>` is DECORATIVE** (`function/eval.rs:66` hardcodes `type_params: Vec::new()`);
  the SEVEN type declarators' `<T>` is LOAD-BEARING, and dropping it makes `T` a **concrete type**,
  not a free var — a plausible TypeMismatch naming a type that does not exist.
- **`List/of` + `char/of` retire** into `List` / `char` (verb-equals-type, the playbook that already
  ran for `vec`→`Vector` and `tuple`→`Tuple`). 72 sites, all tests/probes, zero in `wat/`.
- **296 Stone H** (drawn, not built) now also closes Option/Result's param-spec — they are enum
  VARIANTS, field names stay `value`/`value`/`error`.
- 255 #110 · 285 Map half · 296 OPEN · 295 not started.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **WHEN A STONE TOUCHES N MEMBERS OF A FAMILY, THE ACCEPTANCE ROWS MUST NAME ALL N.** ②-i-b
> shipped `:-` for Tuple/PersistentMap/PersistentVector and not Vector/HashMap/HashSet. I scored it
> green: every new-spelling row I ran landed on the half the rider's diff had touched, and every row
> on the other half used the OLD spelling as a control. **A full green over that split reads exactly
> like a full green over everything.**
>
> ⚠ **THE ROW THAT CAN PASS HOLLOWLY IS THE ONLY ROW THAT MATTERS.** Every other row stays green
> while the new path is silently ignored, because the old path carries everything. Prove it with a
> PERTURBATION — `:- [X Y]` must break what `:- [K V]` builds.
>
> ⚠ **I COUNTED WHERE A THING IS DEFINED AND CALLED IT THE CALL-SITE COUNT** — three sizes for one
> macro in three messages (15 → 4 → ~50). The consumers are the work.
>
> ⚠ **A MACRO BODY MAY NOT CALL A USER-DEFINED FUNCTION AT ALL** (F5, default-deny,
> `src/macros/eval.rs:351`). It fails at DEFINITION and takes the stdlib down — 3029 tests at once.
> Read `109/NOTE-the-F5-allow-list-and-what-a-macro-body-may-call.md` BEFORE editing any macro.
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.`
