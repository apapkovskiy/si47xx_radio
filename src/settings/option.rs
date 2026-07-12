use core::str::FromStr;

use core::fmt::Write;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::rwlock::RwLock;
use heapless::String;

pub trait OptionValidator<const N: usize>: Sync {
    fn validate(&self, raw: &String<N>) -> String<N>;
}

pub trait OptionToString<const N: usize = 64> {
    fn to_string(&self) -> String<N>;
}

impl<const N: usize> OptionToString<N> for u8 {
    fn to_string(&self) -> String<N> {
        String::<N>::try_from(*self).unwrap_or_default()
    }
}

impl<const N: usize> OptionToString<N> for u16 {
    fn to_string(&self) -> String<N> {
        String::<N>::try_from(*self).unwrap_or_default()
    }
}

impl<const N: usize> OptionToString<N> for f32 {
    fn to_string(&self) -> String<N> {
        let mut new = String::<N>::new();
        write!(&mut new, "{}", self).map_err(|_| ()).ok();
        new
    }
}

pub struct OptionString<const N: usize = 64> {
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
    T: Sync + OptionToString<N> + FromStr + Copy + Clone,
{
    fn validate(&self, raw: &String<N>) -> String<N> {
        let value: T = raw.parse().unwrap_or(self.default);
        String::<N>::try_from(value.to_string()).unwrap_or_default()
    }
}

impl<T, const N: usize> ConfigOption<T, N>
where
    T: Sync + OptionToString<N> + FromStr + Copy + Clone,
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
        self.option.set(value.to_string().as_str()).await;
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
