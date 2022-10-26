use std::{borrow::Borrow, hash::Hash, ops::Deref};

use lasso::Spur;
use text_size::TextRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub range: TextRange,
}

impl Span {
    pub fn new<T: Into<std::ops::Range<u32>>>(range: T) -> Self {
        let std::ops::Range { start, end } = range.into();
        let range: TextRange = TextRange::new(start.into(), end.into());
        Self { range }
    }

    /// Convert an iterator of spanned items into a span of all the items
    ///
    /// `[Spanned<T>]` -> `Span`
    ///
    /// Returns `None` if the iterator has no items, as there can be no span
    pub fn span_iter_of_span(iter: impl IntoIterator<Item = Span>) -> Option<Span> {
        let mut iter = iter.into_iter().peekable();
        let first = iter.peek()?;
        let start = first.range.start();
        let mut end = first.range.end();
        iter.for_each(|t| end = t.range.end());
        let span = Span::new(TextRange::new(start, end));
        Some(span)
    }

    /// Convert an iterator of spanned items into a span of all the items
    ///
    /// `[Spanned<T>]` -> `Span`
    ///
    /// Returns `None` if the iterator has no items, as there can be no span
    pub fn span_iter_of_spanned<T, B>(iter: impl IntoIterator<Item = B>) -> Option<Span>
    where
        B: Borrow<Spanned<T>>,
    {
        let mut iter = iter.into_iter().peekable();
        let first = iter.peek()?;
        let start = first.borrow().span.range.start();
        let mut end = first.borrow().span.range.end();
        iter.for_each(|t| end = t.borrow().span.range.end());
        let span = Span::new(TextRange::new(start, end));
        Some(span)
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub inner: T,
    pub span: Span,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T: PartialEq> Eq for Spanned<T> {}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<T> Spanned<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    /// Convert an iterator of spanned items into a spanned collection of unspanned items
    ///
    /// `[Spanned<T>]` -> `Spanned<[T]>`
    ///
    /// Returns `None` if the iterator has no items, as there can be no span
    pub fn spanned_iter<C: FromIterator<T>>(
        iter: impl IntoIterator<Item = Spanned<T>>,
    ) -> Option<Spanned<C>> {
        let mut iter = iter.into_iter().peekable();
        let first = iter.peek()?;
        let start = first.span.range.start();
        let mut end = first.span.range.end();
        let c = C::from_iter(iter.inspect(|t| end = t.span.range.end()).map(|v| v.inner));
        let span = Span::new(TextRange::new(start, end));
        Some(Spanned::new(c, span))
    }

    /// Convert an iterator of spanned items into a spanned collection of unspanned items that have been mapped with the supplied closure
    ///
    /// `[Spanned<A>]` -> `Spanned<[B]>`
    ///
    /// Returns `None` if the iterator has no items, as there can be no span
    pub fn spanned_iter_with<C, F, B>(
        iter: impl IntoIterator<Item = Spanned<T>>,
        f: F,
    ) -> Option<Spanned<C>>
    where
        C: FromIterator<B>,
        F: Fn(T) -> B,
    {
        let mut iter = iter.into_iter().peekable();
        let first = iter.peek()?;
        let start = first.span.range.start();
        let mut end = first.span.range.end();
        let c = C::from_iter(
            iter.inspect(|t| end = t.span.range.end())
                .map(|v| f(v.inner)),
        );
        let span = Span::new(TextRange::new(start, end));
        Some(Spanned::new(c, span))
    }

    // Gets the [`Span`] of an iterator of [`Spanned`] items
    //
    // Returns `None` if the iterator has no items, as there can be no span
    // pub fn span_iter<C: FromIterator<T>>(
    //     iter: impl IntoIterator<Item = Spanned<T>>,
    // ) -> Option<Span> {
    //     let mut iter = iter.into_iter().peekable();
    //     let first = iter.peek()?;
    //     let start = first.span.range.start();
    //     let mut end = first.span.range.end();
    //     iter.for_each(|t| end = t.span.range.end());
    //     let span = Span::new(TextRange::new(start, end));
    //     Some(span)
    // }
}

impl<A> Spanned<A> {
    /// Maps the inner value of a [`Spanned`]
    ///
    /// `Spanned<A>` -> `Spanned<B>`
    ///
    pub fn map<F, B>(self, f: F) -> Spanned<B>
    where
        F: FnOnce(A) -> B,
    {
        Spanned::new(f(self.inner), self.span)
    }

    /// Maps the inner value of an [`Spanned`] passing the values to the closure by reference
    ///
    /// `Spanned<A>` -> `Spanned<B>`
    ///
    pub fn map_ref<F, B>(&self, f: F) -> Spanned<B>
    where
        F: FnOnce(&A) -> B,
    {
        Spanned::new(f(&self.inner), self.span)
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FileId(pub Spur);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFile<T> {
    pub inner: T,
    pub file_id: FileId,
}

impl<T> InFile<T> {
    pub fn new(inner: T, file_id: FileId) -> Self {
        Self { inner, file_id }
    }

    /// Maps the inner value of an [`InFile`]
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn map<F, B>(self, f: F) -> InFile<B>
    where
        F: FnOnce(T) -> B,
    {
        InFile::new(f(self.inner), self.file_id)
    }

    /// Maps the inner value of an [`InFile`] passing the values to the closure by reference
    ///
    /// `InFile<A>` -> `InFile<B>`
    pub fn map_ref<F, B>(&self, f: F) -> InFile<B>
    where
        F: FnOnce(&T) -> B,
    {
        InFile::new(f(&self.inner), self.file_id)
    }
}

impl<T> InFile<Spanned<T>> {
    /// Maps the inner value of an `InFile<Spanned<T>` passing the values to the closure by reference
    ///
    /// `InFile<Spanned<A>>` -> `InFile<Spanned<B>>`
    pub fn map_inner_ref<F, B>(&self, f: F) -> InFile<Spanned<B>>
    where
        F: FnOnce(&T) -> B,
    {
        InFile::new(self.inner.map_ref(|v| f(v)), self.file_id)
    }
}

impl<A: Clone> InFile<A> {}

impl<T> Deref for InFile<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub type FileSpanned<T> = InFile<Spanned<T>>;
