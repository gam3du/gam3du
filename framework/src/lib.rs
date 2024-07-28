// has false positives; enable every now and then to see whether there are actually missed opportunities
#![allow(missing_copy_implementations)]
// usually too noisy. Disable every now and then to see whether there are actually identifiers that need to be improved.
#![allow(clippy::module_name_repetitions)]
#![allow(unused_crate_dependencies)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]
// TODO remove before release
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::panic)]
#![allow(clippy::print_stdout)]
#![allow(clippy::todo)]
#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]

pub mod framework;
pub mod logging;
pub mod python;

use std::sync::atomic::AtomicU16;

pub(crate) static ROTATION: AtomicU16 = AtomicU16::new(0);
