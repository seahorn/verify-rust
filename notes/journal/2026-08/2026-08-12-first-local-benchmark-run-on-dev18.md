# First full benchmark run on dev18: 304/304, and two environment blockers

The ReFrame benchmark (`benchmark/run_benchmark.py`, both tools) had never been
run to completion on dev18 — CI skips the step whenever ctest fails, and the
2026-08-07 sweep that produced `all_benchmarks.txt` was a kani-side script, not
this harness. Ran it locally over the committed list.

## Result

[OBS 2026-08-12] **304/304 cases passed** (151 pairs x 2 tools, plus the 2
fixtures): `0 failure(s), 0 skipped, 0 aborted`, 80 minutes wall clock, serial.
So `all_benchmarks.txt` as regenerated in `cadd21b` is accurate — every pair in
it still verifies under both tools, with no name drift and no stale entries.

| | kani 0.53.0 | seabmc (dev18) |
|---|---|---|
| total verification_time | 403.5s | 331.3s |
| median | 0.312s | 0.010s |
| max | 128.1s | 107.5s |
| total solver_time | 325.9s | 325.5s |
| mean over passed pairs | 2.67s | 2.19s |

[OBS] seabmc is faster on **131 of the 140** pairs where the ratio is defined
(median 8.9x). The other 11 pairs report a seahorn `verification_time` of
exactly 0.00 — the BRUNCH_STAT is printed at 10ms resolution — so no ratio can
be formed and they are excluded rather than counted as wins.

[OBS] **That 8.9x is fixed overhead, not solving power.** Total solver time is a
dead heat: 325.9s vs 325.5s, a 0.1% difference. The gap is per-invocation cost on
jobs where nothing hard happens (kani ~0.5s vs seahorn ~0.01s on
`vec-sort-reverse/entrypt` and friends). On the pairs that do real work the two
tools trade places:

| pair | kani | seabmc |
|---|---|---|
| tinyvec-arrayvec/testfail_splice_panic | 23.3s | **107.5s** |
| tinyvec-arrayvec/test_splice | 18.9s | **40.6s** |
| smallvec-allocation-bound2/test_drain | **72.0s** | 9.8s |
| smallvec-allocation-bound2/test_insert_many | **128.1s** | 74.9s |

[HYP] The split looks structural rather than incidental: CBMC does better on the
splice/panic harnesses, seabmc on the bounded smallvec allocation harnesses. Two
examples each is not enough to claim a rule — worth checking against the next run
before repeating it.

[OBS] `smallvec-allocation-bound2/test_insert_many` took 74.9s here. The
2026-07-24 vacuity audit timed it out at a 90s budget and this session's earlier
`--vac` run needed 483s — consistent, since `--vac` adds the vacuity queries on
top of the plain `unsat` proof.

## Caution: the formula-size axis compares unlike things

[FACT] `vcc_generated` extracts kani's *VCC count* ("N remaining after
simplification") and seahorn's `BRUNCH_STAT bmc.dag_sz`, a DAG node count
(`reframe_test.py:160-165`). Medians this run: kani 121, seahorn 51; totals
82,284 vs 664,051. These are different units. `formula_size_filtered.pdf` plots
one against the other, so read it as a correlation, never as "tool X builds a
smaller formula".

## Neither blocker below applies to CI

[FACT] The dev18 image is built on **Ubuntu 22.04**, which has no gcc-14 in its
archive at all, so the clang/GCC selection trap cannot arise there. Read it off
the registry without pulling:

```bash
TOKEN=$(curl -s "https://ghcr.io/token?scope=repository:seahorn/seahorn-llvm18:pull&service=ghcr.io" \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])')
# manifest -> config digest -> blob; Labels shows org.opencontainers.image.version: 22.04
```

[FACT] The image also needs no `LD_LIBRARY_PATH` — its config sets only `PATH`
and `SEAHORN`, and CI run `31130269881` ran 210 ctest cases on dev18, so the
solver libs resolve by rpath/ldconfig there. Both problems are host-only.

## Two environment blockers, both fatal, neither a verification problem

The first two launch attempts died on these. Both are host-environment skew of
exactly the kind durable/building-llvm-stack-on-ubuntu-24.04.md is about.

### 1. clang++-18 cannot link at all

```
$ echo 'int main(){}' > t.cxx && clang++-18 t.cxx -o t.out
/usr/bin/ld: cannot find -lstdc++
```

cmake rejects the compiler at `project()` time, so the seahorn fixture never
builds and all 151 seahorn cases fail as dependency errors. Same root cause as
the clang-14 case already documented: `/usr/lib/gcc/x86_64-linux-gnu/14` exists
(from `libgcc-14-dev`) without `libstdc++-14-dev`, and clang selects the highest
version it finds. → durable note §1b for the clang-18 form.

[OBS 2026-08-12] `sudo apt install libstdc++-14-dev` is the right fix **here**:
the user confirmed no `build-14` is needed on this box, which is the only thing
that made the package a trap (§1 — clang-14 cannot parse those headers). The
`--gcc-install-dir=…/13` wrappers this run used are the no-sudo equivalent, and
what CI-style hosts carrying both toolchains should use.

### 2. LD_LIBRARY_PATH unset kills every seahorn case

```
seapp: error while loading shared libraries: libz3.so.4.8: cannot open shared object file
```

[FACT] Not set in `~/.bashrc` or `~/.profile`, and ReFrame passes the parent
environment through, so a detached run inherits nothing. Every seahorn ctest
exits non-zero in 0.1s — which looks exactly like a verification failure in the
ReFrame summary, since the sanity function only checks the exit code and the
`Passed` line. Export before running:

```bash
SEA=/home/siddharth/seahorn/seahorn
export LD_LIBRARY_PATH="$SEA/deps/z3-4.8.9/lib:$SEA/deps/yices-2.6.1/lib:$SEA/build-rel/run/lib"
```

Verify in isolation before committing to an 80-minute run:

```bash
ctest --test-dir <fixture>/rfm-build -R '^custom-print_unsat_test$'   # Passed 31.69 sec
```

## Reproducing

Full recipe (reframe venv, the clang wrappers, budget) is in
durable/running-the-benchmark-locally.md. Budget used: `--timeout_sec=300` and
`ulimit -v 20000000`, matching the run `all_benchmarks.txt` was generated from,
so the numbers are comparable to `cadd21b`. CI uses 1800s and no memory cap.

[OPEN] The `run benchmarks` CI step *still* has never executed on dev18 — this
was a local run. Nothing here proves the step works in the container.

## Postprocessing

[OBS] `benchmark/benchmark_postprocess.csv` regenerated from this run's
`report.json`: 302 rows, all `pass`, 151 pairs. The previous contents were 292
rows over 73 pairs with each pair appearing up to 4 times (the script opens the
output in append mode, so re-runs accumulate), 16 of them failures, CRLF line
endings, and dev14 timings.

[FACT] `run_postprocess.py --overwrite` does **not** truncate — the file is
always opened `'a'`. `--overwrite` only suppresses the duplicate-record check, so
re-running against an existing file appends a second copy either way. Write to a
fresh path and move it into place.

[FACT] The notebook is wired to the **EoP study**, not to this list:
`postprocess_visualize.ipynb` cell 2 reads `benchmark_postprocess_eop.csv`, and
cells 12-13 read `benchmark_postprocess_aop.csv` to produce
`aop_vs_eop_time.pdf`. So the committed `verification_time.pdf`,
`solver_time.pdf` and `formula_size_filtered.pdf` are EoP-dataset artifacts.
This run mixes EoP jobs with their `-aop` twins (`all_benchmarks.txt` contains
both `tinyvec-arrayvec` and `tinyvec-arrayvec-aop`), so it is **not** a drop-in
replacement for that dataset and the committed PDFs were left alone. Regenerated
copies from this run's CSV are in the scratchpad only.

## See also

- ../../durable/running-the-benchmark-locally.md
- ../../durable/building-llvm-stack-on-ubuntu-24.04.md
- 2026-08-07-kani-vs-seabmc-sweep.md
- ../../durable/panic-modes-aop-vs-eop.md
