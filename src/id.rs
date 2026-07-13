pub(crate) fn format_id(id: u32) -> String {
    format!("{id:04}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_id_is_zero_padded() {
        assert_eq!(format_id(1), "0001");
        assert_eq!(format_id(12), "0012");
        assert_eq!(format_id(123), "0123");
    }
}
