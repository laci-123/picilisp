/// number of free cells when the [crate::memory::Memory] is constructed
pub const INITIAL_FREE_CELLS: usize = 256;

/// maximum ratio of the number of free cells after garbage collection, compared to the number of used cells
pub const MAXIMUM_FREE_RATIO: f32 = 0.75;

/// when removing free cells after garbage collection, keep as many that the ratio of their number and the number
/// of used cells is at least this big
pub const MINIMUM_FREE_RATIO: f32 = 0.1;

/// when there are no more free cells (not even after garbage collection), allocate this many times the used cells
pub const ALLOCATION_RATIO: f32 = 1.0;

/// maximum depth of recursion before a evaluation is interrupted and a stackoverflow signal is emitted
pub const MAX_RECURSION_DEPTH: usize = 700;

/// the name of the whole application, e.g. it is displayed on the GUI widnow titlebar
pub const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");
