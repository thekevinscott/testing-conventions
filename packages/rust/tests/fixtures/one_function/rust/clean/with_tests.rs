pub fn ratio(numerator: u8, denominator: u8) -> u8 {
    let total = numerator + denominator;
    total / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves() {
        let result = ratio(2, 4);
        assert_eq!(result, 3);
    }

    #[test]
    fn floors() {
        let result = ratio(1, 2);
        assert_eq!(result, 1);
    }
}
