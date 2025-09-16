use super::*;

#[derive(Debug, Clone)]
pub struct FormatterLabel {
    pub message: TokenStream,
    pub child_labels: Vec<TokenStream>,
}

#[derive(Debug, Clone)]
pub struct ReportSegment {
    caret: Caret,
    label: FormatterLabel,
}

#[derive(Debug, Clone)]
/// A collection of overlapping caret segments
pub struct ReportSegments {
    segments: Vec<ReportSegment>,
}
impl Iterator for ReportSegments {
    type Item = ReportSegment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.segments.is_empty() {
            None
        } else {
            Some(self.segments.remove(0))
        }
    }
}
