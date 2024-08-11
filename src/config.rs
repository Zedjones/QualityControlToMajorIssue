use std::{fs, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Config {
    #[clap(long, env)]
    github_token: String,

    #[clap(long, env)]
    github_owner: String,

    #[clap(long, env)]
    github_repo: String,

    #[clap(short, long)]
    issue_title: String,

    #[clap(short, long)]
    qc_file: PathBuf,
}

impl Config {
    pub(crate) fn read_qc_file(&self) -> anyhow::Result<String> {
        Ok(fs::read_to_string(&self.qc_file)?)
    }

    pub(crate) async fn create_issue(&self, text: String) -> anyhow::Result<()> {
        let octocrab = octocrab::OctocrabBuilder::default()
            .personal_token(self.github_token.clone())
            .build()?;

        octocrab
            .issues(&self.github_owner, &self.github_repo)
            .create(&self.issue_title)
            .body(text)
            .send()
            .await?;

        Ok(())
    }
}
