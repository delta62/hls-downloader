use hls::{Line, Tag};

pub struct ManifestWatcher<F>
where
    F: FnMut(String),
{
    first_segment: usize,
    segment_count: usize,
    lines: Vec<Line>,
    data_added: F,
}

impl<F> ManifestWatcher<F>
where
    F: FnMut(String),
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
                    log::debug!("new key: {:?}", attrs);
                }
                Line::Uri(u) => {
                    i += 1;
                    if i > self.segment_count {
                        (self.data_added)(format!("{}", u));
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
