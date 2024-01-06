use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait Name {
    fn name(&self) -> String;
    fn hash(&self) -> u64 {
        let mut hashish = DefaultHasher::new();
        let name = self.name();
        name.hash(&mut hashish);
        hashish.finish()
    }
}
