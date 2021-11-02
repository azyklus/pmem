pub const Allocator = global.Global;
pub const AllocError = errors.AllocError;
pub const AllocOptions = options.AllocOptions;

/// Here, we have our errors.
pub const errors = @import("alloc/errors.zig");

/// The `Layout` type defines the memory layout used in the allocation of memory via our allocator.
pub const layout = @import("alloc/layout.zig");

/// the `Global` type is our global allocator.
pub const global = @import("alloc/global.zig");

/// Options for allocating memory.
pub const options = @import("alloc/options.zig");
