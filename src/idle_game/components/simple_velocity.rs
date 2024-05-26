use bevy::math::Vec3;
use bevy::prelude::Component;

#[derive(Component, Debug)]
pub struct SimpleVelocity{
    pub velocity: Vec3,
}