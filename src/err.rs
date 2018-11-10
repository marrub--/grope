#[derive(Debug)] pub struct StrErr(pub String);

impl<T: ToString> From<T> for StrErr
   {fn from(e: T) -> Self {StrErr(e.to_string())}}

impl StrErr {pub fn new(e: impl ToString) -> Self {StrErr(e.to_string())}}

// EOF
