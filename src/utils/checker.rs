use std::process::{Command, Output};
use std::str;

use colored::Colorize;

use crate::utils::utils;

pub struct DependencyChecker {
    auto_install: bool,
}

impl DependencyChecker {
    pub fn new(auto_install: bool) -> Self {
        Self { auto_install }
    }

    pub fn check(&self) {
        let missing_deps = self.check_dep_status();
        if self.auto_install && !missing_deps.is_empty() {
            let installer = DependencyInstaller::new(&missing_deps);
            installer.install();
        }
    }

    fn check_dep_status(&self) -> Vec<Dependencies> {
        utils::get_system_info().expect("Failed to get system info");
        log::info!("Dependencies:");
        let mut missing_deps = Vec::new();
        if !self.is_fastp_installed() {
            missing_deps.push(Dependencies::Fastp);
        }
        if !self.is_spades_installed() {
            missing_deps.push(Dependencies::Spades);
        }
        missing_deps
    }

    fn is_fastp_installed(&self) -> bool {
        let fastp_output = self.check_fastp();
        match fastp_output {
            Some(out) => {
                log::info!("{:18}: {}", out, "[OK]".green());
                true
            }
            None => {
                log::info!("{:18}: {}", "fastp", "[NOT FOUND]".red());
                false
            }
        }
    }

    fn is_spades_installed(&self) -> bool {
        let spades_output = self.check_spades();
        match spades_output {
            Some(out) => {
                log::info!("{:18}: {}", out, "[OK]".green());
                true
            }
            None => {
                log::info!("{:18}: {}", "spades", "[NOT FOUND]".red());
                false
            }
        }
    }

    fn check_fastp(&self) -> Option<String> {
        let out = Command::new("fastp").arg("--version").output();
        match out {
            Ok(out) => Some(
                str::from_utf8(&out.stdout)
                    .expect("Failed getting fastp name")
                    .trim()
                    .to_string(),
            ),
            Err(_) => None,
        }
    }

    fn check_spades(&self) -> Option<String> {
        let out = Command::new("spades.py").arg("--version").output();
        match out {
            Ok(out) => Some(
                str::from_utf8(&out.stdout)
                    .expect("Failed getting spades name")
                    .trim()
                    .to_string(),
            ),
            Err(_) => None,
        }
    }
}

enum Dependencies {
    Fastp,
    Spades,
}

struct DependencyInstaller<'a> {
    deps: &'a [Dependencies],
}

impl<'a> DependencyInstaller<'a> {
    pub fn new(deps: &'a [Dependencies]) -> Self {
        Self { deps }
    }

    pub fn install(&self) {
        if self.is_conda_installed() {
            log::info!("Installing missing dependencies...");
            self.add_bioconda_channel();
            self.add_conda_forge_channel();
            self.deps.iter().for_each(|dep| match dep {
                Dependencies::Fastp => self.install_deps("fastp"),
                Dependencies::Spades => self.install_deps("spades"),
            });
        }
    }

    fn is_conda_installed(&self) -> bool {
        let conda_output = self.check_conda();
        match conda_output {
            Some(out) => {
                log::info!("{:18}: {}", out, "[OK]".green());
                true
            }
            None => {
                log::warn!("Conda is not installed. Please install conda first.");
                log::info!("Visit https://docs.conda.io/projects/conda/en/latest/user-guide/install/index.html for installation instructions.");
                log::info!("For command-line usage we recommend using Miniconda or Miniforge.");
                false
            }
        }
    }

    fn add_bioconda_channel(&self) {
        log::info!("Adding bioconda channel...");
        let out = Command::new("conda")
            .args(&["config", "--add", "channels", "bioconda"])
            .output();
        match out {
            Ok(out) => self.check_command_status("bioconda", &out),
            Err(e) => log::error!("{}", e),
        }
    }

    fn add_conda_forge_channel(&self) {
        log::info!("Adding conda-forge channel...");
        let out = Command::new("conda")
            .args(&["config", "--add", "channels", "conda-forge"])
            .output();
        match out {
            Ok(out) => self.check_command_status("conda-forge", &out),
            Err(e) => log::error!("{}", e),
        }
    }

    fn install_deps(&self, app: &str) {
        log::info!("Installing {}...", app.to_uppercase());
        let out = Command::new("conda")
            .args(&["install", "-c", "bioconda", app])
            .output();
        match out {
            Ok(out) => self.check_command_status(app, &out),
            Err(e) => log::error!("{}", e),
        }
    }

    fn check_conda(&self) -> Option<String> {
        let out = Command::new("conda").arg("--version").output();
        match out {
            Ok(out) => Some(
                str::from_utf8(&out.stdout)
                    .expect("Failed getting conda name")
                    .trim()
                    .to_string(),
            ),
            Err(_) => None,
        }
    }

    fn check_command_status(&self, app: &str, command_out: &Output) {
        if command_out.status.success() {
            log::info!("{:18}: {}", app, "[SUCCESS]".green());
        } else {
            log::error!(
                "{}",
                str::from_utf8(&command_out.stderr).expect("Failed getting error")
            );
            log::error!("{:18}: {}", "Command", "[FAILED]".red());
        }
    }
}
