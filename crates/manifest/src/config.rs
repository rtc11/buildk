use std::fmt::{Display, Formatter};

use crate::{home::Home, Manifest};

#[derive(Clone, Default)]
pub struct BuildK {
    pub home: Home,
    pub manifest: Option<Manifest>, // not needed if defaults are used
}

impl BuildK  {
    pub fn new() -> Self {
        BuildK  {
            home: Home::default(),
            manifest: Manifest::try_new().ok(),
        }
    }
}

impl Display for BuildK  {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.home)?;

        if let Some(manifest) = &self.manifest {
            write!(f, "{}", manifest)?;
        }

        Ok(())
    }
}
