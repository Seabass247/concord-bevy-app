use bevy::{prelude::*};
use bevy_kajiya::kajiya_render::{KajiyaMeshInstance, KajiyaMeshInstanceBundle, KajiyaMesh};
use concord_logger::console_info;

use crate::{EditorState, NewInstanceSelect, SelectableTag};

#[derive(Component, Copy, Clone)]
pub struct TargetTag;

#[derive(Default, Copy, Clone)]
pub struct Target {
    pub entity: Option<Entity>,
    pub origin: Vec3,
    pub orientation: Quat,
}

pub fn select_new_target(
    commands: &mut Commands,
    editor: &mut EditorState,
    new_target: Target,
) -> bool {
    let entity = new_target.entity.unwrap();

    if let Some(target) = editor.selected_target {
        // If a target is already set and we are trying to set a new (and different) target
        if target.entity.unwrap() != entity {
            unset_entity_target(commands, editor);
            set_entity_target(commands, editor, entity, new_target);
            console_info!("select different new true");

            return true;
        }
        console_info!("select new false");

        false
    } else {
        // The case where a target has not yet been set
        set_entity_target(commands, editor, entity, new_target);
        console_info!("select completely new true");

        true
    }
}

fn set_entity_target(
    commands: &mut Commands,
    editor: &mut EditorState,
    entity: Entity,
    new_target: Target,
) {
    commands.entity(entity).insert(TargetTag);
    editor.selected_target = Some(new_target);

    console_info!("Selected entity");
}

pub fn unset_entity_target(commands: &mut Commands, editor: &mut EditorState) {
    if let Some(target) = editor.selected_target {
        commands
            .entity(target.entity.unwrap())
            .remove::<TargetTag>();
        editor.selected_target = None;
        console_info!("Deselect entity");
    }
}

pub fn update_target(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    mut query_trans: Query<(&mut Transform, &KajiyaMeshInstance)>,
) {
    // Don't need to update/move target from gizmo when instancing mode is enabled
    if editor.new_instancing_enabled {
        return;
    }

    // Handle picked target event from the raycast if there is one
    if let Some(target) = editor.picked_target.take() {
        if select_new_target(&mut commands, &mut editor, target) {
            editor.transform_gizmo.translation_offset = target.origin;
            editor.transform_gizmo.rotation_offset = target.orientation;
            editor.transform_gizmo.last_rotation = target.orientation;
        }

        return;
    }

    // Only perform target query if there is a target selected
    let target = if let Some(target) = editor.selected_target {
        target
    } else {
        return;
    };

    // Get the transform component of the target's entity and mutate it
    if let Ok((mut transform, _mesh)) = query_trans.get_mut(target.entity.unwrap()) {
        transform.translation = editor.transform_gizmo.last_translation;
        transform.rotation = editor.transform_gizmo.last_rotation;
    }

}

pub fn instance_new_target(
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    editor: ResMut<EditorState>,
    mut commands: Commands,
) {
    // Can only be instanced if flag is enabled and is triggered by pressing LShift+LClick
    if editor.new_instancing_enabled &&
    buttons.just_pressed(MouseButton::Left) &&
    keys.pressed(KeyCode::LShift) {
        if let NewInstanceSelect::MeshName(name) = &editor.new_instance_select {
            commands.spawn_bundle(KajiyaMeshInstanceBundle {
                mesh_instance: KajiyaMeshInstance {
                    mesh: KajiyaMesh::Name(name.to_owned()),
                    scale: editor.new_instance_scale,
                },
                transform: Transform::from_translation(editor.transform_gizmo.last_translation),
                ..Default::default()
            }).insert(SelectableTag);
        }
        console_info!("Spawned mesh instance at {}", editor.transform_gizmo.last_translation);
        return;
    }
}