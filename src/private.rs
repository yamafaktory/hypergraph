use std::{fmt, fmt::Write};

pub trait ExtendedDebug<'a> {
    type Debug: 'a;

    fn safe_debug(self) -> Self::Debug;
}

pub struct CustomDebug<'a, T>(&'a T);

impl<'a, T> fmt::Debug for CustomDebug<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for char in format!("{:?}", &self.0).chars() {
            match char {
                '"' | '\\' => f.write_char('\\')?,
                // \l is for left justified line break.
                '\n' => return f.write_str("\\l"),
                _ => {}
            };

            f.write_char(char)?
        }

        Ok(())
    }
}

impl<'a, T> ExtendedDebug<'a> for &'a T {
    type Debug = CustomDebug<'a, T>;

    fn safe_debug(self) -> Self::Debug {
        CustomDebug(self)
    }
}
