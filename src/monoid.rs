//! This module implements a free monoid on a set of annotated string slices along with some extra
//! tools to operate on it. For typical usage you can check `typical_usage` function since
//! abstractions used here are scary, not exposed to the end user and hidden from the doctest.

#[test]
fn typical_usage() {
    let mut m = FreeMonoid::<char>::default();
    m.push('a', "string ").push('b', "more string");

    let mut r = String::new();
    for (a, slice) in &m {
        r += &format!("{}: {:?}; ", a, slice);
    }
    assert_eq!("a: \"string \"; b: \"more string\"; ", r);
}

/// A Free Monoid on set of annotated string slices
///
/// Where identity element is `FreeMonoid::default` and binary operation is `+`
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FreeMonoid<T> {
    payload: String,
    labels: Vec<(std::ops::Range<usize>, T)>,
}

impl<T> Default for FreeMonoid<T> {
    fn default() -> Self {
        Self {
            payload: String::new(),
            labels: Vec::new(),
        }
    }
}

impl<T> FreeMonoid<T> {
    /// Length of stored text in bytes
    ///
    /// Does not account for space required to render the annotations
    pub(crate) fn len(&self) -> usize {
        self.payload.len()
    }

    /// Is there anything inside?
    ///
    /// Returns `true` if both payload and labels are empty
    pub(crate) fn is_empty(&self) -> bool {
        self.payload.is_empty() && self.labels.is_empty()
    }

    /// Clear stored data while retaining the storage capacity
    pub(crate) fn clear(&mut self) {
        self.payload.clear();
        self.labels.clear();
    }

    /// Append an annotated string slice
    pub(crate) fn push<S: AsRef<str>>(&mut self, meta: T, payload: S) -> &mut Self {
        let range = self.payload.len()..self.payload.len() + payload.as_ref().len();
        self.payload.push_str(payload.as_ref());
        self.labels.push((range, meta));
        self
    }

    /// Iterate over annotated fragments
    pub(crate) fn iter(&self) -> AnnotatedSlicesIter<T> {
        AnnotatedSlicesIter {
            current: 0,
            items: self,
        }
    }
}

impl<'a, T> IntoIterator for &'a FreeMonoid<T> {
    type Item = (&'a T, &'a str);
    type IntoIter = AnnotatedSlicesIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Clone> std::ops::Add<&Self> for FreeMonoid<T> {
    type Output = Self;

    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<T: Clone> std::ops::AddAssign<&Self> for FreeMonoid<T> {
    fn add_assign(&mut self, rhs: &Self) {
        self.payload.push_str(&rhs.payload);
        let len = self.payload.len();
        self.labels.extend(
            rhs.labels
                .iter()
                .map(|(range, label)| (range.start + len..range.end + len, label.clone())),
        );
    }
}

impl<T: PartialEq> std::ops::AddAssign<(T, &str)> for FreeMonoid<T> {
    fn add_assign(&mut self, rhs: (T, &str)) {
        self.push(rhs.0, rhs.1);
    }
}

/// Iterate over annotated string slices contained in a [`FreeMonoid`].
///
/// Create with [`FreeMonoid::iter`]
pub(crate) struct AnnotatedSlicesIter<'a, T> {
    current: usize,
    items: &'a FreeMonoid<T>,
}

impl<'a, T> Iterator for AnnotatedSlicesIter<'a, T> {
    type Item = (&'a T, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let (range, label) = self.items.labels.get(self.current)?;
        self.current += 1;
        Some((label, &self.items.payload[range.clone()]))
    }
}
