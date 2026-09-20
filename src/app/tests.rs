use std::path::PathBuf;

use crate::app::App;
use crate::app::state::{AppState, ProfileFocus, PrunePolicyState};
use crate::borg::{BackupArchive, BorgCheckMode};
use crate::config::PrunePolicy;

#[test]
fn test_app_initial_state() {
    let app = App::new();
    assert_eq!(app.state, AppState::Initializing);
    assert!(!app.should_quit);
    assert!(app.mounted_archives.is_empty());
}

#[test]
fn test_validate_and_submit_backup_empty_name() {
    let mut app = App::new();
    app.new_backup_name = "   ".to_string();
    app.validate_and_submit_backup();

    match app.state {
        AppState::ErrorPopup(ref msg) => {
            assert!(msg.contains("não pode ser vazio") || msg.contains("cannot be empty"));
        }
        _ => panic!("Esperava ErrorPopup ao submeter nome vazio"),
    }
}

#[test]
fn test_validate_and_submit_backup_invalid_characters() {
    let mut app = App::new();
    app.new_backup_name = "meu/backup:teste".to_string();
    app.file_browser
        .explicit_includes
        .insert(PathBuf::from("/tmp"));
    app.validate_and_submit_backup();

    match app.state {
        AppState::ErrorPopup(ref msg) => {
            assert!(msg.contains("caracteres reservados") || msg.contains("reserved characters"));
        }
        _ => panic!("Esperava ErrorPopup ao submeter nome com caracteres reservados"),
    }
}

#[test]
fn test_validate_and_submit_backup_no_files_selected() {
    let mut app = App::new();
    app.new_backup_name = "meu_backup".to_string();
    app.file_browser.explicit_includes.clear();
    app.validate_and_submit_backup();

    match app.state {
        AppState::ErrorPopup(ref msg) => {
            assert!(msg.contains("precisa selecionar") || msg.contains("must select"));
        }
        _ => panic!("Esperava ErrorPopup ao submeter sem arquivos selecionados"),
    }
}

#[test]
fn test_active_repo_fallback() {
    let mut app = App::new();
    app.config.active_repo_id = "non-existent-id".to_string();
    let active = app.get_active_repo();
    assert!(active.is_some());
    assert_eq!(
        active.unwrap().id,
        app.config.repositories.first().unwrap().id
    );
}

#[test]
fn test_empty_archive_navigation_no_panic() {
    let mut app = App::new();
    app.archives.clear();
    app.table_state.select(None);

    app.next();
    assert_eq!(app.table_state.selected(), Some(0));

    app.previous();
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn test_ask_delete_archive_no_selection_no_panic() {
    let mut app = App::new();
    app.archives.clear();
    app.table_state.select(None);

    app.ask_delete_archive();
    assert_eq!(app.state, AppState::Initializing);
}

#[test]
fn test_ask_restore_archive_no_selection_no_panic() {
    let mut app = App::new();
    app.archives.clear();
    app.table_state.select(None);

    app.ask_restore_selected_archive();
    assert_eq!(app.state, AppState::Initializing);
}

#[test]
fn test_umount_archive_not_mounted_gives_error() {
    let mut app = App::new();
    app.archives.push(BackupArchive {
        archive: "b1".to_string(),
        barchive: "b1".to_string(),
        id: "1".to_string(),
        name: "b1".to_string(),
        start: "2023-01-01".to_string(),
        time: "2023-01-01".to_string(),
    });
    app.table_state.select(Some(0));

    app.umount_selected_archive();
    match app.state {
        AppState::ErrorPopup(ref msg) => {
            assert!(msg.contains("não está montado") || msg.contains("not mounted"));
        }
        _ => panic!("Esperava ErrorPopup avisando que não está montado"),
    }
}

#[test]
fn test_profile_creation_and_run() {
    let mut app = App::new();
    app.config.profiles.clear();
    app.open_create_profile();
    match app.state {
        AppState::CreatingProfile(ref wizard) => {
            assert_eq!(wizard.focus, ProfileFocus::Name);
        }
        _ => panic!("Esperava CreatingProfile"),
    }

    if let AppState::CreatingProfile(ref mut wizard) = app.state {
        wizard.name = "TEST_PROFILE".to_string();
    }
    app.file_browser
        .explicit_includes
        .insert(PathBuf::from("/tmp/test"));
    app.save_profile();

    assert_eq!(app.config.profiles.len(), 1);
    assert_eq!(app.config.profiles[0].name, "TEST_PROFILE");
    assert_eq!(app.config.profiles[0].counter, 0);

    // Delete profile
    app.delete_profile(0);
    assert_eq!(app.config.profiles.len(), 0);
}

#[test]
fn test_open_diff_wizard_requires_two_archives() {
    let mut app = App::new();
    app.archives.clear();
    app.open_diff_wizard();
    match app.state {
        AppState::ErrorPopup(ref msg) => {
            assert!(msg.contains("2 backups") || msg.contains("at least 2"));
        }
        _ => panic!("Esperava ErrorPopup quando há menos de 2 backups"),
    }

    app.archives.push(BackupArchive {
        archive: "b1".to_string(),
        barchive: "b1".to_string(),
        id: "1".to_string(),
        name: "b1".to_string(),
        start: "2023-01-01".to_string(),
        time: "2023-01-01".to_string(),
    });
    app.archives.push(BackupArchive {
        archive: "b2".to_string(),
        barchive: "b2".to_string(),
        id: "2".to_string(),
        name: "b2".to_string(),
        start: "2023-01-02".to_string(),
        time: "2023-01-02".to_string(),
    });
    app.table_state.select(Some(0));
    app.open_diff_wizard();

    match app.state {
        AppState::DiffWizard(ref wizard) => {
            assert_eq!(wizard.base_archive, "b1");
            assert_eq!(wizard.candidates, vec!["b2"]);
            assert_eq!(wizard.selected_candidate_idx, 0);
            assert!(!wizard.content_only);
        }
        _ => panic!("Esperava DiffWizard com 2 backups"),
    }
}

#[test]
fn test_open_check_wizard_and_mode() {
    let mut app = App::new();
    app.open_check_wizard();

    match app.state {
        AppState::CheckWizard(ref state) => {
            assert_eq!(state.check_mode, BorgCheckMode::Standard);
            assert!(!state.check_archive_only);
        }
        _ => panic!("Esperava AppState::CheckWizard"),
    }
}

#[test]
fn test_prune_policy_state_conversion() {
    let policy = PrunePolicy {
        keep_last: Some(5),
        keep_daily: Some(7),
        keep_weekly: Some(4),
        keep_monthly: Some(12),
        keep_yearly: Some(1),
        prefix: Some("auto-".to_string()),
    };

    let state = PrunePolicyState::from_policy(&policy);
    assert_eq!(state.last_str, "5");
    assert_eq!(state.daily_str, "7");
    assert_eq!(state.weekly_str, "4");
    assert_eq!(state.monthly_str, "12");
    assert_eq!(state.yearly_str, "1");
    assert_eq!(state.prefix_str, "auto-");

    let converted_back = state.to_policy();
    assert_eq!(policy, converted_back);
}

#[test]
fn test_theme_toggle() {
    use crate::ui::theme::ThemeMode;

    let mut app = App::new();
    let initial_mode = app.theme.mode;
    app.toggle_theme();
    assert_ne!(app.theme.mode, initial_mode);
    match app.theme.mode {
        ThemeMode::Rust => assert_eq!(app.config.theme, "rust"),
        ThemeMode::Catppuccin => assert_eq!(app.config.theme, "catppuccin"),
    }
    app.toggle_theme();
    assert_eq!(app.theme.mode, initial_mode);
}

#[test]
fn test_help_modal_navigation() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    let mut app = App::new();
    app.state = AppState::Browsing;

    let key_question = KeyEvent {
        code: KeyCode::Char('?'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    crate::events::handle_key_event(&mut app, key_question);
    assert_eq!(app.state, AppState::HelpModal);

    let key_esc = KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    crate::events::handle_key_event(&mut app, key_esc);
    assert_eq!(app.state, AppState::Browsing);
}
