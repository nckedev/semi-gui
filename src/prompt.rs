use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, Clone)]
pub struct Prompt {
    pub raw: String,
    pub segments: Vec<Segment>,
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

    #[cfg(test)]
    pub fn as_string(&self) -> String {
        let mut buf = String::default();
        for s in &self.segments {
            buf.push_str(&self.raw[s.start..=s.end]);
            // buf.push(' ');
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
    pub kind: SegmentKind,
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
    pub fn expand<'a>(&self, buf: &'a str) -> &'a str {
        match self.kind {
            SegmentKind::Text => &buf[self.start..=self.end],
            SegmentKind::File => "file",
            SegmentKind::Lsp => "lsp",
            SegmentKind::Selected => "selected",
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

fn advance_while<F>(iter: &mut Peekable<CharIndices<'_>>, mut pred: F, start: usize) -> usize
where
    F: FnMut(char) -> bool,
{
    let mut end = start;
    while let Some(&(i, c)) = iter.peek() {
        if !pred(c) {
            break;
        }
        end = i;
        iter.next();
    }
    end
}

fn is_kw(buf: &str, target: &str) -> bool {
    // traget is the last word in buf
    if buf == target {
        return true;
    // buf starts with target but has more chars after, the next char must be a whitespace
    } else if buf.len() > target.len() && buf.starts_with(target) {
        return buf
            .chars()
            .nth(target.len())
            .filter(|c| c.is_whitespace())
            .is_some();
    }

    false
}

fn is_kw_with_args(buf: &str, target: &str) -> bool {
    false
}

fn parse(str: &str) -> Vec<Segment> {
    let mut iter = str.char_indices().peekable();
    let mut segments = vec![];

    while let Some(&(idx, _)) = iter.peek() {
        let start = idx;
        let segment = match &str[idx..] {
            x if x.starts_with("@symbol:") || x.starts_with("@s:") => todo!(),
            x if is_kw(x, "@selected") => {
                let end = advance_while(&mut iter, |c| !c.is_whitespace(), start);
                Segment {
                    start,
                    end,
                    kind: SegmentKind::Selected,
                }
            }
            x if x.starts_with("@") => {
                let end = advance_while(&mut iter, |c| !c.is_whitespace(), start);
                // let take_iter = iter.by_ref().take_while(|(_, c)| !c.is_whitespace());
                Segment {
                    start,
                    end,
                    kind: SegmentKind::File,
                }
            }
            _ => {
                let end = advance_while(&mut iter, |c| c != '@' && c != '#', start);
                // let take_iter = iter.by_ref().take_while(|(_, c)| *c != '@' && *c != '#');
                Segment {
                    start,
                    end,
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
    fn test_prompt() {
        let prompt = Prompt::from("test @test @selected test".to_string());
        let r = prompt.as_string();
        assert_eq!(prompt.segments.len(), 5);
        assert_eq!("test @test @selected test".to_string(), r);
    }

    #[test]
    fn test_parse() {
        let input = "@test @selected test";
        let r = parse(input);
        assert_eq!(
            r,
            vec![
                Segment::new(0, 4, SegmentKind::File),
                Segment::new(5, 5, SegmentKind::Text),
                Segment::new(6, 14, SegmentKind::Selected),
                Segment::new(15, 19, SegmentKind::Text),
            ]
        )
    }
}
