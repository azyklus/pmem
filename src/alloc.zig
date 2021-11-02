/// Here, we have our errors.
pub const errors = @import("alloc/errors.zig");

/// the `Global` type is our global allocator.
pub const global = @import("alloc/global.zig");

/// Options for allocating memory.
pub const options = @import("alloc/options.zig");


pub const Global = @import("alloc/Global.zig");
pub const Layout = @import("alloc/Layout.zig");
pub const AllocError = errors.AllocError;
pub const AllocOptions = options.AllocOptions;
