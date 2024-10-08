use std::fmt::{Display, Write};

use gam3du_framework_common::api;

pub trait PyIdentifier {
    fn parameter(&self) -> impl Display;
    fn function(&self) -> impl Display;
    fn file(&self) -> impl Display;
    fn module(&self) -> impl Display;
}

impl PyIdentifier for api::Identifier {
    fn parameter(&self) -> impl Display {
        LowerSnakeDisplay(self.as_ref())
    }
    fn function(&self) -> impl Display {
        LowerSnakeDisplay(self.as_ref())
    }
    fn file(&self) -> impl Display {
        LowerSnakeDisplay(self.as_ref())
    }
    fn module(&self) -> impl Display {
        LowerSnakeDisplay(self.as_ref())
    }
}

struct LowerSnakeDisplay<'inner>(&'inner str);
impl Display for LowerSnakeDisplay<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for character in self.0.chars() {
            let character = match character {
                ' ' => '_',
                other => other,
            };
            formatter.write_char(character)?;
        }
        Ok(())
    }
}
