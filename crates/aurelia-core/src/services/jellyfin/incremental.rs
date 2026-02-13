pub(super) fn append_incremental_date_filter(base_query: &mut String, since_date: Option<&str>) {
    if let Some(date) = since_date {
        let encoded = urlencoding::encode(date);
        base_query.push_str(&format!(
            "&minDateLastSaved={encoded}&minDateLastSavedForUser={encoded}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::append_incremental_date_filter;

    #[test]
    fn append_incremental_date_filter_encodes_timezone_offset() {
        let mut query = "/Items?userId=test".to_string();
        append_incremental_date_filter(&mut query, Some("2026-02-12T18:10:22+00:00"));

        assert!(query.contains("minDateLastSaved=2026-02-12T18%3A10%3A22%2B00%3A00"));
        assert!(query.contains("minDateLastSavedForUser=2026-02-12T18%3A10%3A22%2B00%3A00"));
    }

    #[test]
    fn append_incremental_date_filter_noop_without_date() {
        let mut query = "/Items?userId=test".to_string();
        append_incremental_date_filter(&mut query, None);
        assert_eq!(query, "/Items?userId=test");
    }
}
