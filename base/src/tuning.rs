
use crate::debug_name;

pub trait Tuning
{
    fn path() -> &'static std::path::Path;

    fn load() -> Self where Self: Sized + Default, for<'de> Self: serde::Deserialize<'de>
    {
        match crate::ronx::read_sync(Self::path())
        {
            Ok(tuning) =>
            {
                tuning
            },
            Err(err) =>
            {
                eprintln!("{} -- Failed to load {}", debug_name!(), Self::path().display());
                debug_assert!(false, "Failed to load {}", Self::path().display());
                Self::default()
            }
        }
    }
}
