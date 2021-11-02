pub const AllocOptions = struct{
   const Self = @This();

   comptime Elem: type = undefined,
   /// `null` means a natural alignment.
   alignment: ?u29,
   sentinel: ?type,

   /// Returns the `Elem` field.
   pub fn elem(self: Self) type {
      return self.Elem;
   }

   /// Returns the `alignment` field.
   pub fn alignment(self: Self) ?u29 {
      return self.alignment;
   }

   /// Returns the `sentinel` field.
   pub fn sentinel(self: Self) ?type {
      return self.sentinel;
   }
};
