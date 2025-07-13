use std::{
    any::Any,
    cell::{Cell, Ref, RefMut},
    num::NonZeroU64,
};

use crate::{query::Query, sparse_set::SparseSets};

/// An opaque id for an entity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(NonZeroU64);

impl Entity {
    /// Returns the unique numeric identifier of this entity.
    pub fn id(&self) -> u64 {
        self.0.get()
    }
}

impl From<u64> for Entity {
    fn from(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap())
    }
}

struct EntityCounter(Cell<NonZeroU64>);

impl EntityCounter {
    fn inc(&self) -> Entity {
        let entity = self.0.get();
        self.0.set(entity.checked_add(1).unwrap());
        Entity(entity)
    }
}

impl Default for EntityCounter {
    fn default() -> Self {
        Self(Cell::new(NonZeroU64::MIN))
    }
}

/// The main entry point to this [crate].
///
/// Invoke [Self::spawn] to add entities and [Self::query] to process them.
#[derive(Default)]
pub struct World {
    entities: EntityCounter,
    pub(crate) sparse_sets: SparseSets,
}

impl World {
    fn insert<C: Any>(&mut self, entity: Entity, component: C) {
        if let Some(mut set) = self.sparse_sets.get_mut::<C>() {
            set.insert(entity, component);
        } else {
            self.sparse_sets.insert(entity, component);
        }
    }

    /// Create a new entity and immediately initialize it with the given components.
    ///
    /// ```rust
    /// # use secs::World;
    /// # let mut world = World::default();
    /// world.spawn(("player", 42));
    /// world.spawn(("animal", 12, 5.3));
    /// ```
    pub fn spawn<C: AttachComponents>(&mut self, components: C) -> Entity {
        components.attach_to(self, self.entities.inc())
    }

    /// Destroy an entity and all its components. Future attempts to use this entity in any way will panic.
    pub fn despawn(&mut self, entity: Entity) {
        self.detach_all(entity);
    }

    /// Attach multiple components to an entity at once.
    pub fn attach<C: AttachComponents>(&mut self, entity: Entity, components: C) {
        components.attach_to(self, entity);
    }

    /// Detach a component and return it if the entity had that component.
    pub fn detach<C: 'static>(&mut self, entity: Entity) -> Option<C> {
        let mut set = self.sparse_sets.get_mut::<C>()?;
        set.remove(entity)
    }

    /// Detach all components from an entity and drop them.
    /// If you want to extract specific components, call [Self::detach] first.
    pub fn detach_all(&mut self, entity: Entity) {
        self.sparse_sets.remove(entity)
    }

    /// Detach all components from an entity and drop them.
    /// If you want to extract specific components, call [Self::detach] first.
    pub fn debug_components(&self, entity: Entity) -> String {
        self.sparse_sets.debug(entity)
    }

    /// Detach all components of a specific type from all entities and drop them.
    ///
    /// This method removes all components of type `C` from every entity in the world.
    /// If no entities have components of type `C`, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use secs::World;
    /// # let mut world = World::default();
    /// # let entity = world.spawn(("player", 42));
    /// world.detach_any::<i32>();
    /// assert!(!world.is_attached::<i32>(entity));
    /// ```
    pub fn detach_any<C: 'static>(&mut self) {
        if let Some(mut set) = self.sparse_sets.get_mut::<C>() {
            set.clear();
        }
    }

    /// Check if an entity has a specific component attached.
    pub fn is_attached<C: 'static>(&self, entity: Entity) -> bool {
        self.sparse_sets
            .get::<C>()
            .is_some_and(|set| set.get(entity).is_some())
    }

    /// Immutable access to an entity's component.
    ///
    /// # Panics
    ///
    /// This will panic if the component is already used mutably either by a [Self::query] or [Self::get_mut].
    pub fn get<C: 'static>(&self, entity: Entity) -> Option<Ref<C>> {
        let set = self.sparse_sets.get::<C>()?;
        Ref::filter_map(set, |set| set.get(entity)).ok()
    }

    /// Mutable access to an entity's component.
    ///
    /// # Panics
    ///
    /// This will panic if the component is already used either by a [Self::query], [Self::get_mut], or [Self::get].
    pub fn get_mut<C: 'static>(&self, entity: Entity) -> Option<RefMut<C>> {
        let set = self.sparse_sets.get_mut::<C>()?;
        RefMut::filter_map(set, |set| set.get_mut(entity)).ok()
    }

    /// Invokes a closure for every entity that has the given components.
    ///
    /// The first component is the one that will be iterated over, so sorting the components to have
    /// the rarest one first is more performant than iterating over a common component and getting
    /// the entities discarded because a later component does not exist for it.
    ///
    /// ```rust
    /// # use secs::World;
    /// # let world = World::default();
    /// world.query(|entity_id, s: &String, u: &u32| {
    ///     println!("{s}: {u}");
    /// });
    /// ```
    pub fn query<Q: Query<T>, T>(&self, f: Q) {
        Q::get_components(self, f)
    }

    /// Same as [Self::query], but only for one component and returns a boolean.
    /// If the boolean is `false` the component will be dropped.
    pub fn query_retain<C: 'static>(&self, mut f: impl for<'a> FnMut(Entity, &'a mut C) -> bool) {
        let Some(mut set) = self.sparse_sets.get_mut::<C>() else {
            return;
        };
        let set = &mut *set;

        let mut idx = 0;
        while let Some(entity) = set.ids.get(idx).copied() {
            let retain = f(entity, &mut set.dense[idx]);
            if retain {
                idx += 1;
            } else {
                set.remove(entity);
            }
        }
    }
}
pub trait AttachComponents {
    fn attach_to(self, world: &mut World, entity: Entity) -> Entity;
}

macro_rules! impl_attach_components {
    ($($T:ident),+) => {
        impl<$($T: Any),+> AttachComponents for ($($T,)+) {
            fn attach_to(self, world: &mut World, entity: Entity) -> Entity {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                $(world.insert(entity, $T);)+
                entity
            }
        }
    };
}

impl_attach_components!(A);
impl_attach_components!(A, B);
impl_attach_components!(A, B, C);
impl_attach_components!(A, B, C, D);
impl_attach_components!(A, B, C, D, E);
impl_attach_components!(A, B, C, D, E, F);
impl_attach_components!(A, B, C, D, E, F, G);
impl_attach_components!(A, B, C, D, E, F, G, H);
impl_attach_components!(A, B, C, D, E, F, G, H, I);
impl_attach_components!(A, B, C, D, E, F, G, H, I, J);
impl_attach_components!(A, B, C, D, E, F, G, H, I, J, K);
impl_attach_components!(A, B, C, D, E, F, G, H, I, J, K, L);
