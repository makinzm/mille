// This file contains a variable that violates name_deny = ["aws"].
pub fn connect() {
    let aws_url = "https://example.com";
    let _ = aws_url;
}
