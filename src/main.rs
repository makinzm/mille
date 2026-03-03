pub mod domain;
pub mod infrastructure;

fn main() {
    println!("Architecture Checker — mille");
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test_for_ci() {
        let expected = 4;
        assert_eq!(2 + 2, expected, "Arithmetic works");
    }
}
