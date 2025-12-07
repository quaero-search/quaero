/// Clips both the start and the end of a string by the specified offsets.
pub trait StringClip {
    /// Clips a string.
    fn clip<'a>(&'a self, start_offset: usize, end_offset: usize) -> &'a str;
}

impl StringClip for str {
    fn clip<'a>(&'a self, start_offset: usize, end_offset: usize) -> &'a str {
        &self[start_offset..self.len() - end_offset]
    }
}
