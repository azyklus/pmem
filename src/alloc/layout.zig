//! Represents the "layout" of a block of memory.
//!
//!
//! # Examples
//!
//! ```zig
//! const Foo = struct{
//!   x: i64
//! };
//! 
//! var layout: Layout = Layout.new(Foo { 1 });
//! ```

const Self = @This();

size: usize,
alignment: usize,

/// Creates a new `Layout` from the supplied type parameter, `T`.
pub fn new(comptime T: type) *Layout {
   return &Layout{
      @sizeOf(T),
      @alignOf(T),
   };
}

/// Creates a new `Layout` from the supplied size and alignment parameters.
pub fn from(comptime s: usize, comptime a: usize) *Layout {
   return &Layout{
      s, a,
   };
}

/// Gets the `size` field from Self.
/// 
/// 
/// # Examples
///
/// ```zig
/// const Foo = struct{
///   x: i64,
/// };
///
/// var x: Layout = Layout.new(Foo { 1 });
///
/// var y: usize = x.size();
/// ```
pub fn size(self: *Self) usize {
   return self.size;
}

/// Gets the `alignment` field from Self.
///
///
/// # Examples
///
/// ```zig
/// const Foo = struct{
///   x: i64,
/// };
///
/// var x: Layout = Layout.new(Foo { 1 });
///
/// var y: usize = x.alignment();
/// ```
pub fn alignment(self: *Self) usize {
   return self.alignment;
}
