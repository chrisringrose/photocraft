//! Optional Info panel readouts. These are display preferences, separate from layer opacity.

use serde::{Deserialize, Serialize};

use crate::PhotocraftApp;

const OPTIONS_PREF: &str = "ui.infoPanelOptions";

/// Info panel display options, kept between launches; older settings leave extra readouts hidden.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InfoPanelState {
    /// Show sampled composite alpha, which can differ from the selected layer's opacity setting.
    pub show_composite_opacity: bool,
}

/// Save display options independently of Workspace > Remember Workspace Changes.
pub(crate) fn set_options(app: &mut PhotocraftApp, options: InfoPanelState) {
    app.ui.info_panel = options;
    if let Ok(value) = serde_json::to_value(options) {
        app.session.prefs.edit(|p| {
            p.dialogs.insert(OPTIONS_PREF.into(), value);
        });
    }
}

/// Restore saved options; missing or unreadable preferences retain the default hidden readout.
pub(crate) fn restore_options(app: &mut PhotocraftApp) {
    if let Some(value) = app.session.prefs().dialogs.get(OPTIONS_PREF)
        && let Ok(options) = serde_json::from_value(value.clone())
    {
        app.ui.info_panel = options;
    }
}

/// The Info panel's menu, shared by docked and floating panels.
pub(crate) fn panel_menu(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let mut options = app.ui.info_panel;
    if ui.checkbox(&mut options.show_composite_opacity, tl!("Show composite opacity")).changed() {
        set_options(app, options);
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Old UI/settings files keep the optional readout hidden; malformed saved options are ignored.
    #[test]
    fn info_panel_options_default_hidden_for_old_or_invalid_settings() {
        let mut value = serde_json::to_value(crate::state::UiState::default()).unwrap();
        value.as_object_mut().unwrap().remove("info_panel");
        let restored: crate::state::UiState = serde_json::from_value(value).unwrap();
        assert!(!restored.info_panel.show_composite_opacity);
        for value in [json!({}), json!({"showCompositeOpacity": "yes"}), json!(null)] {
            let mut session = photocraft_engine::Session::new();
            session.prefs.edit(|p| {
                p.dialogs.insert(OPTIONS_PREF.into(), value);
            });
            let app = PhotocraftApp::new(session, Default::default());
            assert!(!app.ui.info_panel.show_composite_opacity);
        }
    }

    /// A display choice survives preferences serialization and a new shell, without workspace saving.
    #[test]
    fn info_panel_options_survive_restart_independently_of_workspace() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.session.prefs.edit(|p| p.workspace.remember_workspace_changes = false);
        for on in [true, false] {
            set_options(&mut app, InfoPanelState { show_composite_opacity: on });
            let mut session = photocraft_engine::Session::new();
            session.load_prefs_json(&app.session.prefs_to_json()).unwrap();
            let restored = PhotocraftApp::new(session, Default::default());
            assert_eq!(restored.ui.info_panel.show_composite_opacity, on);
        }
    }
}
