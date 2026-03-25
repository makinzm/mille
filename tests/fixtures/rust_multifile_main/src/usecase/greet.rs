use crate::domain::model::User;

pub fn hello() -> User {
    User::new("world")
}
