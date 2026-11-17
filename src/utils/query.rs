use serde::Serialize;

pub const DEFAULT_PER_PAGE: i64 = 20;
pub const MAX_PER_PAGE: i64 = 100;
const MAX_PAGE: i64 = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
}

impl Pagination {
    pub fn new(page: Option<i64>, per_page: Option<i64>) -> Self {
        Self {
            page: page.unwrap_or(1).clamp(1, MAX_PAGE),
            per_page: per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE),
        }
    }

    pub fn limit(&self) -> i64 {
        self.per_page
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
}

#[derive(Debug, Serialize)]
pub struct Paginated<T: Serialize> {
    pub items: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

impl<T: Serialize> Paginated<T> {
    pub fn new(items: Vec<T>, total: i64, p: Pagination) -> Self {
        Self {
            items,
            page: p.page,
            per_page: p.per_page,
            total,
        }
    }
}

/// Builds a `%term%` pattern for ILIKE with LIKE wildcards escaped. None when the term is blank.
pub fn like_pattern(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    let escaped = s
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    Some(format!("%{escaped}%"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = Pagination::new(None, None);
        assert_eq!((p.page, p.per_page, p.offset()), (1, DEFAULT_PER_PAGE, 0));
    }

    #[test]
    fn clamps_bad_input() {
        let p = Pagination::new(Some(-5), Some(10_000));
        assert_eq!((p.page, p.per_page), (1, MAX_PER_PAGE));
        let p = Pagination::new(Some(1), Some(0));
        assert_eq!(p.per_page, 1);
    }

    #[test]
    fn offset_math() {
        let p = Pagination::new(Some(3), Some(25));
        assert_eq!(p.offset(), 50);
        assert_eq!(p.limit(), 25);
    }

    #[test]
    fn like_pattern_escapes_wildcards() {
        assert_eq!(like_pattern(Some("  cs  ")).unwrap(), "%cs%");
        assert_eq!(like_pattern(Some("50%_\\")).unwrap(), "%50\\%\\_\\\\%");
        assert!(like_pattern(Some("   ")).is_none());
        assert!(like_pattern(None).is_none());
    }
}
