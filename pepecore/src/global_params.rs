use once_cell::sync::Lazy;
use std::sync::RwLock;

static GLOBAL_RAYON: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

pub fn rayon_mode(on: bool) {
    let mut param = GLOBAL_RAYON.write().unwrap();
    *param = on;
}
pub fn rayon_get_mode() -> bool {
    *GLOBAL_RAYON.read().unwrap()
}
