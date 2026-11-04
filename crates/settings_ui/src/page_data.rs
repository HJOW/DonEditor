use gpui::{Action as _, App};
use settings::{LanguageSettingsContent, SettingsContent};
use std::sync::Arc;
use strum::IntoDiscriminant as _;
use ui::{IntoElement, SharedString};

use crate::{
    ActionLink, DynamicItem, PROJECT, SettingField, SettingItem, SettingsFieldMetadata, SettingsPage, SettingsPageItem,
    SubPageLink, USER, all_language_names, sub_page_stack,
};

/// 빈 기본값을 공유하는 정적 문자열입니다.
const DEFAULT_SHARED_STRING: SharedString = SharedString::new_static("");
/// 동적 항목과 선택적 설정에서 기본값이 없을 때 사용할 빈 문자열 참조입니다.
const DEFAULT_EMPTY_SHARED_STRING: Option<&SharedString> = Some(&DEFAULT_SHARED_STRING);

/// 설정 화면의 고정 메타데이터만 키로 번역하므로 파일명 등 사용자 값에는 영향을 주지 않습니다.
pub(crate) fn settings_data(cx: &App) -> Vec<SettingsPage> {
    vec![
        SettingsPage {
            title: i18n::t!("settings.text.general"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.general_settings")),
                // [DonEditor] 메뉴·패널·대화상자의 표시 언어 (한국어 UI)
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.ui_language"),
                    description: i18n::t!("settings.text.the_display_language_of_menus_panels_and_dialogs_auto_fo_9b1c3fc9"),
                    aliases: Some(&["language", "locale", "korean", "한국어"]),
                    field: Box::new(SettingField {
                        json_path: Some("ui_language"),
                        pick: |settings_content| settings_content.ui_language.as_ref(),
                        write: |settings_content, value| {
                            settings_content.ui_language = value;
                        },
                    }),
                    metadata: Some(Box::new(SettingsFieldMetadata {
                        should_do_titlecase: Some(false),
                        ..Default::default()
                    })),
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.when_closing_with_no_tabs"),
                    description: i18n::t!("settings.text.what_to_do_when_using_the_close_active_item_action_with_no_tabs"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("when_closing_with_no_tabs"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .when_closing_with_no_tabs
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.when_closing_with_no_tabs = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.on_last_window_closed"),
                    description: i18n::t!("settings.text.what_to_do_when_the_last_window_is_closed"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("on_last_window_closed"),
                        pick: |settings_content| {
                            settings_content.workspace.on_last_window_closed.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.on_last_window_closed = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.use_system_path_prompts"),
                    description: i18n::t!("settings.text.use_native_os_dialogs_for_open_and_save_as"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("use_system_path_prompts"),
                        pick: |settings_content| {
                            settings_content.workspace.use_system_path_prompts.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.use_system_path_prompts = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.use_system_prompts"),
                    description: i18n::t!("settings.text.use_native_os_dialogs_for_confirmations"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("use_system_prompts"),
                        pick: |settings_content| {
                            settings_content.workspace.use_system_prompts.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.use_system_prompts = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.redact_private_values"),
                    description: i18n::t!("settings.text.hide_the_values_of_variables_in_private_files"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("redact_private_values"),
                        pick: |settings_content| {
                            settings_content.editor.redact_private_values.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.redact_private_values = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.private_files"),
                    description: i18n::t!("settings.text.globs_to_match_against_file_paths_to_determine_if_a_file_is_private"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("worktree.private_files"),
                            pick: |settings_content| {
                                settings_content.project.worktree.private_files.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.project.worktree.private_files = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.workspace_restoration")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.restore_unsaved_buffers"),
                    description: i18n::t!("settings.text.whether_or_not_to_restore_unsaved_buffers_on_restart"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("session.restore_unsaved_buffers"),
                        pick: |settings_content| {
                            settings_content
                                .session
                                .as_ref()
                                .and_then(|session| session.restore_unsaved_buffers.as_ref())
                        },
                        write: |settings_content, value| {
                            settings_content
                                .session
                                .get_or_insert_default()
                                .restore_unsaved_buffers = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.restore_on_startup"),
                    description: i18n::t!("settings.text.what_to_restore_from_the_previous_session_when_opening_gram"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("restore_on_startup"),
                        pick: |settings_content| {
                            settings_content.workspace.restore_on_startup.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.restore_on_startup = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.profiles")),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.settings_profiles"),
                    description: i18n::t!("settings.text.any_number_of_settings_profiles_that_are_temporarily_app_1f7ee4da"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("settings_profiles"),
                            pick: |settings_content| {
                                Some(settings_content)
                            },
                            write: |_settings_content, _value| {
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.appearance"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.theme")),
                SettingsPageItem::DynamicItem(DynamicItem {
                    discriminant: SettingItem {
                        files: USER,
                        title: i18n::t!("settings.text.theme_mode"),
                        description: i18n::t!("settings.text.choose_a_static_fixed_theme_or_dynamically_select_themes_7e7ad842"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("theme$"),
                            pick: |settings_content| {
                                Some(&dynamic_variants::<settings::ThemeSelection>()[
                                    settings_content
                                        .theme
                                        .theme
                                        .as_ref()?
                                        .discriminant() as usize])
                            },
                            write: |settings_content, value| {
                                let Some(value) = value else {
                                    settings_content.theme.theme = None;
                                    return;
                                };
                                let settings_value = settings_content.theme.theme.get_or_insert_with(|| {
                                    settings::ThemeSelection::Static(theme::ThemeName(theme::default_theme(theme::SystemAppearance::default().0).into()))
                                });
                                *settings_value = match value {
                                    settings::ThemeSelectionDiscriminants::Static => {
                                        let name = match settings_value {
                                            settings::ThemeSelection::Static(_) => return,
                                            settings::ThemeSelection::Dynamic { mode, light, dark } => {
                                                match mode {
                                                    theme::ThemeAppearanceMode::Light => light.clone(),
                                                    theme::ThemeAppearanceMode::Dark => dark.clone(),
                                                    theme::ThemeAppearanceMode::System => dark.clone(), // no cx, can't determine correct choice
                                                }
                                            },
                                        };
                                        settings::ThemeSelection::Static(name)
                                    },
                                    settings::ThemeSelectionDiscriminants::Dynamic => {
                                        let static_name = match settings_value {
                                            settings::ThemeSelection::Static(theme_name) => theme_name.clone(),
                                            settings::ThemeSelection::Dynamic {..} => return,
                                        };

                                        settings::ThemeSelection::Dynamic {
                                            mode: settings::ThemeAppearanceMode::System,
                                            light: static_name.clone(),
                                            dark: static_name,
                                        }
                                    },
                                };
                            },
                        }),
                        metadata: None,
                    },
                    pick_discriminant: |settings_content| {
                        Some(settings_content.theme.theme.as_ref()?.discriminant() as usize)
                    },
                    fields: dynamic_variants::<settings::ThemeSelection>().iter().map(|variant| {
                        match variant {
                            settings::ThemeSelectionDiscriminants::Static => vec![
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.theme_name"),
                                    description: i18n::t!("settings.text.the_name_of_your_selected_theme"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("theme"),
                                        pick: |settings_content| {
                                            match settings_content.theme.theme.as_ref() {
                                                Some(settings::ThemeSelection::Static(name)) => Some(name),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::ThemeSelection::Static(theme_name)) = settings_content
                                                .theme
                                                .theme.as_mut() { *theme_name = value }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                            settings::ThemeSelectionDiscriminants::Dynamic => vec![
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.mode"),
                                    description: i18n::t!("settings.text.choose_whether_to_use_the_selected_light_or_dark_theme_o_df2c4be7"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("theme.mode"),
                                        pick: |settings_content| {
                                            match settings_content.theme.theme.as_ref() {
                                                Some(settings::ThemeSelection::Dynamic { mode, ..}) => Some(mode),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::ThemeSelection::Dynamic{ mode, ..}) = settings_content
                                                .theme
                                                .theme.as_mut() { *mode = value }
                                        },
                                    }),
                                    metadata: None,
                                },
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.light_theme"),
                                    description: i18n::t!("settings.text.the_theme_to_use_when_mode_is_set_to_light_or_when_mode__75dd10ea"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("theme.light"),
                                        pick: |settings_content| {
                                            match settings_content.theme.theme.as_ref() {
                                                Some(settings::ThemeSelection::Dynamic { light, ..}) => Some(light),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::ThemeSelection::Dynamic{ light, ..}) = settings_content
                                                .theme
                                                .theme.as_mut() { *light = value }
                                        },
                                    }),
                                    metadata: None,
                                },
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.dark_theme"),
                                    description: i18n::t!("settings.text.the_theme_to_use_when_mode_is_set_to_dark_or_when_mode_i_c64ddc88"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("theme.dark"),
                                        pick: |settings_content| {
                                            match settings_content.theme.theme.as_ref() {
                                                Some(settings::ThemeSelection::Dynamic { dark, ..}) => Some(dark),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::ThemeSelection::Dynamic{ dark, ..}) = settings_content
                                                .theme
                                                .theme.as_mut() { *dark = value }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                        }
                    }).collect(),
                }),
                SettingsPageItem::DynamicItem(DynamicItem {
                    discriminant: SettingItem {
                        files: USER,
                        title: i18n::t!("settings.text.icon_theme"),
                        description: i18n::t!("settings.text.the_custom_set_of_icons_don_editor_will_associate_with_f_9e9c69ac"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("icon_theme$"),
                            pick: |settings_content| {
                                Some(&dynamic_variants::<settings::IconThemeSelection>()[
                                    settings_content
                                        .theme
                                        .icon_theme
                                        .as_ref()?
                                        .discriminant() as usize])
                            },
                            write: |settings_content, value| {
                                let Some(value) = value else {
                                    settings_content.theme.icon_theme = None;
                                    return;
                                };
                                let settings_value = settings_content.theme.icon_theme.get_or_insert_with(|| {
                                    settings::IconThemeSelection::Static(settings::IconThemeName(theme::default_icon_theme().name.clone().into()))
                                });
                                *settings_value = match value {
                                    settings::IconThemeSelectionDiscriminants::Static => {
                                        let name = match settings_value {
                                            settings::IconThemeSelection::Static(_) => return,
                                            settings::IconThemeSelection::Dynamic { mode, light, dark } => {
                                                match mode {
                                                    theme::ThemeAppearanceMode::Light => light.clone(),
                                                    theme::ThemeAppearanceMode::Dark => dark.clone(),
                                                    theme::ThemeAppearanceMode::System => dark.clone(), // no cx, can't determine correct choice
                                                }
                                            },
                                        };
                                        settings::IconThemeSelection::Static(name)
                                    },
                                    settings::IconThemeSelectionDiscriminants::Dynamic => {
                                        let static_name = match settings_value {
                                            settings::IconThemeSelection::Static(theme_name) => theme_name.clone(),
                                            settings::IconThemeSelection::Dynamic {..} => return,
                                        };

                                        settings::IconThemeSelection::Dynamic {
                                            mode: settings::ThemeAppearanceMode::System,
                                            light: static_name.clone(),
                                            dark: static_name,
                                        }
                                    },
                                };
                            },
                        }),
                        metadata: None,
                    },
                    pick_discriminant: |settings_content| {
                        Some(settings_content.theme.icon_theme.as_ref()?.discriminant() as usize)
                    },
                    fields: dynamic_variants::<settings::IconThemeSelection>().iter().map(|variant| {
                        match variant {
                            settings::IconThemeSelectionDiscriminants::Static => vec![
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.icon_theme_name"),
                                    description: i18n::t!("settings.text.the_name_of_your_selected_icon_theme"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("icon_theme$string"),
                                        pick: |settings_content| {
                                            match settings_content.theme.icon_theme.as_ref() {
                                                Some(settings::IconThemeSelection::Static(name)) => Some(name),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::IconThemeSelection::Static(theme_name)) = settings_content
                                                .theme
                                                .icon_theme.as_mut() { *theme_name = value }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                            settings::IconThemeSelectionDiscriminants::Dynamic => vec![
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.mode"),
                                    description: i18n::t!("settings.text.choose_whether_to_use_the_selected_light_or_dark_icon_th_aac9aeb9"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("icon_theme"),
                                        pick: |settings_content| {
                                            match settings_content.theme.icon_theme.as_ref() {
                                                Some(settings::IconThemeSelection::Dynamic { mode, ..}) => Some(mode),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::IconThemeSelection::Dynamic{ mode, ..}) = settings_content
                                                .theme
                                                .icon_theme.as_mut() { *mode = value }
                                        },
                                    }),
                                    metadata: None,
                                },
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.light_icon_theme"),
                                    description: i18n::t!("settings.text.the_icon_theme_to_use_when_mode_is_set_to_light_or_when__d8a02fe5"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("icon_theme.light"),
                                        pick: |settings_content| {
                                            match settings_content.theme.icon_theme.as_ref() {
                                                Some(settings::IconThemeSelection::Dynamic { light, ..}) => Some(light),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::IconThemeSelection::Dynamic{ light, ..}) = settings_content
                                                .theme
                                                .icon_theme.as_mut() { *light = value }
                                        },
                                    }),
                                    metadata: None,
                                },
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.dark_icon_theme"),
                                    description: i18n::t!("settings.text.the_icon_theme_to_use_when_mode_is_set_to_dark_or_when_m_9fca8a7e"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("icon_theme.dark"),
                                        pick: |settings_content| {
                                            match settings_content.theme.icon_theme.as_ref() {
                                                Some(settings::IconThemeSelection::Dynamic { dark, ..}) => Some(dark),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::IconThemeSelection::Dynamic{ dark, ..}) = settings_content
                                                .theme
                                                .icon_theme.as_mut() { *dark = value }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                        }
                    }).collect(),
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.buffer_font")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_family"),
                    description: i18n::t!("settings.text.font_family_for_editor_text"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("buffer_font_family"),
                        pick: |settings_content| settings_content.theme.buffer_font_family.as_ref(),
                        write: |settings_content, value|{  settings_content.theme.buffer_font_family = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_size"),
                    description: i18n::t!("settings.text.font_size_for_editor_text"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("buffer_font_size"),
                        pick: |settings_content| settings_content.theme.buffer_font_size.as_ref(),
                        write: |settings_content, value|{  settings_content.theme.buffer_font_size = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_weight"),
                    description: i18n::t!("settings.text.font_weight_for_editor_text_100_900"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("buffer_font_weight"),
                        pick: |settings_content| settings_content.theme.buffer_font_weight.as_ref(),
                        write: |settings_content, value|{  settings_content.theme.buffer_font_weight = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::DynamicItem(DynamicItem {
                    discriminant: SettingItem {
                        files: USER,
                        title: i18n::t!("settings.text.line_height"),
                        description: i18n::t!("settings.text.line_height_for_editor_text"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("buffer_line_height$"),
                            pick: |settings_content| {
                                Some(&dynamic_variants::<settings::BufferLineHeight>()[
                                    settings_content
                                        .theme
                                        .buffer_line_height
                                        .as_ref()?
                                        .discriminant() as usize])
                            },
                            write: |settings_content, value| {
                                let Some(value) = value else {
                                    settings_content.theme.buffer_line_height = None;
                                    return;
                                };
                                let settings_value = settings_content.theme.buffer_line_height.get_or_insert_with(|| {
                                    settings::BufferLineHeight::default()
                                });
                                *settings_value = match value {
                                    settings::BufferLineHeightDiscriminants::Comfortable => {
                                        settings::BufferLineHeight::Comfortable
                                    },
                                    settings::BufferLineHeightDiscriminants::Standard => {
                                        settings::BufferLineHeight::Standard
                                    },
                                    settings::BufferLineHeightDiscriminants::Custom => {
                                        let custom_value = theme::BufferLineHeight::from(*settings_value).value();
                                        settings::BufferLineHeight::Custom(custom_value)
                                    },
                                };
                            },
                        }),
                        metadata: None,
                    },
                    pick_discriminant: |settings_content| {
                        Some(settings_content.theme.buffer_line_height.as_ref()?.discriminant() as usize)
                    },
                    fields: dynamic_variants::<settings::BufferLineHeight>().iter().map(|variant| {
                        match variant {
                            settings::BufferLineHeightDiscriminants::Comfortable => vec![],
                            settings::BufferLineHeightDiscriminants::Standard => vec![],
                            settings::BufferLineHeightDiscriminants::Custom => vec![
                                SettingItem {
                                    files: USER,
                                    title: i18n::t!("settings.text.custom_line_height"),
                                    description: i18n::t!("settings.text.custom_line_height_value_must_be_at_least_1_0"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("buffer_line_height"),
                                        pick: |settings_content| {
                                            match settings_content.theme.buffer_line_height.as_ref() {
                                                Some(settings::BufferLineHeight::Custom(value)) => Some(value),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::BufferLineHeight::Custom(line_height)) = settings_content
                                                .theme
                                                .buffer_line_height.as_mut() { *line_height = f32::max(value, 1.0) }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                        }
                    }).collect(),
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.font_features"),
                    description: i18n::t!("settings.text.the_opentype_features_to_enable_for_rendering_in_text_buffers"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("buffer_font_features"),
                            pick: |settings_content| {
                                settings_content.theme.buffer_font_features.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.theme.buffer_font_features = value;

                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.font_fallbacks"),
                    description: i18n::t!("settings.text.the_font_fallbacks_to_use_for_rendering_in_text_buffers"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("buffer_font_fallbacks"),
                            pick: |settings_content| {
                                settings_content.theme.buffer_font_fallbacks.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.theme.buffer_font_fallbacks = value;

                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.ui_font")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_family"),
                    description: i18n::t!("settings.text.font_family_for_ui_elements"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("ui_font_family"),
                        pick: |settings_content| settings_content.theme.ui_font_family.as_ref(),
                        write: |settings_content, value|{  settings_content.theme.ui_font_family = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_size"),
                    description: i18n::t!("settings.text.font_size_for_ui_elements"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("ui_font_size"),
                        pick: |settings_content| settings_content.theme.ui_font_size.as_ref(),
                        write: |settings_content, value|{  settings_content.theme.ui_font_size = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_weight"),
                    description: i18n::t!("settings.text.font_weight_for_ui_elements_100_900"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("ui_font_weight"),
                        pick: |settings_content| settings_content.theme.ui_font_weight.as_ref(),
                        write: |settings_content, value|{  settings_content.theme.ui_font_weight = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.font_features"),
                    description: i18n::t!("settings.text.the_opentype_features_to_enable_for_rendering_in_ui_elements"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("ui_font_features"),
                            pick: |settings_content| {
                                settings_content.theme.ui_font_features.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.theme.ui_font_features = value;

                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.font_fallbacks"),
                    description: i18n::t!("settings.text.the_font_fallbacks_to_use_for_rendering_in_the_ui"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("ui_font_fallbacks"),
                            pick: |settings_content| {
                                settings_content.theme.ui_font_fallbacks.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.theme.ui_font_fallbacks = value;

                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.text_rendering")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.text_rendering_mode"),
                    description: i18n::t!("settings.text.the_text_rendering_mode_to_use"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("text_rendering_mode"),
                        pick: |settings_content| {
                            settings_content.workspace.text_rendering_mode.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.text_rendering_mode = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.cursor")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.multi_cursor_modifier"),
                    description: i18n::t!("settings.text.modifier_key_for_adding_multiple_cursors"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("multi_cursor_modifier"),
                        pick: |settings_content| {
                            settings_content.editor.multi_cursor_modifier.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.multi_cursor_modifier = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.cursor_blink"),
                    description: i18n::t!("settings.text.whether_the_cursor_blinks_in_the_editor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("cursor_blink"),
                        pick: |settings_content| settings_content.editor.cursor_blink.as_ref(),
                        write: |settings_content, value|{  settings_content.editor.cursor_blink = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.cursor_shape"),
                    description: i18n::t!("settings.text.cursor_shape_for_the_editor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("cursor_shape"),
                        pick: |settings_content| settings_content.editor.cursor_shape.as_ref(),
                        write: |settings_content, value|{  settings_content.editor.cursor_shape = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.hide_mouse"),
                    description: i18n::t!("settings.text.when_to_hide_the_mouse_cursor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("hide_mouse"),
                        pick: |settings_content| settings_content.editor.hide_mouse.as_ref(),
                        write: |settings_content, value|{  settings_content.editor.hide_mouse = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.highlighting")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.unnecessary_code_fade"),
                    description: i18n::t!("settings.text.how_much_to_fade_out_unused_code_0_0_0_9"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("unnecessary_code_fade"),
                        pick: |settings_content| {
                            settings_content.theme.unnecessary_code_fade.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.theme.unnecessary_code_fade = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.current_line_highlight"),
                    description: i18n::t!("settings.text.how_to_highlight_the_current_line"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("current_line_highlight"),
                        pick: |settings_content| {
                            settings_content.editor.current_line_highlight.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.current_line_highlight = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.selection_highlight"),
                    description: i18n::t!("settings.text.highlight_all_occurrences_of_selected_text"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("selection_highlight"),
                        pick: |settings_content| {
                            settings_content.editor.selection_highlight.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.selection_highlight = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.rounded_selection"),
                    description: i18n::t!("settings.text.whether_the_text_selection_should_have_rounded_corners"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("rounded_selection"),
                        pick: |settings_content| settings_content.editor.rounded_selection.as_ref(),
                        write: |settings_content, value|{  settings_content.editor.rounded_selection = value;},
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.minimum_contrast_for_highlights"),
                    description: i18n::t!("settings.text.the_minimum_apca_perceptual_contrast_to_maintain_when_re_c4152458"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("minimum_contrast_for_highlights"),
                        pick: |settings_content| {
                            settings_content
                                .editor
                                .minimum_contrast_for_highlights
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.minimum_contrast_for_highlights = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.guides")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_wrap_guides"),
                    description: i18n::t!("settings.text.show_wrap_guides_vertical_rulers"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("show_wrap_guides"),
                        pick: |settings_content| {
                            settings_content
                                .project
                                .all_languages
                                .defaults
                                .show_wrap_guides
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content

                                .project
                                .all_languages
                                .defaults
                                .show_wrap_guides = value;
                        },
                    }),
                    metadata: None,
                    files: USER | PROJECT,
                }),
                // todo(settings_ui): This needs a custom component
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.wrap_guides"),
                    description: i18n::t!("settings.text.character_counts_at_which_to_show_wrap_guides"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("wrap_guides"),
                            pick: |settings_content| {
                                settings_content
                                    .project
                                    .all_languages
                                    .defaults
                                    .wrap_guides
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.project.all_languages.defaults.wrap_guides = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER | PROJECT,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.decorations")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.client_side_decoration_rounding"),
                    description: i18n::t!("settings.text.defines_window_border_radius_for_platforms_that_use_clie_14c73585"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("client_side_decoration_rounding"),
                        pick: |settings_content| {
                            settings_content.theme.client_side_decoration_rounding.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.theme.client_side_decoration_rounding = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.client_side_decoration_shadow"),
                    description: i18n::t!("settings.text.defines_window_shadow_size_for_platforms_that_use_client_ca206eee"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("client_side_decoration_shadow"),
                        pick: |settings_content| {
                            settings_content.theme.client_side_decoration_shadow.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.theme.client_side_decoration_shadow = value;

                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.keymap"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.keybindings")),
                SettingsPageItem::ActionLink(ActionLink {
                    title: i18n::t!("settings.text.edit_keybindings").into(),
                    description: Some(i18n::t!("ui.text.customize_keybindings_in_the_keymap_editor").into()),
                    button_text: i18n::t!("settings.text.open_keymap").into(),
                    on_click: Arc::new(|settings_window, window, cx| {
                        let Some(original_window) = settings_window.original_window else {
                            return;
                        };
                        original_window
                            .update(cx, |_workspace, original_window, cx| {
                                original_window
                                    .dispatch_action(app_actions::OpenKeymap.boxed_clone(), cx);
                                original_window.activate_window();
                            })
                            .ok();
                        window.remove_window();
                    }),
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.base_keymap")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.base_keymap"),
                    description: i18n::t!("settings.text.the_name_of_a_base_set_of_key_bindings_to_use"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("base_keymap"),
                        pick: |settings_content| settings_content.base_keymap.as_ref(),
                        write: |settings_content, value| {
                            settings_content.base_keymap = value;
                        },
                    }),
                    metadata: Some(Box::new(SettingsFieldMetadata {
                        should_do_titlecase: Some(false),
                        ..Default::default()
                    })),
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.modal_editing")),
                // todo(settings_ui): Vim/Helix Mode should be apart of one type because it's undefined
                // behavior to have them both enabled at the same time
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.vim_mode"),
                    description: i18n::t!("settings.text.enable_vim_mode_and_key_bindings"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("vim_mode"),
                        pick: |settings_content| settings_content.vim_mode.as_ref(),
                        write: |settings_content, value| {
                            settings_content.vim_mode = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.helix_mode"),
                    description: i18n::t!("settings.text.enable_helix_mode_and_key_bindings"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("helix_mode"),
                        pick: |settings_content| settings_content.helix_mode.as_ref(),
                        write: |settings_content, value| {
                            settings_content.helix_mode = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.editor"),
            items: {
                let mut items = vec![
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.auto_save")),
                    SettingsPageItem::DynamicItem(DynamicItem {
                        discriminant: SettingItem {
                            files: USER,
                            title: i18n::t!("settings.text.auto_save_mode"),
                            description: i18n::t!("settings.text.when_to_auto_save_buffer_changes"),
                            aliases: None,
                            field: Box::new(SettingField {
                                json_path: Some("autosave$"),
                                pick: |settings_content| {
                                    Some(&dynamic_variants::<settings::AutosaveSetting>()[
                                        settings_content
                                            .workspace
                                            .autosave
                                            .as_ref()?
                                            .discriminant() as usize])
                                },
                                write: |settings_content, value| {
                                    let Some(value) = value else {
                                        settings_content.workspace.autosave = None;
                                        return;
                                    };
                                    let settings_value = settings_content.workspace.autosave.get_or_insert({
                                        settings::AutosaveSetting::Off
                                    });
                                    *settings_value = match value {
                                        settings::AutosaveSettingDiscriminants::Off => {
                                            settings::AutosaveSetting::Off
                                        },
                                        settings::AutosaveSettingDiscriminants::AfterDelay => {
                                            let milliseconds = match settings_value {
                                                settings::AutosaveSetting::AfterDelay { milliseconds } => *milliseconds,
                                                _ => settings::DelayMs(1000),
                                            };
                                            settings::AutosaveSetting::AfterDelay { milliseconds }
                                        },
                                        settings::AutosaveSettingDiscriminants::OnFocusChange => {
                                            settings::AutosaveSetting::OnFocusChange
                                        },
                                        settings::AutosaveSettingDiscriminants::OnWindowChange => {
                                            settings::AutosaveSetting::OnWindowChange
                                        },
                                    };
                                },
                            }),
                            metadata: None,
                        },
                        pick_discriminant: |settings_content| {
                            Some(settings_content.workspace.autosave.as_ref()?.discriminant() as usize)
                        },
                        fields: dynamic_variants::<settings::AutosaveSetting>().iter().map(|variant| {
                            match variant {
                                settings::AutosaveSettingDiscriminants::Off => vec![],
                                settings::AutosaveSettingDiscriminants::AfterDelay => vec![
                                    SettingItem {
                                        files: USER,
                                        title: i18n::t!("settings.text.delay_milliseconds"),
                                        description: i18n::t!("settings.text.save_after_inactivity_period_in_milliseconds"),
                                        aliases: None,
                                        field: Box::new(SettingField {
                                            json_path: Some("autosave.after_delay.milliseconds"),
                                            pick: |settings_content| {
                                                match settings_content.workspace.autosave.as_ref() {
                                                    Some(settings::AutosaveSetting::AfterDelay { milliseconds }) => Some(milliseconds),
                                                    _ => None
                                                }
                                            },
                                            write: |settings_content, value| {
                                                let Some(value) = value else {
                                                    settings_content.workspace.autosave = None;
                                                    return;
                                                };
                                                if let Some(settings::AutosaveSetting::AfterDelay { milliseconds }) = settings_content
                                                    .workspace
                                                    .autosave.as_mut() { *milliseconds = value }
                                            },
                                        }),
                                        metadata: None,
                                    }
                                ],
                                settings::AutosaveSettingDiscriminants::OnFocusChange => vec![],
                                settings::AutosaveSettingDiscriminants::OnWindowChange => vec![],
                            }
                        }).collect(),
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.which_key")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.which_key"),
                        description: i18n::t!("settings.text.display_menu_with_matching_key_bindings_while_a_multi_st_e282054a"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("which_key.enabled"),
                            pick: |settings_content| {
                                settings_content
                                    .which_key
                                    .as_ref()
                                    .and_then(|settings| settings.enabled.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .which_key
                                    .get_or_insert_default()
                                    .enabled = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.menu_delay"),
                        description: i18n::t!("settings.text.delay_in_milliseconds_before_the_which_key_menu_appears"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("which_key.delay_ms"),
                            pick: |settings_content| {
                                settings_content
                                    .which_key
                                    .as_ref()
                                    .and_then(|settings| settings.delay_ms.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .which_key
                                    .get_or_insert_default()
                                    .delay_ms = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.multibuffer")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.double_click_in_multibuffer"),
                        description: i18n::t!("settings.text.what_to_do_when_multibuffer_is_double_clicked_in_some_of_b2cd4d1f"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("double_click_in_multibuffer"),
                            pick: |settings_content| {
                                settings_content.editor.double_click_in_multibuffer.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.double_click_in_multibuffer = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.expand_excerpt_lines"),
                        description: i18n::t!("settings.text.how_many_lines_to_expand_the_multibuffer_excerpts_by_default"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("expand_excerpt_lines"),
                            pick: |settings_content| {
                                settings_content.editor.expand_excerpt_lines.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.expand_excerpt_lines = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.excerpt_context_lines"),
                        description: i18n::t!("settings.text.how_many_lines_of_context_to_provide_in_multibuffer_exce_39e6fc0e"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("excerpt_context_lines"),
                            pick: |settings_content| {
                                settings_content.editor.excerpt_context_lines.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.excerpt_context_lines = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.expand_outlines_with_depth"),
                        description: i18n::t!("settings.text.default_depth_to_expand_outline_items_in_the_current_file"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("outline_panel.expand_outlines_with_depth"),
                            pick: |settings_content| {
                                settings_content
                                    .outline_panel
                                    .as_ref()
                                    .and_then(|outline_panel| {
                                        outline_panel.expand_outlines_with_depth.as_ref()
                                    })
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .outline_panel
                                    .get_or_insert_default()
                                    .expand_outlines_with_depth = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.scrolling")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.scroll_beyond_last_line"),
                        description: i18n::t!("settings.text.whether_the_editor_will_scroll_beyond_the_last_line"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scroll_beyond_last_line"),
                            pick: |settings_content| {
                                settings_content.editor.scroll_beyond_last_line.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.scroll_beyond_last_line = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.vertical_scroll_margin"),
                        description: i18n::t!("settings.text.the_number_of_lines_to_keep_above_below_the_cursor_when__ab1e843f"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("vertical_scroll_margin"),
                            pick: |settings_content| {
                                settings_content.editor.vertical_scroll_margin.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.vertical_scroll_margin = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.horizontal_scroll_margin"),
                        description: i18n::t!("settings.text.the_number_of_characters_to_keep_on_either_side_when_scr_23bab5b3"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("horizontal_scroll_margin"),
                            pick: |settings_content| {
                                settings_content.editor.horizontal_scroll_margin.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.horizontal_scroll_margin = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.scroll_sensitivity"),
                        description: i18n::t!("settings.text.scroll_sensitivity_multiplier_for_both_horizontal_and_ve_1fa0fb5a"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scroll_sensitivity"),
                            pick: |settings_content| {
                                settings_content.editor.scroll_sensitivity.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.scroll_sensitivity = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.fast_scroll_sensitivity"),
                        description: i18n::t!("settings.text.fast_scroll_sensitivity_multiplier_for_both_horizontal_a_9620e46f"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("fast_scroll_sensitivity"),
                            pick: |settings_content| {
                                settings_content.editor.fast_scroll_sensitivity.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.fast_scroll_sensitivity = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.autoscroll_on_clicks"),
                        description: i18n::t!("settings.text.whether_to_scroll_when_clicking_near_the_edge_of_the_vis_d7de6c3e"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("autoscroll_on_clicks"),
                            pick: |settings_content| {
                                settings_content.editor.autoscroll_on_clicks.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.autoscroll_on_clicks = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.sticky_scroll"),
                        description: i18n::t!("settings.text.whether_to_stick_scopes_to_the_top_of_the_editor"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("sticky_scroll.enabled"),
                            pick: |settings_content| {
                                settings_content.editor.sticky_scroll.as_ref().and_then(|sticky_scroll| sticky_scroll.enabled.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content.editor.sticky_scroll.get_or_insert_default().enabled = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.smooth_scroll"),
                        description: i18n::t!("settings.text.whether_to_animate_scrolling_with_a_smooth_easing_effect"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("smooth_scroll.enabled"),
                            pick: |settings_content| {
                                settings_content.editor.smooth_scroll.as_ref().and_then(|smooth_scroll| smooth_scroll.enabled.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content.editor.smooth_scroll.get_or_insert_default().enabled = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.scroll_animation_duration"),
                        description: i18n::t!("settings.text.duration_of_scroll_animation"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("smooth_scroll.duration"),
                            pick: |settings_content| {
                                settings_content.editor.smooth_scroll.as_ref().and_then(|smooth_scroll| smooth_scroll.duration.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content.editor.smooth_scroll.get_or_insert_default().duration = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),

                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.signature_help")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.auto_signature_help"),
                        description: i18n::t!("settings.text.automatically_show_a_signature_help_pop_up"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("auto_signature_help"),
                            pick: |settings_content| {
                                settings_content.editor.auto_signature_help.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.auto_signature_help = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show_signature_help_after_edits"),
                        description: i18n::t!("settings.text.show_the_signature_help_pop_up_after_completions_or_brac_eb674538"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("show_signature_help_after_edits"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .show_signature_help_after_edits
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.show_signature_help_after_edits = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.snippet_sort_order"),
                        description: i18n::t!("settings.text.determines_how_snippets_are_sorted_relative_to_other_com_a992bfbb"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("snippet_sort_order"),
                            pick: |settings_content| {
                                settings_content.editor.snippet_sort_order.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.snippet_sort_order = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.hover_popover")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.enabled"),
                        description: i18n::t!("settings.text.show_the_informational_hover_box_when_moving_the_mouse_o_8182228c"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("hover_popover_enabled"),
                            pick: |settings_content| {
                                settings_content.editor.hover_popover_enabled.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.hover_popover_enabled = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    // todo(settings ui): add units to this number input
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.delay"),
                        description: i18n::t!("settings.text.time_to_wait_in_milliseconds_before_showing_the_informat_d1629466"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("hover_popover_enabled"),
                            pick: |settings_content| {
                                settings_content.editor.hover_popover_delay.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.hover_popover_delay = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.drag_and_drop_selection")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.enabled"),
                        description: i18n::t!("settings.text.enable_drag_and_drop_selection"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("drag_and_drop_selection.enabled"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .drag_and_drop_selection
                                    .as_ref()
                                    .and_then(|drag_and_drop| drag_and_drop.enabled.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .drag_and_drop_selection
                                    .get_or_insert_default()
                                    .enabled = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.delay"),
                        description: i18n::t!("settings.text.delay_in_milliseconds_before_drag_and_drop_selection_starts"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("drag_and_drop_selection.delay"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .drag_and_drop_selection
                                    .as_ref()
                                    .and_then(|drag_and_drop| drag_and_drop.delay.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .drag_and_drop_selection
                                    .get_or_insert_default()
                                    .delay = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.gutter")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show_line_numbers"),
                        description: i18n::t!("settings.text.show_line_numbers_in_the_gutter"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("gutter.line_numbers"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .gutter
                                    .as_ref()
                                    .and_then(|gutter| gutter.line_numbers.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .gutter
                                    .get_or_insert_default()
                                    .line_numbers = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.relative_line_numbers"),
                        description: i18n::t!("settings.text.controls_line_number_display_in_the_editor_s_gutter_disa_633ea61a"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("relative_line_numbers"),
                            pick: |settings_content| {
                                settings_content.editor.relative_line_numbers.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.relative_line_numbers = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.line_number_scale"),
                        description: i18n::t!("settings.text.relative_size_of_line_numbers_displayed_in_the_editor"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("editor.line_number_scale"),
                            pick: |settings_content| settings_content.editor.line_number_scale.as_ref(),
                            write: |settings_content, value| {
                                settings_content.editor.line_number_scale = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show_runnables"),
                        description: i18n::t!("settings.text.show_runnable_buttons_in_the_gutter"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("gutter.runnables"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .gutter
                                    .as_ref()
                                    .and_then(|gutter| gutter.runnables.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .gutter
                                    .get_or_insert_default()
                                    .runnables = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show_breakpoints"),
                        description: i18n::t!("settings.text.show_breakpoints_in_the_gutter"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("gutter.breakpoints"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .gutter
                                    .as_ref()
                                    .and_then(|gutter| gutter.breakpoints.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .gutter
                                    .get_or_insert_default()
                                    .breakpoints = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show_folds"),
                        description: i18n::t!("settings.text.show_code_folding_controls_in_the_gutter"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("gutter.folds"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .gutter
                                    .as_ref()
                                    .and_then(|gutter| gutter.folds.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content.editor.gutter.get_or_insert_default().folds =
                                    value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.min_line_number_digits"),
                        description: i18n::t!("settings.text.minimum_number_of_characters_to_reserve_space_for_in_the_gutter"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("gutter.min_line_number_digits"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .gutter
                                    .as_ref()
                                    .and_then(|gutter| gutter.min_line_number_digits.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .gutter
                                    .get_or_insert_default()
                                    .min_line_number_digits = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.inline_code_actions"),
                        description: i18n::t!("settings.text.show_code_action_button_at_start_of_buffer_line"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("inline_code_actions"),
                            pick: |settings_content| {
                                settings_content.editor.inline_code_actions.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.inline_code_actions = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.scrollbar")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show"),
                        description: i18n::t!("settings.text.when_to_show_the_scrollbar_in_the_editor"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar"),
                            pick: |settings_content| {
                                settings_content.editor.scrollbar.as_ref()?.show.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .show = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.cursors"),
                        description: i18n::t!("settings.text.show_cursor_positions_in_the_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.cursors"),
                            pick: |settings_content| {
                                settings_content.editor.scrollbar.as_ref()?.cursors.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .cursors = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.git_diff"),
                        description: i18n::t!("settings.text.show_git_diff_indicators_in_the_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.git_diff"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .git_diff
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .git_diff = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.search_results"),
                        description: i18n::t!("settings.text.show_buffer_search_result_indicators_in_the_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.search_results"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .search_results
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .search_results = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.selected_text"),
                        description: i18n::t!("settings.text.show_selected_text_occurrences_in_the_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.selected_text"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .selected_text
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .selected_text = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.selected_symbol"),
                        description: i18n::t!("settings.text.show_selected_symbol_occurrences_in_the_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.selected_symbol"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .selected_symbol
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .selected_symbol = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.diagnostics"),
                        description: i18n::t!("settings.text.which_diagnostic_indicators_to_show_in_the_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.diagnostics"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .diagnostics
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .diagnostics = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.horizontal_scrollbar"),
                        description: i18n::t!("settings.text.when_false_forcefully_disables_the_horizontal_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.axes.horizontal"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .axes
                                    .as_ref()?
                                    .horizontal
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .axes
                                    .get_or_insert_default()
                                    .horizontal = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.vertical_scrollbar"),
                        description: i18n::t!("settings.text.when_false_forcefully_disables_the_vertical_scrollbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("scrollbar.axes.vertical"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .as_ref()?
                                    .axes
                                    .as_ref()?
                                    .vertical
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .scrollbar
                                    .get_or_insert_default()
                                    .axes
                                    .get_or_insert_default()
                                    .vertical = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.minimap")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.show"),
                        description: i18n::t!("settings.text.when_to_show_the_minimap_in_the_editor"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("minimap.show"),
                            pick: |settings_content| {
                                settings_content.editor.minimap.as_ref()?.show.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.editor.minimap.get_or_insert_default().show =
                                    value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.display_in"),
                        description: i18n::t!("settings.text.where_to_show_the_minimap_in_the_editor"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("minimap.display_in"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .minimap
                                    .as_ref()?
                                    .display_in
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .minimap
                                    .get_or_insert_default()
                                    .display_in = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.thumb"),
                        description: i18n::t!("settings.text.when_to_show_the_minimap_thumb"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("minimap.thumb"),
                            pick: |settings_content| {
                                settings_content.editor.minimap.as_ref()?.thumb.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .minimap
                                    .get_or_insert_default()
                                    .thumb = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.thumb_border"),
                        description: i18n::t!("settings.text.border_style_for_the_minimap_s_scrollbar_thumb"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("minimap.thumb_border"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .minimap
                                    .as_ref()?
                                    .thumb_border
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .minimap
                                    .get_or_insert_default()
                                    .thumb_border = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.current_line_highlight"),
                        description: i18n::t!("settings.text.how_to_highlight_the_current_line_in_the_minimap"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("minimap.current_line_highlight"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .minimap
                                    .as_ref()
                                    .and_then(|minimap| minimap.current_line_highlight.as_ref())
                                    .or(settings_content.editor.current_line_highlight.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .minimap
                                    .get_or_insert_default()
                                    .current_line_highlight = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.max_width_columns"),
                        description: i18n::t!("settings.text.maximum_number_of_columns_to_display_in_the_minimap"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("minimap.max_width_columns"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .minimap
                                    .as_ref()?
                                    .max_width_columns
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .minimap
                                    .get_or_insert_default()
                                    .max_width_columns = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.toolbar")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.breadcrumbs"),
                        description: i18n::t!("settings.text.show_breadcrumbs"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("toolbar.breadcrumbs"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .as_ref()?
                                    .breadcrumbs
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .get_or_insert_default()
                                    .breadcrumbs = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.quick_actions"),
                        description: i18n::t!("settings.text.show_quick_action_buttons_e_g_search_selection_editor_controls_etc"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("toolbar.quick_actions"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .as_ref()?
                                    .quick_actions
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .get_or_insert_default()
                                    .quick_actions = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.selections_menu"),
                        description: i18n::t!("settings.text.show_the_selections_menu_in_the_editor_toolbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("toolbar.selections_menu"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .as_ref()?
                                    .selections_menu
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .get_or_insert_default()
                                    .selections_menu = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.code_actions"),
                        description: i18n::t!("settings.text.show_code_action_buttons_in_the_editor_toolbar"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("toolbar.code_actions"),
                            pick: |settings_content| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .as_ref()?
                                    .code_actions
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .editor
                                    .toolbar
                                    .get_or_insert_default()
                                    .code_actions = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                ];
                items.extend(language_settings_data());
                items
            },
        },
        SettingsPage {
            title: i18n::t!("settings.text.languages_tools"),
            items: {
                let mut items = vec![];
                items.extend(non_editor_language_settings_data());
                items.extend([
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.file_types")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.file_type_associations"),
                        description: i18n::t!("settings.text.a_mapping_from_languages_to_files_and_file_extensions_th_34d6f0be"),
                        aliases: None,
                        field: Box::new(
                            SettingField {
                                json_path: Some("file_type_associations"),
                                pick: |settings_content| {
                                    settings_content.project.all_languages.file_types.as_ref()
                                },
                                write: |settings_content, value| {
                                    settings_content.project.all_languages.file_types = value;

                                },
                            }
                            .unimplemented(),
                        ),
                        metadata: None,
                        files: USER | PROJECT,
                    }),
                ]);

                items.extend([
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.diagnostics")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.max_severity"),
                        description: i18n::t!("settings.text.which_level_to_use_to_filter_out_diagnostics_displayed_in_the_editor"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics_max_severity"),
                            pick: |settings_content| settings_content.editor.diagnostics_max_severity.as_ref(),
                            write: |settings_content, value| {
                                settings_content.editor.diagnostics_max_severity = value;

                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.include_warnings"),
                        description: i18n::t!("settings.text.whether_to_show_warnings_or_not_by_default"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.include_warnings"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.include_warnings.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .include_warnings
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.inline_diagnostics")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.enabled"),
                        description: i18n::t!("settings.text.whether_to_show_diagnostics_inline_or_not"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.inline.enabled"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.inline.as_ref()?.enabled.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .inline
                                    .get_or_insert_default()
                                    .enabled
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.update_debounce"),
                        description: i18n::t!("settings.text.the_delay_in_milliseconds_to_show_inline_diagnostics_aft_61a5d105"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.inline.update_debounce_ms"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.inline.as_ref()?.update_debounce_ms.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .inline
                                    .get_or_insert_default()
                                    .update_debounce_ms
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.padding"),
                        description: i18n::t!("settings.text.the_amount_of_padding_between_the_end_of_the_source_line_880419bd"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.inline.padding"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.inline.as_ref()?.padding.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .inline
                                    .get_or_insert_default()
                                    .padding
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.minimum_column"),
                        description: i18n::t!("settings.text.the_minimum_column_at_which_to_display_inline_diagnostics"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.inline.min_column"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.inline.as_ref()?.min_column.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .inline
                                    .get_or_insert_default()
                                    .min_column
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.lsp_pull_diagnostics")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.enabled"),
                        description: i18n::t!("settings.text.whether_to_pull_for_language_server_powered_diagnostics_or_not"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.lsp_pull_diagnostics.enabled"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.lsp_pull_diagnostics.as_ref()?.enabled.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .lsp_pull_diagnostics
                                    .get_or_insert_default()
                                    .enabled
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    // todo(settings_ui): Needs unit
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.debounce"),
                        description: i18n::t!("settings.text.minimum_time_to_wait_before_pulling_diagnostics_from_the_0d82be4b"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("diagnostics.lsp_pull_diagnostics.debounce_ms"),
                            pick: |settings_content| {
                                settings_content.diagnostics.as_ref()?.lsp_pull_diagnostics.as_ref()?.debounce_ms.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content

                                    .diagnostics
                                    .get_or_insert_default()
                                    .lsp_pull_diagnostics
                                    .get_or_insert_default()
                                    .debounce_ms
                                    = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                    SettingsPageItem::SectionHeader(i18n::t!("settings.text.lsp_highlights")),
                    SettingsPageItem::SettingItem(SettingItem {
                        title: i18n::t!("settings.text.debounce"),
                        description: i18n::t!("settings.text.the_debounce_delay_before_querying_highlights_from_the_language"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("lsp_highlight_debounce"),
                            pick: |settings_content| settings_content.editor.lsp_highlight_debounce.as_ref(),
                            write: |settings_content, value| {
                                settings_content.editor.lsp_highlight_debounce = value;
                            },
                        }),
                        metadata: None,
                        files: USER,
                    }),
                ]);

                // todo(settings_ui): Refresh on extension (un)/installed
                // Note that `crates/json_schema_store` solves the same problem, there is probably a way to unify the two
                items.push(SettingsPageItem::SectionHeader(languages_section_header()));
                items.extend(all_language_names(cx).into_iter().map(|language_name| {
                    let link = format!("languages.{language_name}");
                    SettingsPageItem::SubPageLink(SubPageLink {
                        title: language_name,
                        description: None,
                        json_path: Some(link.leak()),
                        in_json: true,
                        files: USER | PROJECT,
                        render: Arc::new(|this, window, cx| {
                            this.render_sub_page_items(
                                language_settings_data()
                                    .iter()
                                    .chain(non_editor_language_settings_data().iter())
                                    .enumerate(),
                                None,
                                window,
                                cx,
                            )
                            .into_any_element()
                        }),
                    })
                }));
                items
            },
        },
        SettingsPage {
            title: i18n::t!("settings.text.search_files"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.search")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.whole_word"),
                    description: i18n::t!("settings.text.search_for_whole_words_by_default"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("search.whole_word"),
                        pick: |settings_content| {
                            settings_content.editor.search.as_ref()?.whole_word.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .editor
                                .search
                                .get_or_insert_default()
                                .whole_word = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.case_sensitive"),
                    description: i18n::t!("settings.text.search_case_sensitively_by_default"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("search.case_sensitive"),
                        pick: |settings_content| {
                            settings_content
                                .editor
                                .search
                                .as_ref()?
                                .case_sensitive
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .editor
                                .search
                                .get_or_insert_default()
                                .case_sensitive = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.use_smartcase_search"),
                    description: i18n::t!("settings.text.whether_to_automatically_enable_case_sensitive_search_ba_c60eb147"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("use_smartcase_search"),
                        pick: |settings_content| {
                            settings_content.editor.use_smartcase_search.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.use_smartcase_search = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.include_ignored"),
                    description: i18n::t!("settings.text.include_ignored_files_in_search_results_by_default"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("search.include_ignored"),
                        pick: |settings_content| {
                            settings_content
                                .editor
                                .search
                                .as_ref()?
                                .include_ignored
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .editor
                                .search
                                .get_or_insert_default()
                                .include_ignored = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.regex"),
                    description: i18n::t!("settings.text.use_regex_search_by_default"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("search.regex"),
                        pick: |settings_content| {
                            settings_content.editor.search.as_ref()?.regex.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.search.get_or_insert_default().regex = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.search_wrap"),
                    description: i18n::t!("settings.text.whether_the_editor_search_results_will_loop"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("search_wrap"),
                        pick: |settings_content| settings_content.editor.search_wrap.as_ref(),
                        write: |settings_content, value| {
                            settings_content.editor.search_wrap = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.center_on_match"),
                    description: i18n::t!("settings.text.whether_to_center_the_current_match_in_the_editor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("editor.search.center_on_match"),
                        pick: |settings_content| {
                            settings_content
                                .editor
                                .search
                                .as_ref()
                                .and_then(|search| search.center_on_match.as_ref())
                        },
                        write: |settings_content, value| {
                            settings_content
                                .editor
                                .search
                                .get_or_insert_default()
                                .center_on_match = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.seed_search_query_from_cursor"),
                    description: i18n::t!("settings.text.when_to_populate_a_new_search_s_query_based_on_the_text__223ea842"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("seed_search_query_from_cursor"),
                        pick: |settings_content| {
                            settings_content
                                .editor
                                .seed_search_query_from_cursor
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.editor.seed_search_query_from_cursor = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.file_finder")),
                // todo: null by default
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.include_ignored_in_search"),
                    description: i18n::t!("settings.text.use_gitignored_files_when_searching"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("file_finder.include_ignored"),
                            pick: |settings_content| {
                                settings_content
                                    .file_finder
                                    .as_ref()?
                                    .include_ignored
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .file_finder
                                    .get_or_insert_default()
                                    .include_ignored = value;
                            },
                        }
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.file_icons"),
                    description: i18n::t!("settings.text.show_file_icons_in_the_file_finder"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("file_finder.file_icons"),
                        pick: |settings_content| {
                            settings_content.file_finder.as_ref()?.file_icons.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .file_finder
                                .get_or_insert_default()
                                .file_icons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.modal_max_width"),
                    description: i18n::t!("settings.text.determines_how_much_space_the_file_finder_can_take_up_in_cc9ecae1"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("file_finder.modal_max_width"),
                        pick: |settings_content| {
                            settings_content
                                .file_finder
                                .as_ref()?
                                .modal_max_width
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .file_finder
                                .get_or_insert_default()
                                .modal_max_width = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.skip_focus_for_active_in_search"),
                    description: i18n::t!("settings.text.whether_the_file_finder_should_skip_focus_for_the_active_363e6868"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("file_finder.skip_focus_for_active_in_search"),
                        pick: |settings_content| {
                            settings_content
                                .file_finder
                                .as_ref()?
                                .skip_focus_for_active_in_search
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .file_finder
                                .get_or_insert_default()
                                .skip_focus_for_active_in_search = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_status"),
                    description: i18n::t!("settings.text.show_the_git_status_in_the_file_finder"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("file_finder.git_status"),
                        pick: |settings_content| {
                            settings_content.file_finder.as_ref()?.git_status.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .file_finder
                                .get_or_insert_default()
                                .git_status = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.file_scan")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.file_scan_exclusions"),
                    description: i18n::t!("settings.text.files_or_globs_of_files_that_will_be_excluded_by_don_edi_bb6d8cff"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("file_scan_exclusions"),
                            pick: |settings_content| {
                                settings_content
                                    .project
                                    .worktree
                                    .file_scan_exclusions
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.project.worktree.file_scan_exclusions = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.file_scan_inclusions"),
                    description: i18n::t!("settings.text.files_or_globs_of_files_that_will_be_included_by_don_edi_fe986050"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("file_scan_inclusions"),
                            pick: |settings_content| {
                                settings_content
                                    .project
                                    .worktree
                                    .file_scan_inclusions
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.project.worktree.file_scan_inclusions = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.restore_file_state"),
                    description: i18n::t!("settings.text.restore_previous_file_state_when_reopening"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("restore_on_file_reopen"),
                        pick: |settings_content| {
                            settings_content.workspace.restore_on_file_reopen.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.restore_on_file_reopen = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.close_on_file_delete"),
                    description: i18n::t!("settings.text.automatically_close_files_that_have_been_deleted"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("close_on_file_delete"),
                        pick: |settings_content| {
                            settings_content.workspace.close_on_file_delete.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.close_on_file_delete = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.window_layout"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.status_bar")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show"),
                    description: i18n::t!("settings.text.display_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.show"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .show
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .show = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.position"),
                    description: i18n::t!("settings.text.show_the_status_bar_at_the_bottom_or_top_of_the_window"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.position"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .position
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .position = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.icon_size"),
                    description: i18n::t!("settings.text.size_of_the_icons_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.icon_size"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .icon_size
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .icon_size = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.active_file"),
                    description: i18n::t!("settings.text.show_the_active_file_name_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.active_file"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .active_file
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .active_file = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.project_panel_button"),
                    description: i18n::t!("settings.text.show_the_project_panel_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.button"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.active_language_button"),
                    description: i18n::t!("settings.text.show_the_active_language_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.active_language_button"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .active_language_button
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .active_language_button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.active_encoding_button"),
                    description: i18n::t!("settings.text.control_when_to_show_the_active_encoding_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.active_encoding_button"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .active_encoding_button
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .active_encoding_button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.cursor_position_button"),
                    description: i18n::t!("settings.text.show_the_cursor_position_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("status_bar.cursor_position_button"),
                        pick: |settings_content| {
                            settings_content
                                .status_bar
                                .as_ref()?
                                .cursor_position_button
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .status_bar
                                .get_or_insert_default()
                                .cursor_position_button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.terminal_button"),
                    description: i18n::t!("settings.text.show_the_terminal_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.button"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.terminal.get_or_insert_default().button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.diagnostics_button"),
                    description: i18n::t!("settings.text.show_the_project_diagnostics_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("diagnostics.button"),
                        pick: |settings_content| {
                            settings_content.diagnostics.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.diagnostics.get_or_insert_default().button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.project_search_button"),
                    description: i18n::t!("settings.text.show_the_project_search_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("search.button"),
                        pick: |settings_content| {
                            settings_content.editor.search.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .editor
                                .search
                                .get_or_insert_default()
                                .button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.debugger_button"),
                    description: i18n::t!("settings.text.show_the_debugger_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.button"),
                        pick: |settings_content| {
                            settings_content.debugger.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.debugger.get_or_insert_default().button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.title_bar")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_title_bar"),
                    description: i18n::t!("settings.text.show_or_hide_the_entire_title_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("title_bar.show"),
                        pick: |settings_content| {
                            settings_content
                                .title_bar
                                .as_ref()?
                                .show
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .title_bar
                                .get_or_insert_default()
                                .show = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_branch_icon"),
                    description: i18n::t!("settings.text.show_the_branch_icon_beside_branch_switcher_in_the_titlebar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("title_bar.show_branch_icon"),
                        pick: |settings_content| {
                            settings_content
                                .title_bar
                                .as_ref()?
                                .show_branch_icon
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .title_bar
                                .get_or_insert_default()
                                .show_branch_icon = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_branch_name"),
                    description: i18n::t!("settings.text.show_the_branch_name_button_in_the_titlebar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("title_bar.show_branch_name"),
                        pick: |settings_content| {
                            settings_content
                                .title_bar
                                .as_ref()?
                                .show_branch_name
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .title_bar
                                .get_or_insert_default()
                                .show_branch_name = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_project_items"),
                    description: i18n::t!("settings.text.show_the_project_host_and_name_in_the_titlebar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("title_bar.show_project_items"),
                        pick: |settings_content| {
                            settings_content
                                .title_bar
                                .as_ref()?
                                .show_project_items
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .title_bar
                                .get_or_insert_default()
                                .show_project_items = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_onboarding_banner"),
                    description: i18n::t!("settings.text.show_banners_announcing_new_features_in_the_titlebar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("title_bar.show_onboarding_banner"),
                        pick: |settings_content| {
                            settings_content
                                .title_bar
                                .as_ref()?
                                .show_onboarding_banner
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .title_bar
                                .get_or_insert_default()
                                .show_onboarding_banner = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_menus"),
                    description: i18n::t!("settings.text.show_the_menus_in_the_titlebar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("title_bar.show_menus"),
                        pick: |settings_content| {
                            settings_content.title_bar.as_ref()?.show_menus.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .title_bar
                                .get_or_insert_default()
                                .show_menus = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.tab_bar")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_tab_bar"),
                    description: i18n::t!("settings.text.show_the_tab_bar_in_the_editor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tab_bar.show"),
                        pick: |settings_content| settings_content.tab_bar.as_ref()?.show.as_ref(),
                        write: |settings_content, value| {
                            settings_content.tab_bar.get_or_insert_default().show = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_git_status_in_tabs"),
                    description: i18n::t!("settings.text.show_the_git_file_status_on_a_tab_item"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tabs.git_status"),
                        pick: |settings_content| {
                            settings_content.tabs.as_ref()?.git_status.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.tabs.get_or_insert_default().git_status = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_file_icons_in_tabs"),
                    description: i18n::t!("settings.text.show_the_file_icon_for_a_tab"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tabs.file_icons"),
                        pick: |settings_content| {
                            settings_content.tabs.as_ref()?.file_icons.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.tabs.get_or_insert_default().file_icons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.tab_close_position"),
                    description: i18n::t!("settings.text.position_of_the_close_button_in_a_tab"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tabs.close_position"),
                        pick: |settings_content| {
                            settings_content.tabs.as_ref()?.close_position.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.tabs.get_or_insert_default().close_position = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.maximum_tabs"),
                    description: i18n::t!("settings.text.maximum_open_tabs_in_a_pane_will_not_close_an_unsaved_tab"),
                    aliases: None,
                    // todo(settings_ui): The default for this value is null and it's use in code
                    // is complex, so I'm going to come back to this later
                    field: Box::new(
                        SettingField {
                            json_path: Some("max_tabs"),
                            pick: |settings_content| settings_content.workspace.max_tabs.as_ref(),
                            write: |settings_content, value| {
                                settings_content.workspace.max_tabs = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.pinned_tabs_layout"),
                    description: i18n::t!("settings.text.show_pinned_tabs_in_a_separate_row_above_unpinned_tabs"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tab_bar.show_pinned_tabs_in_separate_row"),
                        pick: |settings_content| {
                            settings_content
                                .tab_bar
                                .as_ref()?
                                .show_pinned_tabs_in_separate_row
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .tab_bar
                                .get_or_insert_default()
                                .show_pinned_tabs_in_separate_row = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.wrap_tabs"),
                    description: i18n::t!("settings.text.wrap_tabs_onto_additional_rows_when_the_tab_bar_overflows"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tab_bar.wrap_tabs"),
                        pick: |settings_content| {
                            settings_content.tab_bar.as_ref()?.wrap_tabs.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.tab_bar.get_or_insert_default().wrap_tabs = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_navigation_history_buttons"),
                    description: i18n::t!("settings.text.show_the_navigation_history_buttons_in_the_tab_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tab_bar.show_nav_history_buttons"),
                        pick: |settings_content| {
                            settings_content
                                .tab_bar
                                .as_ref()?
                                .show_nav_history_buttons
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .tab_bar
                                .get_or_insert_default()
                                .show_nav_history_buttons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_tab_bar_buttons"),
                    description: i18n::t!("settings.text.show_the_tab_bar_buttons_new_split_pane_zoom"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tab_bar.show_tab_bar_buttons"),
                        pick: |settings_content| {
                            settings_content
                                .tab_bar
                                .as_ref()?
                                .show_tab_bar_buttons
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .tab_bar
                                .get_or_insert_default()
                                .show_tab_bar_buttons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.tab_settings")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.activate_on_close"),
                    description: i18n::t!("settings.text.what_to_do_after_closing_the_current_tab"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tabs.activate_on_close"),
                        pick: |settings_content| {
                            settings_content.tabs.as_ref()?.activate_on_close.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .tabs
                                .get_or_insert_default()
                                .activate_on_close = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.tab_show_diagnostics"),
                    description: i18n::t!("settings.text.which_files_containing_diagnostic_errors_warnings_to_mar_67f03318"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tabs.show_diagnostics"),
                        pick: |settings_content| {
                            settings_content.tabs.as_ref()?.show_diagnostics.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .tabs
                                .get_or_insert_default()
                                .show_diagnostics = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_close_button"),
                    description: i18n::t!("settings.text.controls_the_appearance_behavior_of_the_tab_s_close_button"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("tabs.show_close_button"),
                        pick: |settings_content| {
                            settings_content.tabs.as_ref()?.show_close_button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .tabs
                                .get_or_insert_default()
                                .show_close_button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.preview_tabs")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.preview_tabs_enabled"),
                    description: i18n::t!("settings.text.show_opened_editors_as_preview_tabs"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enabled"),
                        pick: |settings_content| {
                            settings_content.preview_tabs.as_ref()?.enabled.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enabled = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enable_preview_from_project_panel"),
                    description: i18n::t!("settings.text.whether_to_open_tabs_in_preview_mode_when_opened_from_th_a32b0bf4"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enable_preview_from_project_panel"),
                        pick: |settings_content| {
                            settings_content
                                .preview_tabs
                                .as_ref()?
                                .enable_preview_from_project_panel
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enable_preview_from_project_panel = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enable_preview_from_file_finder"),
                    description: i18n::t!("settings.text.whether_to_open_tabs_in_preview_mode_when_selected_from__ac999ca8"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enable_preview_from_file_finder"),
                        pick: |settings_content| {
                            settings_content
                                .preview_tabs
                                .as_ref()?
                                .enable_preview_from_file_finder
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enable_preview_from_file_finder = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enable_preview_from_multibuffer"),
                    description: i18n::t!("settings.text.whether_to_open_tabs_in_preview_mode_when_opened_from_a_multibuffer"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enable_preview_from_multibuffer"),
                        pick: |settings_content| {
                            settings_content
                                .preview_tabs
                                .as_ref()?
                                .enable_preview_from_multibuffer
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enable_preview_from_multibuffer = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enable_preview_multibuffer_from_code_navigation"),
                    description: i18n::t!("settings.text.whether_to_open_tabs_in_preview_mode_when_code_navigatio_56814706"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enable_preview_multibuffer_from_code_navigation"),
                        pick: |settings_content| {
                            settings_content
                                .preview_tabs
                                .as_ref()?
                                .enable_preview_multibuffer_from_code_navigation
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enable_preview_multibuffer_from_code_navigation = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enable_preview_file_from_code_navigation"),
                    description: i18n::t!("settings.text.whether_to_open_tabs_in_preview_mode_when_code_navigatio_e99a63ef"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enable_preview_file_from_code_navigation"),
                        pick: |settings_content| {
                            settings_content
                                .preview_tabs
                                .as_ref()?
                                .enable_preview_file_from_code_navigation
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enable_preview_file_from_code_navigation = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enable_keep_preview_on_code_navigation"),
                    description: i18n::t!("settings.text.whether_to_keep_tabs_in_preview_mode_when_code_navigatio_7d4b8e1d"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("preview_tabs.enable_keep_preview_on_code_navigation"),
                        pick: |settings_content| {
                            settings_content
                                .preview_tabs
                                .as_ref()?
                                .enable_keep_preview_on_code_navigation
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .preview_tabs
                                .get_or_insert_default()
                                .enable_keep_preview_on_code_navigation = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.layout")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.bottom_dock_layout"),
                    description: i18n::t!("settings.text.layout_mode_for_the_bottom_dock"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("bottom_dock_layout"),
                        pick: |settings_content| {
                            settings_content.workspace.bottom_dock_layout.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.bottom_dock_layout = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.centered_layout_left_padding"),
                    description: i18n::t!("settings.text.left_padding_for_centered_layout"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("centered_layout.left_padding"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .centered_layout
                                .as_ref()?
                                .left_padding
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .workspace
                                .centered_layout
                                .get_or_insert_default()
                                .left_padding = value;
                        },
                    }),
                    metadata: None,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.centered_layout_right_padding"),
                    description: i18n::t!("settings.text.right_padding_for_centered_layout"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("centered_layout.right_padding"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .centered_layout
                                .as_ref()?
                                .right_padding
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .workspace
                                .centered_layout
                                .get_or_insert_default()
                                .right_padding = value;
                        },
                    }),
                    metadata: None,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.window")),
                // todo(settings_ui): Should we filter by platform.as_ref()?
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.use_system_window_tabs"),
                    description: i18n::t!("settings.text.macos_only_whether_to_allow_windows_to_tab_together"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("use_system_window_tabs"),
                        pick: |settings_content| {
                            settings_content.workspace.use_system_window_tabs.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.use_system_window_tabs = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.window_decorations"),
                    description: i18n::t!("settings.text.linux_only_whether_don_editor_or_your_compositor_should__f02be8d1"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("window_decorations"),
                        pick: |settings_content| {
                            settings_content.workspace.window_decorations.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.window_decorations = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.option_as_alt"),
                    description: i18n::t!("settings.text.mac_only_which_option_keys_should_the_editor_treat_as_al_24b357c1"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("option_as_alt"),
                        pick: |settings_content| {
                            settings_content.workspace.option_as_alt.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.option_as_alt = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.pane_modifiers")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.inactive_opacity"),
                    description: i18n::t!("settings.text.opacity_of_inactive_panels_0_0_1_0"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("active_pane_modifiers.inactive_opacity"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .active_pane_modifiers
                                .as_ref()?
                                .inactive_opacity
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .workspace
                                .active_pane_modifiers
                                .get_or_insert_default()
                                .inactive_opacity = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.border_size"),
                    description: i18n::t!("settings.text.size_of_the_border_surrounding_the_active_pane"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("active_pane_modifiers.border_size"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .active_pane_modifiers
                                .as_ref()?
                                .border_size
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .workspace
                                .active_pane_modifiers
                                .get_or_insert_default()
                                .border_size = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.zoomed_padding"),
                    description: i18n::t!("settings.text.show_padding_for_zoomed_panes"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("zoomed_padding"),
                        pick: |settings_content| settings_content.workspace.zoomed_padding.as_ref(),
                        write: |settings_content, value| {
                            settings_content.workspace.zoomed_padding = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.pane_split_direction")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.vertical_split_direction"),
                    description: i18n::t!("settings.text.direction_to_split_vertically"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("pane_split_direction_vertical"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .pane_split_direction_vertical
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.pane_split_direction_vertical = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.horizontal_split_direction"),
                    description: i18n::t!("settings.text.direction_to_split_horizontally"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("pane_split_direction_horizontal"),
                        pick: |settings_content| {
                            settings_content
                                .workspace
                                .pane_split_direction_horizontal
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.workspace.pane_split_direction_horizontal = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.panels"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.project_panel")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.project_panel_dock"),
                    description: i18n::t!("settings.text.where_to_dock_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.dock"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.dock.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.project_panel.get_or_insert_default().dock = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.project_panel_default_width"),
                    description: i18n::t!("settings.text.default_width_of_the_project_panel_in_pixels"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.default_width"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .default_width
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .default_width = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.hide_gitignore"),
                    description: i18n::t!("settings.text.whether_to_hide_the_gitignore_entries_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.hide_gitignore"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .hide_gitignore
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .hide_gitignore = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.entry_spacing"),
                    description: i18n::t!("settings.text.spacing_between_worktree_entries_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.entry_spacing"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .entry_spacing
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .entry_spacing = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.file_icons"),
                    description: i18n::t!("settings.text.show_file_icons_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.file_icons"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.file_icons.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .file_icons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.folder_icons"),
                    description: i18n::t!("settings.text.whether_to_show_folder_icons_or_chevrons_for_directories_333b3eae"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.folder_icons"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .folder_icons
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .folder_icons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_status"),
                    description: i18n::t!("settings.text.show_the_git_status_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.git_status"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.git_status.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .git_status = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.indent_size"),
                    description: i18n::t!("settings.text.amount_of_indentation_for_nested_items"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.indent_size"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .indent_size
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .indent_size = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.auto_reveal_entries"),
                    description: i18n::t!("settings.text.whether_to_reveal_entries_in_the_project_panel_automatic_ac333be6"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.auto_reveal_entries"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .auto_reveal_entries
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .auto_reveal_entries = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.starts_open"),
                    description: i18n::t!("settings.text.whether_the_project_panel_should_open_on_startup"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.starts_open"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .starts_open
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .starts_open = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.auto_fold_directories"),
                    description: i18n::t!("settings.text.whether_to_fold_directories_automatically_and_show_compa_a88d2f12"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.auto_fold_dirs"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .auto_fold_dirs
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .auto_fold_dirs = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_scrollbar"),
                    description: i18n::t!("settings.text.show_the_scrollbar_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.scrollbar.show"),
                        pick: |settings_content| {
                            show_scrollbar_or_editor(settings_content, |settings_content| {
                                settings_content
                                    .project_panel
                                    .as_ref()?
                                    .scrollbar
                                    .as_ref()?
                                    .show
                                    .as_ref()
                            })
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .scrollbar
                                .get_or_insert_default()
                                .show = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_diagnostics"),
                    description: i18n::t!("settings.text.which_files_containing_diagnostic_errors_warnings_to_mar_57a8cfdd"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.show_diagnostics"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .show_diagnostics
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .show_diagnostics = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.sticky_scroll"),
                    description: i18n::t!("settings.text.whether_to_stick_parent_directories_at_top_of_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.sticky_scroll"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .sticky_scroll
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .sticky_scroll = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.show_indent_guides"),
                    description: i18n::t!("settings.text.show_indent_guides_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("project_panel.indent_guides.show"),
                            pick: |settings_content| {
                                settings_content
                                    .project_panel
                                    .as_ref()?
                                    .indent_guides
                                    .as_ref()?
                                    .show
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .project_panel
                                    .get_or_insert_default()
                                    .indent_guides
                                    .get_or_insert_default()
                                    .show = value;
                            },
                        }
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.drag_and_drop"),
                    description: i18n::t!("settings.text.whether_to_enable_drag_and_drop_operations_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.drag_and_drop"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .drag_and_drop
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .drag_and_drop = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.hide_root"),
                    description: i18n::t!("settings.text.whether_to_hide_the_root_entry_when_only_one_folder_is_o_e8fc576f"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.drag_and_drop"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.hide_root.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .hide_root = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.hide_hidden"),
                    description: i18n::t!("settings.text.whether_to_hide_the_hidden_entries_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.hide_hidden"),
                        pick: |settings_content| {
                            settings_content
                                .project_panel
                                .as_ref()?
                                .hide_hidden
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .hide_hidden = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.hidden_files"),
                    description: i18n::t!("settings.text.globs_to_match_files_that_will_be_considered_hidden_and__d813cbb0"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("worktree.hidden_files"),
                            pick: |settings_content| {
                                settings_content.project.worktree.hidden_files.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content.project.worktree.hidden_files = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.auto_open_files")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.on_create"),
                    description: i18n::t!("settings.text.whether_to_automatically_open_newly_created_files_in_the_editor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.auto_open.on_create"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.auto_open.as_ref()?.on_create.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.project_panel.get_or_insert_default().auto_open.get_or_insert_default().on_create = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.on_paste"),
                    description: i18n::t!("settings.text.whether_to_automatically_open_files_after_pasting_or_dup_793f745f"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.auto_open.on_paste"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.auto_open.as_ref()?.on_paste.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.project_panel.get_or_insert_default().auto_open.get_or_insert_default().on_paste = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.on_drop"),
                    description: i18n::t!("settings.text.whether_to_automatically_open_files_dropped_from_external_sources"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.auto_open.on_drop"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.auto_open.as_ref()?.on_drop.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.project_panel.get_or_insert_default().auto_open.get_or_insert_default().on_drop = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.should_focus"),
                    description: i18n::t!("settings.text.whether_to_focus_on_files_automatically_opened"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("project_panel.auto_open.should_focus"),
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.auto_open.as_ref()?.should_focus.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.project_panel.get_or_insert_default().auto_open.get_or_insert_default().should_focus = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.sort_mode"),
                    description: i18n::t!("settings.text.sort_order_for_entries_in_the_project_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        pick: |settings_content| {
                            settings_content.project_panel.as_ref()?.sort_mode.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .project_panel
                                .get_or_insert_default()
                                .sort_mode = value;
                        },
                        json_path: Some("project_panel.sort_mode"),
                    }),
                    metadata: Some(Box::new(SettingsFieldMetadata {
                        should_do_titlecase: Some(false),
                        ..Default::default()
                    })),
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.terminal_panel")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.terminal_dock"),
                    description: i18n::t!("settings.text.where_to_dock_the_terminal_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.dock"),
                        pick: |settings_content| settings_content.terminal.as_ref()?.dock.as_ref(),
                        write: |settings_content, value| {
                            settings_content.terminal.get_or_insert_default().dock = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.outline_panel")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.outline_panel_button"),
                    description: i18n::t!("settings.text.show_the_outline_panel_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.button"),
                        pick: |settings_content| {
                            settings_content.outline_panel.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.outline_panel_dock"),
                    description: i18n::t!("settings.text.where_to_dock_the_outline_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.dock"),
                        pick: |settings_content| {
                            settings_content.outline_panel.as_ref()?.dock.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.outline_panel.get_or_insert_default().dock = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.outline_panel_default_width"),
                    description: i18n::t!("settings.text.default_width_of_the_outline_panel_in_pixels"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.default_width"),
                        pick: |settings_content| {
                            settings_content
                                .outline_panel
                                .as_ref()?
                                .default_width
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .default_width = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.file_icons"),
                    description: i18n::t!("settings.text.show_file_icons_in_the_outline_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.file_icons"),
                        pick: |settings_content| {
                            settings_content.outline_panel.as_ref()?.file_icons.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .file_icons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.folder_icons"),
                    description: i18n::t!("settings.text.whether_to_show_folder_icons_or_chevrons_for_directories_99a0813c"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.folder_icons"),
                        pick: |settings_content| {
                            settings_content
                                .outline_panel
                                .as_ref()?
                                .folder_icons
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .folder_icons = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_status"),
                    description: i18n::t!("settings.text.show_the_git_status_in_the_outline_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.git_status"),
                        pick: |settings_content| {
                            settings_content.outline_panel.as_ref()?.git_status.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .git_status = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.indent_size"),
                    description: i18n::t!("settings.text.amount_of_indentation_for_nested_items"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.indent_size"),
                        pick: |settings_content| {
                            settings_content
                                .outline_panel
                                .as_ref()?
                                .indent_size
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .indent_size = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.auto_reveal_entries"),
                    description: i18n::t!("settings.text.whether_to_reveal_when_a_corresponding_outline_entry_becomes_active"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.auto_reveal_entries"),
                        pick: |settings_content| {
                            settings_content
                                .outline_panel
                                .as_ref()?
                                .auto_reveal_entries
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .auto_reveal_entries = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.auto_fold_directories"),
                    description: i18n::t!("settings.text.whether_to_fold_directories_automatically_when_a_directo_e67c6381"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("outline_panel.auto_fold_dirs"),
                        pick: |settings_content| {
                            settings_content
                                .outline_panel
                                .as_ref()?
                                .auto_fold_dirs
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .outline_panel
                                .get_or_insert_default()
                                .auto_fold_dirs = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    files: USER,
                    title: i18n::t!("settings.text.show_indent_guides"),
                    description: i18n::t!("settings.text.when_to_show_indent_guides_in_the_outline_panel"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("outline_panel.indent_guides.show"),
                            pick: |settings_content| {
                                settings_content
                                    .outline_panel
                                    .as_ref()?
                                    .indent_guides
                                    .as_ref()?
                                    .show
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .outline_panel
                                    .get_or_insert_default()
                                    .indent_guides
                                    .get_or_insert_default()
                                    .show = value;
                            },
                        }
                    ),
                    metadata: None,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.git_panel")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_panel_button"),
                    description: i18n::t!("settings.text.show_the_git_panel_button_in_the_status_bar"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.button"),
                        pick: |settings_content| {
                            settings_content.git_panel.as_ref()?.button.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.git_panel.get_or_insert_default().button = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_panel_dock"),
                    description: i18n::t!("settings.text.where_to_dock_the_git_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.dock"),
                        pick: |settings_content| settings_content.git_panel.as_ref()?.dock.as_ref(),
                        write: |settings_content, value| {
                            settings_content.git_panel.get_or_insert_default().dock = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_panel_default_width"),
                    description: i18n::t!("settings.text.default_width_of_the_git_panel_in_pixels"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.default_width"),
                        pick: |settings_content| {
                            settings_content.git_panel.as_ref()?.default_width.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .default_width = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.git_panel_status_style"),
                    description: i18n::t!("settings.text.how_entry_statuses_are_displayed"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.status_style"),
                        pick: |settings_content| {
                            settings_content.git_panel.as_ref()?.status_style.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .status_style = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.fallback_branch_name"),
                    description: i18n::t!("settings.text.default_branch_name_will_be_when_init_defaultbranch_is_n_ba994ccb"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.fallback_branch_name"),
                        pick: |settings_content| {
                            settings_content
                                .git_panel
                                .as_ref()?
                                .fallback_branch_name
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .fallback_branch_name = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.sort_by_path"),
                    description: i18n::t!("settings.text.enable_to_sort_entries_in_the_panel_by_path_disable_to_s_aba4a4f7"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.sort_by_path"),
                        pick: |settings_content| {
                            settings_content.git_panel.as_ref()?.sort_by_path.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .sort_by_path = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.diff_stats"),
                    description: i18n::t!("settings.text.whether_to_show_added_and_deleted_line_counts_for_each_entry"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.diff_stats"),
                        pick: |settings_content| settings_content.git_panel.as_ref()?.diff_stats.as_ref(),
                        write: |settings_content, value| {
                            settings_content.git_panel.get_or_insert_default().diff_stats = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.collapse_untracked_diff"),
                    description: i18n::t!("settings.text.whether_to_collapse_untracked_files_in_the_diff_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.collapse_untracked_diff"),
                        pick: |settings_content| {
                            settings_content
                                .git_panel
                                .as_ref()?
                                .collapse_untracked_diff
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .collapse_untracked_diff = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.tree_view"),
                    description: i18n::t!("settings.text.enable_to_show_entries_in_tree_view_list_disable_to_show_0b40b610"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.tree_view"),
                        pick: |settings_content| {
                            settings_content.git_panel.as_ref()?.tree_view.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .tree_view = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.scroll_bar"),
                    description: i18n::t!("settings.text.how_and_when_the_scrollbar_should_be_displayed"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git_panel.scrollbar.show"),
                        pick: |settings_content| {
                            show_scrollbar_or_editor(settings_content, |settings_content| {
                                settings_content
                                    .git_panel
                                    .as_ref()?
                                    .scrollbar
                                    .as_ref()?
                                    .show
                                    .as_ref()
                            })
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git_panel
                                .get_or_insert_default()
                                .scrollbar
                                .get_or_insert_default()
                                .show = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.debugger_panel")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.debugger_panel_dock"),
                    description: i18n::t!("settings.text.the_dock_position_of_the_debug_panel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.dock"),
                        pick: |settings_content| settings_content.debugger.as_ref()?.dock.as_ref(),
                        write: |settings_content, value| {
                            settings_content.debugger.get_or_insert_default().dock = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.debugger"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.general")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.stepping_granularity"),
                    description: i18n::t!("settings.text.determines_the_stepping_granularity_for_debug_operations"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.stepping_granularity"),
                        pick: |settings_content| {
                            settings_content
                                .debugger
                                .as_ref()?
                                .stepping_granularity
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .debugger
                                .get_or_insert_default()
                                .stepping_granularity = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.save_breakpoints"),
                    description: i18n::t!("settings.text.whether_breakpoints_should_be_reused_across_don_editor_sessions"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.save_breakpoints"),
                        pick: |settings_content| {
                            settings_content
                                .debugger
                                .as_ref()?
                                .save_breakpoints
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .debugger
                                .get_or_insert_default()
                                .save_breakpoints = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.timeout"),
                    description: i18n::t!("settings.text.time_in_milliseconds_until_timeout_error_when_connecting_709d705d"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.timeout"),
                        pick: |settings_content| {
                            settings_content.debugger.as_ref()?.timeout.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.debugger.get_or_insert_default().timeout = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.log_dap_communications"),
                    description: i18n::t!("settings.text.whether_to_log_messages_between_active_debug_adapters_and_gram"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.log_dap_communications"),
                        pick: |settings_content| {
                            settings_content
                                .debugger
                                .as_ref()?
                                .log_dap_communications
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .debugger
                                .get_or_insert_default()
                                .log_dap_communications = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.format_dap_log_messages"),
                    description: i18n::t!("settings.text.whether_to_format_dap_messages_when_adding_them_to_debug_07d18b09"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("debugger.format_dap_log_messages"),
                        pick: |settings_content| {
                            settings_content
                                .debugger
                                .as_ref()?
                                .format_dap_log_messages
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .debugger
                                .get_or_insert_default()
                                .format_dap_log_messages = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.terminal"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.environment")),
                SettingsPageItem::DynamicItem(DynamicItem {
                    discriminant: SettingItem {
                        files: USER | PROJECT,
                        title: i18n::t!("settings.text.shell"),
                        description: i18n::t!("settings.text.what_shell_to_use_when_opening_a_terminal"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("terminal.shell$"),
                            pick: |settings_content| {
                                Some(&dynamic_variants::<settings::Shell>()[
                                    settings_content
                                        .terminal
                                        .as_ref()?
                                        .project
                                        .shell
                                        .as_ref()?
                                        .discriminant() as usize])
                            },
                            write: |settings_content, value| {
                                let Some(value) = value else {
                                    if let Some(terminal) = settings_content.terminal.as_mut() {
                                        terminal.project.shell = None;
                                    }
                                    return;
                                };
                                let settings_value = settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .project
                                    .shell
                                    .get_or_insert_with(settings::Shell::default);
                                let default_shell = if cfg!(target_os = "windows") {
                                    "powershell.exe"
                                } else {
                                    "sh"
                                };
                                *settings_value = match value {
                                    settings::ShellDiscriminants::System => {
                                        settings::Shell::System
                                    },
                                    settings::ShellDiscriminants::Program => {
                                        let program = match settings_value {
                                            settings::Shell::Program(p) => p.clone(),
                                            settings::Shell::WithArguments { program, .. } => program.clone(),
                                            _ => String::from(default_shell),
                                        };
                                        settings::Shell::Program(program)
                                    },
                                    settings::ShellDiscriminants::WithArguments => {
                                        let (program, args, title_override) = match settings_value {
                                            settings::Shell::Program(p) => (p.clone(), vec![], None),
                                            settings::Shell::WithArguments { program, args, title_override } => {
                                                (program.clone(), args.clone(), title_override.clone())
                                            },
                                            _ => (String::from(default_shell), vec![], None),
                                        };
                                        settings::Shell::WithArguments {
                                            program,
                                            args,
                                            title_override,
                                        }
                                    },
                                };
                            },
                        }),
                        metadata: None,
                    },
                    pick_discriminant: |settings_content| {
                        Some(settings_content.terminal.as_ref()?.project.shell.as_ref()?.discriminant() as usize)
                    },
                    fields: dynamic_variants::<settings::Shell>().iter().map(|variant| {
                        match variant {
                            settings::ShellDiscriminants::System => vec![],
                            settings::ShellDiscriminants::Program => vec![
                                SettingItem {
                                    files: USER | PROJECT,
                                    title: i18n::t!("settings.text.program"),
                                    description: i18n::t!("settings.text.the_shell_program_to_use"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("terminal.shell"),
                                        pick: |settings_content| {
                                            match settings_content.terminal.as_ref()?.project.shell.as_ref() {
                                                Some(settings::Shell::Program(program)) => Some(program),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::Shell::Program(program)) = settings_content
                                                .terminal
                                                .get_or_insert_default()
                                                .project
                                                .shell.as_mut() { *program = value }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                            settings::ShellDiscriminants::WithArguments => vec![
                                SettingItem {
                                    files: USER | PROJECT,
                                    title: i18n::t!("settings.text.program"),
                                    description: i18n::t!("settings.text.the_shell_program_to_run"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("terminal.shell.program"),
                                        pick: |settings_content| {
                                            match settings_content.terminal.as_ref()?.project.shell.as_ref() {
                                                Some(settings::Shell::WithArguments { program, .. }) => Some(program),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let Some(value) = value else {
                                                return;
                                            };
                                            if let Some(settings::Shell::WithArguments { program, .. }) = settings_content
                                                .terminal
                                                .get_or_insert_default()
                                                .project
                                                .shell.as_mut() { *program = value }
                                        },
                                    }),
                                    metadata: None,
                                },
                                SettingItem {
                                    files: USER | PROJECT,
                                    title: i18n::t!("settings.text.arguments"),
                                    description: i18n::t!("settings.text.the_arguments_to_pass_to_the_shell_program"),
                                    aliases: None,
                                    field: Box::new(
                                        SettingField {
                                            json_path: Some("terminal.shell.args"),
                                            pick: |settings_content| {
                                                match settings_content.terminal.as_ref()?.project.shell.as_ref() {
                                                    Some(settings::Shell::WithArguments { args, .. }) => Some(args),
                                                    _ => None
                                                }
                                            },
                                            write: |settings_content, value| {
                                                let Some(value) = value else {
                                                    return;
                                                };
                                                if let Some(settings::Shell::WithArguments { args, .. }) = settings_content
                                                    .terminal
                                                    .get_or_insert_default()
                                                    .project
                                                    .shell.as_mut() { *args = value }
                                            },
                                        }
                                        .unimplemented(),
                                    ),
                                    metadata: None,
                                },
                                SettingItem {
                                    files: USER | PROJECT,
                                    title: i18n::t!("settings.text.title_override"),
                                    description: i18n::t!("settings.text.an_optional_string_to_override_the_title_of_the_terminal_tab"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("terminal.shell.title_override"),
                                        pick: |settings_content| {
                                            match settings_content.terminal.as_ref()?.project.shell.as_ref() {
                                                Some(settings::Shell::WithArguments { title_override, .. }) => title_override.as_ref().or(DEFAULT_EMPTY_SHARED_STRING),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            if let Some(settings::Shell::WithArguments { title_override, .. }) = settings_content
                                                .terminal
                                                .get_or_insert_default()
                                                .project
                                                .shell.as_mut() { *title_override = value.filter(|s| !s.is_empty()) }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                        }
                    }).collect(),
                }),
                SettingsPageItem::DynamicItem(DynamicItem {
                    discriminant: SettingItem {
                        files: USER | PROJECT,
                        title: i18n::t!("settings.text.working_directory"),
                        description: i18n::t!("settings.text.what_working_directory_to_use_when_launching_the_terminal"),
                        aliases: None,
                        field: Box::new(SettingField {
                            json_path: Some("terminal.working_directory$"),
                            pick: |settings_content| {
                                Some(&dynamic_variants::<settings::WorkingDirectory>()[
                                    settings_content
                                        .terminal
                                        .as_ref()?
                                        .project
                                        .working_directory
                                        .as_ref()?
                                        .discriminant() as usize])
                            },
                            write: |settings_content, value| {
                                let Some(value) = value else {
                                    if let Some(terminal) = settings_content.terminal.as_mut() {
                                        terminal.project.working_directory = None;
                                    }
                                    return;
                                };
                                let settings_value = settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .project
                                    .working_directory
                                    .get_or_insert(settings::WorkingDirectory::CurrentProjectDirectory);
                                *settings_value = match value {
                                    settings::WorkingDirectoryDiscriminants::CurrentProjectDirectory => {
                                        settings::WorkingDirectory::CurrentProjectDirectory
                                    },
                                    settings::WorkingDirectoryDiscriminants::FirstProjectDirectory => {
                                        settings::WorkingDirectory::FirstProjectDirectory
                                    },
                                    settings::WorkingDirectoryDiscriminants::AlwaysHome => {
                                        settings::WorkingDirectory::AlwaysHome
                                    },
                                    settings::WorkingDirectoryDiscriminants::Always => {
                                        let directory = match settings_value {
                                            settings::WorkingDirectory::Always { .. } => return,
                                            _ => String::new(),
                                        };
                                        settings::WorkingDirectory::Always { directory }
                                    },
                                };
                            },
                        }),
                        metadata: None,
                    },
                    pick_discriminant: |settings_content| {
                        Some(settings_content.terminal.as_ref()?.project.working_directory.as_ref()?.discriminant() as usize)
                    },
                    fields: dynamic_variants::<settings::WorkingDirectory>().iter().map(|variant| {
                        match variant {
                            settings::WorkingDirectoryDiscriminants::CurrentProjectDirectory => vec![],
                            settings::WorkingDirectoryDiscriminants::FirstProjectDirectory => vec![],
                            settings::WorkingDirectoryDiscriminants::AlwaysHome => vec![],
                            settings::WorkingDirectoryDiscriminants::Always => vec![
                                SettingItem {
                                    files: USER | PROJECT,
                                    title: i18n::t!("settings.text.directory"),
                                    description: i18n::t!("settings.text.the_directory_path_to_use_will_be_shell_expanded"),
                                    aliases: None,
                                    field: Box::new(SettingField {
                                        json_path: Some("terminal.working_directory.always"),
                                        pick: |settings_content| {
                                            match settings_content.terminal.as_ref()?.project.working_directory.as_ref() {
                                                Some(settings::WorkingDirectory::Always { directory }) => Some(directory),
                                                _ => None
                                            }
                                        },
                                        write: |settings_content, value| {
                                            let value = value.unwrap_or_default();
                                            if let Some(settings::WorkingDirectory::Always { directory }) = settings_content
                                                .terminal
                                                .get_or_insert_default()
                                                .project
                                                .working_directory.as_mut() { *directory = value }
                                        },
                                    }),
                                    metadata: None,
                                }
                            ],
                        }
                    }).collect(),
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.environment_variables"),
                    description: i18n::t!("settings.text.key_value_pairs_to_add_to_the_terminal_s_environment"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("terminal.env"),
                            pick: |settings_content| {
                                settings_content.terminal.as_ref()?.project.env.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .project
                                    .env = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER | PROJECT,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.detect_virtual_environment"),
                    description: i18n::t!("settings.text.activates_the_python_virtual_environment_if_one_is_found_77c4db73"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("terminal.detect_venv"),
                            pick: |settings_content| {
                                settings_content
                                    .terminal
                                    .as_ref()?
                                    .project
                                    .detect_venv
                                    .as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .project
                                    .detect_venv = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER | PROJECT,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.font")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_size"),
                    description: i18n::t!("settings.text.font_size_for_terminal_text_if_not_set_defaults_to_buffer_font_size"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.font_size"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()
                                .and_then(|terminal| terminal.font_size.as_ref())
                                .or(settings_content.theme.buffer_font_size.as_ref())
                        },
                        write: |settings_content, value| {
                            settings_content.terminal.get_or_insert_default().font_size = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_family"),
                    description: i18n::t!("settings.text.font_family_for_terminal_text_if_not_set_defaults_to_buf_06c26273"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.font_family"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()
                                .and_then(|terminal| terminal.font_family.as_ref())
                                .or(settings_content.theme.buffer_font_family.as_ref())
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .font_family = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_fallbacks"),
                    description: i18n::t!("settings.text.font_fallbacks_for_terminal_text_if_not_set_defaults_to__72993a8f"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("terminal.font_fallbacks"),
                            pick: |settings_content| {
                                settings_content
                                    .terminal
                                    .as_ref()
                                    .and_then(|terminal| terminal.font_fallbacks.as_ref())
                                    .or(settings_content.theme.buffer_font_fallbacks.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .font_fallbacks = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_weight"),
                    description: i18n::t!("settings.text.font_weight_for_terminal_text_in_css_weight_units_100_900"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.font_weight"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.font_weight.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .font_weight = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.font_features"),
                    description: i18n::t!("settings.text.font_features_for_terminal_text"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("terminal.font_features"),
                            pick: |settings_content| {
                                settings_content
                                    .terminal
                                    .as_ref()
                                    .and_then(|terminal| terminal.font_features.as_ref())
                                    .or(settings_content.theme.buffer_font_features.as_ref())
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .font_features = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.display_settings")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.line_height"),
                    description: i18n::t!("settings.text.line_height_for_terminal_text"),
                    aliases: None,
                    field: Box::new(
                        SettingField {
                            json_path: Some("terminal.line_height"),
                            pick: |settings_content| {
                                settings_content.terminal.as_ref()?.line_height.as_ref()
                            },
                            write: |settings_content, value| {
                                settings_content
                                    .terminal
                                    .get_or_insert_default()
                                    .line_height = value;
                            },
                        }
                        .unimplemented(),
                    ),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.cursor_shape"),
                    description: i18n::t!("settings.text.default_cursor_shape_for_the_terminal_bar_block_underline_or_hollow"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.cursor_shape"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.cursor_shape.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .cursor_shape = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.cursor_blinking"),
                    description: i18n::t!("settings.text.sets_the_cursor_blinking_behavior_in_the_terminal"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.blinking"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.blinking.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.terminal.get_or_insert_default().blinking = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.alternate_scroll"),
                    description: i18n::t!("settings.text.whether_alternate_scroll_mode_is_active_by_default_conve_0ff6e646"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.alternate_scroll"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()?
                                .alternate_scroll
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .alternate_scroll = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.minimum_contrast"),
                    description: i18n::t!("settings.text.the_minimum_apca_perceptual_contrast_between_foreground__45619a8c"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.minimum_contrast"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()?
                                .minimum_contrast
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .minimum_contrast = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.behavior_settings")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.option_as_meta"),
                    description: i18n::t!("settings.text.whether_the_option_key_behaves_as_the_meta_key"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.option_as_meta"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.option_as_meta.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .option_as_meta = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.copy_on_select"),
                    description: i18n::t!("settings.text.whether_selecting_text_in_the_terminal_automatically_cop_2086c920"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.copy_on_select"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.copy_on_select.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .copy_on_select = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.keep_selection_on_copy"),
                    description: i18n::t!("settings.text.whether_to_keep_the_text_selection_after_copying_it_to_the_clipboard"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.keep_selection_on_copy"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()?
                                .keep_selection_on_copy
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .keep_selection_on_copy = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.layout_settings")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.default_width"),
                    description: i18n::t!("settings.text.default_width_when_the_terminal_is_docked_to_the_left_or_663280a8"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.default_width"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.default_width.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .default_width = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.default_height"),
                    description: i18n::t!("settings.text.default_height_when_the_terminal_is_docked_to_the_bottom_in_pixels"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.default_height"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.default_height.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .default_height = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.advanced_settings")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.max_scroll_history_lines"),
                    description: i18n::t!("settings.text.maximum_number_of_lines_to_keep_in_scrollback_history_ma_4c17a206"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.max_scroll_history_lines"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()?
                                .max_scroll_history_lines
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .max_scroll_history_lines = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.scroll_multiplier"),
                    description: i18n::t!("settings.text.the_multiplier_for_scrolling_in_the_terminal_with_the_mouse_wheel"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.scroll_multiplier"),
                        pick: |settings_content| {
                            settings_content.terminal.as_ref()?.scroll_multiplier.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .scroll_multiplier = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.toolbar")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.breadcrumbs"),
                    description: i18n::t!("settings.text.display_the_terminal_title_in_breadcrumbs_inside_the_terminal_pane"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.toolbar.breadcrumbs"),
                        pick: |settings_content| {
                            settings_content
                                .terminal
                                .as_ref()?
                                .toolbar
                                .as_ref()?
                                .breadcrumbs
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .toolbar
                                .get_or_insert_default()
                                .breadcrumbs = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.scrollbar")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_scrollbar"),
                    description: i18n::t!("settings.text.when_to_show_the_scrollbar_in_the_terminal"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("terminal.scrollbar.show"),
                        pick: |settings_content| {
                            show_scrollbar_or_editor(settings_content, |settings_content| {
                                settings_content
                                    .terminal
                                    .as_ref()?
                                    .scrollbar
                                    .as_ref()?
                                    .show
                                    .as_ref()
                            })
                        },
                        write: |settings_content, value| {
                            settings_content
                                .terminal
                                .get_or_insert_default()
                                .scrollbar
                                .get_or_insert_default()
                                .show = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.version_control"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.diffs")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.side_by_side_diff"),
                    description: i18n::t!("settings.text.open_repository_diffs_commits_and_file_comparisons_in_two_panels"),
                    aliases: Some(&["Split Diff"]),
                    field: Box::new(SettingField {
                        json_path: Some("git.split_diff"),
                        pick: |content| content.git.as_ref()?.split_diff.as_ref(),
                        write: |content, value| content.git.get_or_insert_default().split_diff = value,
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.git_gutter")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.visibility"),
                    description: i18n::t!("settings.text.control_whether_git_status_is_shown_in_the_editor_s_gutter"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.git_gutter"),
                        pick: |settings_content| settings_content.git.as_ref()?.git_gutter.as_ref(),
                        write: |settings_content, value| {
                            settings_content.git.get_or_insert_default().git_gutter = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                // todo(settings_ui): Figure out the right default for this value in default.jsonc
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.debounce"),
                    description: i18n::t!("settings.text.debounce_threshold_in_milliseconds_after_which_changes_a_1650b653"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.gutter_debounce"),
                        pick: |settings_content| {
                            settings_content.git.as_ref()?.gutter_debounce.as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content.git.get_or_insert_default().gutter_debounce = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.inline_git_blame")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.enabled"),
                    description: i18n::t!("settings.text.whether_or_not_to_show_git_blame_data_inline_in_the_curr_d426efcf"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.inline_blame.enabled"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .inline_blame
                                .as_ref()?
                                .enabled
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .inline_blame
                                .get_or_insert_default()
                                .enabled = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.delay"),
                    description: i18n::t!("settings.text.the_delay_after_which_the_inline_blame_information_is_shown"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.inline_blame.delay_ms"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .inline_blame
                                .as_ref()?
                                .delay_ms
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .inline_blame
                                .get_or_insert_default()
                                .delay_ms = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.padding"),
                    description: i18n::t!("settings.text.padding_between_the_end_of_the_source_line_and_the_start_bc3d22e3"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.inline_blame.padding"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .inline_blame
                                .as_ref()?
                                .padding
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .inline_blame
                                .get_or_insert_default()
                                .padding = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.minimum_column"),
                    description: i18n::t!("settings.text.the_minimum_column_number_at_which_to_show_the_inline_bl_5eba52b9"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.inline_blame.min_column"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .inline_blame
                                .as_ref()?
                                .min_column
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .inline_blame
                                .get_or_insert_default()
                                .min_column = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_commit_summary"),
                    description: i18n::t!("settings.text.show_commit_summary_as_part_of_the_inline_blame"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.inline_blame.show_commit_summary"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .inline_blame
                                .as_ref()?
                                .show_commit_summary
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .inline_blame
                                .get_or_insert_default()
                                .show_commit_summary = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.git_blame_view")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_avatar"),
                    description: i18n::t!("settings.text.show_the_avatar_of_the_author_of_the_commit"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.blame.show_avatar"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .blame
                                .as_ref()?
                                .show_avatar
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .blame
                                .get_or_insert_default()
                                .show_avatar = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.branch_picker")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.show_author_name"),
                    description: i18n::t!("settings.text.show_author_name_as_part_of_the_commit_information_in_branch_picker"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.branch_picker.show_author_name"),
                        pick: |settings_content| {
                            settings_content
                                .git
                                .as_ref()?
                                .branch_picker
                                .as_ref()?
                                .show_author_name
                                .as_ref()
                        },
                        write: |settings_content, value| {
                            settings_content
                                .git
                                .get_or_insert_default()
                                .branch_picker
                                .get_or_insert_default()
                                .show_author_name = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.git_hunks")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.hunk_style"),
                    description: i18n::t!("settings.text.how_git_hunks_are_displayed_visually_in_the_editor"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.hunk_style"),
                        pick: |settings_content| settings_content.git.as_ref()?.hunk_style.as_ref(),
                        write: |settings_content, value| {
                            settings_content.git.get_or_insert_default().hunk_style = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.path_style"),
                    description: i18n::t!("settings.text.should_the_name_or_path_be_displayed_first_in_the_git_view"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("git.path_style"),
                        pick: |settings_content| settings_content.git.as_ref()?.path_style.as_ref(),
                        write: |settings_content, value| {
                            settings_content.git.get_or_insert_default().path_style = value;
                        },
                    }),
                    metadata: None,
                    files: USER,
                }),
            ],
        },
        SettingsPage {
            title: i18n::t!("settings.text.network"),
            items: vec![
                SettingsPageItem::SectionHeader(i18n::t!("settings.text.network")),
                SettingsPageItem::SettingItem(SettingItem {
                    title: i18n::t!("settings.text.proxy"),
                    description: i18n::t!("settings.text.the_proxy_to_use_for_network_requests"),
                    aliases: None,
                    field: Box::new(SettingField {
                        json_path: Some("proxy"),
                        pick: |settings_content| settings_content.proxy.as_ref(),
                        write: |settings_content, value| {
                            settings_content.proxy = value;
                        },
                    }),
                    metadata: Some(Box::new(SettingsFieldMetadata {
                        placeholder: Some("socks5h://localhost:10808"),
                        ..Default::default()
                    })),
                    files: USER,
                }),
            ],
        },
    ]
}

/// 언어별 설정의 섹션 이름은 현재 표시 언어를 따릅니다.
fn languages_section_header() -> &'static str {
    i18n::t!("settings.text.languages")
}

#[cfg(test)]
mod localization_tests {
    use super::*;

    /// 언어를 바꿔도 저장에 사용하는 JSON 경로는 유지하고 제목과 설명만 번역합니다.
    #[test]
    fn language_metadata_translates_without_changing_setting_paths() {
        let english = i18n::with_language(i18n::Language::English, language_settings_data);
        let korean = i18n::with_language(i18n::Language::Korean, language_settings_data);
        assert_eq!(english.len(), korean.len());
        let mut translated = 0;
        // 같은 순서의 필드를 비교하여 언어별 설정의 저장 대상이 달라지지 않음을 확인합니다.
        for (english, korean) in english.iter().zip(&korean) {
            if let (SettingsPageItem::SettingItem(english), SettingsPageItem::SettingItem(korean)) = (english, korean) {
                assert_eq!(english.field.json_path(), korean.field.json_path());
                assert!(!korean.title.starts_with("settings.text."));
                assert_ne!(english.description, korean.description);
                assert!(
                    korean
                        .description
                        .chars()
                        .any(|character| ('가'..='힣').contains(&character))
                );
                translated += 1;
            }
        }
        assert!(translated > 30, "언어별 설정 메타데이터를 충분히 검사해야 합니다");
    }
}

fn current_language() -> Option<SharedString> {
    sub_page_stack()
        .iter()
        .find_map(|page| (page.section_header == languages_section_header()).then(|| page.link.title.clone()))
}

fn language_settings_field<T>(
    settings_content: &SettingsContent,
    get: fn(&LanguageSettingsContent) -> Option<&T>,
) -> Option<&T> {
    let all_languages = &settings_content.project.all_languages;
    if let Some(current_language_name) = current_language()
        && let Some(current_language) = all_languages.languages.0.get(&current_language_name)
    {
        let value = get(current_language);
        if value.is_some() {
            return value;
        }
    }

    get(&all_languages.defaults)
}

fn language_settings_field_mut<T>(
    settings_content: &mut SettingsContent,
    value: Option<T>,
    write: fn(&mut LanguageSettingsContent, Option<T>),
) {
    let all_languages = &mut settings_content.project.all_languages;
    let language_content = if let Some(current_language) = current_language() {
        all_languages.languages.0.entry(current_language).or_default()
    } else {
        &mut all_languages.defaults
    };
    write(language_content, value);
}

fn language_settings_data() -> Vec<SettingsPageItem> {
    let mut items = vec![
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.indentation")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.tab_size"),
            description: i18n::t!("settings.text.how_many_columns_a_tab_should_occupy"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).tab_size"), // TODO(cameron): not JQ syntax because not URL-safe
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.tab_size.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.tab_size = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.hard_tabs"),
            description: i18n::t!("settings.text.whether_to_indent_lines_using_tab_characters_as_opposed__e158811c"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).hard_tabs"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.hard_tabs.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.hard_tabs = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.auto_indent"),
            description: i18n::t!("settings.text.whether_indentation_should_be_adjusted_based_on_the_cont_8a731093"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).auto_indent"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.auto_indent.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.auto_indent = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.auto_indent_on_paste"),
            description: i18n::t!("settings.text.whether_indentation_of_pasted_content_should_be_adjusted_6f738340"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).auto_indent_on_paste"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.auto_indent_on_paste.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.auto_indent_on_paste = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.wrapping")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.soft_wrap"),
            description: i18n::t!("settings.text.how_to_soft_wrap_long_lines_of_text"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).soft_wrap"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.soft_wrap.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.soft_wrap = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_wrap_guides"),
            description: i18n::t!("settings.text.show_wrap_guides_in_the_editor"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).show_wrap_guides"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.show_wrap_guides.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.show_wrap_guides = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.preferred_line_length"),
            description: i18n::t!("settings.text.the_column_at_which_to_soft_wrap_lines_for_buffers_where_d87af1db"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).preferred_line_length"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.preferred_line_length.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.preferred_line_length = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.wrap_guides"),
            description: i18n::t!("settings.text.character_counts_at_which_to_show_wrap_guides_in_the_editor"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).wrap_guides"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.wrap_guides.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.wrap_guides = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.allow_rewrap"),
            description: i18n::t!("settings.text.controls_where_the_editor_rewrap_action_is_allowed_for_this_language"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).allow_rewrap"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.allow_rewrap.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.allow_rewrap = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.indent_guides")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.enabled"),
            description: i18n::t!("settings.text.display_indent_guides_in_the_editor"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).indent_guides.enabled"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language
                            .indent_guides
                            .as_ref()
                            .and_then(|indent_guides| indent_guides.enabled.as_ref())
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.indent_guides.get_or_insert_default().enabled = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.line_width"),
            description: i18n::t!("settings.text.the_width_of_the_indent_guides_in_pixels_between_1_and_10"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).indent_guides.line_width"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language
                            .indent_guides
                            .as_ref()
                            .and_then(|indent_guides| indent_guides.line_width.as_ref())
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.indent_guides.get_or_insert_default().line_width = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.active_line_width"),
            description: i18n::t!("settings.text.the_width_of_the_active_indent_guide_in_pixels_between_1_and_10"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).indent_guides.active_line_width"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language
                            .indent_guides
                            .as_ref()
                            .and_then(|indent_guides| indent_guides.active_line_width.as_ref())
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.indent_guides.get_or_insert_default().active_line_width = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.coloring"),
            description: i18n::t!("settings.text.determines_how_indent_guides_are_colored"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).indent_guides.coloring"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language
                            .indent_guides
                            .as_ref()
                            .and_then(|indent_guides| indent_guides.coloring.as_ref())
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.indent_guides.get_or_insert_default().coloring = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.background_coloring"),
            description: i18n::t!("settings.text.determines_how_indent_guide_backgrounds_are_colored"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).indent_guides.background_coloring"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language
                            .indent_guides
                            .as_ref()
                            .and_then(|indent_guides| indent_guides.background_coloring.as_ref())
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.indent_guides.get_or_insert_default().background_coloring = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.formatting")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.format_on_save"),
            description: i18n::t!("settings.text.whether_or_not_to_perform_a_buffer_format_before_saving"),
            aliases: None,
            field: Box::new(
                // TODO(settings_ui): this setting should just be a bool
                SettingField {
                    json_path: Some("languages.$(language).format_on_save"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.format_on_save.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.format_on_save = value;
                        })
                    },
                },
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.remove_trailing_whitespace_on_save"),
            description: i18n::t!("settings.text.whether_or_not_to_remove_any_trailing_whitespace_from_li_aa1716f8"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).remove_trailing_whitespace_on_save"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.remove_trailing_whitespace_on_save.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.remove_trailing_whitespace_on_save = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.ensure_final_newline_on_save"),
            description: i18n::t!("settings.text.whether_or_not_to_ensure_there_s_a_single_newline_at_the_2d5785a4"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).ensure_final_newline_on_save"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.ensure_final_newline_on_save.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.ensure_final_newline_on_save = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.formatter"),
            description: i18n::t!("settings.text.how_to_perform_a_buffer_format"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).formatter"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.formatter.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.formatter = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.use_on_type_format"),
            description: i18n::t!("settings.text.whether_to_use_additional_lsp_queries_to_format_and_amen_ad0bb49f"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).use_on_type_format"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.use_on_type_format.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.use_on_type_format = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.code_actions_on_format"),
            description: i18n::t!("settings.text.additional_code_actions_to_run_when_formatting"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).code_actions_on_format"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.code_actions_on_format.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.code_actions_on_format = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.autoclose")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.use_autoclose"),
            description: i18n::t!("settings.text.whether_to_automatically_type_closing_characters_for_you_acc73841"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).use_autoclose"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.use_autoclose.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.use_autoclose = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.use_auto_surround"),
            description: i18n::t!("settings.text.whether_to_automatically_surround_text_with_characters_f_c5953f9d"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).use_auto_surround"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.use_auto_surround.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.use_auto_surround = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.always_treat_brackets_as_autoclosed"),
            description: i18n::t!("settings.text.controls_whether_the_closing_characters_are_always_skipp_978950e0"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).always_treat_brackets_as_autoclosed"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.always_treat_brackets_as_autoclosed.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.always_treat_brackets_as_autoclosed = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.jsx_tag_auto_close"),
            description: i18n::t!("settings.text.whether_to_automatically_close_jsx_tags"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).jsx_tag_auto_close"),
                // TODO(settings_ui): this setting should just be a bool
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.jsx_tag_auto_close.as_ref()?.enabled.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.jsx_tag_auto_close.get_or_insert_default().enabled = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.whitespace")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_whitespaces"),
            description: i18n::t!("settings.text.whether_to_show_tabs_and_spaces_in_the_editor"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).show_whitespaces"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.show_whitespaces.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.show_whitespaces = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.space_whitespace_indicator"),
            description: i18n::t!("settings.text.visible_character_used_to_render_space_characters_when_s_3efb606e"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).whitespace_map.space"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| {
                            language.whitespace_map.as_ref()?.space.as_ref()
                        })
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.whitespace_map.get_or_insert_default().space = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.tab_whitespace_indicator"),
            description: i18n::t!("settings.text.visible_character_used_to_render_tab_characters_when_sho_2bd52f1d"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).whitespace_map.tab"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| {
                            language.whitespace_map.as_ref()?.tab.as_ref()
                        })
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.whitespace_map.get_or_insert_default().tab = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.completions")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_completions_on_input"),
            description: i18n::t!("settings.text.whether_to_pop_the_completions_menu_while_typing_in_an_e_5480e189"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).show_completions_on_input"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.show_completions_on_input.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.show_completions_on_input = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_completion_documentation"),
            description: i18n::t!("settings.text.whether_to_display_inline_and_alongside_documentation_fo_5d6dfd94"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).show_completion_documentation"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.show_completion_documentation.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.show_completion_documentation = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.words"),
            description: i18n::t!("settings.text.controls_how_words_are_completed"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).completions.words"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.completions.as_ref()?.words.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.completions.get_or_insert_default().words = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.words_min_length"),
            description: i18n::t!("settings.text.how_many_characters_has_to_be_in_the_completions_query_t_2716f0c8"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).completions.words_min_length"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.completions.as_ref()?.words_min_length.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.completions.get_or_insert_default().words_min_length = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.completion_menu_scrollbar"),
            description: i18n::t!("settings.text.when_to_show_the_scrollbar_in_the_completion_menu"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("editor.completion_menu_scrollbar"),
                pick: |settings_content| settings_content.editor.completion_menu_scrollbar.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.completion_menu_scrollbar = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.completion_menu_item_kind"),
            description: i18n::t!("settings.text.how_to_display_the_lsp_item_kind_function_method_variabl_0b8c513b"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("editor.completion_menu_item_kind"),
                pick: |settings_content| settings_content.editor.completion_menu_item_kind.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.completion_menu_item_kind = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.supertab_mode"),
            description: i18n::t!("settings.text.what_the_supertab_action_should_do_when_not_completing"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("editor.supertab_fallback"),
                pick: |settings_content| settings_content.editor.supertab_fallback.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.supertab_fallback = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.sync_kill_ring"),
            description: i18n::t!("settings.text.whether_to_sync_the_kill_ring_with_clipboard_or_primary_selection"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("editor.sync_kill_ring"),
                pick: |settings_content| settings_content.editor.sync_kill_ring.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.sync_kill_ring = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.inlay_hints")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.enabled"),
            description: i18n::t!("settings.text.global_switch_to_toggle_hints_on_and_off"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.enabled"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.enabled.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().enabled = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_value_hints"),
            description: i18n::t!("settings.text.global_switch_to_toggle_inline_values_on_and_off_when_debugging"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.show_value_hints"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.show_value_hints.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().show_value_hints = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_type_hints"),
            description: i18n::t!("settings.text.whether_type_hints_should_be_shown"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.show_type_hints"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.show_type_hints.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().show_type_hints = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_parameter_hints"),
            description: i18n::t!("settings.text.whether_parameter_hints_should_be_shown"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.show_parameter_hints"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.show_parameter_hints.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().show_parameter_hints = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_other_hints"),
            description: i18n::t!("settings.text.whether_other_hints_should_be_shown"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.show_other_hints"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.show_other_hints.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().show_other_hints = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.show_background"),
            description: i18n::t!("settings.text.show_a_background_for_inlay_hints"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.show_background"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.show_background.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().show_background = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.edit_debounce_ms"),
            description: i18n::t!("settings.text.whether_or_not_to_debounce_inlay_hints_updates_after_buf_b5dc2324"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.edit_debounce_ms"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.edit_debounce_ms.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().edit_debounce_ms = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.scroll_debounce_ms"),
            description: i18n::t!("settings.text.whether_or_not_to_debounce_inlay_hints_updates_after_buf_fee871a0"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).inlay_hints.scroll_debounce_ms"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.inlay_hints.as_ref()?.scroll_debounce_ms.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.inlay_hints.get_or_insert_default().scroll_debounce_ms = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.toggle_on_modifiers_press"),
            description: i18n::t!("settings.text.toggles_inlay_hints_hides_or_shows_when_the_user_presses_d65add73"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).inlay_hints.toggle_on_modifiers_press"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| {
                            language.inlay_hints.as_ref()?.toggle_on_modifiers_press.as_ref()
                        })
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.inlay_hints.get_or_insert_default().toggle_on_modifiers_press = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
    ];
    if current_language().is_none() {
        items.push(SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.lsp_document_colors"),
            description: i18n::t!("settings.text.how_to_render_lsp_color_previews_in_the_editor"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("lsp_document_colors"),
                pick: |settings_content| settings_content.editor.lsp_document_colors.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.lsp_document_colors = value;
                },
            }),
            metadata: None,
            files: USER,
        }))
    }
    items.extend([
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.tasks")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.enabled"),
            description: i18n::t!("settings.text.whether_tasks_are_enabled_for_this_language"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).tasks.enabled"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.tasks.as_ref()?.enabled.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.tasks.get_or_insert_default().enabled = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.variables"),
            description: i18n::t!("settings.text.extra_task_variables_to_set_for_a_particular_language"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).tasks.variables"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| {
                            language.tasks.as_ref()?.variables.as_ref()
                        })
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.tasks.get_or_insert_default().variables = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.prefer_lsp"),
            description: i18n::t!("settings.text.use_lsp_tasks_over_don_editor_language_extension_tasks"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).tasks.prefer_lsp"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.tasks.as_ref()?.prefer_lsp.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.tasks.get_or_insert_default().prefer_lsp = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.miscellaneous")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.word_diff_enabled"),
            description: i18n::t!("settings.text.whether_to_enable_word_diff_highlighting_in_the_editor_w_973023ce"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).word_diff_enabled"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.word_diff_enabled.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.word_diff_enabled = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.debuggers"),
            description: i18n::t!("settings.text.preferred_debuggers_for_this_language"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).debuggers"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.debuggers.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.debuggers = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.middle_click_paste"),
            description: i18n::t!("settings.text.enable_middle_click_paste_on_linux"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).editor.middle_click_paste"),
                pick: |settings_content| settings_content.editor.middle_click_paste.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.middle_click_paste = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.extend_comment_on_newline"),
            description: i18n::t!("settings.text.whether_to_start_a_new_line_with_a_comment_when_a_previo_505df2d0"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).extend_comment_on_newline"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.extend_comment_on_newline.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.extend_comment_on_newline = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.colorize_brackets"),
            description: i18n::t!("settings.text.whether_to_colorize_brackets_in_the_editor"),
            aliases: Some(&["Rainbow Parentheses", "Rainbow Brackets"]),
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).colorize_brackets"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.colorize_brackets.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.colorize_brackets = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
    ]);

    if current_language().is_none() {
        items.extend([
            SettingsPageItem::SettingItem(SettingItem {
                title: i18n::t!("settings.text.image_viewer"),
                description: i18n::t!("settings.text.the_unit_for_image_file_sizes"),
                aliases: None,
                field: Box::new(SettingField {
                    json_path: Some("image_viewer.unit"),
                    pick: |settings_content| {
                        settings_content
                            .image_viewer
                            .as_ref()
                            .and_then(|image_viewer| image_viewer.unit.as_ref())
                    },
                    write: |settings_content, value| {
                        settings_content.image_viewer.get_or_insert_default().unit = value;
                    },
                }),
                metadata: None,
                files: USER,
            }),
            SettingsPageItem::SettingItem(SettingItem {
                title: i18n::t!("settings.text.drop_size_target"),
                description: i18n::t!(
                    "settings.text.relative_size_of_the_drop_target_in_the_editor_that_will_dca40e29"
                ),
                aliases: None,
                field: Box::new(SettingField {
                    json_path: Some("drop_target_size"),
                    pick: |settings_content| settings_content.workspace.drop_target_size.as_ref(),
                    write: |settings_content, value| {
                        settings_content.workspace.drop_target_size = value;
                    },
                }),
                metadata: None,
                files: USER,
            }),
        ]);
    }
    items
}

/// LanguageSettings items that should be included in the "Languages & Tools" page
/// not the "Editor" page
fn non_editor_language_settings_data() -> Vec<SettingsPageItem> {
    vec![
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.lsp")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.enable_language_server"),
            description: i18n::t!("settings.text.whether_to_use_language_servers_to_provide_code_intelligence"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).enable_language_server"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.enable_language_server.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.enable_language_server = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.language_servers"),
            description: i18n::t!("settings.text.the_list_of_language_servers_to_use_or_disable_for_this_language"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).language_servers"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.language_servers.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.language_servers = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.linked_edits"),
            description: i18n::t!("settings.text.whether_to_perform_linked_edits_of_associated_ranges_if__d741364a"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).linked_edits"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.linked_edits.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.linked_edits = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.go_to_definition_fallback"),
            description: i18n::t!("settings.text.whether_to_follow_up_empty_go_to_definition_responses_fr_ef20e5ff"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("go_to_definition_fallback"),
                pick: |settings_content| settings_content.editor.go_to_definition_fallback.as_ref(),
                write: |settings_content, value| {
                    settings_content.editor.go_to_definition_fallback = value;
                },
            }),
            metadata: None,
            files: USER,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.lsp_completions")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.enabled"),
            description: i18n::t!("settings.text.whether_to_fetch_lsp_completions_or_not"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).completions.lsp"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.completions.as_ref()?.lsp.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.completions.get_or_insert_default().lsp = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.fetch_timeout_milliseconds"),
            description: i18n::t!("settings.text.when_fetching_lsp_completions_determines_how_long_to_wai_231296e2"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).completions.lsp_fetch_timeout_ms"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.completions.as_ref()?.lsp_fetch_timeout_ms.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.completions.get_or_insert_default().lsp_fetch_timeout_ms = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.insert_mode"),
            description: i18n::t!("settings.text.controls_how_lsp_completions_are_inserted"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).completions.lsp_insert_mode"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.completions.as_ref()?.lsp_insert_mode.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.completions.get_or_insert_default().lsp_insert_mode = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.debuggers")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.debuggers"),
            description: i18n::t!("settings.text.preferred_debuggers_for_this_language"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).debuggers"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| language.debuggers.as_ref())
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.debuggers = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SectionHeader(i18n::t!("settings.text.prettier")),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.allowed"),
            description: i18n::t!("settings.text.enables_or_disables_formatting_with_prettier_for_a_given_language"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).prettier.allowed"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| {
                        language.prettier.as_ref()?.allowed.as_ref()
                    })
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.prettier.get_or_insert_default().allowed = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.parser"),
            description: i18n::t!("settings.text.forces_prettier_integration_to_use_a_specific_parser_nam_7e722ad6"),
            aliases: None,
            field: Box::new(SettingField {
                json_path: Some("languages.$(language).prettier.parser"),
                pick: |settings_content| {
                    language_settings_field(settings_content, |language| language.prettier.as_ref()?.parser.as_ref())
                },
                write: |settings_content, value| {
                    language_settings_field_mut(settings_content, value, |language, value| {
                        language.prettier.get_or_insert_default().parser = value;
                    })
                },
            }),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.plugins"),
            description: i18n::t!("settings.text.forces_prettier_integration_to_use_specific_plugins_when_1eb1bcd8"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).prettier.plugins"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| {
                            language.prettier.as_ref()?.plugins.as_ref()
                        })
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.prettier.get_or_insert_default().plugins = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
        SettingsPageItem::SettingItem(SettingItem {
            title: i18n::t!("settings.text.options"),
            description: i18n::t!("settings.text.default_prettier_options_in_the_format_as_in_package_jso_18285809"),
            aliases: None,
            field: Box::new(
                SettingField {
                    json_path: Some("languages.$(language).prettier.options"),
                    pick: |settings_content| {
                        language_settings_field(settings_content, |language| {
                            language.prettier.as_ref()?.options.as_ref()
                        })
                    },
                    write: |settings_content, value| {
                        language_settings_field_mut(settings_content, value, |language, value| {
                            language.prettier.get_or_insert_default().options = value;
                        })
                    },
                }
                .unimplemented(),
            ),
            metadata: None,
            files: USER | PROJECT,
        }),
    ]
}

fn show_scrollbar_or_editor(
    settings_content: &SettingsContent,
    show: fn(&SettingsContent) -> Option<&settings::ShowScrollbar>,
) -> Option<&settings::ShowScrollbar> {
    show(settings_content).or(settings_content
        .editor
        .scrollbar
        .as_ref()
        .and_then(|scrollbar| scrollbar.show.as_ref()))
}

fn dynamic_variants<T>() -> &'static [T::Discriminant]
where
    T: strum::IntoDiscriminant,
    T::Discriminant: strum::VariantArray,
{
    <<T as strum::IntoDiscriminant>::Discriminant as strum::VariantArray>::VARIANTS
}
