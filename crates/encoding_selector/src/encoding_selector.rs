//! [DonEditor] 인코딩(캐릭터셋) 선택. GPL-3.0-or-later.
//!
//! 상태 표시줄의 인코딩 이름을 누르면 선택 화면이 열리고, 고른 인코딩으로 **디스크의 파일을 강제로 다시 불러온다.**
//! 파일을 이미 연 뒤 글자가 깨져 보일 때(또는 한 파일에 여러 인코딩이 섞여 있어 부분씩 복원할 때) 쓰는 기능이므로,
//! 다시 불러오기는 **절대 저장하지 않으며** 저장은 사용자가 직접 할 때만 현재 선택된 인코딩으로 일어난다.

mod active_buffer_encoding;

pub use active_buffer_encoding::ActiveBufferEncoding;

use anyhow::Result;
use editor::Editor;
use encoding_rs::Encoding;
use fuzzy::{StringMatch, StringMatchCandidate, match_strings};
use gpui::{
    Action, App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, ParentElement, PromptLevel,
    Render, Styled, WeakEntity, Window, actions,
};
use language::Buffer;
use picker::{Picker, PickerDelegate};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use ui::{HighlightedLabel, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt;
use workspace::{ModalView, Toast, Workspace, notifications::NotificationId};

actions!(
    encoding_selector,
    [
        /// Toggles the encoding selector, which reloads the file from disk with the chosen encoding.
        Toggle
    ]
);

/// 현재 파일을 지정한 인코딩으로 디스크에서 다시 불러온다. 디스크의 파일은 바꾸지 않는다.
/// 선택 화면을 거치지 않고 MCP·키 바인딩에서 직접 쓸 수 있다.
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = encoding_selector)]
#[serde(deny_unknown_fields)]
pub struct ReloadWithEncoding {
    /// 인코딩 이름. 예: `utf-8`, `euc-kr`, `shift_jis`, `utf-16le`, `windows-1252`
    #[serde(default)]
    pub encoding: String,
    /// `true` 이면 저장하지 않은 변경이 있어도 확인 없이 버리고 다시 불러온다.
    #[serde(default)]
    pub force: bool,
}

/// 앱 시작 때 선택 화면과 다시 불러오기 액션을 작업 공간마다 등록한다.
pub fn init(cx: &mut App) {
    cx.observe_new(EncodingSelector::register).detach();
}

/// 인코딩 선택 화면 (모달)
pub struct EncodingSelector {
    picker: Entity<Picker<EncodingSelectorDelegate>>,
}

impl EncodingSelector {
    fn register(workspace: &mut Workspace, _window: Option<&mut Window>, _: &mut Context<Workspace>) {
        workspace.register_action(|workspace, _: &Toggle, window, cx| {
            Self::toggle(workspace, window, cx);
        });
        workspace.register_action(|workspace, action: &ReloadWithEncoding, window, cx| {
            Self::reload_active_buffer(workspace, action, window, cx);
        });
    }

    /// 활성 편집기가 보여 주는 버퍼를 돌려준다 (편집기가 아니면 `None`).
    fn active_buffer(workspace: &Workspace, cx: &App) -> Option<Entity<Buffer>> {
        let (_, buffer, _) = workspace
            .active_item(cx)?
            .act_as::<Editor>(cx)?
            .read(cx)
            .active_excerpt(cx)?;
        Some(buffer)
    }

    /// 선택 화면을 열거나 닫는다.
    pub fn toggle(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) -> Option<()> {
        let buffer = Self::active_buffer(workspace, cx)?;
        let workspace_handle = workspace.weak_handle();
        workspace.toggle_modal(window, cx, move |window, cx| {
            EncodingSelector::new(buffer, workspace_handle, window, cx)
        });
        Some(())
    }

    /// `ReloadWithEncoding` 액션: 이름으로 인코딩을 찾아 활성 파일을 다시 불러온다.
    fn reload_active_buffer(
        workspace: &mut Workspace,
        action: &ReloadWithEncoding,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        let Some(buffer) = Self::active_buffer(workspace, cx) else {
            return;
        };
        let Some(encoding) = don_charset::encoding_for_label(&action.encoding) else {
            show_toast(
                workspace.weak_handle(),
                format!("Unknown encoding: {}", action.encoding),
                cx,
            );
            return;
        };
        reload_buffer_with_encoding(buffer, workspace.weak_handle(), encoding, action.force, window, cx);
    }

    fn new(
        buffer: Entity<Buffer>,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = EncodingSelectorDelegate::new(cx.entity().downgrade(), buffer, workspace, cx);
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx));
        Self { picker }
    }
}

impl Render for EncodingSelector {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("EncodingSelector")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

impl Focusable for EncodingSelector {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl EventEmitter<DismissEvent> for EncodingSelector {}
impl ModalView for EncodingSelector {}

/// 선택 화면의 항목 한 줄
struct EncodingCandidate {
    encoding: &'static Encoding,
    /// 검색과 표시에 쓰는 글자. 예: `EUC-KR — 한국어`
    label: String,
}

pub struct EncodingSelectorDelegate {
    encoding_selector: WeakEntity<EncodingSelector>,
    buffer: Entity<Buffer>,
    workspace: WeakEntity<Workspace>,
    /// 지금 버퍼가 쓰는 인코딩 (목록에서 체크 표시)
    current: &'static Encoding,
    candidates: Vec<EncodingCandidate>,
    string_candidates: Vec<StringMatchCandidate>,
    matches: Vec<StringMatch>,
    selected_index: usize,
}

impl EncodingSelectorDelegate {
    fn new(
        encoding_selector: WeakEntity<EncodingSelector>,
        buffer: Entity<Buffer>,
        workspace: WeakEntity<Workspace>,
        cx: &App,
    ) -> Self {
        let current = buffer.read(cx).encoding();
        let mut candidates = Vec::new();
        // 자동 감지가 목록에 없는 인코딩을 고른 경우에도 현재 인코딩을 맨 위에 보여 준다
        if !don_charset::supported_charsets()
            .iter()
            .any(|entry| entry.encoding == current)
        {
            candidates.push(EncodingCandidate {
                encoding: current,
                label: current.name().to_string(),
            });
        }
        // 지원 목록의 모든 인코딩을 "이름 — 언어 계열" 형태로 만든다
        for entry in don_charset::supported_charsets() {
            candidates.push(EncodingCandidate {
                encoding: entry.encoding,
                label: format!("{} — {}", entry.encoding.name(), i18n::t(entry.group.i18n_key())),
            });
        }
        let string_candidates = candidates
            .iter()
            .enumerate()
            .map(|(id, candidate)| StringMatchCandidate::new(id, &candidate.label))
            .collect();

        Self {
            encoding_selector,
            buffer,
            workspace,
            current,
            candidates,
            string_candidates,
            matches: Vec::new(),
            selected_index: 0,
        }
    }
}

impl PickerDelegate for EncodingSelectorDelegate {
    type ListItem = ListItem;

    fn placeholder_text(&self, _window: &mut Window, _cx: &mut App) -> Arc<str> {
        i18n::t!("encoding_selector.placeholder").into()
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        if let Some(mat) = self.matches.get(self.selected_index) {
            let encoding = self.candidates[mat.candidate_id].encoding;
            let buffer = self.buffer.clone();
            let workspace = self.workspace.clone();
            // 선택 화면을 닫은 뒤 확인 창이 떠야 하므로 다음 틱에서 실행한다
            cx.defer_in(window, move |_, window, cx| {
                reload_buffer_with_encoding(buffer, workspace, encoding, false, window, cx);
            });
        }
        self.dismissed(window, cx);
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.encoding_selector
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, ix: usize, _window: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = ix;
    }

    fn update_matches(&mut self, query: String, window: &mut Window, cx: &mut Context<Picker<Self>>) -> gpui::Task<()> {
        let background = cx.background_executor().clone();
        let candidates = self.string_candidates.clone();
        cx.spawn_in(window, async move |this, cx| {
            let matches = if query.is_empty() {
                // 검색어가 없으면 모든 항목을 목록 순서대로 보여 준다
                candidates
                    .into_iter()
                    .enumerate()
                    .map(|(index, candidate)| StringMatch {
                        candidate_id: index,
                        string: candidate.string,
                        positions: Vec::new(),
                        score: 0.0,
                    })
                    .collect()
            } else {
                match_strings(&candidates, &query, false, true, 100, &Default::default(), background).await
            };

            this.update(cx, |this, cx| {
                let delegate = &mut this.delegate;
                delegate.matches = matches;
                // 처음 열 때는 현재 인코딩 항목에 커서를 둔다
                if query_is_initial(&delegate.matches, &delegate.candidates) {
                    delegate.selected_index = delegate
                        .matches
                        .iter()
                        .position(|mat| delegate.candidates[mat.candidate_id].encoding == delegate.current)
                        .unwrap_or(0);
                } else {
                    delegate.selected_index = delegate.selected_index.min(delegate.matches.len().saturating_sub(1));
                }
                cx.notify();
            })
            .log_err();
        })
    }

    fn render_match(
        &self,
        ix: usize,
        selected: bool,
        _: &mut Window,
        _cx: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let mat = self.matches.get(ix)?;
        let candidate = &self.candidates[mat.candidate_id];
        let mut item = ListItem::new(ix)
            .inset(true)
            .spacing(ListItemSpacing::Sparse)
            .toggle_state(selected)
            .child(HighlightedLabel::new(candidate.label.clone(), mat.positions.clone()));
        // 현재 버퍼가 쓰는 인코딩에는 체크 표시를 한다
        if candidate.encoding == self.current {
            item = item.end_slot(Icon::new(IconName::Check).color(Color::Muted));
        }
        Some(item)
    }
}

/// 검색어가 없는 처음 상태(모든 항목이 목록 순서 그대로 나열됨)인지 판단한다.
fn query_is_initial(matches: &[StringMatch], candidates: &[EncodingCandidate]) -> bool {
    matches.len() == candidates.len()
        && matches
            .iter()
            .enumerate()
            .all(|(index, mat)| mat.candidate_id == index && mat.positions.is_empty())
}

/// 작업 공간 모서리에 잠깐 뜨는 안내 메시지를 보여 준다.
fn show_toast(workspace: WeakEntity<Workspace>, message: String, cx: &mut App) {
    struct EncodingToast;
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<EncodingToast>(), message).autohide(),
                cx,
            );
        })
        .log_err();
}

/// 버퍼의 파일을 지정한 인코딩으로 디스크에서 다시 불러온다.
///
/// - 저장하지 않은 변경이 있으면(그리고 `force` 가 아니면) 버릴지 **한 번 확인**한다. 취소하면 아무것도 바꾸지 않는다.
/// - 다시 불러오면 파일을 새로 연 것과 같이 실행 취소 기록이 지워지고 변경 없음 상태가 된다.
/// - **디스크의 파일은 건드리지 않는다.** 이후 저장은 여기서 고른 인코딩으로 한다.
pub fn reload_buffer_with_encoding(
    buffer: Entity<Buffer>,
    workspace: WeakEntity<Workspace>,
    encoding: &'static Encoding,
    force: bool,
    window: &mut Window,
    cx: &mut App,
) {
    // 이 컴퓨터에 저장된 파일만 다시 불러올 수 있다 (제목 없는 새 파일·원격 파일은 제외)
    let has_local_file = buffer.read(cx).file().is_some_and(|file| file.as_local().is_some());
    if !has_local_file {
        show_toast(workspace, i18n::t!("encoding_selector.error.no_file").to_string(), cx);
        return;
    }

    let name = encoding.name();
    // 저장하지 않은 변경이 있으면 버려도 되는지 묻는다
    let answer = (!force && buffer.read(cx).is_dirty()).then(|| {
        window.prompt(
            PromptLevel::Warning,
            &i18n::t!("encoding_selector.confirm.title", encoding = name),
            Some(i18n::t!("encoding_selector.confirm.detail")),
            &[i18n::t!("encoding_selector.confirm.reload"), i18n::t!("common.cancel")],
            cx,
        )
    });

    window
        .spawn(cx, async move |cx| -> Result<()> {
            if let Some(answer) = answer
                && answer.await != Ok(0)
            {
                return Ok(());
            }
            let reload = buffer.update(cx, |buffer, cx| buffer.reload_with_encoding(encoding, cx))?;
            let message = match reload.await {
                Ok(false) => i18n::t!("encoding_selector.reloaded", encoding = name),
                Ok(true) => i18n::t!("encoding_selector.warning.decode_errors", encoding = name),
                Err(error) => i18n::t!("encoding_selector.error.reload_failed", encoding = name, error = error),
            };
            cx.update(|_, cx| show_toast(workspace, message, cx))?;
            Ok(())
        })
        .detach_and_log_err(cx);
}
