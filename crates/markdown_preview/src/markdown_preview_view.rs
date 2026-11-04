use std::cmp::min;
use std::sync::Arc;
use std::time::Duration;
use std::{ops::Range, path::PathBuf};

use anyhow::Result;
use editor::scroll::Autoscroll;
use editor::{Editor, EditorEvent, MultiBufferOffset, SelectionEffects};
use gpui::{
    App, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement, IsZero,
    ListOffset, ListState, ParentElement, Render, RetainAllImageCache, Styled, Subscription, Task, WeakEntity, Window,
    list,
};
use language::LanguageRegistry;
use settings::Settings;
use theme::ThemeSettings;
use ui::{WithScrollbar, prelude::*};
use workspace::item::{Item, ItemHandle};
use workspace::{Pane, Workspace};

use crate::markdown_elements::{
    MarkdownParagraphChunk, ParsedMarkdownElement, ParsedMarkdownHeading, ParsedMarkdownText,
};
use crate::markdown_renderer::{CheckboxClickedEvent, MermaidState};
use crate::{
    OpenFollowingPreview, OpenPreview, OpenPreviewToTheSide, ScrollPageDown, ScrollPageUp,
    markdown_elements::ParsedMarkdown,
    markdown_parser::parse_markdown,
    markdown_renderer::{RenderContext, render_markdown_block},
};
use crate::{ScrollDown, ScrollDownByItem, ScrollToBottom, ScrollToTop, ScrollUp, ScrollUpByItem};

const REPARSE_DEBOUNCE: Duration = Duration::from_millis(200);

pub struct MarkdownPreviewView {
    workspace: WeakEntity<Workspace>,
    image_cache: Entity<RetainAllImageCache>,
    active_editor: Option<EditorState>,
    focus_handle: FocusHandle,
    contents: Option<ParsedMarkdown>,
    selected_block: usize,
    list_state: ListState,
    language_registry: Arc<LanguageRegistry>,
    mermaid_state: MermaidState,
    parsing_markdown_task: Option<Task<Result<()>>>,
    mode: MarkdownPreviewMode,
    /// [DonEditor] 다음 구문 분석이 끝나면 이 원본 위치(바이트 오프셋)를 담은 블록이 맨 위에 오도록 스크롤한다.
    pending_scroll_to_source_offset: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkdownPreviewMode {
    /// The preview will always show the contents of the provided editor.
    Default,
    /// The preview will "follow" the currently active editor.
    Follow,
    /// [DonEditor] 편집기 탭 안에 내장되어 편집기 본문 대신 그려지는 미리보기 (미리보기/편집 모드 전환용).
    Embedded,
}

struct EditorState {
    // [DonEditor] 내장 미리보기는 편집기가 이 화면을 소유하므로, 편집기를 약한 참조로 가져야 순환 참조가 생기지 않는다.
    editor: WeakEntity<Editor>,
    /// 별도 탭 미리보기는 원본 편집기 탭을 닫아도 계속 보이도록 강한 참조를 함께 보관한다. 내장 미리보기는 `None`.
    _keep_alive: Option<Entity<Editor>>,
    _subscription: Subscription,
}

impl MarkdownPreviewView {
    pub fn register(workspace: &mut Workspace, _window: &mut Window, _cx: &mut Context<Workspace>) {
        workspace.register_action(move |workspace, _: &OpenPreview, window, cx| {
            if let Some(editor) = Self::resolve_active_item_as_markdown_editor(workspace, cx) {
                let view = Self::create_markdown_view(workspace, editor.clone(), window, cx);
                workspace.active_pane().update(cx, |pane, cx| {
                    if let Some(existing_view_idx) = Self::find_existing_independent_preview_item_idx(pane, &editor, cx)
                    {
                        pane.activate_item(existing_view_idx, true, true, window, cx);
                    } else {
                        pane.add_item(Box::new(view.clone()), true, true, None, window, cx)
                    }
                });
                cx.notify();
            }
        });

        workspace.register_action(move |workspace, _: &OpenPreviewToTheSide, window, cx| {
            if let Some(editor) = Self::resolve_active_item_as_markdown_editor(workspace, cx) {
                let view = Self::create_markdown_view(workspace, editor.clone(), window, cx);
                let pane = workspace
                    .find_pane_in_direction(workspace::SplitDirection::Right, cx)
                    .unwrap_or_else(|| {
                        workspace.split_pane(
                            workspace.active_pane().clone(),
                            workspace::SplitDirection::Right,
                            window,
                            cx,
                        )
                    });
                pane.update(cx, |pane, cx| {
                    if let Some(existing_view_idx) = Self::find_existing_independent_preview_item_idx(pane, &editor, cx)
                    {
                        pane.activate_item(existing_view_idx, true, true, window, cx);
                    } else {
                        pane.add_item(Box::new(view.clone()), false, false, None, window, cx)
                    }
                });
                editor.focus_handle(cx).focus(window, cx);
                cx.notify();
            }
        });

        workspace.register_action(move |workspace, _: &OpenFollowingPreview, window, cx| {
            if let Some(editor) = Self::resolve_active_item_as_markdown_editor(workspace, cx) {
                // Check if there's already a following preview
                let existing_follow_view_idx = {
                    let active_pane = workspace.active_pane().read(cx);
                    active_pane
                        .items_of_type::<MarkdownPreviewView>()
                        .find(|view| view.read(cx).mode == MarkdownPreviewMode::Follow)
                        .and_then(|view| active_pane.index_for_item(&view))
                };

                if let Some(existing_follow_view_idx) = existing_follow_view_idx {
                    workspace.active_pane().update(cx, |pane, cx| {
                        pane.activate_item(existing_follow_view_idx, true, true, window, cx);
                    });
                } else {
                    let view = Self::create_following_markdown_view(workspace, editor, window, cx);
                    workspace.active_pane().update(cx, |pane, cx| {
                        pane.add_item(Box::new(view.clone()), true, true, None, window, cx)
                    });
                }
                cx.notify();
            }
        });
    }

    fn find_existing_independent_preview_item_idx(pane: &Pane, editor: &Entity<Editor>, cx: &App) -> Option<usize> {
        pane.items_of_type::<MarkdownPreviewView>()
            .find(|view| {
                let view_read = view.read(cx);
                // Only look for independent (Default mode) previews, not Follow previews
                view_read.mode == MarkdownPreviewMode::Default
                    && view_read
                        .active_editor
                        .as_ref()
                        .is_some_and(|active_editor| active_editor.editor.entity_id() == editor.entity_id())
            })
            .and_then(|view| pane.index_for_item(&view))
    }

    pub fn resolve_active_item_as_markdown_editor(
        workspace: &Workspace,
        cx: &mut Context<Workspace>,
    ) -> Option<Entity<Editor>> {
        if let Some(editor) = workspace.active_item(cx).and_then(|item| item.act_as::<Editor>(cx))
            && Self::is_markdown_file(&editor, cx)
        {
            return Some(editor);
        }
        None
    }

    fn create_markdown_view(
        workspace: &mut Workspace,
        editor: Entity<Editor>,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Entity<MarkdownPreviewView> {
        let language_registry = workspace.project().read(cx).languages().clone();
        let workspace_handle = workspace.weak_handle();
        MarkdownPreviewView::new(
            MarkdownPreviewMode::Default,
            editor,
            workspace_handle,
            language_registry,
            window,
            cx,
        )
    }

    fn create_following_markdown_view(
        workspace: &mut Workspace,
        editor: Entity<Editor>,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Entity<MarkdownPreviewView> {
        let language_registry = workspace.project().read(cx).languages().clone();
        let workspace_handle = workspace.weak_handle();
        MarkdownPreviewView::new(
            MarkdownPreviewMode::Follow,
            editor,
            workspace_handle,
            language_registry,
            window,
            cx,
        )
    }

    pub fn new(
        mode: MarkdownPreviewMode,
        active_editor: Entity<Editor>,
        workspace: WeakEntity<Workspace>,
        language_registry: Arc<LanguageRegistry>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        // [DonEditor] 내장 미리보기는 편집기의 포커스 핸들을 같이 쓴다. 작업 공간이 편집기 탭에 초점을 주면
        // 이 화면이 초점을 받아 미리보기 키 바인딩(스크롤 등)이 동작하고, 편집 모드로 돌아가도 초점이 유지된다.
        let shared_focus_handle =
            (mode == MarkdownPreviewMode::Embedded).then(|| active_editor.read(cx).focus_handle(cx));
        cx.new(|cx| {
            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(1000.));

            let mut this = Self {
                selected_block: 0,
                active_editor: None,
                focus_handle: shared_focus_handle.unwrap_or_else(|| cx.focus_handle()),
                workspace: workspace.clone(),
                contents: None,
                list_state,
                language_registry,
                mermaid_state: Default::default(),
                parsing_markdown_task: None,
                image_cache: RetainAllImageCache::new(cx),
                mode,
                pending_scroll_to_source_offset: None,
            };

            this.set_editor(active_editor, window, cx);

            if mode == MarkdownPreviewMode::Follow {
                if let Some(workspace) = &workspace.upgrade() {
                    cx.observe_in(workspace, window, |this, workspace, window, cx| {
                        let item = workspace.read(cx).active_item(cx);
                        this.workspace_updated(item, window, cx);
                    })
                    .detach();
                } else {
                    log::error!("Failed to listen to workspace updates");
                }
            }

            this
        })
    }

    fn workspace_updated(
        &mut self,
        active_item: Option<Box<dyn ItemHandle>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(item) = active_item
            && item.item_id() != cx.entity_id()
            && let Some(editor) = item.act_as::<Editor>(cx)
            && Self::is_markdown_file(&editor, cx)
        {
            self.set_editor(editor, window, cx);
        }
    }

    pub fn is_markdown_file<V>(editor: &Entity<Editor>, cx: &mut Context<V>) -> bool {
        let buffer = editor.read(cx).buffer().read(cx);
        if let Some(buffer) = buffer.as_singleton()
            && let Some(language) = buffer.read(cx).language()
        {
            return language.name() == "Markdown".into();
        }
        false
    }

    fn set_editor(&mut self, editor: Entity<Editor>, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = &self.active_editor
            && active.editor.entity_id() == editor.entity_id()
        {
            return;
        }

        let subscription = cx.subscribe_in(&editor, window, |this, editor, event: &EditorEvent, window, cx| {
            match event {
                EditorEvent::Edited { .. }
                | EditorEvent::BufferEdited
                | EditorEvent::DirtyChanged
                | EditorEvent::ExcerptsEdited { .. } => {
                    this.parse_markdown_from_active_editor(true, window, cx);
                }
                EditorEvent::SelectionsChanged { .. } => {
                    let selection_range = editor.update(cx, |editor, cx| {
                        editor
                            .selections
                            .last::<MultiBufferOffset>(&editor.display_snapshot(cx))
                            .range()
                    });
                    this.selected_block = this.get_block_index_under_cursor(selection_range);
                    this.list_state.scroll_to_reveal_item(this.selected_block);
                    cx.notify();
                }
                _ => {}
            };
        });

        // [DonEditor] 내장 미리보기만 약한 참조로 둔다 (편집기 → 미리보기 → 편집기 순환 방지)
        let keep_alive = (self.mode != MarkdownPreviewMode::Embedded).then(|| editor.clone());
        self.active_editor = Some(EditorState {
            editor: editor.downgrade(),
            _keep_alive: keep_alive,
            _subscription: subscription,
        });

        self.parse_markdown_from_active_editor(false, window, cx);
    }

    fn parse_markdown_from_active_editor(
        &mut self,
        wait_for_debounce: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(state) = &self.active_editor {
            // if there is already a task to update the ui and the current task is also debounced (not high priority), do nothing
            if wait_for_debounce && self.parsing_markdown_task.is_some() {
                return;
            }
            self.parsing_markdown_task =
                Some(self.parse_markdown_in_background(wait_for_debounce, state.editor.clone(), window, cx));
        }
    }

    fn parse_markdown_in_background(
        &mut self,
        wait_for_debounce: bool,
        editor: WeakEntity<Editor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Result<()>> {
        let language_registry = self.language_registry.clone();

        cx.spawn_in(window, async move |view, cx| {
            if wait_for_debounce {
                // Wait for the user to stop typing
                cx.background_executor().timer(REPARSE_DEBOUNCE).await;
            }

            // [DonEditor] 편집기를 약한 참조로 받으므로, 대기 중 편집기가 닫혔으면 분석을 그만둔다.
            let source = view.update(cx, |_, cx| {
                let editor = editor.upgrade()?;
                let editor = editor.read(cx);
                let contents = editor.buffer().read(cx).snapshot(cx).text();
                let file_location = MarkdownPreviewView::get_folder_for_active_editor(editor, cx);
                Some((contents, file_location))
            })?;
            let Some((contents, file_location)) = source else {
                return view.update(cx, |view, _| view.parsing_markdown_task = None);
            };

            let parsing_task =
                cx.background_spawn(
                    async move { parse_markdown(&contents, file_location, Some(language_registry)).await },
                );
            let contents = parsing_task.await;

            view.update(cx, move |view, cx| {
                view.mermaid_state.update(&contents, cx);
                let markdown_blocks_count = contents.children.len();
                view.contents = Some(contents);
                let scroll_top = view.list_state.logical_scroll_top();
                view.list_state.reset(markdown_blocks_count);
                // [DonEditor] 미리보기 모드로 막 바뀐 경우에는 편집기에서 보던 위치의 블록을 맨 위로 보여 준다.
                let scroll_top = match view.pending_scroll_to_source_offset.take() {
                    Some(offset) => ListOffset {
                        item_ix: view.block_index_for_source_offset(offset),
                        offset_in_item: px(0.),
                    },
                    None => scroll_top,
                };
                view.list_state.scroll_to(scroll_top);
                view.parsing_markdown_task = None;
                cx.notify();
            })
        })
    }

    fn move_cursor_to_block(&self, window: &mut Window, cx: &mut Context<Self>, selection: Range<MultiBufferOffset>) {
        if let Some(editor) = self.active_editor.as_ref().and_then(|state| state.editor.upgrade()) {
            editor.update(cx, |editor, cx| {
                editor.change_selections(
                    SelectionEffects::scroll(Autoscroll::center()),
                    window,
                    cx,
                    |selections| selections.select_ranges(vec![selection]),
                );
                // [DonEditor] 내장 미리보기에서 블록을 두 번 누르면 편집 모드로 바꿔 그 위치에서 바로 고칠 수 있게 한다.
                // 커서를 옮기며 요청한 자동 스크롤이 편집 화면을 다시 그릴 때 적용된다.
                if self.mode == MarkdownPreviewMode::Embedded {
                    editor.set_alternate_view(None, cx);
                }
                window.focus(&editor.focus_handle(cx), cx);
            });
        }
    }

    /// [DonEditor] 이 미리보기가 편집기 탭 안에 내장된 것인지 돌려준다.
    pub fn is_embedded(&self) -> bool {
        self.mode == MarkdownPreviewMode::Embedded
    }

    /// [DonEditor] 다음 구문 분석이 끝나면 원본 위치 `offset` 을 담은 블록이 맨 위에 오도록 예약한다.
    pub fn scroll_to_source_offset_after_parse(&mut self, offset: usize) {
        self.pending_scroll_to_source_offset = Some(offset);
    }

    /// [DonEditor] 현재 미리보기 맨 위에 보이는 블록의 원본 시작 위치(바이트 오프셋)를 돌려준다.
    /// 원본 범위가 없는 블록이면 그 앞쪽에서 범위가 있는 가장 가까운 블록을 쓴다.
    pub fn top_visible_source_offset(&self) -> Option<usize> {
        let contents = self.contents.as_ref()?;
        source_offset_at_or_before_block(&contents.children, self.list_state.logical_scroll_top().item_ix)
    }

    /// [DonEditor] 원본 위치 `offset` 을 담고 있거나, 그 앞에서 가장 가까운 블록 번호를 돌려준다.
    fn block_index_for_source_offset(&self, offset: usize) -> usize {
        self.contents
            .as_ref()
            .map_or(0, |contents| block_index_at_source_offset(&contents.children, offset))
    }

    /// The absolute path of the file that is currently being previewed.
    fn get_folder_for_active_editor(editor: &Editor, cx: &App) -> Option<PathBuf> {
        if let Some(file) = editor.file_at(MultiBufferOffset(0), cx) {
            if let Some(file) = file.as_local() {
                file.abs_path(cx).parent().map(|p| p.to_path_buf())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_block_index_under_cursor(&self, selection_range: Range<MultiBufferOffset>) -> usize {
        let mut block_index = None;
        let cursor = selection_range.start.0;

        let mut last_end = 0;
        if let Some(content) = &self.contents {
            for (i, block) in content.children.iter().enumerate() {
                let Some(Range { start, end }) = block.source_range() else {
                    continue;
                };

                // Check if the cursor is between the last block and the current block
                if last_end <= cursor && cursor < start {
                    block_index = Some(i.saturating_sub(1));
                    break;
                }

                if start <= cursor && end >= cursor {
                    block_index = Some(i);
                    break;
                }
                last_end = end;
            }

            if block_index.is_none() && last_end < cursor {
                block_index = Some(content.children.len().saturating_sub(1));
            }
        }

        block_index.unwrap_or_default()
    }

    fn should_apply_padding_between(
        current_block: &ParsedMarkdownElement,
        next_block: Option<&ParsedMarkdownElement>,
    ) -> bool {
        !(current_block.is_list_item() && next_block.map(|b| b.is_list_item()).unwrap_or(false))
    }

    fn scroll_page_up(&mut self, _: &ScrollPageUp, _window: &mut Window, cx: &mut Context<Self>) {
        let viewport_height = self.list_state.viewport_bounds().size.height;
        if viewport_height.is_zero() {
            return;
        }

        self.list_state.scroll_by(-viewport_height);
        cx.notify();
    }

    fn scroll_page_down(&mut self, _: &ScrollPageDown, _window: &mut Window, cx: &mut Context<Self>) {
        let viewport_height = self.list_state.viewport_bounds().size.height;
        if viewport_height.is_zero() {
            return;
        }

        self.list_state.scroll_by(viewport_height);
        cx.notify();
    }

    fn scroll_up(&mut self, _: &ScrollUp, window: &mut Window, cx: &mut Context<Self>) {
        let scroll_top = self.list_state.logical_scroll_top();
        if let Some(bounds) = self.list_state.bounds_for_item(scroll_top.item_ix) {
            let item_height = bounds.size.height;
            // Scroll no more than the rough equivalent of a large headline
            let max_height = window.rem_size() * 2;
            let scroll_height = min(item_height, max_height);
            self.list_state.scroll_by(-scroll_height);
        }
        cx.notify();
    }

    fn scroll_down(&mut self, _: &ScrollDown, window: &mut Window, cx: &mut Context<Self>) {
        let scroll_top = self.list_state.logical_scroll_top();
        if let Some(bounds) = self.list_state.bounds_for_item(scroll_top.item_ix) {
            let item_height = bounds.size.height;
            // Scroll no more than the rough equivalent of a large headline
            let max_height = window.rem_size() * 2;
            let scroll_height = min(item_height, max_height);
            self.list_state.scroll_by(scroll_height);
        }
        cx.notify();
    }

    fn scroll_up_by_item(&mut self, _: &ScrollUpByItem, _window: &mut Window, cx: &mut Context<Self>) {
        let scroll_top = self.list_state.logical_scroll_top();
        if let Some(bounds) = self.list_state.bounds_for_item(scroll_top.item_ix) {
            self.list_state.scroll_by(-bounds.size.height);
        }
        cx.notify();
    }

    fn scroll_down_by_item(&mut self, _: &ScrollDownByItem, _window: &mut Window, cx: &mut Context<Self>) {
        let scroll_top = self.list_state.logical_scroll_top();
        if let Some(bounds) = self.list_state.bounds_for_item(scroll_top.item_ix) {
            self.list_state.scroll_by(bounds.size.height);
        }
        cx.notify();
    }

    fn scroll_to_top(&mut self, _: &ScrollToTop, _window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        cx.notify();
    }

    fn scroll_to_bottom(&mut self, _: &ScrollToBottom, _window: &mut Window, cx: &mut Context<Self>) {
        let count = self.list_state.item_count();
        if count > 0 {
            self.list_state.scroll_to(ListOffset {
                item_ix: count - 1,
                offset_in_item: px(0.),
            });
        }
        cx.notify();
    }

    fn scroll_to_heading(&mut self, id: &str, _window: &mut Window, _cx: &mut Context<Self>) {
        if let Some(contents) = &self.contents {
            let id = &id[1..];
            // Fid index of header that matches the ID
            if let Some((idx, _)) = contents.children.iter().enumerate().find(|(_, child)| match child {
                ParsedMarkdownElement::Heading(ParsedMarkdownHeading { contents, .. }) => {
                    contents.iter().any(|content| match content {
                        MarkdownParagraphChunk::Text(ParsedMarkdownText { contents, .. }) => {
                            contents.to_lowercase().replace(" ", "-").contains(id)
                        }
                        _ => false,
                    })
                }
                _ => false,
            }) {
                self.list_state.scroll_to_reveal_item(idx);
            }
        }
    }
}

/// [DonEditor] 원본 위치 `offset` 을 담고 있거나, 그 앞에서 가장 가까운 블록 번호 (없으면 0).
fn block_index_at_source_offset(blocks: &[ParsedMarkdownElement], offset: usize) -> usize {
    let mut index = 0;
    // 블록은 원본 순서대로 놓여 있으므로, 시작 위치가 offset 이하인 마지막 블록을 고른다
    for (ix, block) in blocks.iter().enumerate() {
        match block.source_range() {
            Some(range) if range.start <= offset => index = ix,
            Some(_) => break,
            None => {}
        }
    }
    index
}

/// [DonEditor] `index` 번 블록의 원본 시작 위치. 그 블록에 원본 범위가 없으면 앞쪽에서 가장 가까운 블록의 값을 쓴다.
fn source_offset_at_or_before_block(blocks: &[ParsedMarkdownElement], index: usize) -> Option<usize> {
    // 지정한 블록부터 앞쪽으로 거슬러 올라가며 원본 범위가 있는 블록을 찾는다
    blocks[..blocks.len().min(index + 1)]
        .iter()
        .rev()
        .find_map(|block| block.source_range())
        .map(|range| range.start)
}

impl Focusable for MarkdownPreviewView {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for MarkdownPreviewView {}

impl Item for MarkdownPreviewView {
    type Event = ();

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::FileDoc))
    }

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        self.active_editor
            .as_ref()
            .and_then(|editor_state| {
                let editor = editor_state.editor.upgrade()?;
                let buffer = editor.read(cx).buffer().read(cx);
                let buffer = buffer.as_singleton()?;
                let file = buffer.read(cx).file()?;
                let local_file = file.as_local()?;
                local_file
                    .abs_path(cx)
                    .file_name()
                    .map(|name| format!("Preview {}", name.to_string_lossy()).into())
            })
            .unwrap_or_else(|| SharedString::from("Markdown Preview"))
    }

    fn to_item_events(_event: &Self::Event, _f: impl FnMut(workspace::item::ItemEvent)) {}
}

impl Render for MarkdownPreviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let buffer_size = ThemeSettings::get_global(cx).buffer_font_size(cx);
        let buffer_line_height = ThemeSettings::get_global(cx).buffer_line_height;

        v_flex()
            .image_cache(self.image_cache.clone())
            .id("MarkdownPreview")
            .key_context("MarkdownPreview")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(MarkdownPreviewView::scroll_page_up))
            .on_action(cx.listener(MarkdownPreviewView::scroll_page_down))
            .on_action(cx.listener(MarkdownPreviewView::scroll_up))
            .on_action(cx.listener(MarkdownPreviewView::scroll_down))
            .on_action(cx.listener(MarkdownPreviewView::scroll_up_by_item))
            .on_action(cx.listener(MarkdownPreviewView::scroll_down_by_item))
            .on_action(cx.listener(MarkdownPreviewView::scroll_to_top))
            .on_action(cx.listener(MarkdownPreviewView::scroll_to_bottom))
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .p_4()
            .text_size(buffer_size)
            .line_height(relative(buffer_line_height.value()))
            .child(div().flex_grow().map(|this| {
                this.child(
                    list(
                        self.list_state.clone(),
                        cx.processor(|this, ix, window, cx| {
                            let Some(contents) = &this.contents else {
                                return div().into_any();
                            };

                            let mut render_cx =
                                RenderContext::new(Some(this.workspace.clone()), &this.mermaid_state, window, cx)
                                    .with_fragment_jump_callback(cx.listener(move |this, id: &str, window, cx| {
                                        this.scroll_to_heading(id, window, cx);
                                        cx.notify();
                                    }))
                                    .with_checkbox_clicked_callback(cx.listener(
                                        move |this, e: &CheckboxClickedEvent, window, cx| {
                                            if let Some(editor) =
                                                this.active_editor.as_ref().and_then(|s| s.editor.upgrade())
                                            {
                                                editor.update(cx, |editor, cx| {
                                                    let task_marker = if e.checked() { "[x]" } else { "[ ]" };

                                                    editor.edit(
                                                        [(
                                                            MultiBufferOffset(e.source_range().start)
                                                                ..MultiBufferOffset(e.source_range().end),
                                                            task_marker,
                                                        )],
                                                        cx,
                                                    );
                                                });
                                                this.parse_markdown_from_active_editor(false, window, cx);
                                                cx.notify();
                                            }
                                        },
                                    ));

                            let block = contents.children.get(ix).unwrap();
                            let rendered_block = render_markdown_block(block, &mut render_cx);

                            let should_apply_padding =
                                Self::should_apply_padding_between(block, contents.children.get(ix + 1));

                            let selected_block = this.selected_block;
                            let scaled_rems = render_cx.scaled_rems(1.0);
                            div()
                                .id(ix)
                                .when(should_apply_padding, |this| this.pb(render_cx.scaled_rems(0.75)))
                                .group("markdown-block")
                                .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                                    if event.click_count() == 2
                                        && let Some(source_range) = this
                                            .contents
                                            .as_ref()
                                            .and_then(|c| c.children.get(ix))
                                            .and_then(|block: &ParsedMarkdownElement| block.source_range())
                                    {
                                        this.move_cursor_to_block(
                                            window,
                                            cx,
                                            MultiBufferOffset(source_range.start)
                                                ..MultiBufferOffset(source_range.start),
                                        );
                                    }
                                }))
                                .map(move |container| {
                                    let indicator = div()
                                        .h_full()
                                        .w(px(4.0))
                                        .when(ix == selected_block, |this| this.bg(cx.theme().colors().border))
                                        .group_hover("markdown-block", |s| {
                                            if ix == selected_block {
                                                s
                                            } else {
                                                s.bg(cx.theme().colors().border_variant)
                                            }
                                        })
                                        .rounded_xs();

                                    container.child(
                                        div()
                                            .relative()
                                            .child(div().pl(scaled_rems).child(rendered_block))
                                            .child(indicator.absolute().left_0().top_0()),
                                    )
                                })
                                .into_any()
                        }),
                    )
                    .size_full(),
                )
            }))
            .vertical_scrollbar_for(&self.list_state, window, cx)
    }
}

// [DonEditor] 미리보기/편집 모드 전환 시 위치를 잇는 계산의 단위 테스트
#[cfg(test)]
mod tests {
    use super::{block_index_at_source_offset, source_offset_at_or_before_block};
    use crate::markdown_parser::parse_markdown;

    /// 제목·문단·목록이 섞인 예시 문서
    const SAMPLE: &str = "# 제목\n\n첫 문단입니다.\n\n## 둘째\n\n- 항목 하나\n- 항목 둘\n\n끝 문단\n";

    /// 원본 위치가 그 위치를 담은 블록(블록 사이면 앞 블록) 번호로 바뀌는지 확인한다.
    #[gpui::test]
    async fn source_offset_maps_to_the_block_that_contains_it() {
        let parsed = parse_markdown(SAMPLE, None, None).await;
        let blocks = &parsed.children;
        let second_heading = SAMPLE.find("## 둘째").unwrap();
        let last_paragraph = SAMPLE.find("끝 문단").unwrap();

        // 문서 맨 앞은 첫 블록
        assert_eq!(block_index_at_source_offset(blocks, 0), 0);
        // 블록 시작 위치와 블록 안쪽 위치는 그 블록
        let heading_index = block_index_at_source_offset(blocks, second_heading);
        assert_eq!(blocks[heading_index].source_range().unwrap().start, second_heading);
        assert_eq!(block_index_at_source_offset(blocks, second_heading + 3), heading_index);
        // 블록 사이 빈 줄은 앞 블록, 문서 끝을 넘으면 마지막 블록
        assert_eq!(
            block_index_at_source_offset(blocks, second_heading - 1),
            heading_index - 1
        );
        assert_eq!(
            block_index_at_source_offset(blocks, SAMPLE.len() + 10),
            blocks.len() - 1
        );
        assert_eq!(
            blocks[block_index_at_source_offset(blocks, last_paragraph)]
                .source_range()
                .unwrap()
                .start,
            last_paragraph
        );
    }

    /// 블록 번호 → 원본 위치 → 블록 번호 변환이 제자리로 돌아오고, 범위 밖·빈 문서도 안전한지 확인한다.
    #[gpui::test]
    async fn block_index_round_trips_to_its_source_offset() {
        let parsed = parse_markdown(SAMPLE, None, None).await;
        let blocks = &parsed.children;
        // 모든 블록에 대해 "블록 → 원본 위치 → 블록" 이 같은 블록으로 돌아와야 한다
        for (index, block) in blocks.iter().enumerate() {
            let offset = source_offset_at_or_before_block(blocks, index).unwrap();
            assert_eq!(offset, block.source_range().unwrap().start);
            assert_eq!(block_index_at_source_offset(blocks, offset), index);
        }
        // 범위를 벗어난 번호는 마지막 블록의 위치, 빈 문서는 None
        assert_eq!(
            source_offset_at_or_before_block(blocks, blocks.len() + 5),
            blocks
                .last()
                .and_then(|block| block.source_range())
                .map(|range| range.start)
        );
        assert_eq!(source_offset_at_or_before_block(&[], 0), None);
        assert_eq!(block_index_at_source_offset(&[], 3), 0);
    }
}
