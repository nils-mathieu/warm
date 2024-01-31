use ash::vk;

/// An error that might occur while creating a [`Library`] instance.
#[derive(Debug, Clone, Copy)]
pub struct Error(vk::Result);

impl From<vk::Result> for Error {
    #[inline]
    fn from(value: vk::Result) -> Self {
        Self(value)
    }
}

/// The result type for the crate.
pub type Result<T> = std::result::Result<T, Error>;
