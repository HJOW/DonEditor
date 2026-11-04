use app_actions::{ToggleFocus as ToggleDebugPanel, dev};
use gpui::{App, Menu, MenuItem, OsAction};
use language_tools::lsp_config_view::OpenLanguageServerConfig;
use release_channel::ReleaseChannel;
use terminal_view::terminal_panel;

pub fn app_menus(cx: &mut App) -> Vec<Menu> {
    use app_actions::Quit;

    // [DonEditor] 메뉴 문자열은 i18n 키로 가져온다 (표시 언어는 설정 `ui_language`). {app} 은 제품 이름.
    let app_name = ReleaseChannel::try_global(cx).map_or("Don Editor", |channel| channel.display_name());

    let mut view_items = vec![
        MenuItem::action(
            i18n::t!("menu.view.zoom_in"),
            app_actions::IncreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            i18n::t!("menu.view.zoom_out"),
            app_actions::DecreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            i18n::t!("menu.view.reset_zoom"),
            app_actions::ResetBufferFontSize { persist: false },
        ),
        MenuItem::action(
            i18n::t!("menu.view.reset_all_zoom"),
            app_actions::ResetAllZoom { persist: false },
        ),
        MenuItem::separator(),
        MenuItem::action(i18n::t!("menu.view.toggle_left_dock"), workspace::ToggleLeftDock),
        MenuItem::action(i18n::t!("menu.view.toggle_right_dock"), workspace::ToggleRightDock),
        MenuItem::action(i18n::t!("menu.view.toggle_bottom_dock"), workspace::ToggleBottomDock),
        MenuItem::action(i18n::t!("menu.view.toggle_all_docks"), workspace::ToggleAllDocks),
        MenuItem::submenu(Menu {
            name: i18n::t!("menu.view.editor_layout").into(),
            items: vec![
                MenuItem::action(i18n::t!("menu.view.split_up"), workspace::SplitUp),
                MenuItem::action(i18n::t!("menu.view.split_down"), workspace::SplitDown),
                MenuItem::action(i18n::t!("menu.view.split_left"), workspace::SplitLeft),
                MenuItem::action(i18n::t!("menu.view.split_right"), workspace::SplitRight),
            ],
        }),
        MenuItem::separator(),
        MenuItem::action(
            i18n::t!("menu.view.project_panel"),
            app_actions::project_panel::ToggleFocus,
        ),
        MenuItem::action(i18n::t!("menu.view.outline_panel"), outline_panel::ToggleFocus),
        MenuItem::action(i18n::t!("menu.view.terminal_panel"), terminal_panel::ToggleFocus),
        MenuItem::action(i18n::t!("menu.view.debugger_panel"), ToggleDebugPanel),
        // [DonEditor] 숨겨진 Agent 패널과 로컬 MCP 서버에 메뉴로도 접근한다.
        MenuItem::action(i18n::t!("menu.view.agent_panel"), agent_panel::ToggleFocus),
        MenuItem::action(i18n::t!("menu.view.mcp_toggle"), mcp_server::ToggleServer),
        MenuItem::action(i18n::t!("menu.view.mcp_connection"), mcp_server::ShowConnectionInfo),
        MenuItem::separator(),
        MenuItem::action(i18n::t!("menu.view.diagnostics"), diagnostics::Deploy),
        MenuItem::separator(),
    ];

    if ReleaseChannel::try_global(cx) == Some(ReleaseChannel::Dev) {
        view_items.push(MenuItem::action(
            i18n::t!("menu.view.toggle_inspector"),
            dev::ToggleInspector,
        ));
        view_items.push(MenuItem::separator());
    }

    let menus = vec![
        // [DonEditor] 제품 이름 변경 (Gram → Don Editor): 메뉴 이름, 정보, 종료 항목
        Menu {
            name: app_name.into(),
            items: vec![
                MenuItem::action(i18n::t!("menu.app.about", app = app_name), app_actions::About),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: i18n::t!("menu.app.settings").into(),
                    items: vec![
                        MenuItem::action(i18n::t!("menu.app.settings.open"), app_actions::OpenSettings),
                        MenuItem::action(i18n::t!("menu.app.settings.open_file"), super::OpenSettingsFile),
                        MenuItem::action(
                            i18n::t!("menu.app.settings.open_project"),
                            app_actions::OpenProjectSettings,
                        ),
                        MenuItem::action(
                            i18n::t!("menu.app.settings.open_project_file"),
                            super::OpenProjectSettingsFile,
                        ),
                        MenuItem::action(i18n::t!("menu.app.settings.open_default"), super::OpenDefaultSettings),
                        MenuItem::action(i18n::t!("menu.app.settings.language_servers"), OpenLanguageServerConfig),
                        MenuItem::separator(),
                        MenuItem::action(i18n::t!("menu.app.settings.open_keymap"), app_actions::OpenKeymap),
                        MenuItem::action(
                            i18n::t!("menu.app.settings.open_keymap_file"),
                            app_actions::OpenKeymapFile,
                        ),
                        MenuItem::action(
                            i18n::t!("menu.app.settings.open_default_keymap"),
                            app_actions::OpenDefaultKeymap,
                        ),
                        MenuItem::separator(),
                        MenuItem::action(
                            i18n::t!("menu.app.settings.select_theme"),
                            app_actions::theme_selector::Toggle::default(),
                        ),
                        MenuItem::action(
                            i18n::t!("menu.app.settings.select_icon_theme"),
                            app_actions::icon_theme_selector::Toggle::default(),
                        ),
                    ],
                }),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::os_submenu(i18n::t!("menu.app.services"), gpui::SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.app.extensions"), app_actions::Extensions::default()),
                #[cfg(not(target_os = "windows"))]
                MenuItem::action(i18n::t!("menu.app.install_cli"), install_cli::InstallCliBinary),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::action(i18n::t!("menu.app.hide", app = app_name), super::Hide),
                #[cfg(target_os = "macos")]
                MenuItem::action(i18n::t!("menu.app.hide_others"), super::HideOthers),
                #[cfg(target_os = "macos")]
                MenuItem::action(i18n::t!("menu.app.show_all"), super::ShowAll),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.app.quit", app = app_name), Quit),
            ],
        },
        Menu {
            name: i18n::t!("menu.file").into(),
            items: vec![
                MenuItem::action(i18n::t!("menu.file.new"), workspace::NewFile),
                MenuItem::action(i18n::t!("menu.file.new_window"), workspace::NewWindow),
                MenuItem::separator(),
                #[cfg(not(target_os = "macos"))]
                MenuItem::action(i18n::t!("menu.file.open_file"), workspace::OpenFiles),
                MenuItem::action(
                    if cfg!(not(target_os = "macos")) {
                        i18n::t!("menu.file.open_folder")
                    } else {
                        i18n::t!("menu.file.open")
                    },
                    workspace::Open,
                ),
                MenuItem::action(
                    i18n::t!("menu.file.open_recent"),
                    app_actions::OpenRecent {
                        create_new_window: false,
                    },
                ),
                MenuItem::action(
                    i18n::t!("menu.file.open_remote"),
                    app_actions::OpenRemote {
                        create_new_window: false,
                        from_existing_connection: false,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.file.add_folder"), workspace::AddFolderToProject),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.file.save"), workspace::Save { save_intent: None }),
                MenuItem::action(i18n::t!("menu.file.save_as"), workspace::SaveAs),
                MenuItem::action(i18n::t!("menu.file.save_all"), workspace::SaveAll { save_intent: None }),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::t!("menu.file.close_editor"),
                    workspace::CloseActiveItem {
                        save_intent: None,
                        close_pinned: true,
                    },
                ),
                MenuItem::action(i18n::t!("menu.file.close_project"), workspace::CloseProject),
                MenuItem::action(i18n::t!("menu.file.close_window"), workspace::CloseWindow),
            ],
        },
        Menu {
            name: i18n::t!("menu.edit").into(),
            items: vec![
                MenuItem::os_action(i18n::t!("menu.edit.undo"), editor::actions::Undo, OsAction::Undo),
                MenuItem::os_action(i18n::t!("menu.edit.redo"), editor::actions::Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action(i18n::t!("menu.edit.cut"), editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action(i18n::t!("menu.edit.copy"), editor::actions::Copy, OsAction::Copy),
                MenuItem::action(i18n::t!("menu.edit.copy_and_trim"), editor::actions::CopyAndTrim),
                MenuItem::os_action(i18n::t!("menu.edit.paste"), editor::actions::Paste, OsAction::Paste),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.edit.find"), search::buffer_search::Deploy::find()),
                MenuItem::action(i18n::t!("menu.edit.find_in_project"), workspace::DeploySearch::find()),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::t!("menu.edit.toggle_comment"),
                    editor::actions::ToggleComments::default(),
                ),
            ],
        },
        Menu {
            name: i18n::t!("menu.selection").into(),
            items: vec![
                MenuItem::os_action(
                    i18n::t!("menu.selection.select_all"),
                    editor::actions::SelectAll,
                    OsAction::SelectAll,
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.expand"),
                    editor::actions::SelectLargerSyntaxNode,
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.shrink"),
                    editor::actions::SelectSmallerSyntaxNode,
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.next_sibling"),
                    editor::actions::SelectNextSyntaxNode,
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.previous_sibling"),
                    editor::actions::SelectPreviousSyntaxNode,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::t!("menu.selection.cursor_above"),
                    editor::actions::AddSelectionAbove { skip_soft_wrap: true },
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.cursor_below"),
                    editor::actions::AddSelectionBelow { skip_soft_wrap: true },
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.next_occurrence"),
                    editor::actions::SelectNext { replace_newest: false },
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.previous_occurrence"),
                    editor::actions::SelectPrevious { replace_newest: false },
                ),
                MenuItem::action(
                    i18n::t!("menu.selection.all_occurrences"),
                    editor::actions::SelectAllMatches,
                ),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.selection.move_line_up"), editor::actions::MoveLineUp),
                MenuItem::action(i18n::t!("menu.selection.move_line_down"), editor::actions::MoveLineDown),
                MenuItem::action(i18n::t!("menu.selection.duplicate"), editor::actions::DuplicateLineDown),
            ],
        },
        Menu {
            name: i18n::t!("menu.view").into(),
            items: view_items,
        },
        Menu {
            name: i18n::t!("menu.go").into(),
            items: vec![
                MenuItem::action(i18n::t!("menu.go.back"), workspace::GoBack),
                MenuItem::action(i18n::t!("menu.go.forward"), workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::t!("menu.go.command_palette"),
                    app_actions::command_palette::Toggle,
                ),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.go.file"), workspace::ToggleFileFinder::default()),
                // MenuItem::action("Go to Symbol in Project", project_symbols::Toggle),
                MenuItem::action(i18n::t!("menu.go.symbol"), app_actions::outline::ToggleOutline),
                MenuItem::action(i18n::t!("menu.go.line"), editor::actions::ToggleGoToLine),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.go.definition"), editor::actions::GoToDefinition),
                MenuItem::action(i18n::t!("menu.go.declaration"), editor::actions::GoToDeclaration),
                MenuItem::action(i18n::t!("menu.go.type_definition"), editor::actions::GoToTypeDefinition),
                MenuItem::action(
                    i18n::t!("menu.go.references"),
                    editor::actions::FindAllReferences::default(),
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::t!("menu.go.next_problem"),
                    editor::actions::GoToDiagnostic::default(),
                ),
                MenuItem::action(
                    i18n::t!("menu.go.previous_problem"),
                    editor::actions::GoToPreviousDiagnostic::default(),
                ),
            ],
        },
        Menu {
            name: i18n::t!("menu.run").into(),
            items: vec![
                MenuItem::action(
                    i18n::t!("menu.run.spawn_task"),
                    app_actions::Spawn::ViaModal { reveal_target: None },
                ),
                MenuItem::action(i18n::t!("menu.run.start_debugger"), debugger_ui::Start),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.run.edit_tasks"), crate::gram::OpenProjectTasks),
                MenuItem::action(i18n::t!("menu.run.edit_debug"), app_actions::OpenProjectDebugTasks),
                MenuItem::separator(),
                MenuItem::action(i18n::t!("menu.run.continue"), debugger_ui::Continue),
                MenuItem::action(i18n::t!("menu.run.step_over"), debugger_ui::StepOver),
                MenuItem::action(i18n::t!("menu.run.step_into"), debugger_ui::StepInto),
                MenuItem::action(i18n::t!("menu.run.step_out"), debugger_ui::StepOut),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::t!("menu.run.toggle_breakpoint"),
                    editor::actions::ToggleBreakpoint,
                ),
                MenuItem::action(i18n::t!("menu.run.edit_breakpoint"), editor::actions::EditLogBreakpoint),
                MenuItem::action(i18n::t!("menu.run.clear_breakpoints"), debugger_ui::ClearAllBreakpoints),
            ],
        },
        Menu {
            name: i18n::t!("menu.window").into(),
            items: vec![
                MenuItem::action(i18n::t!("menu.window.minimize"), super::Minimize),
                MenuItem::action(i18n::t!("menu.window.zoom"), super::Zoom),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: i18n::t!("menu.help").into(),
            items: vec![
                MenuItem::action(i18n::t!("menu.help.license"), app_actions::ViewLicense),
                MenuItem::action(i18n::t!("menu.help.welcome"), onboarding::ShowWelcome),
                MenuItem::separator(),
                // [DonEditor] 자체 안내를 먼저 제공하며 원본 도움말도 그대로 유지한다.
                MenuItem::action(
                    i18n::t!("menu.help.don_guide"),
                    app_actions::OpenDocsAt {
                        path: "don/index".into(),
                    },
                ),
                MenuItem::action(
                    i18n::t!("menu.help.documentation"),
                    super::OpenGramUrl {
                        url: "gram://docs/".into(),
                    },
                ),
                MenuItem::action(i18n::t!("menu.help.repository"), super::OpenGramRepo),
            ],
        },
    ];

    // [DonEditor] 명령 팔레트가 한국어 이름을 함께 보여 줄 수 있게, 메뉴에 있는 액션의 표시 이름을 i18n 에 등록한다.
    register_action_labels(&menus);
    menus
}

/// 메뉴 항목의 이름을 해당 액션의 표시 이름으로 등록한다 (하위 메뉴 포함).
/// 이름 끝의 "..." / "…" 는 명령 이름에는 어울리지 않으므로 뗀다.
fn register_action_labels(menus: &[Menu]) {
    // 메뉴와 하위 메뉴의 모든 항목을 훑는다
    for menu in menus {
        for item in &menu.items {
            match item {
                MenuItem::Action { name, action, .. } => {
                    let label = name.trim_end_matches(['…', '.']).trim();
                    i18n::set_action_label(action.name(), label);
                }
                MenuItem::Submenu(submenu) => register_action_labels(std::slice::from_ref(submenu)),
                MenuItem::Separator | MenuItem::SystemMenu(_) => {}
            }
        }
    }
}
