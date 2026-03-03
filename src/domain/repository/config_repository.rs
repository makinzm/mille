use crate::domain::entity::config::MilleConfig;

pub trait ConfigRepository {
    fn load(&self, path: &str) -> std::io::Result<MilleConfig>;
}
