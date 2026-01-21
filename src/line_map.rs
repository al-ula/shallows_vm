use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{Deref, DerefMut},
    path::PathBuf,
};
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Line {
    pub idx: usize,
    pub content: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Lines(Vec<Line>);
impl Deref for Lines {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Lines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Lines {
    type Item = Line;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Lines {
    type Item = &'a Line;
    type IntoIter = std::slice::Iter<'a, Line>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Lines {
    type Item = &'a mut Line;
    type IntoIter = std::slice::IterMut<'a, Line>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl FromIterator<Line> for Lines {
    fn from_iter<I: IntoIterator<Item = Line>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Vec<Line>> for Lines {
    fn from(v: Vec<Line>) -> Self {
        Self(v)
    }
}
impl Lines {
    pub fn new() -> Lines {
        Lines(Vec::<Line>::new())
    }
    pub fn from_path(path: PathBuf) -> Result<Lines, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = Lines::new();

        for (index, line_result) in reader.lines().enumerate() {
            let line_content = line_result?;
            if line_content.is_empty() {
                continue;
            }

            let line = Line {
                idx: index,
                content: line_content,
            };

            lines.push(line);
        }

        Ok(lines)
    }
}

#[derive(Debug)]
pub struct LineReader {
    // We use std::io::Lines which is an iterator over BufReader
    inner_lines: std::io::Lines<BufReader<File>>,
    current_idx: usize,
}

impl LineReader {
    pub fn from_path(path: PathBuf) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(Self {
            inner_lines: reader.lines(),
            current_idx: 0,
        })
    }
}

impl Iterator for LineReader {
    // We return a Result because file I/O can fail mid-stream
    type Item = io::Result<Line>;

    fn next(&mut self) -> Option<Self::Item> {
        // We loop to handle the "skip empty lines" logic within the stream
        loop {
            // Get the next raw line from the file
            match self.inner_lines.next() {
                Some(Ok(content)) => {
                    // Check logic from original snippet
                    if content.is_empty() {
                        self.current_idx += 1; // Still increment index? (Assuming yes based on "line: index")
                        continue; // Skip this iteration and try the next line
                    }

                    let line = Line {
                        idx: self.current_idx,
                        content,
                    };

                    self.current_idx += 1;
                    return Some(Ok(line));
                }
                Some(Err(e)) => return Some(Err(e)), // Propagate IO errors
                None => return None,                 // End of file
            }
        }
    }
}
