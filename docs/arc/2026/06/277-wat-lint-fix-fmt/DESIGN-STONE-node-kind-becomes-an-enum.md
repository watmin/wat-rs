# DESIGN — STONE: Node.kind becomes an enum (and the Rust boundary gets a gate)

> **Builder, 2026-09-06:** *"we should rip the bandaid and move off of strings for the rules"* ·
> *"break first, then node"* · *"ok.... node-kind next..."*

`[[DESIGN-STONE-break-kind-becomes-an-enum]]` was the small proving case: 23 sites, one file, no
boundary. **This is seven times larger and it has a problem `Break` did not** — `Node.kind` is fed by
a **Rust intrinsic that returns a String**.

## THE SCOPE — a census, corpus-wide

```
49  files consume :wat::grep::Node
155  kind-string comparisons        "keyword" 88 · "list" 23 · "vector" 21
                                    "symbol" 10 · "map" 8 · "set" 5
```

Not just the formatter:

```
12  wat-scripts/fmt/rules/*.wat        the formatter
 7  wat-scripts/grep/*.wat             wat-grep's own rules
11  wat-scripts/fixes/*.wat            RECORDED MIGRATIONS
 3  tests/cli/wat_grep__*.wat          gated CLI tests
 6  wat-scripts/scratch-pad/277-*.wat  this arc's probes
 2  wat-scripts/fmt/fixtures/*.wat     kw-table, pair-table
 1  wat/grep.wat                       the PRODUCER
```

⚠ **The 11 recorded migrations in `wat-scripts/fixes/` get rewritten too, and that is correct.** A
recorded codemod is live code under `every_wat_scripts_file_loads` — it must parse and type-check on
the current runtime or it rots. Git preserves what each one was; the working tree carries what still
compiles. This is the same reasoning that put them under the gate in the first place.

## ★ THE CLOSED SET IS 14, AND IT IS RUST'S

`:wat::core::ast-kind` is an intrinsic returning `:wat::core::String`. Its arms, read from
`src/edn/render.rs` — **not from the doc comment** (`[[feedback_a_header_is_not_the_file]]`), though
the two agree exactly:

```
IntLit "int"      FloatLit "float"    RationalLit "rational"  BigIntLit "bigint"
CharLit "char"    BoolLit "bool"      StringLit "string"      NilLit "nil"
Keyword "keyword" Symbol "symbol"     List "list"             Vector "vector"
Set "set"         Map "map"
```

**One arm per `WatAST` variant. Fourteen.**

## THE ENUM MIRRORS `WatAST`'s VARIANT NAMES — and that is measured, not aesthetic

```wat
(:wat::core::defenum :wat::grep::NodeKind :wat::enum::Pure
  :IntLit [] :FloatLit [] :RationalLit [] :BigIntLit [] :CharLit [] :BoolLit []
  :StringLit [] :NilLit [] :Keyword [] :Symbol [] :List [] :Vector [] :Set [] :Map [])
```

⚠ **A variant named `:String` is REFUSED** — measured across all fourteen candidate short names, and
it is the only one:

```
:Int ok · :Float ok · :Rational ok · :BigInt ok · :Char ok · :Bool ok
:String  REFUSED — "bare primitive type ':String' is retired (arc 109 slice 1c)"
:Nil ok · :Keyword ok · :Symbol ok · :List ok · :Vector ok · :Set ok · :Map ok
```

So the short-name spelling is not available anyway, and mirroring `WatAST` is the shape that both
sidesteps it and makes the Rust↔wat correspondence **readable by inspection** rather than a
coincidence a reader has to reconstruct.

## ★★ THE BOUNDARY — the honest part, and the thing `Break` did not have

`grep.wat:192-194` builds each fact from `(:wat::core::ast-kind node)`, a **String**. A
`String → NodeKind` map **cannot be exhaustive on its input.** So this stone does **not** remove a
raise the way `Break` did — it **relocates** one:

```
BEFORE   a latent hole at 155 comparison sites: a typo'd "lst" silently matches nothing
AFTER    ONE raise, at ONE place, the moment an unknown discriminant crosses the boundary
```

That is a real improvement and it is not the same claim as `Break`'s. **Say it that way.**

Proven end-to-end (`wat-scripts/scratch-pad/277-the-node-kind-boundary.wat`): `roundtrip=list`, `same=true`, `cross=false` —
`kind-of` maps in, `kind-name` maps back, and `core::=` discriminates between variants.

## ★★★ AND THE CLASS THIS STONE MUST CLOSE — DRIFT

Nothing keeps `NodeKind`'s 14 variants in step with `ast-kind`'s 14 arms. Add a 15th `WatAST`
variant tomorrow and:

- **Rust is guarded** — `eval_ast_kind` goes `E0004` non-exhaustive, so the author MUST add an arm
  and MUST invent a fifteenth string.
- **wat is NOT** — `kind-of` silently falls to `:else` and raises at runtime, the first time some
  file happens to contain the new node shape.

**A gate must fail when the two lists diverge**, in the `tests/lint/` style this repo already uses
(`one_name_grammar.rs`, `no_angle_suffix_strip.rs`). **This is the only Rust in the stone**, and
without it the migration ships a new drift class in exchange for the one it closes.

## THE CONTRACT DECISION

**`NodeKind` lives in `wat/grep.wat` beside `Node`, mirrors `WatAST`'s variant names exactly, and has
no catch-all variant.** The boundary raises; the consumers cannot. A `:Unknown` variant would push
the hole back out to all 155 sites, which is the thing being removed.

## WHAT THIS STONE IS NOT

- **NOT a change to `ast-kind`.** It stays a `String`-returning intrinsic with a documented `@ret`.
  Changing it is a far wider Rust blast radius and it is not needed: one mapping site is enough.
- **NOT `if` / `cond`.** Ruled, queued, two new rule files.
- **NOT the `RhsUnresolvableOperand` diagnostic**, whose `accepted` list omits the call form and is
  the class that taught the String in the first place. Rust, its own stone, named so it is not lost.

## FILES

```
wat/grep.wat                       NodeKind · Node.kind's type · the boundary map at :194
wat-scripts/fixes/<name>.wat       NEW — the recorded codemod
49 consumer .wat files             rewritten BY the codemod, never by hand
tests/lint/<name>.rs               NEW — the ast-kind ↔ NodeKind sync gate (the only Rust)
```

⚠ **`wat/*.wat` is FROZEN into the release binary** — `cargo build --release` between an edit and any
measurement. `wat-scripts/` is read from disk.

⚠ **Expect a very wide red mid-migration** — a `String` field and an enum field are different types,
so the checker is the cascade and the fail count is the progress meter, not a crisis
(`docs/SUBSTRATE-AS-TEACHER.md`). 155 sites is the biggest sweep this arc has run.
