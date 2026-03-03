fn main() {
    println!("Architecture Checker — mille");
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test_for_ci() {
        // Failing test for RED phase
        assert_eq!(2 + 2, 5);
    }
}
