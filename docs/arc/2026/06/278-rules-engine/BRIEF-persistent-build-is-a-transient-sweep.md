# BRIEF — sweep the 35 rpds rebuild-loop sites to the `_mut` transient form

## Your role

You are a rider. **Ending your turn ENDS you** — it does not suspend you, and nothing will wake you.
There is no notification coming. So do the whole edit, then write your report in the same turn.

This is a **text-edit strike**. The orchestrator compiles and measures centrally, once, after the
tree is quiescent — that is deliberate, because a build is a serial resource. Do not run cargo,
build, or tests; your turn ends when the edits are made and reported, and nothing about your report
depends on a compiler.

## The work, in one paragraph

`rpds` persistent collections expose two families: a **copying** one (`push_back`, `insert`, …) that
returns a new structure, and a **`_mut`** one (`push_back_mut`, `insert_mut`, …) that writes in
place. `Vector::push_back(&self)` begins with `self.clone()`, which raises every trie node's
refcount to 2, so the `SharedPointer::make_mut` inside `assoc` is **forced to copy the whole
root→leaf path** — and the previous version is then dropped unread. When a structure is being built
in a loop and reassigned to itself, every one of those copies is waste. Convert those sites to the
`_mut` twin. The values produced are byte-identical; only the build strategy changes.

## The exemplar to mirror — already landed, read it first

`0416d1a5`, `src/rete/kernel.rs`, `hashmap_to_pm`. Two lines, and it took `out:production` from
28.53 ms to 4.47 ms on a 40,000-element materialisation:

```rust
// before
pv = pv.push_back(v);
pm = pm.insert(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
// after
pv.push_back_mut(v);
pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
```

That is the entire shape. Every edit below is that edit.

## The transform

```
x = x.push_back(v);   ->   x.push_back_mut(v);
x = x.insert(k, v);   ->   x.insert_mut(k, v);
```

The binding is already `let mut` at every site (it had to be, to be reassigned), so no declaration
changes. Keep arguments, formatting, and surrounding comments exactly as they are.

**The rule that makes this safe, and the only rule you need:** convert **only** where the identifier
on the left of `=` is the *same identifier* as the receiver. That shape proves the previous version
is dead — it was just overwritten — so the copy it forces can never be observed by anyone. A site
that genuinely needs the old value binds a *different* name and is not in this list.

## Rooms — the exact sites (35), and why each file is here

Each line below is `file:line` at HEAD `0416d1a5`, with the current text. Line numbers may drift by
a line or two as you edit within a file; match on the **text**, not the number.

### `src/rete/kernel.rs` (18) — the hot one; `:905` is inside a path measured at 18.8 ms
```
  244  matches_pv = matches_pv.push_back(tuple);
  303  pv = pv.push_back(native_token_to_value(tok));
  305  pm = pm.insert(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
  350  pm = pm.insert(k.clone(), v.clone());
  406  pv = pv.push_back(native_element_to_value(el));
  408  pm = pm.insert(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
  605  new_pv = new_pv.push_back(Value::i64(*cid));
  905  new_bindings = new_bindings.insert(k.clone(), v.clone());
  924  m = m.insert(k.clone(), v.clone());
 1257  pv = pv.push_back(fact.clone());
 1533  pv = pv.push_back(v);
 1543  pv = pv.push_back(fact.clone());
 1558  pm = pm.insert(k, Value::wat__core__PersistentVector(pv.push_back(fact.clone())));   ⚠ SEE BELOW
 1571  pv = pv.push_back(Value::i64(acc_var_i64(el, &var)));
 3055  stratum_pv = stratum_pv.push_back(rule_val.clone());
 3107  nm = nm.insert(Value::i64(*id), dedupe_filter_children(v, &active_ids));
 3166  prod_pv = prod_pv.push_back(d.clone());
 3493  support_pm = support_pm.insert(derived_fact, support_value);
```

### `src/edn_shim.rs` (7) — two on the EDN decode path, five in `#[cfg(test)]` bodies
```
 2895  pm = pm.insert(k_val, v_val);
 2914  pv = pv.push_back(val);
 4493  pv = pv.push_back(Value::i64(10));
 4494  pv = pv.push_back(Value::i64(20));
 4495  pv = pv.push_back(Value::i64(30));
 4533  m = m.insert(Value::String(Arc::new("a".to_string())), Value::i64(1));
 4534  m = m.insert(Value::String(Arc::new("b".to_string())), Value::i64(2));
```
The five test-body sites are in scope on purpose: the wall that follows this sweep is armed at
**zero**, so a cosmetic site left behind would keep it red forever.

### `src/collection/eval.rs` (5)
```
  783  out = out.push_back(elem.clone());
  786  out = out.push_back(elem.clone());
 1205  map = map.insert(k, v);
 1535  pv = pv.push_back(v);
 1654  out = out.push_back(elem.clone());
```

### `src/rete/matcher.rs` (3)
```
  192  pm = pm.insert(k.clone(), v.clone());
 1016  constraints_pv = constraints_pv.push_back(Value::wat__WatAST(Arc::new(substituted)));
 1028  step_bindings_pm = step_bindings_pm.insert(key, v.clone());
```

### `src/collection/transform.rs` (1)
```
   58  out = out.push_back(elem.clone());
```

### `src/rete/collect.rs` (1)
```
   73  out = out.push_back(rule);
```

## ⚠ The one trap — `src/rete/kernel.rs:1558`

```rust
pm = pm.insert(k, Value::wat__core__PersistentVector(pv.push_back(fact.clone())));
//  ^^^^^^^^^^ OUTER: self-reassignment  -> convert to pm.insert_mut(...)
//                    INNER: pv.push_back(...) -> LEAVE EXACTLY AS IT IS
```

`pv` is a **borrowed** vector whose previous version is still live and still needed, so the inner
copy is correct and required. Convert the outer call only. The result must read:

```rust
pm.insert_mut(k, Value::wat__core__PersistentVector(pv.push_back(fact.clone())));
```

A transform that rewrites by *method name* instead of by the LHS-equals-receiver shape will corrupt
this line. It is the only nested instance in the list.

## Blast radius

Those **six files only**. No new types, no signature changes, no `use` changes, no reformatting of
untouched lines, no comment rewrites except where a comment literally describes the old copying
behaviour (there are none in this list). Nothing under `wat/`, nothing in `tests/`, no `.wat` files.

## STOP triggers — each rejects; none of them lets you ship less

1. **STOP-1** — a site's text does not match what is listed above (the code has changed under you).
   Stop, leave that site alone, and name it in your report. Do not guess at the intended edit.
2. **STOP-2** — you find a `x = x.<method>(…)` self-reassignment that is **not** in the list above.
   Stop and report it with `file:line`. It means the detector missed a case and the orchestrator
   needs to know before the wall is armed. Do not convert it silently.
3. **STOP-3** — a site where converting would change behaviour (the old value is read after the
   reassignment). Stop and report; do not convert it and do not work around it.

## Your report

- The count converted, per file.
- `kernel.rs:1558` quoted verbatim as you left it, so the nested inner call can be verified by eye.
- Anything a STOP fired on.
- Nothing else — no summary of what rpds is, no speculation about performance.
