#![cfg(test)]

use uuid::Uuid;

use crate::{permissioms::model::PermissionModel, user::public_user::PublicUser};

#[test]
fn create_new_permission_model() {
    let id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    
    assert_eq!(permission_model.value(&calling_user).unwrap(), value);
}

#[test]
fn get_value_throw_err() {
    let id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };
    
    assert_eq!(permission_model.value(&calling_user).unwrap_err().message, "User does not have permission to view this value");
}

#[test]
fn add_viewer() {
    let id = Uuid::new_v4();
    let viewer_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_viewer = PublicUser { id: viewer_id.to_string(), first_name: None };

    let result = permission_model.add_viewer(new_viewer.clone(), &calling_user);
    
    assert!(result.is_ok());
    assert_eq!(permission_model.value(&new_viewer).unwrap(), value);
}

#[test]
fn add_viewer_no_permissions() {
    let id = Uuid::new_v4();
    let viewer_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };
    let new_viewer = PublicUser { id: viewer_id.to_string(), first_name: None };

    let result = permission_model.add_viewer(new_viewer.clone(), &calling_user);
    
    assert_eq!(result.unwrap_err().message, "User does not have permission to add a viewer");
}

#[test]
fn add_viewer_owner() {
    let id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };

    let result = permission_model.add_viewer(calling_user.clone(), &calling_user);
    
    assert_eq!(result.unwrap_err().message, "User is owner");
}

#[test]
fn add_viewer_twice_err() {
    let id = Uuid::new_v4();
    let viewer_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_viewer = PublicUser { id: viewer_id.to_string(), first_name: None };

    let result = permission_model.add_viewer(new_viewer.clone(), &calling_user);
    
    assert!(result.is_ok());

    let second_result = permission_model.add_viewer(new_viewer.clone(), &calling_user);
    assert_eq!(second_result.unwrap_err().message, "User already has permission to view this value");
    assert_eq!(permission_model.value(&new_viewer).unwrap(), value);
}

#[test]
fn add_viewer_cannot_mut() {
    let id = Uuid::new_v4();
    let viewer_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_viewer = PublicUser { id: viewer_id.to_string(), first_name: None };

    let result = permission_model.add_viewer(new_viewer.clone(), &calling_user);
    
    assert!(result.is_ok());
    assert_eq!(permission_model.value_mut(&new_viewer).unwrap_err().message, "User does not have permission to view this mut value");
}

#[test]
fn add_editor() {
    let id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_editor = PublicUser { id: editor_id.to_string(), first_name: None };

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);
    
    assert!(result.is_ok());
    assert_eq!(permission_model.value(&new_editor).unwrap(), value);
}

#[test]
fn add_editor_no_permissions() {
    let id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };
    let new_editor = PublicUser { id: editor_id.to_string(), first_name: None };

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);
    
    assert_eq!(result.unwrap_err().message, "User does not have permission to add an editor");
}

#[test]
fn add_editor_as_editor() {
    let id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let second_editor_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_editor = PublicUser { id: editor_id.to_string(), first_name: None };
    let second_editor = PublicUser { id: second_editor_id.to_string(), first_name: None };

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);

    assert!(result.is_ok());

    let second_result = permission_model.add_editor(second_editor.clone(), &new_editor);

    assert!(second_result.is_ok());
    assert_eq!(permission_model.value(&new_editor).unwrap(), value.clone());
    assert_eq!(permission_model.value(&second_editor).unwrap(), value);
}

#[test]
fn add_editor_owner() {
    let id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };

    let result = permission_model.add_editor(calling_user.clone(), &calling_user);
    
    assert_eq!(result.unwrap_err().message, "User is owner");
}

#[test]
fn add_editor_twice_err() {
    let id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_editor = PublicUser { id: editor_id.to_string(), first_name: None };

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);
    
    assert!(result.is_ok());

    let second_result = permission_model.add_editor(new_editor.clone(), &calling_user);
    assert_eq!(second_result.unwrap_err().message, "User already has permission to edit this value");
    assert_eq!(permission_model.value(&new_editor).unwrap(), value);
}

#[test]
fn add_editor_can_mut() {
    let id = Uuid::new_v4();
    let editor_id = Uuid::new_v4();
    let mut value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_editor = PublicUser { id: editor_id.to_string(), first_name: None };

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);
    
    assert!(result.is_ok());
    assert_eq!(permission_model.value_mut(&new_editor).unwrap(), &mut value);
}

#[test]
fn get_value_mut() {
    let id = Uuid::new_v4();
    let mut value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    
    assert_eq!(permission_model.value_mut(&calling_user).unwrap(), &mut value);
}

#[test]
fn remove_viewer() {
    let id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_editor = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };
    let new_viewer = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };

    let add_viewer_result = permission_model.add_viewer(new_viewer.clone(), &calling_user);

    assert!(add_viewer_result.is_ok());
    assert_eq!(permission_model.value(&new_viewer).unwrap(), value);

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);
    
    assert!(result.is_ok());

    let remove_result = permission_model.remove_viewer(new_viewer.clone(), &new_editor);
    assert!(remove_result.is_ok());
    assert!(permission_model.value(&new_viewer).is_err());
}

#[test]
fn remove_editor() {
    let id = Uuid::new_v4();
    let value: String = "whoop".into();
    let key: String = "blah".into();
    let mut permission_model = PermissionModel::new(id.to_string(), value.clone(), key.clone());
    
    let calling_user = PublicUser { id: id.to_string(), first_name: None };
    let new_editor = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };
    let new_viewer = PublicUser { id: Uuid::new_v4().to_string(), first_name: None };

    let add_viewer_result = permission_model.add_viewer(new_viewer.clone(), &calling_user);

    assert!(add_viewer_result.is_ok());
    assert_eq!(permission_model.value(&new_viewer).unwrap(), value);

    let result = permission_model.add_editor(new_editor.clone(), &calling_user);
    
    assert!(result.is_ok());

    let remove_result = permission_model.remove_viewer(new_viewer.clone(), &new_editor);
    assert!(remove_result.is_ok());
    assert!(permission_model.value(&new_viewer).is_err());
}