pub fn is_success(status_code: u16) -> bool {
    (200..300).contains(&status_code)
}
