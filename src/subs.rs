use std::time::Duration;

use aspasia::{AssSubtitle, Moment, Subtitle};

use crate::qc_issue::ASSEvent;

pub(crate) struct Subs {
    pub(crate) subtitle_file: AssSubtitle,
}

impl Subs {
    pub fn choices_for_timecode(&self, timecode: &Duration) -> Vec<ASSEvent> {
        let timecode_as_moment = Moment::from(timecode.as_millis() as i64);
        self.subtitle_file
            .events()
            .into_iter()
            .filter(|&event| event.start < timecode_as_moment && timecode_as_moment < event.end)
            .map(|event| event.into())
            .collect()
    }
}
