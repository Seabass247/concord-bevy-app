use bevy::{math::Vec3A, prelude::*};

use bevy_kajiya::kajiya_render::{mesh::Aabb, KajiyaMeshInstance, KajiyaMeshInstanceBundle, KajiyaMesh};
use concord_logger::console_info;
use egui_gizmo::{math, Ray};

use crate::{
    target::{select_new_target, TargetTag, Target, unset_entity_target},
    EditorState, NewInstanceSelect,
};

/// A 3D ray, with an origin and direction. The direction is guaranteed to be normalized.
#[derive(Debug, Copy, Clone)]
pub struct RayCast {
    pub(crate) ray: Ray,
}

impl Default for RayCast {
    fn default() -> Self {
        Self {
            ray: Ray {
                origin: math::Vec3::ZERO,
                direction: math::Vec3::ZERO,
            },
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct SelectableTag;

impl RayCast {
    /// Constructs a `RayCast`, normalizing the direction vector.
    pub fn with_ray(ray: Ray) -> Self {
        Self { ray }
    }

    /// Checks if the ray intersects with an AABB of a mesh.
    pub fn intersects_aabb(&self, aabb: &Aabb, model_to_world: &Mat4) -> Option<[f32; 2]> {
        // Transform the ray to model space
        let world_to_model = model_to_world.inverse();
        let ray_dir: [f32; 3] = self.ray.direction().into();
        let ray_origin: [f32; 3] = self.ray.origin().into();
        let ray_dir: Vec3A = world_to_model.transform_vector3(ray_dir.into()).into();
        let ray_origin: Vec3A = world_to_model.transform_point3(ray_origin.into()).into();
        // Check if the ray intersects the mesh's AABB. It's useful to work in model space because
        // we can do an AABB intersection test, instead of an OBB intersection test.

        let t_0: Vec3A = (Vec3A::from(aabb.min()) - ray_origin) / ray_dir;
        let t_1: Vec3A = (Vec3A::from(aabb.max()) - ray_origin) / ray_dir;
        let t_min: Vec3A = t_0.min(t_1);
        let t_max: Vec3A = t_0.max(t_1);

        let mut hit_near = t_min.x;
        let mut hit_far = t_max.x;

        if hit_near > t_max.y || t_min.y > hit_far {
            return None;
        }

        if t_min.y > hit_near {
            hit_near = t_min.y;
        }
        if t_max.y < hit_far {
            hit_far = t_max.y;
        }

        if (hit_near > t_max.z) || (t_min.z > hit_far) {
            return None;
        }

        if t_min.z > hit_near {
            hit_near = t_min.z;
        }
        if t_max.z < hit_far {
            hit_far = t_max.z;
        }
        Some([hit_near, hit_far])
    }
}

pub fn pick_meshes(
    buttons: Res<Input<MouseButton>>,
    mut editor: ResMut<EditorState>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    query: Query<
        (Entity, &GlobalTransform, &KajiyaMeshInstance),
        (With<SelectableTag>, Without<TargetTag>),
    >,
) {
    // Handle mouse object picking events for action LCtrl+LeftClick
    if buttons.just_pressed(MouseButton::Left) && keys.pressed(KeyCode::LControl) {
        for (entity, mesh_transform, mesh) in query.iter() {
            console_info!("queried mesh {:?}", mesh.mesh);

            let mesh_aabb = Aabb::from_center_padding(Vec3::ZERO, 1.0);
            if let Some([_, far]) = editor
                .last_ray_cast
                .intersects_aabb(&mesh_aabb, &mesh_transform.compute_matrix())
            {
                if far > 0.0 {
                    console_info!("found clickable target");

                    let target = Target {
                        entity: Some(entity),
                        origin: mesh_transform.translation,
                        orientation: mesh_transform.rotation,
                    };
                    editor.picked_target = Some(target);
                }
            }
        }

        // Clear target selection if clicked on nothing
        if editor.picked_target.is_none() {
            console_info!("try unset entity (clicked on nothing)");
            unset_entity_target(&mut commands, &mut editor);
        }
    }
}
