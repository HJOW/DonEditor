//! [DonEditor] 기본 설정 화면의 Agent API 키 입력란. GPL-3.0-or-later.
//! 키는 settings.json에 쓰지 않고 OS 자격 증명 저장소에만 저장하며, 화면에서는 마스킹한다.

use std::rc::Rc;

use agent_panel::{
    credentials::{self, KeySource},
    provider::Provider,
};
use editor::Editor;
use gpui::{App, Entity, Focusable as _, IntoElement, RenderOnce, SharedString, WeakEntity, Window, div};
use ui::{Button, ButtonStyle, prelude::*};

/// 설정 항목의 값 자리에 놓는 표식 타입이다. 실제 키 값은 담지 않고 제공자만 가리킨다.
#[derive(Clone, Copy, PartialEq)]
pub struct AgentApiKeyField(pub Provider);

/// 저장·삭제 결과를 입력란 아래에 보여 주는 메시지와 오류 여부이다.
type Feedback = Option<(SharedString, bool)>;

/// 제공자 하나의 API 키를 입력·저장·삭제하는 화면 요소이다.
#[derive(IntoElement)]
pub struct AgentApiKeyInput {
    provider: Provider,
    tab_index: Option<isize>,
}

impl AgentApiKeyInput {
    /// 지정한 제공자의 키 입력란을 만든다.
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            tab_index: None,
        }
    }

    /// 키보드 탭 이동 순서를 지정한다.
    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = Some(tab_index);
        self
    }
}

/// 결과 메시지를 갱신하고 다시 그리게 한다.
fn set_feedback(feedback: &WeakEntity<Feedback>, value: Feedback, cx: &mut App) {
    feedback
        .update(cx, |feedback, cx| {
            *feedback = value;
            cx.notify();
        })
        .ok();
}

/// 작업 결과를 성공 메시지 또는 오류 메시지로 바꿔 표시한다.
fn report(feedback: WeakEntity<Feedback>, task: gpui::Task<anyhow::Result<()>>, success: &'static str, cx: &mut App) {
    cx.spawn(async move |cx| {
        let value = match task.await {
            Ok(()) => (SharedString::new_static(success), false),
            Err(error) => (SharedString::from(error.to_string()), true),
        };
        cx.update(|cx| set_feedback(&feedback, Some(value), cx)).ok();
    })
    .detach();
}

impl RenderOnce for AgentApiKeyInput {
    /// 입력란·저장/삭제 버튼·현재 등록 상태를 그린다.
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let provider = self.provider;
        let source = credentials::key_source(provider, cx);
        // 저장·삭제할 때마다 세대 번호가 바뀌어 빈 입력란을 새로 만들고 실행 취소 기록도 버린다.
        let generation = credentials::generation(cx);
        let editor: Entity<Editor> = window.use_keyed_state(
            SharedString::from(format!("agent-api-key-{}-{generation}", provider.id())),
            cx,
            |window, cx| {
                let mut editor = Editor::single_line(window, cx);
                editor.set_masked(true, cx);
                editor.set_placeholder_text(i18n::t!("settings.agent.api_key_placeholder"), window, cx);
                editor
            },
        );
        let feedback: Entity<Feedback> = window.use_keyed_state(
            SharedString::from(format!("agent-api-key-feedback-{}", provider.id())),
            cx,
            |_, _| None,
        );
        // Enter와 저장 버튼이 같은 저장 동작을 사용한다.
        let save: Rc<dyn Fn(&mut App)> = {
            let editor = editor.downgrade();
            let feedback = feedback.downgrade();
            Rc::new(move |cx: &mut App| {
                let Some(editor) = editor.upgrade() else {
                    return;
                };
                let key = editor.read(cx).text(cx);
                if key.trim().is_empty() {
                    let message = SharedString::new_static(i18n::t!("agent.error.empty_key"));
                    set_feedback(&feedback, Some((message, true)), cx);
                    return;
                }
                let task = credentials::save(provider, key, cx);
                report(feedback.clone(), task, i18n::t!("agent.key_saved"), cx);
            })
        };
        let status = match source {
            KeySource::Stored => SharedString::new_static(i18n::t!("settings.agent.key_stored")),
            KeySource::Environment => SharedString::from(i18n::t!(
                "settings.agent.key_environment",
                name = provider.environment_key()
            )),
            KeySource::Missing => SharedString::new_static(i18n::t!("settings.agent.key_missing")),
            KeySource::Unknown => SharedString::new_static(i18n::t!("settings.agent.key_checking")),
        };
        let status_color = if source.is_available() {
            Color::Success
        } else {
            Color::Muted
        };
        let theme_colors = cx.theme().colors();
        let id = provider.id();

        // 다른 설정 입력란과 같은 너비 안에서 입력란·버튼·상태 문구를 세로로 쌓아 오른쪽으로 넘치지 않게 한다.
        v_flex()
            .w_64()
            .gap_1()
            .child(
                div()
                    .w_full()
                    .py_1()
                    .px_2()
                    .rounded_md()
                    .border_1()
                    .border_color(theme_colors.border)
                    .bg(theme_colors.editor_background)
                    .when_some(self.tab_index, |this, tab_index| {
                        let focus_handle = editor.focus_handle(cx).tab_index(tab_index).tab_stop(true);
                        this.track_focus(&focus_handle)
                            .focus(|style| style.border_color(theme_colors.border_focused))
                    })
                    .on_action::<menu::Confirm>({
                        let save = save.clone();
                        move |_, _, cx| save(cx)
                    })
                    .child(editor),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new(
                            SharedString::from(format!("agent-save-key-{id}")),
                            i18n::t!("agent.save_key"),
                        )
                        .style(ButtonStyle::Filled)
                        .on_click(move |_, _, cx| save(cx)),
                    )
                    .child(
                        Button::new(
                            SharedString::from(format!("agent-delete-key-{id}")),
                            i18n::t!("agent.delete_key"),
                        )
                        // 환경 변수 키는 OS 저장소에 없으므로 저장된 키가 있을 때만 삭제할 수 있다.
                        .disabled(source != KeySource::Stored)
                        .on_click({
                            let feedback = feedback.downgrade();
                            move |_, _, cx| {
                                let task = credentials::delete(provider, cx);
                                report(feedback.clone(), task, i18n::t!("agent.key_deleted"), cx);
                            }
                        }),
                    ),
            )
            .child(Label::new(status).size(LabelSize::Small).color(status_color))
            .when_some(feedback.read(cx).clone(), |this, (message, is_error)| {
                this.child(Label::new(message).size(LabelSize::Small).color(if is_error {
                    Color::Error
                } else {
                    Color::Success
                }))
            })
    }
}
