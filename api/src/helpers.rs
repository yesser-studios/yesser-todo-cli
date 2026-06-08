pub fn is_success(status_code: u16) -> bool {
    (200..300).contains(&status_code)
}

#[cfg(test)]
mod test {
    use crate::helpers::is_success;

    #[test]
    fn is_success_returns_true_for_ok() {
        for n in 200..300 {
            assert!(is_success(n));
        }
    }

    #[test]
    fn is_success_returns_false_for_input_error() {
        for n in 400..500 {
            assert!(!is_success(n));
        }
    }

    #[test]
    fn is_success_returns_false_for_server_error() {
        for n in 500..600 {
            assert!(!is_success(n));
        }
    }
}
