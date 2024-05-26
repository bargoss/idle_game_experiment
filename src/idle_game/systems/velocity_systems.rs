
use bevy::prelude::*;
use super::super::components::*;
use bevy_voxel_world::prelude::*;
pub fn velocity_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &SimpleVelocity)>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.velocity * time.delta_seconds();
    }
}
pub fn gravity_system(
    time: Res<Time>,
    mut query: Query<(&Transform, &mut SimpleVelocity)>,
) {
    for (transform, mut velocity) in query.iter_mut() {
        velocity.velocity.y -= 9.8 * time.delta_seconds();
    }
}