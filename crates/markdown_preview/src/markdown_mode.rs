//! [DonEditor] Markdown 파일 탭의 미리보기/편집 모드. GPL-3.0-or-later.
//!
//! Markdown 파일을 열면 같은 탭 안에서 미리보기 화면부터 보여 주고, 툴바(현재 단락 제목 줄) 오른쪽의
//! 토글 버튼이나 `markdown::TogglePreviewMode` 액션으로 편집 화면과 오간다.
//! 탭 자체는 언제나 원래의 편집기 항목이므로 저장·변경 표시·닫기 확인·세션 복원·MCP 도구가 그대로 동작한다.

use editor::scroll::ScrollAnchor;
use editor::{Editor, MultiBufferOffset, ToOffset as _};
use gpui::{App, Context, Entity, EventEmitter, Focusable, IntoElement, Render, Subscription, WeakEntity, Window};
use ui::{ToggleButtonGroup, ToggleButtonGroupStyle, ToggleButtonSimple, Tooltip, prelude::*};
use workspace::item::{ItemEvent, ItemHandle};
use workspace::{ToolbarItemEvent, ToolbarItemLocation, ToolbarItemView, Workspace};

use crate::TogglePreviewMode;
use crate::markdown_preview_view::{MarkdownPreviewMode, MarkdownPreviewView};

/// Markdown 언어 이름 (`crates/languages/src/markdown/config.toml` 의 name)
const MARKDOWN_LANGUAGE_NAME: &str = "Markdown";

/// 작업 공간마다 전환 액션과 "열 때 미리보기로 시작" 처리를 등록한다.
pub(crate) fn register(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    workspace.register_action(|workspace, _: &TogglePreviewMode, window, cx| {
        let Some(editor) = workspace.active_item(cx).and_then(|item| item.downcast::<Editor>()) else {
            return;
        };
        if !is_markdown_editor(editor.read(cx), cx) && !is_preview_mode(&editor, cx) {
            return;
        }
        toggle_preview_mode(&editor, workspace.weak_handle(), window, cx);
    });

    // 탭이 추가될 때(파일 열기·세션 복원·명령줄/파일 연결로 열기) Markdown 파일이면 미리보기로 시작한다.
    let workspace_entity = cx.entity();
    cx.subscribe_in(
        &workspace_entity,
        window,
        |workspace, _, event: &workspace::Event, window, cx| {
            if let workspace::Event::ItemAdded { item } = event {
                start_in_preview_if_markdown_file(workspace, item.as_ref(), window, cx);
            }
        },
    )
    .detach();
}

/// 새로 추가된 탭이 처음 여는 Markdown 파일 편집기이면 미리보기 모드로 바꾼다.
fn start_in_preview_if_markdown_file(
    workspace: &mut Workspace,
    item: &dyn ItemHandle,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let Some(editor) = item.downcast::<Editor>() else {
        return;
    };
    // 이미 모드를 정한 편집기(다른 분할로 옮긴 탭)나 분할 복제본은 사용자가 고른 모드를 유지한다.
    if editor.read(cx).alternate_view_initialized() {
        return;
    }
    // Markdown 이 아니면 건드리지 않는다 (다른 미리보기 종류, 예: Typst 가 같은 표시를 쓰므로 가로채면 안 된다)
    if !is_markdown_editor(editor.read(cx), cx) {
        return;
    }
    // Markdown 은 여기서 모드 결정을 마친다. 내용이 없으면 편집 모드로 둔다.
    editor.update(cx, |editor, _| editor.mark_alternate_view_initialized());
    if !is_markdown_file_with_content(editor.read(cx), cx) {
        return;
    }
    set_preview_mode(&editor, true, workspace.weak_handle(), window, cx);
}

/// 디스크 파일에서 연 Markdown 문서이고 내용이 있으면 `true`.
/// 빈 파일(새로 만들 파일 포함)은 미리보기할 내용이 없으므로 편집 모드로 연다.
fn is_markdown_file_with_content(editor: &Editor, cx: &App) -> bool {
    let Some(buffer) = editor.buffer().read(cx).as_singleton() else {
        return false;
    };
    let buffer = buffer.read(cx);
    buffer.file().is_some() && !buffer.is_empty() && is_markdown_editor(editor, cx)
}

/// 단일 버퍼 전체 편집기이고, 그 버퍼가 Markdown 이면 `true`.
/// 버퍼 언어가 아직 로드되지 않았으면(파일을 막 연 직후) 사용자 파일 형식 설정을 반영한 파일 경로로 판단한다.
pub fn is_markdown_editor(editor: &Editor, cx: &App) -> bool {
    if !editor.mode().is_full() {
        return false;
    }
    let Some(buffer) = editor.buffer().read(cx).as_singleton() else {
        return false;
    };
    let buffer = buffer.read(cx);
    if let Some(language) = buffer.language() {
        return language.name().as_ref() == MARKDOWN_LANGUAGE_NAME;
    }
    let (Some(file), Some(project)) = (buffer.file(), editor.project()) else {
        return false;
    };
    project
        .read(cx)
        .languages()
        .language_for_file(file, None, cx)
        .is_some_and(|language| language.name().as_ref() == MARKDOWN_LANGUAGE_NAME)
}

/// 편집기 탭 안에 내장된 미리보기를 돌려준다 (편집 모드면 `None`).
fn embedded_preview(editor: &Entity<Editor>, cx: &App) -> Option<Entity<MarkdownPreviewView>> {
    editor
        .read(cx)
        .alternate_view()?
        .clone()
        .downcast::<MarkdownPreviewView>()
        .ok()
        .filter(|preview| preview.read(cx).is_embedded())
}

/// 편집기가 지금 미리보기 모드이면 `true`.
pub fn is_preview_mode(editor: &Entity<Editor>, cx: &App) -> bool {
    embedded_preview(editor, cx).is_some()
}

/// 미리보기 모드와 편집 모드를 서로 바꾸고 편집기 탭에 초점을 준다.
pub fn toggle_preview_mode(
    editor: &Entity<Editor>,
    workspace: WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    let enabled = !is_preview_mode(editor, cx);
    set_preview_mode(editor, enabled, workspace, window, cx);
    let focus_handle = editor.read(cx).focus_handle(cx);
    window.focus(&focus_handle, cx);
}

/// 미리보기 모드(`true`) 또는 편집 모드(`false`)로 바꾼다. 이미 그 모드이면 아무것도 하지 않는다.
/// 모드를 바꿀 때 보던 위치를 이어서 보여 준다.
pub fn set_preview_mode(
    editor: &Entity<Editor>,
    enabled: bool,
    workspace: WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    if is_preview_mode(editor, cx) == enabled {
        return;
    }

    if enabled {
        let Some(language_registry) = editor
            .read(cx)
            .project()
            .map(|project| project.read(cx).languages().clone())
        else {
            return;
        };
        // 편집 화면에서 보이던 위치: 특정 줄로 열기처럼 아직 적용되지 않은 자동 스크롤이 있으면 커서, 아니면 화면 맨 윗줄
        let source_offset = {
            let editor = editor.read(cx);
            let snapshot = editor.buffer().read(cx).snapshot(cx);
            let anchor = if editor.scroll_manager.has_pending_autoscroll() {
                editor.selections.newest_anchor().head()
            } else {
                editor.scroll_manager.anchor().anchor
            };
            anchor.to_offset(&snapshot).0
        };
        let preview = MarkdownPreviewView::new(
            MarkdownPreviewMode::Embedded,
            editor.clone(),
            workspace,
            language_registry,
            window,
            cx,
        );
        preview.update(cx, |preview, _| {
            preview.scroll_to_source_offset_after_parse(source_offset)
        });
        editor.update(cx, |editor, cx| editor.set_alternate_view(Some(preview.into()), cx));
    } else {
        // 미리보기 맨 위에 보이던 블록을 편집 화면 맨 위에 둔다
        let top_offset = embedded_preview(editor, cx).and_then(|preview| preview.read(cx).top_visible_source_offset());
        editor.update(cx, |editor, cx| {
            if let Some(offset) = top_offset {
                let snapshot = editor.buffer().read(cx).snapshot(cx);
                let anchor = snapshot.anchor_before(MultiBufferOffset(offset).min(snapshot.len()));
                // 미리보기 중에 쌓인 자동 스크롤 요청이 방금 정한 위치를 덮어쓰지 않게 비운다
                editor.scroll_manager.take_autoscroll_request();
                editor.set_scroll_anchor(
                    ScrollAnchor {
                        offset: Default::default(),
                        anchor,
                    },
                    window,
                    cx,
                );
            }
            editor.set_alternate_view(None, cx);
        });
    }
}

/// 툴바(현재 단락 제목 줄) 오른쪽의 "미리보기 | 편집" 토글 버튼. Markdown 편집기 탭에서만 보인다.
pub struct MarkdownModeToggle {
    workspace: WeakEntity<Workspace>,
    active_editor: Option<WeakEntity<Editor>>,
    _subscription: Option<Subscription>,
}

impl MarkdownModeToggle {
    pub fn new(workspace: &Workspace) -> Self {
        Self {
            workspace: workspace.weak_handle(),
            active_editor: None,
            _subscription: None,
        }
    }

    /// 현재 탭에 맞는 툴바 위치. Markdown 편집기이거나 미리보기 중이면 오른쪽에 보인다.
    fn location(&self, cx: &App) -> ToolbarItemLocation {
        let Some(editor) = self.active_editor.as_ref().and_then(|editor| editor.upgrade()) else {
            return ToolbarItemLocation::Hidden;
        };
        if is_preview_mode(&editor, cx) || is_markdown_editor(editor.read(cx), cx) {
            ToolbarItemLocation::PrimaryRight
        } else {
            ToolbarItemLocation::Hidden
        }
    }
}

impl EventEmitter<ToolbarItemEvent> for MarkdownModeToggle {}

impl Render for MarkdownModeToggle {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(editor) = self.active_editor.as_ref().and_then(|editor| editor.upgrade()) else {
            return div().into_any_element();
        };
        let preview_mode = is_preview_mode(&editor, cx);

        // 버튼마다 대상 편집기와 작업 공간의 약한 참조를 들고, 누르면 해당 모드로 바꾼다
        let mode_button = |label: &'static str, tooltip: &'static str, enabled: bool| {
            let editor = editor.downgrade();
            let workspace = self.workspace.clone();
            ToggleButtonSimple::new(label, move |_, window, cx| {
                if let Some(editor) = editor.upgrade() {
                    set_preview_mode(&editor, enabled, workspace.clone(), window, cx);
                    let focus_handle = editor.read(cx).focus_handle(cx);
                    window.focus(&focus_handle, cx);
                }
            })
            .tooltip(Tooltip::text(tooltip))
        };

        ToggleButtonGroup::single_row(
            "markdown-preview-mode",
            [
                mode_button(
                    i18n::t!("markdown_preview.mode.preview"),
                    i18n::t!("markdown_preview.mode.preview_tooltip"),
                    true,
                ),
                mode_button(
                    i18n::t!("markdown_preview.mode.edit"),
                    i18n::t!("markdown_preview.mode.edit_tooltip"),
                    false,
                ),
            ],
        )
        .style(ToggleButtonGroupStyle::Outlined)
        .auto_width()
        .selected_index(if preview_mode { 0 } else { 1 })
        .into_any_element()
    }
}

impl ToolbarItemView for MarkdownModeToggle {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ToolbarItemLocation {
        cx.notify();
        self.active_editor = None;
        self._subscription = None;

        let Some(item) = active_pane_item else {
            return ToolbarItemLocation::Hidden;
        };
        let Some(editor) = item.downcast::<Editor>() else {
            return ToolbarItemLocation::Hidden;
        };

        // 언어 로드 완료·모드 전환·파일 이름 변경은 모두 UpdateBreadcrumbs 로 전달되므로 그때 위치를 다시 계산한다.
        let this = cx.entity().downgrade();
        self._subscription = Some(item.subscribe_to_item_events(
            window,
            cx,
            Box::new(move |event, _, cx| {
                if let ItemEvent::UpdateBreadcrumbs = event {
                    this.update(cx, |this, cx| {
                        let location = this.location(cx);
                        cx.emit(ToolbarItemEvent::ChangeLocation(location));
                        cx.notify();
                    })
                    .ok();
                }
            }),
        ));
        self.active_editor = Some(editor.downgrade());
        self.location(cx)
    }
}
