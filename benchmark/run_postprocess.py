"""
This script processes a JSON input file and converts it into a CSV format.

The CSV file contains the following columns:
1. Test_name: Extracted test information from the test case name.
  - Derived from the "name" field in the JSON test case.
  - Processed to extract specific details (e.g., test name and parameters).

2. Test_suite: The suite (dire) associated with the test case.
  
3. Tool: The tool associated with the test case.
  - Extracted from the "name" field in the JSON test case.

4. Verification_Time: The time taken for verification.
  - Extracted from the "perfvalues" field in the JSON test case.
  - If no performance value is available, this field will be empty.

5. Result: The result of the test case.
  - If the test case failed, this column will contain "fail" along with the failure reason.
  - Otherwise, it will contain the result string (e.g., "pass").
"""
import json
import csv
import sys
import os
import logging
import re

import pdb  # Importing pdb for debugging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')


def process_input(input_file, output_file, overwrite=False):
  try:
    # Load the JSON input file
    with open(input_file, 'r') as infile:
      report = json.load(infile)
    # Prepare data for CSV
    csv_data = []
    for run in report.get("runs", []):
      for testcase in run.get("testcases", []):
        # Extract test name and tool
        test_name = testcase.get("name", "")
        if not test_name:
            logging.warning("Test name is empty. Skipping this testcase.")
            continue
        parts = test_name.split("%")
        if len(parts) < 3:
          logging.info(f"Unexpected test name format: {test_name}. Skipping this testcase.")
          continue
        # Use regex to extract test_info and tool
        # Assuming test_name format: "BmcProofTest %test=('value1', 'value2') %tool=tool_name"
        match = re.match(r"^BmcProofTest\s+%test=\(('([^']*)',\s*'([^']*)')\)\s+%tool=([\w-]+)", test_name)
        if not match:
            logging.info(f"Unexpected test name format: {test_name}. Skipping this testcase.")
            continue
        # Extracted values
        name = match.group(2)
        test_suite = match.group(3)
        tool = match.group(4)  # Extract tool name
        wall_time = testcase.get('time_run', None)
        # Skip if the result is "fail"
        if testcase.get("result") == "fail":
          logging.info(f"testcase '{test_name}' 'failed'.")
          fail_reason = testcase.get("fail_reason", "Unknown reason")
          csv_data.append([name, test_suite, tool, 0, # formula_size
                          0, # solver_time
                          0, # verification_time
                          wall_time, 
                          f"fail: {fail_reason}"])
        else:
          # Extract pass string
          expect_pass = testcase.get("result")
          # Extract performance values
          perf_verification_values = testcase.get("perfvalues", {}).get("generic:default:verification_time", [])
          verification_time = perf_verification_values[0] if len(perf_verification_values) > 0 else None
          perf_solver_values = testcase.get("perfvalues", {}).get("generic:default:solver_time", [])
          solver_time = perf_solver_values[0] if len(perf_solver_values) > 0 else None
          formula_size_values = testcase.get("perfvalues", {}).get("generic:default:vcc_generated", []) 
          formula_size = formula_size_values[0] if len(formula_size_values) > 0 else None
          # Append row to CSV data
          csv_data.append([name, test_suite, tool, formula_size, solver_time, verification_time, wall_time, expect_pass])
        logging.info(f"Added to csv_data: {csv_data[-1]}")

    # Check if output file exists
    file_exists = os.path.exists(output_file)
    existing_records = set()

    if file_exists:
      # Read existing records if the file exists
      with open(output_file, 'r', newline='') as outfile:
        reader = csv.reader(outfile)
        _header = next(reader, None)  # Skip the header
        for row in reader:
          if len(row) >= 2:  # Ensure there are enough columns
            existing_records.add((row[0], row[1]))  # Add (Test_Info, Tool) to the set

    # Check for duplicate records
    for row in csv_data:
      if (row[0], row[1]) in existing_records and not overwrite:
        raise FileExistsError(
          f"Record with Test_Info '{row[0]}' and Tool '{row[1]}' already exists in '{output_file}'. "
          "Use overwrite option to replace it."
        )

    # Write to CSV file
    # Open the file in write mode if it doesn't exist, otherwise append to preserve existing records
    with open(output_file, 'a', newline='') as outfile:
      writer = csv.writer(outfile)
      if not file_exists:
        # Write header only if the file is being created
        writer.writerow(["Test_Name", "Test_Suite", "Tool", "Formula_Size", "Solver_Time", "Verification_Time", "Wall_Time", "Result"])
      # Write rows, skipping duplicates
      for row in csv_data:
        if (row[0], row[1]) not in existing_records or overwrite:
          writer.writerow(row)

    logging.info(f"Data successfully written to '{output_file}'.")

  except FileExistsError as e:
    logging.error(e)
    sys.exit(1)
  except Exception as e:
    logging.error(f"An error occurred: {e}")
    sys.exit(1)

def main():
  if len(sys.argv) < 3:
    logging.error("Usage: python run_postprocess.py <input_file> <output_file> [--overwrite]")
    sys.exit(1)

  input_file = sys.argv[1]
  output_file = sys.argv[2]
  overwrite = "--overwrite" in sys.argv

  process_input(input_file, output_file, overwrite)

if __name__ == "__main__":
  main()