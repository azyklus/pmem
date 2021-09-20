pub(crate) mod result;
pub use self::result::{AllocResult, AllocError};

/// # Low-level "Entities"
///
/// In this case, ertities are a unit of allocation that contain
/// a set of [`Component`]s and are managed
/// by the [`System`] in which they reside.
///
/// [`Component`]: crate::allocations::ecs::component::Component
/// [`System`]: crate::allocations::ecs::system::System
#[cfg(feature="ecs")]
pub mod entity;

/// # Low-level "Component" structures
///
/// Components are anatomical 'pieces' of entities and describe the
/// functionality of a [`System`] on that level.
///
/// [`System`]: crate::allocations::ecs::system::System
#[cfg(feature="ecs")]
pub mod component;

/// # High-level "System" architecture
///
/// `System`s are housed within a [`World`] structure and describe a
/// piece of a larger host.
///
/// Examples of such systems will be housed in a test module located
/// at `/tests/ecs/systems` at a later date.
///
/// [`World`]: crate::allocations::ecs::world::World
#[cfg(feature="ecs")]
pub mod system;

/// # Host-level "World" structure
///
/// A `World` is a collection of [`System`]s and special [`Component`]s.
///
/// The `World` acts as a 'commander' of these systems and behaves as a
/// kind of runtime environment.
///
/// [`System`]: crate::allocations::ecs::system::System
/// [`Component`]: crate::allocations::ecs::component::Component
#[cfg(feature="ecs")]
pub mod world;
