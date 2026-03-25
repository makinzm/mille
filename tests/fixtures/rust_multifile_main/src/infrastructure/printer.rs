use crate::domain::model::User;

pub fn print(user: &User) {
    println!("Hello, {}!", user.name);
}
