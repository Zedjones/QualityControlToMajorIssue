use std::collections::HashMap;
use std::time::Duration;

use hhmmss::Hhmmss;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "mpvqc.pest"]
pub struct MPVQCParser;

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub struct QCIssue {
    pub timecode: Duration,
    pub issue_type: String,
    pub issue_text: String,
}

impl Ord for QCIssue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timecode.cmp(&other.timecode)
    }
}

fn format_into_md(issue_map: HashMap<String, Vec<QCIssue>>) -> String {
    let mut markdown_string = String::new();
    let mut issue_map_vec = issue_map
        .into_iter()
        .collect::<Vec<(String, Vec<QCIssue>)>>();

    issue_map_vec.sort_by(|(_, issues), (_, other_issues)| issues.len().cmp(&other_issues.len()));
    issue_map_vec.reverse();

    for (issue_type, issues) in issue_map_vec.iter_mut() {
        markdown_string += &format!("# {}\n", issue_type);
        issues.sort();
        for issue in issues {
            markdown_string +=
                &format!("- [ ] {} - {}\n", issue.timecode.hhmmss(), issue.issue_text);
        }
        markdown_string += "\n";
    }
    markdown_string
}

fn main() -> anyhow::Result<()> {
    let test_str = include_str!("../[QC]_[WhiteClover] CLANNAD - 02 (BD 1080p) [68FC64DD]_ame.txt");
    let pairs = MPVQCParser::parse(Rule::qc_file, test_str)?;
    let mut issues = Vec::new();
    for pair in pairs.flatten() {
        match pair.as_rule() {
            Rule::data_line => {
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
                            timecode = Duration::from_secs(
                                ((hour * 60 * 60) + (minute * 60) + second) as u64,
                            );
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

                issues.push(QCIssue {
                    timecode,
                    issue_type,
                    issue_text,
                });
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

    let markdown = format_into_md(issue_map);
    //println!("{}", markdown);

    let edited = edit::edit(markdown)?;
    termimad::print_text(&edited);

    Ok(())
}
