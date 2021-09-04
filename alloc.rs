pub unsafe trait Allocator
{
   // TODO: Implement the Allocator trait.
}

/// # Implements an ECS allocator
pub mod ecs;

/// # Defines memory layout structure
pub mod layout;

/// # Implements a simple page allocator
pub mod paging;
