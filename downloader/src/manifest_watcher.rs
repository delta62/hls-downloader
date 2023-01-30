use hls::{Line, Tag};

#[derive(Debug)]
pub enum FileAdd {
    Segment(String),
    Key(String),
}

pub struct ManifestWatcher<F>
where
    F: FnMut(FileAdd),
{
    first_segment: usize,
    segment_count: usize,
    lines: Vec<Line>,
    data_added: F,
}

impl<F> ManifestWatcher<F>
where
    F: FnMut(FileAdd),
{
    pub fn new(data_added: F) -> Self {
        let first_segment = 0;
        let lines = Vec::new();
        let segment_count = 0;

        Self {
            first_segment,
            lines,
            data_added,
            segment_count,
        }
    }

    pub fn update(&mut self, new_manifest: Vec<Line>) {
        let mut i = 0;

        for line in &new_manifest {
            match line {
                Line::Tag(Tag::Key(attrs)) => {
                    (self.data_added)(FileAdd::Key(
                        attrs.uri.as_ref().map(|s| s.clone()).unwrap_or_default(),
                    ));
                }
                Line::Uri(u) => {
                    i += 1;
                    if i > self.segment_count {
                        self.segment_count += 1;
                        (self.data_added)(FileAdd::Segment(u.to_owned()));
                    }
                }
                Line::Tag(t) => {
                    // log::debug!("other tag: {:?}", t);
                }
            }
        }

        self.lines = new_manifest
    }
}
