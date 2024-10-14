use std::collections::HashMap;
use std::time::Duration;

use dotenv::dotenv;

use clap::Parser as ClapParser;
use config::{Config, ReferenceFormat};
use editor::Editor;
use hhmmss::Hhmmss;
use pest::{iterators::Pair, Parser};
use qc_issue::{MPVQCParser, QCIssue, Rule};
use subs::Subs;

mod config;
mod editor;
mod qc_issue;
mod subs;

fn format_into_md(config: &Config, issue_map: HashMap<String, Vec<QCIssue>>) -> String {
    let mut markdown_string = String::new();
    let mut issue_map_vec = issue_map
        .into_iter()
        .collect::<Vec<(String, Vec<QCIssue>)>>();

    issue_map_vec.sort_by_key(|(_, issues)| issues.len());
    issue_map_vec.reverse();

    for (issue_type, issues) in issue_map_vec.iter_mut() {
        markdown_string += &format!("# {}\n", issue_type);
        issues.sort();
        for issue in issues {
            if config.reference_options.include_references
                && config
                    .reference_options
                    .reference_categories
                    .contains(&issue_type)
            {
                if issue.matching_lines.is_empty() {
                    markdown_string += "> \n";
                } else {
                    for line in &issue.matching_lines {
                        markdown_string += &format!("> {}\n", {
                            if config.reference_options.reference_format == ReferenceFormat::Full {
                                line.0.to_string()
                            } else {
                                line.0.text.clone()
                            }
                        });
                    }
                }
            }
            markdown_string +=
                &format!("* [ ] {} - {}\n", issue.timecode.hhmmss(), issue.issue_text);
        }
        markdown_string += "\n";
    }
    markdown_string
}

fn parse_data_line(pair: Pair<Rule>, sub_file: &Option<Subs>) -> QCIssue {
    let (mut timecode, mut issue_type, mut issue_text) =
        (Duration::new(0, 0), String::new(), String::new());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::timecode => {
                let split_iter: Vec<&str> = inner_pair.as_str().split(":").collect();
                let (hour, minute, second) = (
                    split_iter[0].parse::<u16>().unwrap(),
                    split_iter[1].parse::<u16>().unwrap(),
                    split_iter[2].parse::<u16>().unwrap(),
                );
                timecode = Duration::from_secs(((hour * 60 * 60) + (minute * 60) + second) as u64);
            }
            Rule::issue_type => {
                issue_type = inner_pair.as_str().to_string();
            }
            Rule::issue_text => {
                issue_text = inner_pair.as_str().to_string();
            }
            _ => {}
        }
    }

    let matching_lines = sub_file
        .as_ref()
        .map(|sub_file| sub_file.choices_for_timecode(&timecode))
        .unwrap_or(Vec::new());

    QCIssue {
        timecode,
        issue_type,
        issue_text,
        matching_lines,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let args: Config = Config::parse();

    let qc_file_str = args.read_qc_file()?;

    let dialogue_file = args.read_dialogue_file()?;
    let sub_file: Option<Subs> = dialogue_file.map(|dialogue| Subs {
        subtitle_file: dialogue,
    });

    let pairs = MPVQCParser::parse(Rule::qc_file, &qc_file_str)?;
    let mut issues = Vec::new();
    for pair in pairs.flatten() {
        match pair.as_rule() {
            Rule::data_line => {
                issues.push(parse_data_line(pair, &sub_file));
            }
            _ => {}
        }
    }
    let issue_map = issues.into_iter().fold(HashMap::new(), |mut map, issue| {
        map.entry(issue.issue_type.clone())
            .or_insert_with(Vec::new)
            .push(issue);
        map
    });

    // for (_, issues) in &issue_map {
    //     for issue in issues {
    //         println!("{:#?}", issue);
    //     }
    // }
    // todo!();

    let markdown = format_into_md(&args, issue_map);
    clearscreen::clear()?;

    //let text = termimad::term_text(&markdown).to_string();
    let edited: String;
    if args.skip_edit {
        edited = markdown;
    } else {
        let mut editor = Editor::new(markdown);
        edited = editor.prompt()?;
    }

    args.output_action(edited).await?;

    Ok(())
}
