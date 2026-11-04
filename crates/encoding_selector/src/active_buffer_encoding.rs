//! [DonEditor] 상태 표시줄의 인코딩 표시. 언어 이름 옆에 현재 파일의 인코딩을 보여 주고,
//! 누르면 다른 인코딩으로 다시 불러오는 선택 화면을 연다.

use editor::Editor;
use encoding_rs::{Encoding, UTF_8};
use gpui::{
    Context, Entity, EntityId, IntoElement, ParentElement, Render, Styled, Subscription, WeakEntity, Window, div,
};
use language::Buffer;
use ui::{Button, ButtonCommon, Clickable, Tooltip};
use workspace::{
    StatusBarSettings, StatusItemView, Workspace,
    item::{ItemHandle, Settings},
};

use crate::{EncodingSelector, Toggle};

pub struct ActiveBufferEncoding {
    active_encoding: Option<&'static Encoding>,
    workspace: WeakEntity<Workspace>,
    _observe_active_editor: Option<Subscription>,
    /// 인코딩이 바뀌는 것(다른 인코딩으로 다시 불러오기, 저장 등)을 바로 반영하려고 버퍼도 지켜본다
    _observe_active_buffer: Option<Subscription>,
    /// 지금 지켜보는 버퍼. 편집기가 알림을 보낼 때마다 구독을 다시 만들지 않으려고 기억해 둔다.
    observed_buffer: Option<EntityId>,
    has_bom: bool,
}

impl ActiveBufferEncoding {
    pub fn new(workspace: &Workspace) -> Self {
        Self {
            active_encoding: None,
            workspace: workspace.weak_handle(),
            _observe_active_editor: None,
            _observe_active_buffer: None,
            observed_buffer: None,
            has_bom: false,
        }
    }

    /// 활성 편집기의 버퍼에서 인코딩과 BOM 여부를 읽어 온다. 버퍼가 바뀌었으면 그 버퍼를 새로 지켜본다.
    fn update_encoding(&mut self, editor: Entity<Editor>, _: &mut Window, cx: &mut Context<Self>) {
        self.active_encoding = None;

        let editor = editor.read(cx);
        if let Some((_, buffer, _)) = editor.active_excerpt(cx) {
            if self.observed_buffer != Some(buffer.entity_id()) {
                self.observed_buffer = Some(buffer.entity_id());
                self._observe_active_buffer = Some(cx.observe(&buffer, |this, buffer: Entity<Buffer>, cx| {
                    let buffer = buffer.read(cx);
                    this.active_encoding = Some(buffer.encoding());
                    this.has_bom = buffer.has_bom();
                    cx.notify();
                }));
            }
            let buffer = buffer.read(cx);
            self.active_encoding = Some(buffer.encoding());
            self.has_bom = buffer.has_bom();
        }

        cx.notify();
    }
}

impl Render for ActiveBufferEncoding {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(active_encoding) = self.active_encoding else {
            return div().hidden();
        };

        let display_option = StatusBarSettings::get_global(cx).active_encoding_button;
        let is_utf8 = active_encoding == UTF_8;
        if !display_option.should_show(is_utf8, self.has_bom) {
            return div().hidden();
        }

        let mut text = active_encoding.name().to_string();
        if self.has_bom {
            text.push_str(" (BOM)");
        }

        let icon_size = StatusBarSettings::get_global(cx).icon_size;

        div().child(
            Button::new("change-encoding", text)
                .label_size(icon_size.label_size())
                // 누르면 다른 인코딩으로 파일을 다시 불러오는 선택 화면을 연다
                .on_click(cx.listener(|this, _, window, cx| {
                    if let Some(workspace) = this.workspace.upgrade() {
                        workspace.update(cx, |workspace, cx| {
                            EncodingSelector::toggle(workspace, window, cx);
                        });
                    }
                }))
                .tooltip(|_window, cx| Tooltip::for_action(i18n::t!("status.select_encoding"), &Toggle, cx)),
        )
    }
}

impl StatusItemView for ActiveBufferEncoding {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(editor) = active_pane_item.and_then(|item| item.downcast::<Editor>()) {
            self._observe_active_editor = Some(cx.observe_in(&editor, window, Self::update_encoding));
            self.update_encoding(editor, window, cx);
        } else {
            self.active_encoding = None;
            self.has_bom = false;
            self._observe_active_editor = None;
            self._observe_active_buffer = None;
            self.observed_buffer = None;
        }

        cx.notify();
    }
}
