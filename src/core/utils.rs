use itertools::Itertools;

pub(crate) fn are_arrays_equal(a: &[usize], b: &[usize]) -> bool {
    // Early guard if lengths are different.
    if a.len() != b.len() {
        return false;
    }

    a.iter().zip_eq(b).fold(true, |acc, (a, b)| acc && a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_matching() {
        assert!(are_arrays_equal(&[], &[]));
        assert!(are_arrays_equal(&[1], &[1]));
        assert!(are_arrays_equal(&[1, 2, 3], &[1, 2, 3]));
    }
    #[test]
    fn check_not_matching() {
        assert!(!are_arrays_equal(&[], &[1]));
        assert!(!are_arrays_equal(&[1], &[]));
        assert!(!are_arrays_equal(&[1], &[2]));
        assert!(!are_arrays_equal(&[1, 2, 3], &[1, 2, 4]));
        assert!(!are_arrays_equal(&[1, 2, 3], &[1, 2, 3, 4]));
    }
}
