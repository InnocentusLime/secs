use secs::World;

#[test]
fn check_component_attached() {
    let mut world = World::default();

    let entity = world.spawn((1_u32,));

    assert!(world.is_attached::<u32>(entity));

    world.detach::<u32>(entity);
    assert!(!world.is_attached::<u32>(entity));
}

#[test]
fn detach() {
    let mut world = World::default();
    let entity = world.spawn((String::new(),));
    world.detach::<String>(entity).unwrap();
    assert_eq!(None, world.detach::<String>(entity));
}

#[test]
fn detach_any() {
    let mut world = World::default();
    let entity = world.spawn((1_u32, "foo"));

    world.detach_any::<u32>();

    assert!(!world.is_attached::<u32>(entity));
}
