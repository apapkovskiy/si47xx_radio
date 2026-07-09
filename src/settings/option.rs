use core::str::FromStr;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::rwlock::RwLock;

use heapless::String;

pub trait OptionValidator<const N: usize>: Sync {
    fn validate(&self, raw: &String<N>) -> String<N>;
}

pub struct OptionString<const N: usize> {
    key: &'static str,
    pub(crate) str: RwLock<CriticalSectionRawMutex, String<N>>,
    validator: &'static dyn OptionValidator<N>,
    description: &'static str,
}

pub struct ConfigOption<T, const N: usize = 64> {
    pub(crate) option: OptionString<N>,
    default: T,
}

impl<T, const N: usize> OptionValidator<N> for ConfigOption<T, N>
where
    T: Sync + AsRef<str> + FromStr + Copy + Clone,
{
    fn validate(&self, raw: &String<N>) -> String<N> {
        let value: T = raw.parse().unwrap_or(self.default);
        let mut str = String::<N>::new();
        let _ = str.push_str(value.as_ref());
        str
    }
}

impl<T, const N: usize> ConfigOption<T, N>
where
    T: Sync + AsRef<str> + FromStr + Copy + Clone,
{
    pub const fn new(
        key: &'static str,
        default: T,
        validator: &'static dyn OptionValidator<N>,
        description: &'static str,
    ) -> Self {
        Self {
            option: OptionString::new(key, validator, description),
            default,
        }
    }

    pub async fn set(&self, value: &T) {
        self.option.set(value.as_ref()).await;
    }

    pub async fn get(&self) -> T {
        self.option.convert(self.default).await
    }
}

impl<const N: usize> OptionString<N> {
    pub const fn new(
        key: &'static str,
        validator: &'static dyn OptionValidator<N>,
        description: &'static str,
    ) -> Self {
        Self {
            key,
            str: RwLock::new(String::<N>::new()),
            validator,
            description,
        }
    }

    pub fn get_key(&self) -> &'static str {
        self.key
    }

    pub fn get_description(&self) -> &'static str {
        self.description
    }

    pub async fn set(&self, value: &str) {
        let mut str = self.str.write().await;
        str.clear();
        let _ = str.push_str(value);
    }

    pub async fn get(&self) -> String<N> {
        let str = self.str.read().await;
        self.validator.validate(&str)
    }

    pub async fn convert<T>(&self, default: T) -> T
    where
        T: FromStr,
    {
        let str = self.str.read().await;
        str.parse().unwrap_or(default)
    }
}
