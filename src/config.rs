use std::{fs, path::PathBuf};

use aspasia::{AssSubtitle, Subtitle};
use clap::{builder::ArgPredicate, Args, Parser};
use colored::Colorize;

#[derive(Args, Debug)]
pub(crate) struct IssueConfig {
    #[clap(
        short,
        long,
        help = "Creates GitHub issue; if not specified, Markdown will be printed to stdout",
        action,
        requires = "github_token",
        requires = "github_owner",
        requires = "github_repo",
        requires = "issue_title",
        help_heading = Some("Issue Config"),
    )]
    pub(crate) create_issue: bool,

    #[clap(
        long,
        env,
        help = "GitHub token with access to modify issues",
        help_heading = Some("Issue Config"),
    )]
    github_token: Option<String>,

    #[clap(long, env, help = "Owner of target GitHub repo", help_heading = Some("Issue Config"))]
    github_owner: Option<String>,

    #[clap(long, env, help = "Name of target GitHub repo", help_heading = Some("Issue Config"))]
    github_repo: Option<String>,

    #[clap(long, help = "Title for issue being created", help_heading = Some("Issue Config"))]
    issue_title: Option<String>,
}

#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub(crate) enum ReferenceFormat {
    Full,
    Text,
}

#[derive(Args, Debug)]
pub(crate) struct ReferenceConfig {
    #[clap(
        short('r'),
        long, 
        help = "Add quotation blocks for line references above report entries; defaults to true if --dialogue-file is specified", 
        default_value_if("dialogue_file", ArgPredicate::IsPresent, "true"),
        action, 
        help_heading = Some("Reference Config")
    )]
    pub include_references: bool,

    #[clap(
        long, 
        help = "Categories of reports to include references for",
        help_heading = Some("Reference Config"), 
        default_values = &["Linebreak", "Translation", "Spelling", "Punctuation", "Phrasing", "Note"]
    )]
    pub reference_categories: Vec<String>,

    #[clap(long, help = "Skips picker for refs; will include all refs if set", action, help_heading = Some("Reference Config"))]
    pub skip_reference_picker: bool,

    #[clap(
        value_enum,
        long,
        help = "Format for references; \"full\" includes the full line, \"text\" includes only text",
        help_heading = Some("Reference Config"),
        default_value_t = ReferenceFormat::Full
    )]
    pub reference_format: ReferenceFormat,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Config {
    #[clap(
        short,
        long,
        help = "Path to the mpvQC file to be processed",
        required = true
    )]
    qc_file: PathBuf,

    #[clap(
        short,
        long,
        help = "Path to dialogue file; will include relevant line references for notes if provided"
    )]
    dialogue_file: Option<PathBuf>,

    #[clap(
        short,
        long,
        action,
        help = "Skips the editing prompt; useful for scripts"
    )]
    pub(crate) skip_edit: bool,

    #[command(flatten)]
    pub(crate) reference_options: ReferenceConfig,

    #[command(flatten)]
    pub(crate) issue_options: IssueConfig,
}

impl Config {
    pub(crate) fn read_qc_file(&self) -> anyhow::Result<String> {
        Ok(fs::read_to_string(&self.qc_file)?)
    }

    async fn create_issue(&self, text: String) -> anyhow::Result<()> {
        let octocrab = octocrab::OctocrabBuilder::default()
            .personal_token(self.issue_options.github_token.as_ref().unwrap().clone())
            .build()?;

        octocrab
            .issues(
                &self.issue_options.github_owner.as_ref().unwrap().clone(),
                &self.issue_options.github_repo.as_ref().unwrap().clone(),
            )
            .create(&self.issue_options.issue_title.as_ref().unwrap().clone())
            .body(text)
            .send()
            .await?;

        Ok(())
    }

    pub(crate) async fn output_action(&self, text: String) -> anyhow::Result<()> {
        if self.issue_options.create_issue {
            self.create_issue(text).await.and_then(|_| {
                println!("{}", "Uploading issue...".blue());
                Ok(())
            })?
        } else {
            println!("{}", text);
        }
        Ok(())
    }

    pub(crate) fn read_dialogue_file(&self) -> anyhow::Result<Option<AssSubtitle>> {
        if let Some(dialogue_file) = &self.dialogue_file {
            return Ok(Some(AssSubtitle::from_path(dialogue_file)?));
        }
        Ok(None)
    }
}
