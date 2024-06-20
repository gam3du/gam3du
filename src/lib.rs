#![warn(clippy::all, clippy::pedantic)]
// usually too noisy. Disable every now and then to see whether there are actually identifiers that need to be improved.
#![allow(clippy::module_name_repetitions)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]
// TODO remove before release
#![allow(clippy::missing_panics_doc)]

// TODO enable hand-picked clippy lints from the `restriction` group

pub mod application;
pub mod framework;
pub mod logging;
pub mod python;
mod scene;
pub mod transform;

use std::sync::atomic::AtomicU16;

pub(crate) static ROTATION: AtomicU16 = AtomicU16::new(0);
