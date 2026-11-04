//! [DonEditor] 도구 실행 전 변경 내용을 읽고 승인하는 모달.
use editor::Editor;
use futures::channel::oneshot;
use gpui::{DismissEvent, Entity, EventEmitter, FocusHandle, Focusable};
use ui::{Button, ButtonCommon, Clickable, prelude::*};
use workspace::ModalView;

pub struct ToolApproval {
    preview: Entity<Editor>,
    answer: Option<oneshot::Sender<bool>>,
}

impl ToolApproval {
    /// 전체 변경안을 읽기 전용 편집기에 표시한다. 닫거나 Esc를 누르면 거절로 처리된다.
    pub fn new(detail: String, answer: oneshot::Sender<bool>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let preview = cx.new(|cx| {
            let mut editor = Editor::multi_line(window, cx);
            editor.set_text(detail, window, cx);
            editor.set_read_only(true);
            editor
        });
        Self {
            preview,
            answer: Some(answer),
        }
    }

    /// 승인/거절 결과를 정확히 한 번 전달하고 모달을 닫는다.
    fn finish(&mut self, accepted: bool, cx: &mut Context<Self>) {
        if let Some(answer) = self.answer.take() {
            let _ = answer.send(accepted);
        }
        cx.emit(DismissEvent);
    }
}

impl Focusable for ToolApproval {
    /// 키보드 탐색을 미리보기 편집기로 보낸다.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.preview.focus_handle(cx)
    }
}
impl EventEmitter<DismissEvent> for ToolApproval {}
impl ModalView for ToolApproval {}
impl Render for ToolApproval {
    /// 큰 변경도 스크롤해서 전부 검토할 수 있는 승인 창을 그린다.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // MCP 연결 종료나 Agent 중단으로 수신자가 사라졌으면 남은 승인 창도 닫는다.
        if self.answer.as_ref().is_some_and(|answer| answer.is_canceled()) {
            cx.emit(DismissEvent);
        }
        v_flex()
            .w(px(760.))
            .max_w_full()
            .h(px(560.))
            .p_3()
            .gap_2()
            .child(Label::new(i18n::t!("tools.approval.title")))
            .child(div().flex_1().min_h_0().child(self.preview.clone()))
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("reject", i18n::t!("tools.approval.reject"))
                            .on_click(cx.listener(|this, _, _, cx| this.finish(false, cx))),
                    )
                    .child(
                        Button::new("approve", i18n::t!("tools.approval.accept"))
                            .on_click(cx.listener(|this, _, _, cx| this.finish(true, cx))),
                    ),
            )
    }
}
