# DESIGN — a `path:line` citation is two claims and only the first is ever checked

## Why

Work-list **F2-e**: *"`:wat::rete::insert-all-spec` IS A PHANTOM, cited three times with a line number
past the end of its file."*

**Driven at HEAD `c22cfe6e3`, the row is wrong about what it is and short about how much of it there
is.**

## It is not a phantom — it is a retirement prose never followed

`wat-scripts/fixes/rete-oracle-sigil.wat` is the **recorded codemod that retired the name**:

```
:wat::rete::insert-all-spec  ->  :wat::rete::insert-all$oracle
```

and `:44` carries the rune saying so — *"pre-`$oracle` spelling of the wat reference impl, retired by
the very rewrite recorded below."* **The retirement is correct and correctly documented.** What rotted
is the prose that still cites the old spelling.

## The citation is wrong three ways, in five files

| | cited | actual |
|---|---|---|
| name | `insert-all-spec` | retired → **`insert-all$oracle`** |
| file | `wat/rete.wat` | **`wat/rete/oracle/insert.wat:45`** |
| line | `:1508` | **that file is 533 lines** |

**Five files, six citations** — `wat/seq.wat` ×2, `wat/core.wat`, and **three under `wat-tests/core/`
that the row never mentions**. `rete.wat:1508` appears **five times across four files**.

## The root: two orthogonal gaps

`no_stale_path_in_doc` is the gate for this class, and it misses on both axes:

1. **Scope.** It walks `vec![root.join("src/rete")]` (`:88`). **Nothing scans `wat/` or `wat-tests/`
   prose at all** — which is where all six citations live.
2. **Depth.** It checks a path **exists**. **Nothing checks that a cited `:LINE` is inside the file.**
   That is how `rete.wat:1508` survived in a 533-line file, five times.

**A `path:line` is two claims. The second one is checked nowhere in this repo.**

## The contract decision, pinned

**Gate the second claim, and widen the scope to where the prose actually is.**

- A `path:line` cited in a comment must have the path resolve **and** the line be within that file's
  length. Out-of-range is a hard fail naming both numbers.
- The scanned roots gain `wat/` and `wat-tests/`.
- **The six citations are cured by naming the live symbol and NO line number.** A symbol beats a line
  — the lesson C14 landed when two of its citations had rotted by line drift. A citation that cannot
  rot is better than one that is currently right.

⛔ **This does NOT gate retired names in prose.** `rete_names_in_wat_scripts_resolve` rules
deliberately that *"prose may name a retired form"* — accurate history is not a defect. This strike
checks only that a cited **location exists**, which is orthogonal and does not touch that ruling.

## Out of scope = REJECTED

- **Flagging retired names in comments.** Explicitly ruled legitimate; see above.
- **`src/` prose beyond what `no_stale_path_in_doc` already covers.** The depth check applies
  wherever the gate scans, but widening `src/` scope is a separate measurement.
- **Auto-fixing citations.** Six is a hand edit; a codemod for six sites is more machinery than the
  problem.
