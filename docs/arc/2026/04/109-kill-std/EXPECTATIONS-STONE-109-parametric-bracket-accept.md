# EXPECTATIONS — 109 step ① (bracket accept)

Written BEFORE the strike. The rider does not see this.

| # | what | expected |
|---|---|---|
| 1 | type position accepts the bracket | `(:wat::type::Vector [:wat::core::i64])` after `<-` → exit 0 |
| 2 | nesting | `(Tuple [i64 (HashSet [f64])])` → exit 0 |
| 3 | value position, HashMap | `(HashMap [String i64] "a" 1)` → `{"a" 1}` |
| 4 | value position, PersistentMap — the odd one | builds, or a clean STOP-3 report |
| 5 | empty instance | `(HashMap [String i64])` → `{}` |
| 6 | ★ **ADDITIVE**: angle form still checks | `:wat::core::Vector<wat::core::i64>` → exit 0 |
| 7 | ★ no collision | `[A :-> B]` standalone → exit 0 |
| 8 | lexer untouched | `git diff --stat crates/wat-reader/` empty |
| 9 | renderer untouched | `format_type_inner` still emits `<>` — that is ③'s |
| 10 | `.wat` / `tests/` untouched | empty diff |
| 11 | one helper, six call sites | `infer_*_constructor` fns show no diff |
| 12 | floor | 4818/4818 |
| 13 | clippy | 0 |

★ **Rows 6 and 7 are the load-bearing rows, not 1–5.** This step ADDS a spelling. A rider optimising
toward "the new form works" can trivially satisfy 1–5 by breaking the old path — and the corpus is
still entirely angle-form, so row 12 would catch it, but rows 6/7 name it precisely instead of
leaving me to bisect a 3,000-failure floor.

★ **Row 9 is a real trap.** The design RULES that rendering follows — but at ③, not ①. A rider that
reads the design and "helpfully" flips `format_type_inner` moves 113 goldens inside an additive step
and makes the floor unreadable.

## Independent prediction

**Runtime: 20–35 min.** Two rooms, both mapped to the line, one helper. The `PersistentMap` asymmetry
is the only place real judgement is spent.

## Trap doors

- **`PersistentMap`** rejects leading type keywords today. If its `infer_*` has no leading-type path,
  STOP-3 fires and that is the correct outcome, not a failure.
- **`Peer'<…>`** — 29 primed heads. Not ①'s business (no lexer change), but if the rider touches
  head parsing it must survive.
- **`Tuple`** is special-cased in `parse_type_form` (`raw_head == "wat::core::Tuple"` → `TypeExpr::Tuple`,
  not `Parametric`). The bracket must feed that arm the same way.
- **`:wat::type::Infer`** — `infer_hashmap_constructor` has a dedicated `INFER_TYPE_PATH` arm. It must
  keep working inside the bracket.

## What would make me reject

Row 6 or 7 red · a lexer diff · a renderer diff · `.wat` or `tests/` touched · an `infer_*` fn edited
without a STOP-3 report.
