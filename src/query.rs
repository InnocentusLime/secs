use std::cell::{Ref, RefMut};

use crate::{
    sparse_set::SparseSet,
    world::{Entity, World},
};

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid query",
    label = "",
    note = "only tuples with 1 or up to 5 elements can be used as queries"
)]
pub trait Query<ARGS>: Sized {
    fn get_components(world: &World, f: Self);
}

/// A marker trait preventing `Option` from being used as the first field in a query tuple.
/// This needs to be prevented, as it does not iterate over all entity ids, but only the ones
/// within that list, always producing `Some`. In that case you can just leave off the `Option`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be the first element of a query",
    label = "",
    note = "move another element to the front of the list"
)]
pub trait Always {}

impl<T> Always for &T {}
impl<T> Always for &mut T {}

/// A helper that allows more copy paste
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a query component",
    label = "",
    note = "only references and `Option`s of references can be components"
)]
pub trait SparseSetGetter {
    type Val<'b>;
    type StorageRef<'c>;
    fn get_set(world: &World) -> Option<Self::StorageRef<'_>>;
    fn get_entity<'b>(
        storage: &'b mut Self::StorageRef<'_>,
        entity: Entity,
    ) -> Option<Self::Val<'b>>;
    fn iter<'b>(
        storage: &'b mut Self::StorageRef<'_>,
    ) -> impl Iterator<Item = (Entity, Self::Val<'b>)>
    where
        Self: Always;
}

impl<C: 'static> SparseSetGetter for &C {
    type Val<'b> = &'b C;
    type StorageRef<'c> = Ref<'c, SparseSet<C>>;
    fn get_set(world: &World) -> Option<Self::StorageRef<'_>> {
        world.sparse_sets.get()
    }
    fn get_entity<'b>(iter: &'b mut Self::StorageRef<'_>, entity: Entity) -> Option<Self::Val<'b>> {
        iter.get(entity)
    }
    fn iter<'b>(
        iter: &'b mut Self::StorageRef<'_>,
    ) -> impl Iterator<Item = (Entity, Self::Val<'b>)> {
        iter.iter()
    }
}

impl<T: SparseSetGetter> SparseSetGetter for Option<T> {
    type Val<'b> = Option<T::Val<'b>>;
    type StorageRef<'c> = T::StorageRef<'c>;
    fn get_set(world: &World) -> Option<Self::StorageRef<'_>> {
        T::get_set(world)
    }
    fn get_entity<'b>(
        storage: &'b mut Self::StorageRef<'_>,
        entity: Entity,
    ) -> Option<Self::Val<'b>> {
        Some(T::get_entity(storage, entity))
    }
    fn iter<'b>(
        _storage: &'b mut Self::StorageRef<'_>,
    ) -> impl Iterator<Item = (Entity, Self::Val<'b>)>
    where
        Self: Always,
    {
        std::iter::once_with(|| unreachable!())
    }
}

impl<C: 'static> SparseSetGetter for &mut C {
    type Val<'b> = &'b mut C;
    type StorageRef<'c> = RefMut<'c, SparseSet<C>>;
    fn get_set(world: &World) -> Option<Self::StorageRef<'_>> {
        world.sparse_sets.get_mut()
    }
    fn get_entity<'b>(
        storage: &'b mut Self::StorageRef<'_>,
        entity: Entity,
    ) -> Option<Self::Val<'b>> {
        storage.get_mut(entity)
    }
    fn iter<'b>(
        storage: &'b mut Self::StorageRef<'_>,
    ) -> impl Iterator<Item = (Entity, Self::Val<'b>)> {
        storage.iter_mut()
    }
}

macro_rules! impl_query {
    ($($T:ident),*) => {
        impl<A: SparseSetGetter + Always, $($T: SparseSetGetter,)* Z> Query<(A, $($T,)*)> for Z
        where
            Z: FnMut(Entity, A::Val<'_>, $($T::Val<'_>,)*),
            Z: FnMut(Entity, A, $($T,)*),
        {
            fn get_components(world: &World, mut f: Z) {
                #[allow(non_snake_case)]
                if let (Some(mut a), $(Some(mut $T),)*) = (A::get_set(world), $($T::get_set(world),)*) {
                    for (entity, a) in A::iter(&mut a) {
                        $(let Some($T) = $T::get_entity(&mut $T, entity) else { continue };)*
                        f(entity, a, $($T,)*);

                    }
                }
            }
        }
    };
}

impl_query!();
impl_query!(B);
impl_query!(B, C);
impl_query!(B, C, D);
impl_query!(B, C, D, E);
impl_query!(B, C, D, E, F);
impl_query!(B, C, D, E, F, G);
impl_query!(B, C, D, E, F, G, H);
impl_query!(B, C, D, E, F, G, H, I);
impl_query!(B, C, D, E, F, G, H, I, J);
impl_query!(B, C, D, E, F, G, H, I, J, K);
impl_query!(B, C, D, E, F, G, H, I, J, K, L);
