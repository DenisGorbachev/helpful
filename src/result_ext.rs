use crate::Error;
use core::result::Result as CoreResult;

/// An extension trait to convert `Result` to `helpful::Result`
pub trait ResultExt {
    type Output;

    fn helpful(self) -> Self::Output;
}

impl<T, E: Into<Error>> ResultExt for CoreResult<T, E> {
    type Output = crate::Result<T>;

    fn helpful(self) -> Self::Output {
        self.map_err(Into::into)
    }
}
