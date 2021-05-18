use itertools::Itertools;

pub(crate) fn are_arrays_equal(a: &[usize], b: &[usize]) -> bool {
    a.iter().zip_eq(b).fold(true, |acc, (a, b)| acc && a == b)
}
