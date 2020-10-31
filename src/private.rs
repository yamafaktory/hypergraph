use std::fmt::{Debug, Formatter, Result, Write};

pub trait ExtendedDebug<'a> {
    type Debug: 'a;

    fn safe_debug(self) -> Self::Debug;
}

/// Borrowed from https://github.com/petgraph/petgraph/blob/master/src/dot.rs.
struct Escaper<W>(W);

impl<W> Write for Escaper<W>
where
    W: Write,
{
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        match c {
            '"' | '\\' => self.0.write_char('\\')?,
            // \l is for left justified linebreak
            '\n' => return self.0.write_str("\\l"),
            _ => {}
        }

        self.0.write_char(c)
    }
}

pub struct CustomDebug<'a, T>(&'a T);

impl<'a, T> Debug for CustomDebug<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(&mut Escaper(f), "{:#?}", &self.0)
    }
}

impl<'a, T> ExtendedDebug<'a> for &'a T {
    type Debug = CustomDebug<'a, T>;

    fn safe_debug(self) -> Self::Debug {
        CustomDebug(self)
    }
}
