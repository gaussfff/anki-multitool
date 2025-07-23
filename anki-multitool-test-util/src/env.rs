use anyhow::Result;
use std::{env, path::PathBuf};
use tempfile::TempDir;

#[allow(dead_code)]
pub struct TestEnv {
    temp_env_dir: TempDir,
    orig_env_dir: PathBuf,
}

impl TestEnv {
    pub fn init() -> Result<Self> {
        let temp_env_dir = TempDir::new()?;
        let orig_env_dir = env::current_dir()?;

        env::set_current_dir(&temp_env_dir)?;

        Ok(Self {
            temp_env_dir,
            orig_env_dir,
        })
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        env::set_current_dir(&self.orig_env_dir).expect("failed to restore original directory");
    }
}
