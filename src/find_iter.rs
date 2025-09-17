pub struct FindIter<'a> {
    haystack: &'a str,
    needle: &'a str,
    last_index: usize,
}

impl<'a> FindIter<'a> {
    pub fn new<P: AsRef<str> + ?Sized, H: AsRef<str> + ?Sized>(
        haystack: &'a H,
        needle: &'a P,
    ) -> Self {
        Self {
            haystack: haystack.as_ref(),
            needle: needle.as_ref(),
            last_index: 0,
        }
    }
}

impl<'a> Iterator for FindIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_index >= self.haystack.len() {
            return None;
        }
        if let Some(pos) = &self.haystack[self.last_index..].find(self.needle) {
            let found_index = self.last_index + pos;
            self.last_index = found_index + self.needle.len();
            Some(found_index)
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for FindIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.last_index == 0 {
            return None;
        }
        if let Some(pos) = self.haystack[..self.last_index].rfind(self.needle) {
            let found_index = pos;
            self.last_index = found_index;
            Some(found_index)
        } else {
            None
        }
    }
}
pub struct FindRevIter<'a> {
    haystack: &'a str,
    needle: &'a str,
    last_index: usize,
}

impl<'a> FindRevIter<'a> {
    pub fn new<P: AsRef<str> + ?Sized, H: AsRef<str> + ?Sized>(
        haystack: &'a H,
        needle: &'a P,
    ) -> Self {
        Self {
            haystack: haystack.as_ref(),
            needle: needle.as_ref(),
            last_index: haystack.as_ref().len(),
        }
    }
}

impl<'a> Iterator for FindRevIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_index == 0 {
            return None;
        }
        if let Some(pos) = self.haystack[..self.last_index].rfind(self.needle) {
            let found_index = pos;
            self.last_index = found_index;
            Some(found_index)
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for FindRevIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.last_index >= self.haystack.len() {
            return None;
        }
        if let Some(pos) = self.haystack[self.last_index..].find(self.needle) {
            let found_index = self.last_index + pos;
            self.last_index = found_index + self.needle.len();
            Some(found_index)
        } else {
            None
        }
    }
}

pub trait Find<'a> {
    fn find_iter(&'a self, needle: &'a str) -> FindIter<'a>;
}

impl<T: AsRef<str>> Find<'_> for T {
    fn find_iter<'a>(&'a self, needle: &'a str) -> FindIter<'a> {
        FindIter::new(self.as_ref(), needle)
    }
}

impl Find<'_> for str {
    fn find_iter<'a>(&'a self, needle: &'a str) -> FindIter<'a> {
        FindIter::new(self, needle)
    }
}

pub trait FindRev<'a> {
    fn find_rev_iter(&'a self, needle: &'a str) -> FindRevIter<'a>;
}

impl<T: AsRef<str>> FindRev<'_> for T {
    fn find_rev_iter<'a>(&'a self, needle: &'a str) -> FindRevIter<'a> {
        FindRevIter::new(self.as_ref(), needle)
    }
}

impl FindRev<'_> for str {
    fn find_rev_iter<'a>(&'a self, needle: &'a str) -> FindRevIter<'a> {
        FindRevIter::new(self, needle)
    }
}
