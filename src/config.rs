use std::{fs, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Config {
    #[clap(long, env, help = "GitHub token with access to modify issues")]
    github_token: String,

    #[clap(long, env, help = "Owner of target GitHub repo")]
    github_owner: String,

    #[clap(long, env, help = "Name of target GitHub repo")]
    github_repo: String,

    #[clap(short, long, help = "Title for issue being created")]
    issue_title: String,

    #[clap(short, long, help = "Path to the mpvQC file to be proceesed")]
    qc_file: PathBuf,

    #[clap(
        short,
        long,
        action,
        help = "Skips the editing prompt, useful for scripts"
    )]
    pub(crate) skip_edit: bool,
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
