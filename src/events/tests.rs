use crate::app::App;
use crate::app::state::{
    AppState, CheckResultState, CheckWizardState, DiffViewState, DiffWizardState, ProfileFocus,
};
use crate::borg::{BorgCheckMode, CheckResult};
use crate::events::handle_key_event;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

#[test]
fn test_browsing_v_opens_check_wizard() {
    let mut app = App::new();
    app.state = AppState::Browsing;

    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('v')));
    match app.state {
        AppState::CheckWizard(ref state) => {
            assert_eq!(state.check_mode, BorgCheckMode::Standard);
        }
        _ => panic!("Esperava CheckWizard ao pressionar 'v'"),
    }
}

#[test]
fn test_check_wizard_navigation_and_toggle() {
    let mut app = App::new();
    app.state = AppState::CheckWizard(CheckWizardState {
        target_archive: Some("archive1".to_string()),
        check_mode: BorgCheckMode::Standard,
        check_archive_only: false,
    });

    // Tab should toggle check_archive_only
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Tab));
    if let AppState::CheckWizard(ref state) = app.state {
        assert!(state.check_archive_only);
    }

    // Down should cycle to VerifyData
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
    if let AppState::CheckWizard(ref state) = app.state {
        assert_eq!(state.check_mode, BorgCheckMode::VerifyData);
    }

    // Up should cycle back to Standard
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Up));
    if let AppState::CheckWizard(ref state) = app.state {
        assert_eq!(state.check_mode, BorgCheckMode::Standard);
    }

    // Esc should exit back to browsing
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.state, AppState::Browsing);
}

#[test]
fn test_check_result_view_scroll_and_exit() {
    let mut app = App::new();
    app.state = AppState::CheckResultView(CheckResultState {
        result: CheckResult {
            success: true,
            warnings: false,
            log_output: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        },
        target_display: "Repo".to_string(),
        mode_display: "Standard".to_string(),
        log_scroll: 0,
    });

    // Down should scroll
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
    if let AppState::CheckResultView(ref state) = app.state {
        assert_eq!(state.log_scroll, 1);
    }

    // Esc should exit
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.state, AppState::Browsing);
}

#[test]
fn test_diff_wizard_navigation_and_toggle() {
    let mut app = App::new();
    app.state = AppState::DiffWizard(DiffWizardState {
        base_archive: "base".to_string(),
        candidates: vec!["cand1".to_string(), "cand2".to_string()],
        selected_candidate_idx: 0,
        content_only: false,
    });

    // Tab toggles content_only
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Tab));
    if let AppState::DiffWizard(ref state) = app.state {
        assert!(state.content_only);
    }

    // Down moves to candidate 1
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
    if let AppState::DiffWizard(ref state) = app.state {
        assert_eq!(state.selected_candidate_idx, 1);
    }

    // Esc returns to browsing
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.state, AppState::Browsing);
}

#[test]
fn test_managing_profiles_navigation_and_shortcuts() {
    let mut app = App::new();
    app.state = AppState::Browsing;

    // 'b' opens profiles view
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('b')));
    assert_eq!(app.state, AppState::ManagingProfiles { selected_index: 0 });

    // 'a' opens create profile wizard
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('a')));
    match app.state {
        AppState::CreatingProfile(ref wizard) => {
            assert_eq!(wizard.focus, ProfileFocus::Name);
        }
        _ => panic!("Esperava CreatingProfile"),
    }

    // Tab cycles focus to Compression
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Tab));
    if let AppState::CreatingProfile(ref wizard) = app.state {
        assert_eq!(wizard.focus, ProfileFocus::Compression);
    }

    // Esc returns to ManagingProfiles
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.state, AppState::ManagingProfiles { selected_index: 0 });
}

#[test]
fn test_diff_view_navigation() {
    let mut app = App::new();
    app.state = AppState::DiffView(DiffViewState {
        archive1: "a1".to_string(),
        archive2: "a2".to_string(),
        entries: vec![
            crate::borg::DiffEntry {
                path: "p1".to_string(),
                changes: vec![],
            },
            crate::borg::DiffEntry {
                path: "p2".to_string(),
                changes: vec![],
            },
        ],
        selected_index: 0,
    });

    handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
    if let AppState::DiffView(ref state) = app.state {
        assert_eq!(state.selected_index, 1);
    }

    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('q')));
    assert_eq!(app.state, AppState::Browsing);
}

#[test]
fn test_log_viewer_navigation_and_filters() {
    let mut app = App::new();
    app.state = AppState::Browsing;

    // 'L' opens log viewer
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('L')));
    assert!(matches!(app.state, AppState::LogViewer(_)));

    if let AppState::LogViewer(ref mut state) = app.state {
        state.all_lines = vec![
            "[2026-09-19 22:00:00.000] [INFO] Info 1".to_string(),
            "[2026-09-19 22:00:01.000] [WARN] Warn 1".to_string(),
            "[2026-09-19 22:00:02.000] [ERROR] Error 1".to_string(),
        ];
    }

    // Key '2' sets filter to Info
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('2')));
    if let AppState::LogViewer(ref state) = app.state {
        assert_eq!(state.filter, crate::app::state::LogFilterLevel::Info);
        assert_eq!(state.filtered_indices().len(), 1);
    }

    // Key '4' sets filter to Error
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('4')));
    if let AppState::LogViewer(ref state) = app.state {
        assert_eq!(state.filter, crate::app::state::LogFilterLevel::Error);
        assert_eq!(state.filtered_indices().len(), 1);
    }

    // Key '1' sets filter back to All
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('1')));
    if let AppState::LogViewer(ref state) = app.state {
        assert_eq!(state.filter, crate::app::state::LogFilterLevel::All);
        assert_eq!(state.filtered_indices().len(), 3);
    }

    // Esc returns to Browsing
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.state, AppState::Browsing);
}

#[test]
fn test_log_viewer_clear_and_reload() {
    let mut app = App::new();
    app.state = AppState::Browsing;

    // 'o' also opens log viewer
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('o')));
    assert!(matches!(app.state, AppState::LogViewer(_)));

    // 'c' clears logs
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('c')));
    if let AppState::LogViewer(ref state) = app.state {
        assert!(state.all_lines.iter().any(|l| l.contains("Logs limpos")));
    }

    // 'q' closes log viewer
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('q')));
    assert_eq!(app.state, AppState::Browsing);
}

#[test]
fn test_settings_navigation_and_actions() {
    let mut app = App::new();
    app.state = AppState::Browsing;

    // 's' opens Settings modal
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('s')));
    assert_eq!(app.state, AppState::Settings(crate::app::state::SettingsState { selected_index: 0 }));

    // Item 0 is Language: Enter toggles language
    let initial_lang = app.t.lang;
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Enter));
    assert_ne!(app.t.lang, initial_lang);

    // Down moves to Item 1 (Theme)
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
    if let AppState::Settings(ref state) = app.state {
        assert_eq!(state.selected_index, 1);
    }

    // Enter toggles theme
    let initial_theme = app.theme.mode;
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Enter));
    assert_ne!(app.theme.mode, initial_theme);

    // Down moves to Item 2 (Repositories)
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Enter));
    assert_eq!(app.state, AppState::ManagingRepos);

    // Esc from repos returns to Browsing
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.state, AppState::Browsing);

    // 's' re-opens settings
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('s')));
    // 'q' or 's' or 'Esc' closes settings
    handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('s')));
    assert_eq!(app.state, AppState::Browsing);
}
