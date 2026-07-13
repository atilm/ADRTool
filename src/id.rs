pub(crate) fn format_id(id: u32) -> String {
    format!("{id:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_id_is_zero_padded() {
        assert_eq!(format_id(1), "001");
        assert_eq!(format_id(12), "012");
        assert_eq!(format_id(123), "123");
    }
}
