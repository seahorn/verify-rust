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

def get_kani_tests(directory):
  lib_rs = Path(directory) / "lib.rs"
  if not lib_rs.exists():
    return []
  with lib_rs.open("r", encoding="utf-8") as f:
    contents = f.read()
  pattern = re.compile(
    r'#\[cfg_attr\(kani,\s*kani::proof\)\](?:\n#.*)*\nfn\s+(\w+)',
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

def run_reframe(timeout_sec: int, mode: RunMode, suite_names: list[str] = None, report_filename: str = None):
  
  command = [
    "reframe",
    "-c", f"{REFRAME_DIR}/reframe_test.py",
    "-S", f"sourcesdir={SOURCES_DIR}",
    "-S", f"timeout_sec={timeout_sec}",
    "-r", "--performance-report",
    "--report-file", report_filename,
    "--exec-policy", "serial",
  ]
  dirs = None
  if mode == RunMode.SUITES:
    if not suite_names:
      raise ValueError("Suite name must be provided when mode is 'suite'")
    dirs = suite_names
  elif mode == RunMode.ALL:
    dirs = DIRECTORIES  
  test_dir_pairs = make_test_dir_pair_list(RUST_JOBS, dirs)
  env = os.environ.copy()
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

# add post processing on success
# report["runs"][0]["testcases"][3]["perfvalues"]["generic:default:verification_time"]

def main():
  parser = argparse.ArgumentParser(description="Run reframe benchmark tests.")
  parser.add_argument("--timeout_sec", type=int, required=True, help="Timeout in seconds.")
  parser.add_argument("--mode", type=RunMode, choices=list(RunMode), required=True, help="Run mode: suite or all.")
  parser.add_argument("--suite_names", type=str, nargs='+', help="List of suite names to run (required if mode is 'suite').")
  parser.add_argument("--report-file", type=str, required=True, help="Name of the report file.")
  args = parser.parse_args()
  
  run_reframe(
    timeout_sec=args.timeout_sec,
    mode=args.mode,
    suite_names=args.suite_names,
    report_filename=args.report_file
  )

if __name__ == "__main__":
  main()