import subprocess
import argparse
from enum import Enum
import os
import re
from pathlib import Path

DIRECTORIES = [
  'tinyvec-arrayvec',
]


SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
RUST_JOBS = os.path.join(SCRIPT_DIR, "../src/rust-jobs")
REFRAME_DIR = SCRIPT_DIR
SOURCES_DIR = Path(SCRIPT_DIR).resolve().parent

class RunMode(Enum):
  SUITES = "suites"
  ALL = "all"
  TESTCASES = "testcases"

def get_kani_tests(directory):
  lib_rs = Path(directory) / "lib.rs"
  if not lib_rs.exists():
    return []
  with lib_rs.open("r", encoding="utf-8") as f:
    contents = f.read()
  pattern = re.compile(
    r'#\[cfg_attr\(kani,\s*kani::proof\)\](?:\n#.*)*\n^.*fn\s+(\w+)',
    re.MULTILINE,
  )
  return pattern.findall(contents)

def make_test_dir_pair_list(base_dir, directories):
  r = []
  for dir in directories:
    test_dir = os.path.join(base_dir, dir)
    test_names = get_kani_tests(test_dir)
    test_names_tuple = [(test_name, dir) for test_name in test_names]
    r.extend(test_names_tuple)
  return r

def run_reframe(timeout_sec: int, mode: RunMode, suite_names: list[str] = None, list_tests : bool = False, test_cases : list[(str,str)]= None, report_filename: str = None, 
                cxx_compiler: str = "clang++-14",
                c_compiler: str = "clang-14",
                seahorn_root: str = "/home/siddharth/seahorn/seahorn/build-rel/run/",
                rust_compiler: str = "/home/siddharth/.cargo/bin/rustc",
                rust_cargo: str = "/home/siddharth/.cargo/bin/cargo",
                extra_reframe_options: list[str] = None):
  # TODO: we may need to pass a full <test>.<fixture>.<var> syntax for reframe
  command = [
    "reframe",
    "-c", f"{REFRAME_DIR}/reframe_test.py",
    "-S", f"sourcesdir={SOURCES_DIR}",
    "-S", f"timeout_sec={timeout_sec}",
    "-S", f"BmcProofTest.seahorn_build_artefacts.cxx_compiler={cxx_compiler}",
    "-S", f"BmcProofTest.seahorn_build_artefacts.c_compiler={c_compiler}",
    "-S", f"BmcProofTest.seahorn_build_artefacts.seahorn_root={seahorn_root}",
    "-S", f"BmcProofTest.seahorn_build_artefacts.rust_compiler={rust_compiler}",
    "-S", f"BmcProofTest.seahorn_build_artefacts.rust_cargo={rust_cargo}",
    "-l" if list_tests else "-r", 
    "--performance-report",
    "--report-file", report_filename,
    "--exec-policy", "serial",
    *(extra_reframe_options or [])
  ]
  env = os.environ.copy()
  if mode == RunMode.TESTCASES:
    env['BMC_TESTS'] = ";".join([f"{test},{dir}" for test, dir in test_cases])
  else:
    dirs = None
    if mode == RunMode.SUITES:
      if not suite_names:
        raise ValueError("Suite name must be provided when mode is 'suite'")
      dirs = suite_names
    elif mode == RunMode.ALL:
      dirs = DIRECTORIES  
    test_dir_pairs = make_test_dir_pair_list(RUST_JOBS, dirs)
    env['BMC_TESTS'] = ";".join([f"{test},{dir}" for test, dir in test_dir_pairs])
  try:
    process = subprocess.Popen(command, env=env)
    process.wait()
  except KeyboardInterrupt:
    print("KeyboardInterrupt received, terminating subprocess...")
    process.terminate()
    process.wait()
  except subprocess.CalledProcessError as e:
    print(f"Error while running reframe: {e}")
  except Exception as e:
    print(f"Unexpected error: {e}")

def main():
  parser = argparse.ArgumentParser(description="Run reframe benchmark tests.")
  parser.add_argument("--timeout_sec", type=int, required=True, help="Timeout in seconds.")
  parser.add_argument("--mode", type=RunMode, choices=list(RunMode), required=True, help="Run mode: suite or all.")
  parser.add_argument("--suite-names", type=str, nargs='+', help="List of suite names to run (required if mode is 'suite').")
  parser.add_argument("--report-file", type=str, required=True, help="Name of the report file.")
  parser.add_argument("--extra-reframe-options", type=str, nargs='*', default=[], help="Additional options to pass to reframe.")
  parser.add_argument("--testcases", type=lambda s: tuple(s.split(',')), nargs='*', help="List of test_name and test_dir pairs in the format 'test_name,test_dir' (required if mode is 'testcases').")
  parser.add_argument("--list-tests", action="store_true", help="If set, lists the tests instead of running them.")
  parser.add_argument("--cxx-compiler", type=str, default="clang++-14", help="Path to the C++ compiler (default: clang++-14).")
  parser.add_argument("--c-compiler", type=str, default="clang-14", help="Path to the C compiler (default: clang-14).")
  parser.add_argument("--seahorn-root", type=str, default="/home/siddharth/seahorn/seahorn/build-rel/run/", help="Path to the Seahorn root directory.")
  parser.add_argument("--rust-compiler", type=str, default="/home/siddharth/.cargo/bin/rustc", help="Path to the Rust compiler (default: /home/siddharth/.cargo/bin/rustc).")
  parser.add_argument("--rust-cargo", type=str, default="/home/siddharth/.cargo/bin/cargo", help="Path to the Rust Cargo (default: /home/siddharth/.cargo/bin/cargo).")

  args = parser.parse_args()

  run_reframe( 
    timeout_sec=args.timeout_sec,
    mode=args.mode,
    suite_names=args.suite_names,
    list_tests = args.list_tests,
    test_cases= args.testcases,
    report_filename=args.report_file,
    cxx_compiler=args.cxx_compiler,
    c_compiler=args.c_compiler,
    seahorn_root=args.seahorn_root,
    rust_compiler=args.rust_compiler,
    rust_cargo=args.rust_cargo,
    extra_reframe_options = args.extra_reframe_options
  )

if __name__ == "__main__":
  main()