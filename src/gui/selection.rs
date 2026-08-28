pub fn selected_entry_index(selected_line: i32, row_count: i32) -> Option<usize> {
    if selected_line > 0 {
        Some((selected_line - 1) as usize)
    } else if row_count > 0 {
        Some(0)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_uses_the_selected_row() {
        assert_eq!(selected_entry_index(3, 5), Some(2));
    }

    #[test]
    fn selection_defaults_to_the_first_row() {
        assert_eq!(selected_entry_index(0, 2), Some(0));
    }

    #[test]
    fn selection_is_absent_when_there_are_no_rows() {
        assert_eq!(selected_entry_index(0, 0), None);
    }
}
