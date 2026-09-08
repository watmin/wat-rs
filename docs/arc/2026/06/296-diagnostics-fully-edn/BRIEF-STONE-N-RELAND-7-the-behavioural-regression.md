# BRIEF — STONE N RELAND 7: ⛔ A BEHAVIOURAL REGRESSION HAS BEEN HIDING UNDER THE FAILURE COUNT

> **34 of the remaining 45 are not migration failures. They are WRONG ANSWERS.**
> This is the priority. The rest of this brief is secondary to it.

## THE FINDING — verbatim, from the captured arm

```
thread 'probe_arc278_journal_query::journal_query_metrics_reads_back_and_filters_by_time_window'
panicked at tests/services/probe_arc278_journal_query.rs:20:5:
  expected broad query to return 2 metrics and narrow query to return 1 (encoded 2*10+1=21);
  got i64(-11)   (a value like 2N means the narrow window's time filter failed to drop the t=1s metric)
```

```
thread 'probe_arc278_journal_query_logs::…' panicked at …probe_arc278_journal_query_logs.rs:13:5:
  expected broad=2, narrow=1 (2*10+1=21); got i64(-11)
```

**No type-check error. No spelling error. A WRONG VALUE.** The services run and return the wrong
answer, and `-11` is a sentinel — the test's own encoding says what it means.

## THE 45, CLASSIFIED

```
27  wat::services   journal · sift · span · call_context · tagged_keys_store · mem_store
 7  wat::rete       smem_roundtrip ×5 · sqlite_store_differential ×2
                    ⛔ THESE 34 ARE BEHAVIOURAL. THEY ARE THE STONE.
 5  wat::wat_lang   wat_core_try ×3 · probe_arc241_stone15_zombie_purge ×2
                    .wat.bad fixtures whose BARE spelling is INCIDENTAL; the wall fires before
                    the error the test measures. RELAND 4 fixed ONE of these
                    (try_with_zero_args) and did not sweep the class.
 2  wat::cli        wat_mcp counter ×2
 4  misc            collection · diagnostics · lint(wat_scripts loader) · reflection
```

## ⛔ THE WORK — DIAGNOSE THE 34 BEFORE TOUCHING ANYTHING ELSE

**1. Bisect it against this campaign's own save points.** Every reland committed locally:

```
434d7fac6  M2 corpus            49f03f179  N wall + rename
d1f64f188  RELAND-1 Path B      e77311bf9  RELAND-2 fmap + fold removed
…           RELAND-3/4/5/6
```

Run one journal test at each. **Name the first commit where the answer goes wrong.** ⚠ And read
that commit's DIFF before calling it the cause — a first-bad commit names where a symptom became
VISIBLE, never what caused it. `[[feedback_a_bisect_names_visibility_not_causation]]`

**2. The suspects, in the order I would look — but MEASURE, do not assume:**

```
enum_runtime_value / try_eval_enum_map_ctor   does the map unpack in DECLARATION ORDER?
                                              A two-field variant filled in the wrong order is
                                              silently wrong and type-checks perfectly.
Path B's synthesized recv (runtime.rs)        RELAND 1 rewrote Message/Lost bodies to {:msg …}/
                                              {:cause …}. A wrong key is a wrong binding.
rete lower_construct (RELAND 4)               unpacks (:E::V {:k v}) via parse_key_first_pairs —
                                              the smem/sqlite failures are rete's.
the wrap's own emissions                      did any site get its fields in map-literal order
                                              rather than declaration order?
```

★ **The shape to suspect first is ORDER.** Every one of these changes turned a positional
construction into a named one. If any consumer still reads positionally while the producer now
writes by name — or vice versa — the value is built with its fields transposed. It type-checks. It
returns the wrong number. **That is exactly `i64(-11)` instead of `21`.**

**3. Only after the 34 are understood:** sweep the `.wat.bad` class (5), and diagnose the 4 misc.

## STOP TRIGGERS

- **STOP-1 — the 34 are called migration residue.** They are wrong answers, not stale syntax.
- **STOP-2 — a test expectation is updated to match the new value.** ⛔ The value is WRONG. Changing
  the assertion would bury a live defect and is the worst available outcome of this stone.
- **STOP-3 — the bisect's first-bad commit is reported as the cause without reading its diff.**
- **STOP-4 — the `.wat.bad` sweep happens first because it is easier.** The 34 are the stone.
- **STOP-5 — the wall is weakened.** It is not implicated: these are runtime values.
