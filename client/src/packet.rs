use glam::{Quat, Vec3};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
}

pub type VehicleId = u32;

#[derive(Debug, Deserialize)]
pub struct Vehicle {
    pub id: VehicleId,
    pub position: Vec3,
}

#[derive(Debug, Deserialize)]
pub struct Packet {
    pub camera: Camera,
    pub vehicles: Vec<Vehicle>
}