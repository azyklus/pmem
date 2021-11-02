// 2021 11 02 >
//
// YK: began implementing an interface to a global allocator.
// YK: will pick up tomorrow. (02:58EST) 

const Self = @This();

const AllocError = @import("errors.zig").AllocError;
const AllocOptions = @import("options.zig").AllocOptions;

const math = @import("std").math;

pub fn allocate(self: *Self, layout: Layout) AllocError![]u8 {}

/// Allocates an array of `n` items and sets all of the items to `undefined`.
///
/// Depending on the allocator implementation, it may be required to call `free`
/// once the memory is no longer needed, to avoid a resource leak. If the allocator
/// implementation is unknown, then correct code will call `free` when done.
/// 
/// For allocating a single item, please see `create`.
pub fn allocate_array(self: *Self, comptime T: type, n: usize) AllocError![]T {
}

pub const Exact = enum {
   exact, at_least,
};

/// Allocates an 
pub fn with_options_ret_addr(
   self: *Self,
   layout: Layout,
   ret_addr: usize,
   comptime options: AllocOptions,
) AllocError!AllocateWithOptionsPayload(options) {
   var n: usize = layout.size();

   if (options.sentinel()) |sentinel| {
      const ptr = try self.advanced_with_ret_addr(.exact, ret_addr, layout, );
      ptr[n] = sentinel;
      return ptr[0..n :sentinel];
   } else {
      return self.advanced_with_ret_addr(options.elem(), options.alignment(), n, .exact, ret_addr);
   }
}

fn AllocateWithOptionsPayload(comptime options: AllocOptions) type {
   if (options.sentinel()) |s| {
      return [:s]align(options.alignment orelse @alignOf(options.Elem)) options.Elem;
   } else {
      return []align(options.alignment orelse @alignOf(options.Elem)) options.Elem;
   }
}

pub fn advanced_with_ret_addr(
   self: *Self,
   exact: Exact,
   ret_addr: usize,
   comptime layout: Layout,
   comptime T: type,   
) AllocError![]align(layout.alignment() orelse @alignOf(T)) T {
}
