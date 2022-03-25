use std::fs;

use crate::{
    raycast::{pick_meshes, RayCast, pointer_ray},
    target::{update_target, TargetTag, instance_new_target},
    EditorState, NewInstanceSelect, scene::{save_scene_system, load_scene_system}, SelectableTag,
};
use bevy::prelude::*;
use bevy_kajiya::{
    egui::{LayerId, ScrollArea, Slider},
    kajiya_egui::{egui},
    kajiya_render::{
        camera::ExtractedCamera,
        plugin::{KajiyaRenderApp, KajiyaRenderStage, RenderWorld},
        KajiyaCamera, EnvironmentSettings, mesh::Aabb, KajiyaMeshInstance,
    },
};
use concord_logger::{console_info, get_console_logs};
use egui_gizmo::GizmoMode;

#[derive(Default)]
pub struct ConcordEditorPlugin;

impl Plugin for ConcordEditorPlugin {
    fn build(&self, app: &mut App) {
        let editor_state = EditorState::default();
        app.insert_resource(editor_state);
        app.add_startup_system(setup_gui);
        app.add_system(process_input);
        app.add_system(update_target);
        app.add_system(instance_new_target);
        app.add_system(pick_meshes);
        app.add_system(pointer_ray);
        app.add_system(save_scene_system);
        app.add_startup_system(load_scene_system);

        app.register_type::<KajiyaCamera>();
        app.register_type::<EnvironmentSettings>();
        app.register_type::<Aabb>();
        app.register_type::<KajiyaMeshInstance>();
        app.register_type::<SelectableTag>();

        app.sub_app_mut(KajiyaRenderApp)
            .add_system_to_stage(KajiyaRenderStage::Extract, update_transform_gizmo)
            .add_system_to_stage(KajiyaRenderStage::Extract, move_transform_gizmo)
            .add_system_to_stage(
                KajiyaRenderStage::Extract,
                process_gui.exclusive_system().at_end(),
            );

        console_info!("Editor Plugin Initialized");
    }
}

pub fn process_input(mut editor: ResMut<EditorState>, keys: Res<Input<KeyCode>>) {

    if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::E){
        editor.hide_gui = !editor.hide_gui;
    }

    if keys.just_pressed(KeyCode::Tab) {
        editor.new_instancing_enabled = !editor.new_instancing_enabled;
    }

    if keys.just_pressed(KeyCode::T) {
        editor.transform_gizmo.mode = match editor.transform_gizmo.mode {
            GizmoMode::Rotate => GizmoMode::Scale,
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Scale => GizmoMode::Translate,
        }
    }
}

pub fn setup_gui(mut editor: ResMut<EditorState>) {
    get_dir_mesh_list(&mut editor);
    editor.new_instance_scale = 1.0;
}

fn get_dir_mesh_list(editor: &mut EditorState) {
    let paths = fs::read_dir("assets/meshes/").unwrap();

    for path in paths {
        let mut mesh_name = path.unwrap().path().display().to_string();
        mesh_name = mesh_name.replace("assets/meshes/", "");
        editor.meshes_list.insert(mesh_name);
    }

    editor.new_instance_scale = 1.0;
}

pub fn process_gui(egui: Res<bevy_kajiya::Egui>, mut editor: ResMut<EditorState>) {
    if editor.hide_gui {
        return;
    }
    egui::SidePanel::left("backend_panel")
        .resizable(false)
        .show(egui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Editor");
            });

            ui.separator();

            ui.label("Transform Tool");
            egui::ComboBox::from_id_source("transform_mode_combo_box")
                .selected_text(format!("{:?}", editor.transform_gizmo.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut editor.transform_gizmo.mode,
                        GizmoMode::Rotate,
                        "Rotate Mode",
                    );
                    ui.selectable_value(
                        &mut editor.transform_gizmo.mode,
                        GizmoMode::Translate,
                        "Translate Mode",
                    );
                    ui.selectable_value(
                        &mut editor.transform_gizmo.mode,
                        GizmoMode::Scale,
                        "Scale Mode",
                    );
                    // ui.selectable_value(&mut editor.transform_gizmo.mode, GizmoMode::Scale, "Scale");
                });

            ui.checkbox(&mut editor.transform_gizmo.snapping_off, "Disable Tool Snapping");
            
            ui.separator();
            
            ui.label("Snapping distance");
            ui.add(
                Slider::new(&mut editor.transform_gizmo.snap_distance, (0.0)..=(1.0))
                    .clamp_to_range(true)
                    .smart_aim(true)
                    .text("units"),
            );
            ui.label("Snapping Angle");
            ui.add(
                Slider::new(&mut editor.transform_gizmo.snap_angle, (0.0)..=(90.0))
                    .clamp_to_range(true)
                    .smart_aim(true)
                    .text("deg"),
            );

            ui.separator();

            if ui.button("Refresh mesh list").clicked() {
                get_dir_mesh_list(&mut editor);
            }

            ui.checkbox(&mut editor.new_instancing_enabled, "Spawn New Instance");
            egui::ComboBox::from_id_source("new_instance_combo_box")
                .selected_text(format!("{}", editor.new_instance_select))
                .show_ui(ui, |ui| {
                    for mesh in editor.meshes_list.clone() {
                        ui.selectable_value(
                            &mut editor.new_instance_select,
                            NewInstanceSelect::MeshName(mesh.clone()),
                            mesh,
                        );
                    }
                    // ui.selectable_value(&mut editor.transform_gizmo.mode, GizmoMode::Scale, "Scale");
                });
            ui.label("Instanced scale");
            ui.add(
                Slider::new(&mut editor.new_instance_scale, (0.0)..=(1.0))
                    .clamp_to_range(true)
                    .smart_aim(true)
            );

            ui.separator();

            ui.label("Selected Instance");

            ui.horizontal(|ui| {
                ui.label("Emission");

                ui.add(
                    Slider::new(&mut editor.selected_emission, (0.0)..=(10.0))
                        .clamp_to_range(false)
                        .smart_aim(true)
                );
            });

            let mut translation_str = "".to_string();
            let mut rotation_str = "".to_string();
            if let Some(target) = editor.selected_target {
                translation_str = format!("{:?}", target.origin);
                rotation_str = format!("{:?}", target.orientation);
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut translation_str).interactive(false));
                ui.label("Position");
            });
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut rotation_str).interactive(false));
                ui.label("Rotation");
            });

        });

    egui::TopBottomPanel::bottom("bottom_panel")
        .min_height(100.0)
        .max_height(400.0)
        .resizable(true)
        .show(egui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Console");

                ScrollArea::vertical()
                    .enable_scrolling(true)
                    .stick_to_bottom()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for log_message in get_console_logs() {
                                ui.label(log_message);
                            }
                        });
                    });
            });
        });

    // Update and show gizmo UI
    if editor.selected_target.is_some() || editor.new_instancing_enabled {
        egui::Area::new("viewport")
            .fixed_pos((0.0, 0.0))
            .show(egui.ctx(), |ui| {
                ui.with_layer_id(LayerId::background(), |ui| {
                    let last_response = editor.transform_gizmo.gizmo().interact(ui);
                    editor.transform_gizmo.last_response = last_response;
                });
            });
    } else {
        editor.transform_gizmo.last_response = None;
    }
}

pub fn update_transform_gizmo(
    mut editor: ResMut<EditorState>,
    render_world: Res<RenderWorld>,
) {
    let extracted_camera = render_world.get_resource::<ExtractedCamera>().unwrap();

    let view_matrix = KajiyaCamera::view_matrix_from_pos_rot(extracted_camera.transform);
    let projection_matrix = extracted_camera.camera.projection_matrix();

    if let Some(gizmo_response) = editor.transform_gizmo.last_response {
        let transform = Mat4::from_cols_array_2d(&gizmo_response.transform);
        let translation = transform.to_scale_rotation_translation().2;
        editor.transform_gizmo.last_translation = translation;
        editor.transform_gizmo.translation_offset = editor.transform_gizmo.last_translation;

        match gizmo_response.mode {
            egui_gizmo::GizmoMode::Rotate => {
                let delta: Vec3 = gizmo_response.value.into();
                let delta = delta * -1.0;

                let mut rotation = Quat::from_rotation_x(delta.x);
                rotation *= Quat::from_rotation_y(delta.y);
                rotation *= Quat::from_rotation_z(delta.z);
                editor.transform_gizmo.last_rotation = rotation * editor.transform_gizmo.rotation_offset;
            }
            egui_gizmo::GizmoMode::Scale => {
                let delta: Vec3 = gizmo_response.value.into();
                let delta = (delta[0] + delta[1] + delta[2]) / 3.0 - 1.0;

                editor.transform_gizmo.last_scale = editor.transform_gizmo.scale_offset + editor.transform_gizmo.scale_origin * Vec3::splat(delta);
            }
            _ => {}
        }

    } else {
        editor.transform_gizmo.last_translation = editor.transform_gizmo.translation_offset;
        editor.transform_gizmo.rotation_offset = editor.transform_gizmo.last_rotation;
        editor.transform_gizmo.scale_offset = editor.transform_gizmo.last_scale;
    }

    editor.transform_gizmo.model_matrix =
        Mat4::from_translation(editor.transform_gizmo.last_translation).to_cols_array_2d();

    editor.transform_gizmo.view_matrix = view_matrix.to_cols_array_2d();
    editor.transform_gizmo.projection_matrix = projection_matrix.to_cols_array_2d();
}

pub fn move_transform_gizmo(
    mut editor: ResMut<EditorState>,
) {

}