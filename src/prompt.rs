#[derive(Debug, Clone)]
pub struct Prompt {
    raw: String,
    segments: Vec<Segment>,
}

impl Prompt {
    pub fn expand(&self) -> String {
        let mut buf = String::default();
        for s in &self.segments {
            buf.push_str(s.expand(&self.raw));
            buf.push(' ');
        }
        buf
    }
}

impl From<String> for Prompt {
    fn from(value: String) -> Self {
        let segments = parse(&value);
        Self {
            raw: value,
            segments,
        }
    }
}

/// A range representing a part of the prompt, the end is inclusive
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    start: usize,
    /// end is inclusive
    end: usize,
    kind: SegmentKind,
}

impl Segment {
    pub fn new(start: usize, end: usize, kind: SegmentKind) -> Self {
        Self { start, end, kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentKind {
    Text,
    File,
    Lsp,
    Selected,
}

impl Segment {
    fn expand<'a>(&self, buf: &'a str) -> &'a str {
        match self.kind {
            SegmentKind::Text => &buf[self.start..=self.end],
            SegmentKind::File => todo!(),
            SegmentKind::Lsp => todo!(),
            SegmentKind::Selected => todo!(),
        }
    }

    pub const fn prefix_len(&self) -> usize {
        match self.kind {
            SegmentKind::Text => 0,
            SegmentKind::File => 1,
            SegmentKind::Lsp => 2,
            SegmentKind::Selected => 1,
        }
    }
}

fn parse(str: &str) -> Vec<Segment> {
    let mut iter = str.char_indices();
    let mut segments = vec![];

    while let Some((idx, _)) = iter.next() {
        let segment = match &str[idx..] {
            x if x.starts_with("@symbol:") || x.starts_with("@s:") => todo!(),
            x if x.starts_with("@") => {
                let take_iter = iter.by_ref().take_while(|(_, c)| !c.is_whitespace());
                Segment {
                    start: idx,
                    end: take_iter.count() + idx,
                    kind: SegmentKind::File,
                }
            }
            _ => {
                let take_iter = iter.by_ref().take_while(|(_, c)| *c != '@' && *c != '#');
                Segment {
                    start: idx,
                    end: take_iter.count() + idx,
                    kind: SegmentKind::Text,
                }
            }
        };
        assert!(segment.end < str.len(), "end is out of bounds");
        assert!(
            segment.start <= segment.end,
            "segment start should be <= segment end"
        );
        segments.push(segment)
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "test @test";
        let r = parse(input);
        assert_eq!(
            r,
            vec![
                Segment::new(0, 4, SegmentKind::Text),
                Segment::new(5, 9, SegmentKind::File)
            ]
        )
    }
}
