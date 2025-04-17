import reframe as rfm
import reframe.utility.sanity as sn
import reframe.utility.typecheck as typ
from pathlib import Path
import json
import re
import os

def load_tests():
    bmc_tests = os.getenv('BMC_TESTS')
    if bmc_tests:
        try:
            return [tuple(part.strip() for part in test.split(',')) for test in bmc_tests.split(';')]
        except ValueError:
            raise ValueError("BMC_TESTS environment variable must be in the format 'test1,dir1;test2,dir2;...'")
    return []
    
class kani_build_tests(rfm.CompileOnlyRegressionTest):
    build_system = 'CustomBuild'
    #test = parameter(TESTS)

    @run_before('compile')
    def prepare_build(self):
        #TEST_HARNESSES = get_harnesses(f'rust-jobs/{self.directory}')
        #directory = self.test[1]
        self.build_system.commands = [
            f'cargo kani --only-codegen --manifest-path src/rust-jobs/tinyvec-arrayvec/Cargo.toml'
        ]

class seahorn_build_tests(rfm.CompileOnlyRegressionTest):
    build_system = 'CustomBuild'

    @run_before('compile')
    def prepare_build(self):  
        self.build_system.builddir = 'rfm-build'
        self.build_system.commands = ['mkdir -p rfm-build && cd rfm-build',
                                    ' '.join([
                                    'cmake',
                                    '-DCMAKE_CXX_COMPILER=clang++-14',
                                    '-DCMAKE_C_COMPILER=clang-14',
                                    '-DSEAHORN_ROOT=/home/siddharth/seahorn/seahorn/build-rel/run/',
                                    '-DRust_COMPILER=/home/siddharth/.cargo/bin/rustc',
                                    '-DRust_CARGO=/home/siddharth/.cargo/bin/cargo',
                                    '../',
                                    '-GNinja']),
                                    'cmake --build .']

@rfm.simple_test
class BmcProofTest(rfm.RunOnlyRegressionTest):
    #test = variable(typ.Tuple[str, str], value=('', ''))
    test = parameter(load_tests())
    tool = parameter(['kani', 'seahorn'])
    timeout_sec = variable(int, value=0)
    valid_systems = ['*']
    valid_prog_environs = ['builtin']
    kani_build_artefacts = fixture(kani_build_tests, scope='environment')
    seahorn_build_artefacts = fixture(seahorn_build_tests, scope='environment')

    @run_before('run')
    def prepare_test(self):
        self.time_limit = str(self.timeout_sec) + 's'
        harness, directory = self.test
        testname = f"{directory}_{harness}"
        if self.tool == 'kani':
            self.executable = 'cargo'
            self.executable_opts = [
                'kani', f'--harness {harness}',
                f'--manifest-path {self.kani_build_artefacts.stagedir}/src/rust-jobs/{directory}/Cargo.toml']
        elif self.tool == 'seahorn':
            self.executable = 'ctest'
            self.executable_opts = [
                f'--test-dir {self.seahorn_build_artefacts.stagedir}/rfm-build', f'-R {testname}_unsat_test']

    @sanity_function
    def validate_exit(self):
        if self.tool == 'kani':
            return sn.assert_eq(self.job.exitcode, 0)
        elif self.tool == 'seahorn':
            return sn.assert_eq(self.job.exitcode, 0) & sn.assert_found(
                r"\w+_unsat_test.*Passed\s+[\d.]+\s+sec", self.stdout
            )

    @performance_function('s')
    def verification_time(self):
        if self.tool == 'kani':
            return sn.extractsingle(r"Verification Time:\s*([\d.]+)s", self.stdout, 1, float)
        elif self.tool == 'seahorn':
            return sn.extractsingle(r"(?P<test_name>\w+)_unsat_test.*Passed\s+([\d.]+)\s+sec", self.stdout, 2, float)
    # @run_after('setup')
    # def setup_test(self):
    #     self.descr = "Run Kani proofs and report results"
    #     self.valid_systems = ['*']
    #     self.valid_prog_environs = ['*']
    #     self.
    #     self.executable_opts = ['kani']
    #     self.sanity_patterns = sn.assert_found(r'VERIFICATION:- SUCCESSFUL', self.stdout)
    #     self.sourcesdir = self.directory
    #     # Check if the directory exists
    #     if not Path(self.directory).exists():
    #         raise FileNotFoundError(f"Directory not found: {self.directory}")
    
    # @run_before('run')
    # def prepare_tests(self):
    #     all_tests = self.get_kani_tests()
    #     if self.tests:
    #         all_tests = [test for test in all_tests if test in self.tests]
    #     if self.excluded_tests:
    #         all_tests = [test for test in all_tests if test not in self.excluded_tests]
    #     self.test_cases = all_tests

    # def get_kani_tests(self):
    #     lib_rs = Path(self.directory) / "lib.rs"
    #     if not lib_rs.exists():
    #         self.logger.error(f"File not found: {lib_rs}")
    #         return []
    #     with lib_rs.open("r", encoding="utf-8") as f:
    #         contents = f.read()
    #     pattern = re.compile(
    #         r'#\[cfg_attr\(kani,\s*kani::proof\)\](?:\n#.*)*\nfn\s+(\w+)',
    #         re.MULTILINE,
    #     )
    #     return pattern.findall(contents)

    # @run_after('run')
    # def parse_results(self):
    #     results = []
    #     for test in self.test_cases:
    #         result = self.run_kani_test(test)
    #         results.append(result)
    #     with open('kani_results.json', 'w') as f:
    #         json.dump(results, f, indent=2)

    # def run_kani_test(self, testname):
        
    #     self.job.options = [f'--time={self.timeout_sec}s'] if self.timeout_sec else []
    #     self.run()
    #     # Read the output from the stdout file
    #     with open(self.stdout.evaluate(), 'r') as f:
    #         output = f.read()
    #     result, verification_time = self.parse_kani_test_result(output)
    #     return {
    #         "testname": testname,
    #         "result": result,
    #         "verification_time": verification_time,
    #     }

    # def parse_kani_test_result(self, output):
    #     result = "fail"
    #     verification_time = None
    #     if "VERIFICATION:- SUCCESSFUL" in output:
    #         result = "pass"
    #     if output is None or not isinstance(output, str):
    #         self.logger.error("Output is not valid for parsing.")
    #         return result, verification_time

    #     match = re.findall(r"Verification Time:\s*([\d.]+)s", output)
    #     if match:
    #         verification_time = float(match[-1])
    #     return result, verification_time
    