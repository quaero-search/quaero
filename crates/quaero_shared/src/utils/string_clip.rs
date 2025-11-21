pub trait StringClip {
    fn clip<'a>(&'a self, start_offset: usize, end_offset: usize) -> &'a str;
}

impl StringClip for str {
    fn clip<'a>(&'a self, start_offset: usize, end_offset: usize) -> &'a str {
        &self[start_offset..self.len() - end_offset]
    }
}
