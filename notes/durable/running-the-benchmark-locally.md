# Running the ReFrame benchmark locally

[FACT] The benchmark is the only thing that exercises **both** tools on the same
harnesses. CI runs it (`.github/workflows/main.yml`, `run benchmarks`) but only
after ctest passes, and inside the Docker image — so on a workstation none of the
image's setup applies and four things must be supplied by hand. All four are
verified: a full 304-case run completed 2026-08-12.
→ ../journal/2026-08/2026-08-12-first-local-benchmark-run-on-dev18.md

## What the host does not provide

| need | why it is missing | fix |
|---|---|---|
| `reframe` | never installed outside the image; PEP 668 blocks `pip install --user` on 24.04 | venv |
| linkable `clang++-18` | gcc-14 selected, no `libstdc++-14-dev` | `apt install libstdc++-14-dev`, or the wrapper below without sudo |
| `LD_LIBRARY_PATH` | not exported by any shell rc | export, below |
| `cargo-kani` | — | already installed here (`~/.kani/kani-0.53.0`) |

## Recipe

```bash
# 1. reframe in a venv (must match the Dockerfile pin)
uv venv rfmenv && uv pip install --python ./rfmenv/bin/python 'ReFrame-HPC==4.8.0'

# 2. clang++-18 must be able to link. Either fix it once for the host
#      sudo apt install libstdc++-14-dev      # safe here: no dev14 build on this box
#    or, without sudo, wrap it (see building-llvm-stack-on-ubuntu-24.04.md §1b)
mkdir -p bin
for c in clang-18 clang++-18; do
  printf '#!/bin/sh\nexec /usr/bin/%s --gcc-install-dir=/usr/lib/gcc/x86_64-linux-gnu/13 "$@"\n' "$c" > bin/$c
  chmod +x bin/$c
done
export PATH="$PWD/bin:$PWD/rfmenv/bin:$PATH"

# 3. solvers
SEA=/home/siddharth/seahorn/seahorn
export LD_LIBRARY_PATH="$SEA/deps/z3-4.8.9/lib:$SEA/deps/yices-2.6.1/lib:$SEA/build-rel/run/lib"

# 4. cap memory -- a single cbmc on the vector jobs has been seen at 22-30GB RSS
ulimit -v 20000000

python3 <repo>/benchmark/run_benchmark.py \
  --timeout_sec=300 --report-file=report.json --mode=testcases \
  --cxx-compiler=clang++-18 --c-compiler=clang-18 \
  --seahorn-root=$SEA/build-rel/run/ \
  --rust-compiler=$(rustup which rustc) --rust-cargo=$(rustup which cargo) \
  --testcases $(cat <repo>/benchmark/all_benchmarks.txt)
```

Run it from a scratch directory, not the repo: ReFrame stages a **full copy of
the source tree per fixture** and builds the whole CMake project there.

## Facts worth knowing before trusting a run

[FACT] Both failure modes below make every seahorn case fail *identically to a
verification failure* — `BmcProofTest ... %tool=seahorn` FAIL, sanity phase. The
tell is that **all** of them fail, including trivial jobs, and in ~0.1s. Read one
stage dir's `rfm_job.out` before believing any seahorn-side red.

[FACT] `--timeout_sec` is the ReFrame per-case limit. 300s is what
`all_benchmarks.txt` was measured under (`cadd21b`); CI uses 1800s. A case that
exceeds the budget or the `ulimit` is reported as a failure, not as a timeout —
resource limits and verdicts are indistinguishable in the report.

[FACT] Serial execution (`--exec-policy serial`) is set in `run_benchmark.py` and
should stay that way. Parallel cbmc runs are what OOM-killed the box on
2026-08-07.

[EMP] Cost, verified 2026-08-12: two fixture builds ~4 min (full CMake build of
63 jobs + `cargo install kani-verifier`), then 302 cases in ~76 min. Stage tree
is a few GB.

## Postprocessing

[FACT] `run_postprocess.py <report.json> <out.csv>` always opens the output in
**append** mode; `--overwrite` only skips the duplicate check, it does not
truncate. To regenerate a CSV rather than grow it, write to a new path and move
it over.

[FACT] `postprocess_visualize.ipynb` is pinned to the AoP/EoP study — cell 2
reads `benchmark_postprocess_eop.csv`, cells 12-13 read
`benchmark_postprocess_aop.csv`. Re-executing it regenerates
`verification_time.pdf`, `solver_time.pdf`, `formula_size_filtered.pdf` and
`aop_vs_eop_time.pdf` from *that* dataset. Running an `all_benchmarks.txt` report
through it means repointing cell 2 first, and the result is a different study —
that list mixes EoP jobs with their `-aop` twins.

[FACT] `vcc_generated` is kani's VCC count vs seahorn's `bmc.dag_sz` — different
units on one axis. Correlation only.

## See also

- building-llvm-stack-on-ubuntu-24.04.md (§1, §1b — the clang/GCC selection trap)
- kani-version-to-rust-toolchain-map.md (why the pin is exactly 0.53.0)
- ../journal/2026-08/2026-08-12-first-local-benchmark-run-on-dev18.md
