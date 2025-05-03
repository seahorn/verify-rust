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
    KANI_VERSION = '0.43.0'

    @run_before('compile')
    def prepare_build(self):
        # add kani to path
        self.env_vars['PATH'] = f'kani/bin/:{os.environ["PATH"]}'
        # TODO: codegen for all passed directories
        self.build_system.commands = [
            'rm -f rust-toolchain.toml',
            'rustup override set 1.81.0',
            f'cargo install kani-verifier --root $PWD/kani --version {self.KANI_VERSION}',
            f'kani/bin/cargo-kani --only-codegen --manifest-path src/rust-jobs/tinyvec-arrayvec/Cargo.toml',
            'kani/bin/cargo-kani --version'
        ]

    @sanity_function
    def validate_version(self):
        return sn.assert_found(rf"cargo-kani {self.KANI_VERSION}", self.stdout)

class seahorn_build_tests(rfm.CompileOnlyRegressionTest):
    build_system = 'CustomBuild'
    cxx_compiler = variable(str, value='clang++-14')
    c_compiler = variable(str, value='clang-14')
    seahorn_root = variable(str, value='/home/siddharth/seahorn/seahorn/build-rel/run/')
    rust_compiler = variable(str, value='/home/siddharth/.cargo/bin/rustc')
    rust_cargo = variable(str, value='/home/siddharth/.cargo/bin/cargo')

    @run_before('compile')
    def prepare_build(self):  
        self.build_system.builddir = 'rfm-build'
        self.build_system.commands = ['mkdir -p rfm-build && cd rfm-build',
                        ' '.join([
                        'cmake',
                        f'-DCMAKE_CXX_COMPILER={self.cxx_compiler}',
                        f'-DCMAKE_C_COMPILER={self.c_compiler}',
                        f'-DSEAHORN_ROOT={self.seahorn_root}',
                        f'-DRust_COMPILER={self.rust_compiler}',
                        f'-DRust_CARGO={self.rust_cargo}',
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
        kani_bin_dir = f'{self.kani_build_artefacts.stagedir}/kani/bin'
        self.env_vars['PATH'] = f'{kani_bin_dir}:{os.environ["PATH"]}'
        self.time_limit = str(self.timeout_sec) + 's'
        harness, directory = self.test
        if self.tool == 'kani':
            self.executable = f'{self.kani_build_artefacts.stagedir}/kani/bin/cargo-kani'
            self.executable_opts = [
                f'--harness {harness}',
                '--no-unwinding-checks', # for speed
                '--no-assertion-reach-checks', # for speed
                f'--manifest-path {self.kani_build_artefacts.stagedir}/src/rust-jobs/{directory}/Cargo.toml']
        elif self.tool == 'seahorn':
            ctestname = f"{directory}_{harness}" if harness != 'entrypt' else directory
            ctestname += f"_{'sat' if harness.startswith('testfail') else 'unsat'}_test"
            # use latest ctest 3.x so that timeout is working
            # see https://gitlab.kitware.com/cmake/cmake/-/merge_requests/8851
            self.executable = 'ctest'
            self.executable_opts = [
                '--verbose', # verbose needed to get timing of solver
                '--timeout 86400', # 24 hours, this will be ignored 
                                   # since we expect reframe timeout to be smaller than this.
                f'--test-dir {self.seahorn_build_artefacts.stagedir}/rfm-build', f'-R {ctestname}']

    @sanity_function
    def validate_exit(self):
        if self.tool == 'kani':
            return sn.assert_eq(self.job.exitcode, 0)
        elif self.tool == 'seahorn':
            # can be sat or unsat
            return sn.assert_eq(self.job.exitcode, 0) & sn.assert_found(
                r"\w+_(un)?sat_test.*Passed\s+[\d.]+\s+sec", self.stdout
            )

    @performance_function('s')
    def verification_time(self):
        if self.tool == 'kani':
            return sn.extractsingle(r"Verification Time:\s*([\d.]+)s", self.stdout, 1, float)
        elif self.tool == 'seahorn':
            return sn.extractsingle(r"BRUNCH_STAT seahorn_total\s+([\d.]+)", self.stdout, 1, float)
            #return sn.extractsingle(r"(?P<test_name>\w+)_unsat_test.*Passed\s+([\d.]+)\s+sec", self.stdout, 2, float)
    
    @performance_function('formula size')
    def vcc_generated(self):
        if self.tool == 'kani':
            return sn.extractsingle(r"Generated \d+ VCC\(s\), (\d+) remaining after simplification", self.stdout, 1, int)
        elif self.tool == 'seahorn':
            return sn.extractsingle(r"BRUNCH_STAT bmc\.dag_sz\s+(\d+)", self.stdout, 1, int)

    @performance_function('s')
    def solver_time(self):
        if self.tool == 'kani':
            #solver_times = sn.extractall(r"Runtime Solver:\s*([\d.]+)s", self.stdout, 1, float)
            decision_times = sn.extractall(r"Runtime decision procedure:\s*([\d.]+)s", self.stdout, 1, float)
            return sn.sum(decision_times) # + sn.sum(solver_times)
        elif self.tool == 'seahorn':
            bmc_solve_time = sn.extractall(r"BRUNCH_STAT BMC\.solve\s+([\d.]+)", self.stdout, 1, float)
            opsem_simplify_time = sn.extractall(r"BRUNCH_STAT opsem\.simplify\s+([\d.]+)", self.stdout, 1, float)
            return sn.sum(bmc_solve_time) + sn.sum(opsem_simplify_time)