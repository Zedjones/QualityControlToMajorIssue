use std::time::Duration;

use aspasia::{AssSubtitle, Moment, Subtitle};

use crate::qc_issue::ASSEvent;

pub(crate) struct Subs {
    pub(crate) subtitle_file: AssSubtitle,
}

impl Subs {
    pub fn choices_for_timecode(&self, timecode: &Duration) -> Vec<ASSEvent> {
        let start = Moment::from(((timecode.as_millis() / 1000) * 1000) as i64);
        let end = Moment::from(((timecode.as_millis() / 1000 + 1) * 1000) as i64);
        self.subtitle_file
            .events()
            .into_iter()
            .filter(|&event| event.start < end && event.end > start)
            .filter(|&event| !event.text.contains("\\pos("))
            .map(|event| event.into())
            .collect()
    }
}
