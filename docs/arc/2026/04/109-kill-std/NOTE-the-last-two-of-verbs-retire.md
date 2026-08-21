# NOTE (arc 109) — the last two `/of` verbs retire. The playbook already ran twice.

**Filed 2026-08-20. Builder-ruled.** Measured against `target/release/wat` at `a254d0d7e`
(⚠ the binary warns STALE because a rider is mid-flight in `src/` — these numbers are HEAD's
behaviour, not the working tree's, which is what this NOTE wants).

> Builder: *"those are scheduled for deletion .. we do not use `of` enough for me to want to keep
> it.. its illogical to have them as well.. **we have constructor for them**....."*

## The whole population is TWO

```
:wat::core::List/of    5 dispatch arms    wat/ 0    wat-scripts/ 11    tests/ 35
:wat::core::char/of    6 dispatch arms    wat/ 0    wat-scripts/  0    tests/ 26
```

Neither appears anywhere in `wat/` — **not one site in the stdlib**. 72 sites total, every one a
test or a probe. `List/of` is also the ONLY List constructor (`List/conj`, `List/get`,
`List/length`, `List/empty?`, `List/contains?` are the rest), and `char/of` is the ONLY `char/*`
verb of any kind.

## Why "illogical" is exact, and why this is not a new decision

The **verb-equals-type** playbook is arc 109's own, and it has already shipped twice:

| retired | became | where |
|---|---|---|
| `:wat::core::vec` | `:wat::core::Vector` | slice **1f** (`src/remedy/retirement.rs:110`) |
| `:wat::core::tuple` | `:wat::core::Tuple` | slice **1g** (`SLICE-1G.md`, `INVENTORY.md:135`) |
| `:wat::core::List/of` | `:wat::core::List` | ⛔ **never ran** |
| `:wat::core::char/of` | `:wat::core::char` | ⛔ **never ran** |

So the constructor the builder means is the *pattern*: **a type's constructor is spelled with the
type's own name.** `/of` is a second way to say the same thing, which is what makes it illogical
rather than merely unused — and `[[feedback_wat_llm_first_design]]`'s "two ways to do one thing" is
the standing objection.

Confirmed neither replacement head exists yet: `(:wat::core::List 1 2 3)` and
`(:wat::core::char "x")` both return `UnknownFunction`. This is a rename that must MINT the new
head, exactly as 1f and 1g did, not a redirect to something already there.

## The two halves are NOT the same size

- **`char/of` → `char`** is a pure rename. `char` is not parametric, so no param-spec, no
  `split_type_param_bracket`, nothing from ②-i-b. It could ship at any time.
- **`List/of` → `List`** is a rename PLUS the param-spec, because `List<T>` IS parametric and the
  mandatory-vec ruling (`3821db4ba`) reaches it. That is the *"same treatment as Tuple"* the builder
  ruled, and it **depends on ②-i-b's `split_type_param_bracket`** — one call site through the door,
  not a new rule.

## Sequencing

1. ②-i-b lands (the door exists).
2. `List/of` → `List` with the param-spec, through that door. `char/of` → `char` rides along or
   ships alone; it shares only the retirement paperwork.
3. Both get a `src/remedy/retirement.rs` entry so the checker hands each of the 72 sites its own
   fix, the way 1f and 1g did.

★ The 72 sites are ALL in `tests/` and `wat-scripts/`. There is no stdlib consumer to break, and
`every_wat_scripts_file_loads` plus the floor will name every one of them. **Do not survey for the
worklist** — impose the retirement and read the screams.
`[[feedback_impose_the_check_and_read_the_screams]]`

## The open this does NOT close

`Some` / `None` / `Ok` / `Err` still have no param-spec form, and `Option<T>` / `Result<T,E>` are
parametric under a ruling that forbids inference. Tracked in
`NOTE-six-parametric-constructors-never-got-the-bracket.md`; the builder is still weighing it.
