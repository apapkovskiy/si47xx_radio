use core::str::FromStr;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::rwlock::RwLock;

use heapless::String;

pub struct OptionString<const N: usize> {
    key: &'static str,
    pub str: RwLock<CriticalSectionRawMutex, String<N>>,
    default: &'static str,
}

impl<const N: usize> OptionString<N> {
    pub const fn new(key: &'static str, default: &'static str) -> Self {
        Self {
            key,
            str: RwLock::new(String::<N>::new()),
            default,
        }
    }

    pub fn get_key(&self) -> &'static str {
        self.key
    }

    pub fn get_default(&self) -> String<N> {
        String::from_str(self.default).unwrap_or_default()
    }

    pub async fn set(&self, value: &str) {
        let mut str = self.str.write().await;
        str.clear();
        let _ = str.push_str(value);
    }

    pub async fn get(&self) -> String<N> {
        let str = self.str.read().await;
        if str.is_empty() {
            self.get_default()
        } else {
            str.clone()
        }
    }
}
