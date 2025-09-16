/*
use ::std::ops::Index;
pub struct Range {
    pub start: usize,
    pub end: usize,
}
impl Range {
    pub fn into_std(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}
pub trait IntoRange {
    fn into_range(self) -> Range;
}
impl IntoRange for Range {
    fn into_range(self) -> Range {
        self
    }
}
impl IntoRange for std::ops::Range<usize> {
    fn into_range(self) -> Range {
        Range {
            start: self.start,
            end: self.end,
        }
    }
}
impl IntoRange for std::ops::RangeInclusive<usize> {
    fn into_range(self) -> Range {
        Range {
            start: *self.start(),
            end: *self.end() + 1,
        }
    }
}
impl IntoRange for std::ops::RangeFrom<usize> {
    fn into_range(self) -> Range {
        Range {
            start: self.start,
            end: usize::MAX,
        }
    }
}
impl IntoRange for std::ops::RangeTo<usize> {
    fn into_range(self) -> Range {
        Range {
            start: 0,
            end: self.end,
        }
    }
}
impl IntoRange for std::ops::RangeFull {
    fn into_range(self) -> Range {
        Range {
            start: 0,
            end: usize::MAX,
        }
    }
}

pub trait Indexable: std::ops::Index<std::ops::Range<usize>> {
    type Output: ?Sized = <Self as std::ops::Index<std::ops::Range<usize>>>::Output;
    fn len(&self) -> usize;
    fn get<'a, R: IntoRange>(&'a self, index: R) -> &'a <Self as Indexable>::Output
    where
        Self: Index<std::ops::Range<usize>>;
}

impl Indexable for str {
    type Output = str;
    fn len(&self) -> usize {
        self.chars().count()
    }

    fn get<'a, R: IntoRange>(&'a self, index: R) -> &'a <Self as Indexable>::Output
    where
        Self: Index<std::ops::Range<usize>>,
    {
        &self[index.into_range().into_std()]
    }
}
impl<A: AsRef<str> + std::ops::Index<std::ops::Range<usize>>> Indexable for A {
    type Output = str;
    fn len(&self) -> usize {
        self.as_ref().chars().count()
    }
    fn get<'a, R: IntoRange>(&'a self, index: R) -> &'a <Self as Indexable>::Output
    where
        Self: Index<std::ops::Range<usize>>,
    {
        &self.as_ref()[index.into_range().into_std()]
    }
}

// We have the self type, which needs to be able to PartialEq to <T>
// and we have <T> which needs to be able to PartialEq to self
pub trait Pattern<'a, T: Sized + std::ops::Index<std::ops::Range<usize>>> {
    fn find_pattern(&'a self, haystack: &T) -> Option<usize>;
}
impl<'a, A: AsRef<str>, T: AsRef<str> + std::ops::Index<std::ops::Range<usize>>> Pattern<'a, T>
    for A
{
    fn find_pattern(&self, haystack: &T) -> Option<usize> {
        haystack.as_ref().find(self.as_ref())
    }
}

impl<A: AsRef<str> + std::ops::Index<std::ops::Range<usize>>> Pattern<'_, A> for str {
    fn find_pattern(&self, haystack: &A) -> Option<usize> {
        haystack.as_ref().find(self)
    }
} */

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
