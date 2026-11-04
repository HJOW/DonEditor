use gpui::ElementId;
use gpui::{AnyElement, Entity};
use picker::Picker;
use repl::{
    ExecutionState, JupyterSettings, Kernel, KernelSpecification, KernelStatus, Session, SessionSupport,
    components::{KernelPickerDelegate, KernelSelector},
    worktree_id_for_editor,
};
use ui::{
    ButtonLike, CommonAnimationExt, ContextMenu, IconWithIndicator, Indicator, IntoElement, PopoverMenu,
    PopoverMenuHandle, Tooltip, prelude::*,
};
use util::ResultExt;

use super::QuickActionBar;

const GRAM_REPL_DOCUMENTATION: &str = "gram://docs/repl";

struct ReplMenuState {
    tooltip: SharedString,
    icon: IconName,
    icon_color: Color,
    icon_is_animating: bool,
    popover_disabled: bool,
    indicator: Option<Indicator>,

    status: KernelStatus,
    kernel_name: SharedString,
    kernel_language: SharedString,
}

impl QuickActionBar {
    pub fn render_repl_menu(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !JupyterSettings::enabled(cx) {
            return None;
        }

        let editor = self.active_editor()?;

        let is_local_project = editor
            .read(cx)
            .workspace()
            .is_some_and(|workspace| workspace.read(cx).project().read(cx).is_local());

        if !is_local_project {
            return None;
        }

        let has_nonempty_selection = {
            editor.update(cx, |this, cx| {
                if this.selections.count().ne(&0) {
                    {
                        let snapshot = this.display_snapshot(cx);
                        let latest = this.selections.newest_display(&snapshot);
                        !latest.is_empty()
                    }
                } else {
                    Default::default()
                }
            })
        };

        let session = repl::session(editor.downgrade(), cx);
        let session = match session {
            SessionSupport::ActiveSession(session) => session,
            SessionSupport::Inactive(spec) => {
                return Some(self.render_repl_launch_menu(spec, cx));
            }
            SessionSupport::RequiresSetup(language) => {
                return Some(self.render_repl_setup(language.as_ref(), cx));
            }
            SessionSupport::Unsupported => return None,
        };

        let menu_state = session_state(session.clone(), cx);

        let id = "repl-menu";

        let element_id = |suffix| ElementId::Name(format!("{id}-{suffix}").into());

        let editor = editor.downgrade();
        let dropdown_menu = PopoverMenu::new(element_id("menu"))
            .menu(move |window, cx| {
                let editor = editor.clone();
                let session = session.clone();
                ContextMenu::build(window, cx, move |menu, _, cx| {
                    let menu_state = session_state(session, cx);
                    let status = menu_state.status;
                    let editor = editor.clone();

                    menu.map(|menu| {
                        let status = status.clone();
                        if status.is_connected() {
                            menu.custom_row(move |_window, _cx| {
                                h_flex()
                                    .child(
                                        Label::new(format!(
                                            "kernel: {} ({})",
                                            menu_state.kernel_name, menu_state.kernel_language
                                        ))
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                    )
                                    .into_any_element()
                            })
                            .custom_row(move |_window, _cx| {
                                h_flex()
                                    .child(
                                        Label::new(status.to_string())
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .into_any_element()
                            })
                        } else {
                            menu.custom_row(move |_window, _cx| {
                                h_flex()
                                    .child(
                                        Label::new(format!("{}...", status))
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .into_any_element()
                            })
                        }
                    })
                    .separator()
                    .custom_entry(
                        move |_window, _cx| {
                            Label::new(if has_nonempty_selection {
                                i18n::t!("ui.text.run_selection")
                            } else {
                                i18n::t!("ui.text.run_line")
                            })
                            .into_any_element()
                        },
                        {
                            let editor = editor.clone();
                            move |window, cx| {
                                repl::run(editor.clone(), true, window, cx).log_err();
                            }
                        },
                    )
                    .custom_entry(
                        move |_window, _cx| {
                            Label::new(i18n::t!("ui.text.interrupt"))
                                .size(LabelSize::Small)
                                .color(Color::Error)
                                .into_any_element()
                        },
                        {
                            let editor = editor.clone();
                            move |_, cx| {
                                repl::interrupt(editor.clone(), cx);
                            }
                        },
                    )
                    .custom_entry(
                        move |_window, _cx| {
                            Label::new(i18n::t!("ui.text.clear_outputs"))
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .into_any_element()
                        },
                        {
                            let editor = editor.clone();
                            move |_, cx| {
                                repl::clear_outputs(editor.clone(), cx);
                            }
                        },
                    )
                    .separator()
                    .custom_entry(
                        move |_window, _cx| {
                            Label::new(i18n::t!("ui.text.shut_down_kernel"))
                                .size(LabelSize::Small)
                                .color(Color::Error)
                                .into_any_element()
                        },
                        {
                            let editor = editor.clone();
                            move |window, cx| {
                                repl::shutdown(editor.clone(), window, cx);
                            }
                        },
                    )
                    .custom_entry(
                        move |_window, _cx| {
                            Label::new(i18n::t!("ui.text.restart_kernel"))
                                .size(LabelSize::Small)
                                .color(Color::Error)
                                .into_any_element()
                        },
                        {
                            move |window, cx| {
                                repl::restart(editor.clone(), window, cx);
                            }
                        },
                    )
                    .separator()
                    .action(i18n::t!("ui.text.view_sessions"), Box::new(repl::Sessions))
                    // TODO: Add shut down all kernels action
                    // .action(i18n::t!("ui.text.shut_down_all_kernels"), Box::new(gpui::NoAction))
                })
                .into()
            })
            .trigger_with_tooltip(
                ButtonLike::new_rounded_right(element_id("dropdown"))
                    .child(
                        Icon::new(IconName::ChevronDown)
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    )
                    .width(rems(1.))
                    .disabled(menu_state.popover_disabled),
                Tooltip::text(i18n::t!("ui.text.repl_menu")),
            );

        let button = ButtonLike::new_rounded_left("toggle_repl_icon")
            .child(if menu_state.icon_is_animating {
                Icon::new(menu_state.icon)
                    .color(menu_state.icon_color)
                    .with_rotate_animation(5)
                    .into_any_element()
            } else {
                IconWithIndicator::new(
                    Icon::new(IconName::ReplNeutral).color(menu_state.icon_color),
                    menu_state.indicator,
                )
                .indicator_border_color(Some(cx.theme().colors().toolbar_background))
                .into_any_element()
            })
            .size(ButtonSize::Compact)
            .style(ButtonStyle::Subtle)
            .tooltip(Tooltip::text(menu_state.tooltip))
            .on_click(|_, window, cx| window.dispatch_action(Box::new(repl::Run {}), cx))
            .into_any_element();

        Some(
            h_flex()
                .child(self.render_kernel_selector(cx))
                .child(button)
                .child(dropdown_menu)
                .into_any_element(),
        )
    }
    pub fn render_repl_launch_menu(
        &self,
        kernel_specification: KernelSpecification,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let tooltip: SharedString =
            SharedString::from(i18n::t!("ui.text.start_repl_for", value0 = kernel_specification.name()));

        h_flex()
            .child(self.render_kernel_selector(cx))
            .child(
                IconButton::new("toggle_repl_icon", IconName::ReplNeutral)
                    .size(ButtonSize::Compact)
                    .icon_color(Color::Muted)
                    .style(ButtonStyle::Subtle)
                    .tooltip(Tooltip::text(tooltip))
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(repl::Run {}), cx)),
            )
            .into_any_element()
    }

    pub fn render_kernel_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let editor = if let Some(editor) = self.active_editor() {
            editor
        } else {
            return div().into_any_element();
        };

        let Some(worktree_id) = worktree_id_for_editor(editor.downgrade(), cx) else {
            return div().into_any_element();
        };

        let session = repl::session(editor.downgrade(), cx);

        let current_kernelspec = match session {
            SessionSupport::ActiveSession(session) => Some(session.read(cx).kernel_specification.clone()),
            SessionSupport::Inactive(kernel_specification) => Some(kernel_specification),
            SessionSupport::RequiresSetup(_language_name) => None,
            SessionSupport::Unsupported => None,
        };

        let current_kernel_name = current_kernelspec.as_ref().map(repl::KernelSpecification::name);

        let menu_handle: PopoverMenuHandle<Picker<KernelPickerDelegate>> = PopoverMenuHandle::default();
        KernelSelector::new(
            {
                Box::new(move |kernelspec, window, cx| {
                    repl::assign_kernelspec(kernelspec, editor.downgrade(), window, cx).ok();
                })
            },
            worktree_id,
            ButtonLike::new("kernel-selector")
                .style(ButtonStyle::Subtle)
                .size(ButtonSize::Compact)
                .child(
                    h_flex()
                        .w_full()
                        .gap_0p5()
                        .child(
                            div().overflow_x_hidden().flex_grow().whitespace_nowrap().child(
                                Label::new(if let Some(name) = current_kernel_name {
                                    name
                                } else {
                                    SharedString::from(i18n::t!("ui.text.select_kernel"))
                                })
                                .size(LabelSize::Small)
                                .color(if current_kernelspec.is_some() {
                                    Color::Default
                                } else {
                                    Color::Placeholder
                                })
                                .into_any_element(),
                            ),
                        )
                        .child(
                            Icon::new(IconName::ChevronDown)
                                .color(Color::Muted)
                                .size(IconSize::XSmall),
                        ),
                ),
            Tooltip::text(i18n::t!("ui.text.select_kernel")),
        )
        .with_handle(menu_handle)
        .into_any_element()
    }

    pub fn render_repl_setup(&self, language: &str, cx: &mut Context<Self>) -> AnyElement {
        let tooltip: SharedString = SharedString::from(i18n::t!(
            "ui.text.setup_don_editor_repl_for_language",
            language = language
        ));
        h_flex()
            .gap(DynamicSpacing::Base06.rems(cx))
            .child(self.render_kernel_selector(cx))
            .child(
                IconButton::new("toggle_repl_icon", IconName::ReplNeutral)
                    .style(ButtonStyle::Subtle)
                    .shape(ui::IconButtonShape::Square)
                    .icon_size(ui::IconSize::Small)
                    .icon_color(Color::Muted)
                    .tooltip(Tooltip::text(tooltip))
                    .on_click(|_, _window, cx| cx.open_url(&format!("{GRAM_REPL_DOCUMENTATION}#installation"))),
            )
            .into_any_element()
    }
}

fn session_state(session: Entity<Session>, cx: &mut App) -> ReplMenuState {
    let session = session.read(cx);

    let kernel_name = session.kernel_specification.name();
    let kernel_language: SharedString = session.kernel_specification.language();

    let fill_fields = || {
        ReplMenuState {
            tooltip: i18n::t!("ui.text.nothing_running").into(),
            icon: IconName::ReplNeutral,
            icon_color: Color::Default,
            icon_is_animating: false,
            popover_disabled: false,
            indicator: None,
            kernel_name: kernel_name.clone(),
            kernel_language: kernel_language.clone(),
            // TODO: Technically not shutdown, but indeterminate
            status: KernelStatus::Shutdown,
            // current_delta: Duration::default(),
        }
    };

    let transitional = |tooltip: SharedString, animating: bool, popover_disabled: bool| ReplMenuState {
        tooltip,
        icon_is_animating: animating,
        popover_disabled,
        icon_color: Color::Muted,
        indicator: Some(Indicator::dot().color(Color::Muted)),
        status: session.kernel.status(),
        ..fill_fields()
    };

    let starting = || {
        transitional(
            i18n::t!("ui.text.kernel_name_is_starting", kernel_name = kernel_name).into(),
            true,
            true,
        )
    };
    let restarting = || transitional(format!("Restarting {kernel_name}").into(), true, true);
    let shutting_down = || {
        transitional(
            i18n::t!("ui.text.kernel_name_is_shutting_down", kernel_name = kernel_name).into(),
            false,
            true,
        )
    };
    let auto_restarting = || transitional(format!("Auto-restarting {kernel_name}").into(), true, true);
    let unknown = || {
        transitional(
            i18n::t!("ui.text.kernel_name_state_unknown", kernel_name = kernel_name).into(),
            false,
            true,
        )
    };
    let other = |state: &str| transitional(format!("{kernel_name} state: {state}").into(), false, true);

    let shutdown = || ReplMenuState {
        tooltip: i18n::t!("ui.text.nothing_running").into(),
        icon: IconName::ReplNeutral,
        icon_color: Color::Default,
        icon_is_animating: false,
        popover_disabled: false,
        indicator: None,
        status: KernelStatus::Shutdown,
        ..fill_fields()
    };

    match &session.kernel {
        Kernel::Restarting => restarting(),
        Kernel::Running(kernel) => match &kernel.execution_state() {
            ExecutionState::Idle => ReplMenuState {
                tooltip: i18n::t!(
                    "ui.text.run_code_on_kernel_name_kernel_language",
                    kernel_name = kernel_name,
                    kernel_language = kernel_language
                )
                .into(),
                indicator: Some(Indicator::dot().color(Color::Success)),
                status: session.kernel.status(),
                ..fill_fields()
            },
            ExecutionState::Busy => ReplMenuState {
                tooltip: format!("Interrupt {kernel_name} ({kernel_language})").into(),
                icon_is_animating: true,
                popover_disabled: false,
                indicator: None,
                status: session.kernel.status(),
                ..fill_fields()
            },
            ExecutionState::Unknown => unknown(),
            ExecutionState::Starting => starting(),
            ExecutionState::Restarting => restarting(),
            ExecutionState::Terminating => shutting_down(),
            ExecutionState::AutoRestarting => auto_restarting(),
            ExecutionState::Dead => shutdown(),
            ExecutionState::Other(state) => other(state),
        },
        Kernel::Starting(_) => starting(),
        Kernel::ErroredLaunch(e) => ReplMenuState {
            tooltip: i18n::t!(
                "ui.text.error_with_kernel_kernel_name_e",
                kernel_name = kernel_name,
                e = e
            )
            .into(),
            popover_disabled: false,
            indicator: Some(Indicator::dot().color(Color::Error)),
            status: session.kernel.status(),
            ..fill_fields()
        },
        Kernel::ShuttingDown => shutting_down(),
        Kernel::Shutdown => shutdown(),
    }
}
