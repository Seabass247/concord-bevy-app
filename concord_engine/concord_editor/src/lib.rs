use std::fmt;

use bevy::{prelude::*, utils::HashSet};
use bevy_kajiya::kajiya_egui::egui::Color32;
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation, GizmoResult, GizmoVisuals};
use raycast::RayCast;

mod raycast;
mod target;

use crate::target::Target;

pub mod plugin;
pub mod scene;

pub use plugin::*;
pub use raycast::SelectableTag;

/// The default snapping distance for rotation in radians
pub const DEFAULT_SNAP_ANGLE: f32 = 15.0;
/// The default snapping distance for translation
pub const DEFAULT_SNAP_DISTANCE: f32 = 1.0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NewInstanceSelect {
    MeshName(String),
    None,
}

impl fmt::Display for NewInstanceSelect {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            NewInstanceSelect::MeshName(name) => write!(f, "{}", name),
            NewInstanceSelect::None => write!(f, "{}", "None"),
        }
        
    }
}

impl Default for NewInstanceSelect {
    fn default() -> Self {
        Self::None
    }
}

pub struct TransformGizmo {
    last_translation: Vec3,
    last_rotation: Quat,
    last_scale: Vec3,
    translation_offset: Vec3,
    rotation_offset: Quat,
    scale_offset: Vec3,
    scale_origin: Vec3,
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    model_matrix: [[f32; 4]; 4],
    mode: GizmoMode,
    visuals: GizmoVisuals,
    orientation: GizmoOrientation,
    last_response: Option<GizmoResult>,
    last_transformation: Option<(GizmoMode, [f32; 3])>,
    snapping_off: bool,
    snap_angle: f32,
    snap_distance: f32,
}

impl Default for TransformGizmo {
    fn default() -> Self {
        let last_translation = Vec3::ZERO;
        let last_rotation = Quat::IDENTITY;
        let last_scale = Vec3::ONE;
        let translation_offset = Vec3::ZERO;
        let scale_offset = Vec3::ZERO;
        let scale_origin = Vec3::ONE;
        let rotation_offset = Quat::IDENTITY;
        let view_matrix = [[0.0; 4]; 4];
        let projection_matrix = [[0.0; 4]; 4];
        let model_matrix = Mat4::IDENTITY.to_cols_array_2d();
        let mode = GizmoMode::Translate;
        let visuals = GizmoVisuals {
            x_color: Color32::from_rgb(255, 0, 128),
            y_color: Color32::from_rgb(128, 255, 0),
            z_color: Color32::from_rgb(0, 128, 255),
            inactive_alpha: 0.6,
            s_color: Color32::TRANSPARENT,
            stroke_width: 6.0,
            gizmo_size: 100.0,
            ..Default::default()
        };
        let orientation = GizmoOrientation::Global;

        Self {
            last_translation,
            last_rotation,
            last_scale,
            translation_offset,
            rotation_offset,
            scale_offset,
            scale_origin,
            view_matrix,
            projection_matrix,
            model_matrix,
            mode,
            visuals,
            orientation,
            last_response: None,
            last_transformation: None,
            snapping_off: true,
            snap_angle: DEFAULT_SNAP_ANGLE,
            snap_distance: DEFAULT_SNAP_DISTANCE,
        }
    }
}

impl TransformGizmo {
    pub fn gizmo(&self) -> Gizmo {
        let Self {
            last_translation,
            last_rotation,
            last_scale,
            translation_offset,
            rotation_offset,
            scale_offset,
            scale_origin,
            view_matrix,
            projection_matrix,
            model_matrix,
            mode,
            visuals,
            orientation,
            last_response: _,
            last_transformation: _,
            snapping_off,
            snap_angle,
            snap_distance,
        } = *self;

        Gizmo::new("My gizmo")
            .view_matrix(view_matrix)
            .projection_matrix(projection_matrix)
            .model_matrix(model_matrix)
            .mode(mode)
            .orientation(orientation)
            .snapping(!snapping_off)
            .snap_angle(snap_angle.to_radians())
            .snap_distance(snap_distance)
            .visuals(visuals)
    }
}

#[derive(Default)]
pub struct EditorState {
    pub selected_target: Option<Target>,
    pub picked_target: Option<(Target)>,
    pub meshes_list: HashSet<String>,
    pub new_instance_select: NewInstanceSelect,
    pub new_instance_scale: f32,
    pub new_instancing_enabled: bool,
    pub selected_emission: f32,
    transform_gizmo: TransformGizmo,
    hide_gui: bool,
    last_ray_cast: RayCast,
    pointer: Vec2,
}
