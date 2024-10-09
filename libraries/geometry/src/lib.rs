#![allow(missing_docs, reason = "TOFO add later")]

mod camera;
mod projection;

pub use camera::Camera;
use glam::{IVec2, IVec3, IVec4, Mat4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
pub use projection::Projection;

pub const SIZE_OF_VEC2: u64 = size_of::<Vec2>() as u64;
pub const SIZE_OF_VEC3: u64 = size_of::<Vec3>() as u64;
pub const SIZE_OF_VEC4: u64 = size_of::<Vec4>() as u64;

pub const SIZE_OF_MAT4: u64 = size_of::<Mat4>() as u64;

pub const SIZE_OF_IVEC2: u64 = size_of::<IVec2>() as u64;
pub const SIZE_OF_IVEC3: u64 = size_of::<IVec3>() as u64;
pub const SIZE_OF_IVEC4: u64 = size_of::<IVec4>() as u64;

pub const SIZE_OF_UVEC2: u64 = size_of::<UVec2>() as u64;
pub const SIZE_OF_UVEC3: u64 = size_of::<UVec3>() as u64;
pub const SIZE_OF_UVEC4: u64 = size_of::<UVec4>() as u64;
