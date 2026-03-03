fn main() {
    println!("Architecture Checker — mille");
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test_for_ci() {
        // Passing test for GREEN phase
        assert_eq!(2 + 2, 4);
    }
}
