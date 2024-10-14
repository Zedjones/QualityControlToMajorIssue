use aspasia::substation::ass::AssEvent;
use pest_derive::Parser;
use std::cmp::Ordering;
use std::time::Duration;

#[derive(Debug)]
pub struct ASSEvent(pub AssEvent);

impl Clone for ASSEvent {
    fn clone(&self) -> Self {
        ASSEvent(AssEvent {
            kind: self.0.kind.clone(),
            layer: self.0.layer,
            start: self.0.start.clone(),
            end: self.0.end.clone(),
            style: self.0.style.clone(),
            name: self.0.name.clone(),
            margin_l: self.0.margin_l,
            margin_r: self.0.margin_r,
            margin_v: self.0.margin_v,
            effect: self.0.effect.clone(),
            text: self.0.text.clone(),
        })
    }
}

/// Implement `From<&AssEvent>` for `MyCloneableAssEvent`
impl From<&AssEvent> for ASSEvent {
    fn from(ass_event: &AssEvent) -> Self {
        ASSEvent(AssEvent {
            kind: ass_event.kind.clone(),
            layer: ass_event.layer,
            start: ass_event.start.clone(),
            end: ass_event.end.clone(),
            style: ass_event.style.clone(),
            name: ass_event.name.clone(),
            margin_l: ass_event.margin_l,
            margin_r: ass_event.margin_r,
            margin_v: ass_event.margin_v,
            effect: ass_event.effect.clone(),
            text: ass_event.text.clone(),
        })
    }
}

#[derive(Parser)]
#[grammar = "mpvqc.pest"]
pub struct MPVQCParser;

#[derive(Debug)]
pub struct QCIssue {
    pub timecode: Duration,
    pub issue_type: String,
    pub issue_text: String,
    pub matching_lines: Vec<String>,
}

impl Ord for QCIssue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timecode.cmp(&other.timecode)
    }
}

impl PartialEq for QCIssue {
    fn eq(&self, other: &Self) -> bool {
        self.timecode == other.timecode
            && self.issue_type == other.issue_type
            && self.issue_text == other.issue_text
    }
}

// Since Ord implies PartialOrd, we can derive PartialOrd
impl PartialOrd for QCIssue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other)) // Use the Ord implementation
    }
}

impl Eq for QCIssue {}
