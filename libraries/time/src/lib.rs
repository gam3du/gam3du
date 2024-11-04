#![allow(missing_docs, reason = "TODO add later")]
#![expect(
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    reason = "TODO remove before release"
)]

use web_time::Instant;

#[must_use]
pub fn elapsed_as_vec(start_time: Instant) -> [u32; 2] {
    let elapsed = start_time.elapsed();
    let seconds = u32::try_from(elapsed.as_secs()).unwrap();
    let subsec_nanos = u64::from(elapsed.subsec_nanos());
    // map range of nanoseconds to value range of u32 with rounding
    let subseconds = ((subsec_nanos << u32::BITS) + 500_000_000) / 1_000_000_000;

    [
        seconds,
        u32::try_from(subseconds)
            .unwrap_or_else(|err| unreachable!("math should have prevented this error: {err}")),
    ]
}
