// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::process::Command as StdCommand;

use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    sub: SubCommand,
}

impl Command {
    fn run(self) {
        match self.sub {
            SubCommand::Build(cmd) => cmd.run(),
            SubCommand::Check(cmd) => cmd.run(),
            SubCommand::Lint(cmd) => cmd.run(),
            SubCommand::Test(cmd) => cmd.run(),
        }
    }
}

#[derive(Subcommand)]
enum SubCommand {
    #[clap(about = "Compile workspace packages.")]
    Build(CommandBuild),
    #[clap(about = "Check the library with each optional feature independently.")]
    Check(CommandCheck),
    #[clap(about = "Run format and clippy checks.")]
    Lint(CommandLint),
    #[clap(about = "Run offline tests, or opt into real PostgreSQL workflows.")]
    Test(CommandTest),
}

#[derive(Parser)]
struct CommandBuild {
    #[arg(long, help = "Assert that `Cargo.lock` will remain unchanged.")]
    locked: bool,
}

impl CommandBuild {
    fn run(self) {
        run_command(make_build_cmd(self.locked));
    }
}

#[derive(Parser)]
struct CommandCheck;

impl CommandCheck {
    fn run(self) {
        // Keep this list in sync with pqb/Cargo.toml. Each invocation avoids workspace feature
        // unification with the driver tests and example applications.
        for feature in [
            None,
            Some("postgres"),
            Some("sqlx"),
            Some("with-chrono"),
            Some("with-json"),
            Some("with-rust_decimal"),
            Some("with-uuid"),
        ] {
            let mut cmd = find_command("cargo");
            cmd.args([
                "check",
                "--locked",
                "--package",
                "pqb",
                "--lib",
                "--no-default-features",
            ]);
            if let Some(feature) = feature {
                cmd.args(["--features", feature]);
            }
            run_command(cmd);
        }
    }
}

#[derive(Parser)]
struct CommandTest {
    #[arg(long, help = "Do not capture test output.")]
    no_capture: bool,

    #[arg(long, help = "Run database tests using PQB_TEST_DATABASE_URL.")]
    postgres: bool,
}

impl CommandTest {
    fn run(self) {
        if self.postgres {
            assert!(
                std::env::var_os("PQB_TEST_DATABASE_URL").is_some_and(|url| !url.is_empty()),
                "set PQB_TEST_DATABASE_URL to a disposable PostgreSQL database"
            );
            let mut cmd = find_command("cargo");
            cmd.args([
                "test",
                "--locked",
                "--package",
                "tests-integration",
                "--test",
                "postgres",
                "--",
                "--ignored",
            ]);
            if self.no_capture {
                cmd.arg("--nocapture");
            }
            run_command(cmd);
            return;
        }
        run_command(make_test_cmd(self.no_capture, false));
        run_command(make_test_cmd(self.no_capture, true));
    }
}

#[derive(Parser)]
#[clap(name = "lint")]
struct CommandLint {
    #[arg(long, help = "Automatically apply lint suggestions.")]
    fix: bool,
}

impl CommandLint {
    fn run(self) {
        run_command(make_clippy_cmd(self.fix));
        run_command(make_format_cmd(self.fix));
        run_command(make_taplo_cmd(self.fix));
        run_command(make_typos_cmd());
        run_command(make_hawkeye_cmd(self.fix));
    }
}

fn find_command(cmd: &str) -> StdCommand {
    match which::which(cmd) {
        Ok(exe) => {
            let mut cmd = StdCommand::new(exe);
            cmd.current_dir(env!("CARGO_WORKSPACE_DIR"));
            cmd
        }
        Err(err) => {
            panic!("{cmd} not found: {err}");
        }
    }
}

fn ensure_installed(bin: &str, crate_name: &str) {
    if which::which(bin).is_err() {
        let mut cmd = find_command("cargo");
        cmd.args(["install", crate_name]);
        run_command(cmd);
    }
}

fn run_command(mut cmd: StdCommand) {
    println!("{cmd:?}");
    let status = cmd.status().expect("failed to execute process");
    assert!(status.success(), "command failed: {status}");
}

fn make_build_cmd(locked: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args([
        "build",
        "--workspace",
        "--all-features",
        "--tests",
        "--examples",
        "--benches",
        "--bins",
    ]);
    if locked {
        cmd.arg("--locked");
    }
    cmd
}

fn make_test_cmd(no_capture: bool, all_features: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args(["test", "--locked"]);
    if all_features {
        cmd.args(["--workspace", "--all-features"]);
    } else {
        cmd.args(["--package", "pqb", "--no-default-features"]);
    }
    if no_capture {
        cmd.args(["--", "--nocapture"]);
    }
    cmd
}

fn make_format_cmd(fix: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args(["+nightly", "fmt", "--all"]);
    if !fix {
        cmd.arg("--check");
    }
    cmd
}

fn make_clippy_cmd(fix: bool) -> StdCommand {
    let mut cmd = find_command("cargo");
    cmd.args([
        "+nightly",
        "clippy",
        "--tests",
        "--all-features",
        "--all-targets",
        "--workspace",
    ]);
    if fix {
        cmd.args(["--allow-staged", "--allow-dirty", "--fix"]);
    } else {
        cmd.args(["--", "-D", "warnings"]);
    }
    cmd
}

fn make_hawkeye_cmd(fix: bool) -> StdCommand {
    ensure_installed("hawkeye", "hawkeye");
    let mut cmd = find_command("hawkeye");
    if fix {
        cmd.args(["format"]);
    } else {
        cmd.args(["check"]);
    }
    cmd
}

fn make_typos_cmd() -> StdCommand {
    ensure_installed("typos", "typos-cli");
    find_command("typos")
}

fn make_taplo_cmd(fix: bool) -> StdCommand {
    ensure_installed("taplo", "taplo-cli");
    let mut cmd = find_command("taplo");
    if fix {
        cmd.args(["format"]);
    } else {
        cmd.args(["format", "--check"]);
    }
    cmd
}

fn main() {
    let cmd = Command::parse();
    cmd.run()
}
