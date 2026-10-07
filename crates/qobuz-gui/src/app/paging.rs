//! One search results section (albums or tracks) paged independently, plus the
//! rules for when another page exists and how a page is merged in.

pub(super) const PAGE_SIZE: u32 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Albums,
    Tracks,
}

/// One fetched page, already reduced to display rows.
#[derive(Debug, Clone)]
pub(super) struct Page<T> {
    pub(super) items: Vec<T>,
    pub(super) total: Option<u32>,
}

#[derive(Debug, Clone)]
pub(super) struct Section<T> {
    pub(super) items: Vec<T>,
    /// Where the next page starts: every item the API returned so far,
    /// duplicates included, so a page of repeats still moves the cursor on.
    next_offset: u32,
    pub(super) total: Option<u32>,
    last_page_len: usize,
    pub(super) loading: bool,
}

impl<T> Default for Section<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_offset: 0,
            total: None,
            last_page_len: 0,
            loading: false,
        }
    }
}

impl<T> Section<T> {
    pub(super) fn first(page: Page<T>) -> Self {
        Self {
            next_offset: page.items.len() as u32,
            total: page.total,
            last_page_len: page.items.len(),
            items: page.items,
            loading: false,
        }
    }

    pub(super) fn next_offset(&self) -> u32 {
        self.next_offset
    }

    /// Without a total, a full last page is the only hint that more exist. An
    /// empty page ends paging even under a larger total, so an inconsistent
    /// total can't offer a control that never yields anything.
    pub(super) fn has_more(&self) -> bool {
        if self.last_page_len == 0 {
            return false;
        }
        match self.total {
            Some(total) => self.next_offset < total,
            None => self.last_page_len == PAGE_SIZE as usize,
        }
    }

    /// Merge a page, skipping items whose id is already shown: later pages can
    /// repeat results when the catalog shifts between requests.
    pub(super) fn append(&mut self, page: Page<T>, id: impl Fn(&T) -> &str) {
        self.last_page_len = page.items.len();
        self.next_offset += page.items.len() as u32;
        if page.total.is_some() {
            self.total = page.total;
        }
        for item in page.items {
            if !self.items.iter().any(|seen| id(seen) == id(&item)) {
                self.items.push(item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(ids: &[&str], total: Option<u32>) -> Page<String> {
        Page {
            items: ids.iter().map(|s| s.to_string()).collect(),
            total,
        }
    }

    fn full_page(total: Option<u32>) -> Page<String> {
        Page {
            items: (0..PAGE_SIZE).map(|i| format!("p1-{i}")).collect(),
            total,
        }
    }

    #[test]
    fn first_page_keeps_every_item() {
        let s = Section::first(full_page(Some(140)));
        assert_eq!(s.items.len(), PAGE_SIZE as usize);
    }

    #[test]
    fn more_while_below_total() {
        let s = Section::first(full_page(Some(140)));
        assert_eq!(s.next_offset(), 25);
        assert!(s.has_more());
    }

    #[test]
    fn exhausted_at_total() {
        let mut s = Section::first(full_page(Some(30)));
        s.append(page(&["a", "b", "c", "d", "e"], Some(30)), String::as_str);
        assert_eq!(s.next_offset(), 30);
        assert!(!s.has_more());
    }

    #[test]
    fn unknown_total_with_full_page_has_more() {
        assert!(Section::first(full_page(None)).has_more());
    }

    #[test]
    fn unknown_total_with_short_page_is_done() {
        assert!(!Section::first(page(&["a", "b"], None)).has_more());
    }

    #[test]
    fn empty_page_ends_paging_despite_total() {
        let mut s = Section::first(full_page(Some(140)));
        s.append(page(&[], Some(140)), String::as_str);
        assert!(!s.has_more());
    }

    #[test]
    fn duplicates_are_skipped() {
        let mut s = Section::first(page(&["a", "b"], None));
        s.append(page(&["b", "c"], None), String::as_str);
        assert_eq!(s.items, vec!["a", "b", "c"]);
    }

    #[test]
    fn offset_advances_over_duplicates() {
        let mut s = Section::first(page(&["a", "b"], Some(10)));
        s.append(page(&["a", "b"], Some(10)), String::as_str);
        assert_eq!(s.items.len(), 2);
        assert_eq!(s.next_offset(), 4);
    }

    #[test]
    fn missing_total_keeps_known_total() {
        let mut s = Section::first(page(&["a"], Some(10)));
        s.append(page(&["b"], None), String::as_str);
        assert_eq!(s.total, Some(10));
    }
}
