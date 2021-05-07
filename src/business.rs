use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use once_cell::sync::Lazy;

pub static BUSINESS_COUNTER: Lazy<Arc<AtomicUsize>> = Lazy::new(|| Arc::new(AtomicUsize::new(0)));

pub struct BusinessToken;

impl BusinessToken {
    #[inline]
    pub fn new() -> BusinessToken {
        BUSINESS_COUNTER.fetch_add(1, Ordering::SeqCst);
        BusinessToken
    }
}

impl Default for BusinessToken {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BusinessToken {
    #[inline]
    fn drop(&mut self) {
        BUSINESS_COUNTER.fetch_sub(1, Ordering::SeqCst);
    }
}

#[inline]
pub fn is_busy() -> bool {
    BUSINESS_COUNTER.load(Ordering::SeqCst) == 0
}

#[inline]
pub fn busy() -> BusinessToken {
    Default::default()
}
