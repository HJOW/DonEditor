//! [DonEditor] Typst(및 Markdown→Typst 변환) 문서의 쪽 미리보기 화면. GPL-3.0-or-later.
//!
//! 편집기 탭이 본문 대신 이 화면을 그린다(`Editor::set_alternate_view`). 문서를 백그라운드에서 컴파일해
//! 쪽마다 SVG 로 그린 그림을 세로로 늘어놓고, 컴파일 오류는 화면 위쪽에 줄 번호와 함께 보여 준다.
//! 오류가 있을 때는 마지막으로 성공한 쪽 그림을 그대로 둬서 입력 중에 화면이 깜빡이지 않는다.

use std::sync::Arc;
use std::time::Duration;

use don_typst::Diagnostic;
use editor::{Editor, EditorEvent};
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement, IsZero, ListAlignment,
    ListOffset, ListState, ParentElement, Render, RenderImage, Styled, Subscription, Task, WeakEntity, Window, img,
    list,
};
use markdown_preview::{ScrollDown, ScrollPageDown, ScrollPageUp, ScrollToBottom, ScrollToTop, ScrollUp};
use ui::{WithScrollbar, prelude::*};

/// 입력이 멈춘 뒤 컴파일을 시작하기까지 기다리는 시간 (Typst 컴파일은 문서가 크면 시간이 걸린다)
const RECOMPILE_DEBOUNCE: Duration = Duration::from_millis(450);

/// 포인트(1/72 인치)를 화면 픽셀(96 DPI 기준)로 바꾸는 비율
const POINTS_TO_PIXELS: f32 = 96.0 / 72.0;

/// 미리보기에 보여 줄 문서 종류
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewSource {
    /// Typst 문서
    Typst,
    /// Markdown 문서를 Typst 로 바꿔서 그린다 (현재는 인쇄·PDF 변환에서만 쓰고, 미리보기는 기존 Markdown 미리보기를 쓴다)
    Markdown,
}

/// 컴파일에 성공한 쪽 하나
struct PreviewPage {
    image: Arc<RenderImage>,
    /// 화면에 보여 줄 크기(논리 픽셀)
    width: f32,
    height: f32,
}

/// 백그라운드 컴파일 결과
struct CompileOutcome {
    pages: Result<Vec<(Arc<RenderImage>, f32, f32)>, Vec<Diagnostic>>,
    warning_count: usize,
}

pub struct TypstPreviewView {
    /// 원본 편집기. 편집기가 이 화면을 소유하므로 순환 참조를 피하려고 약한 참조로 갖는다.
    editor: WeakEntity<Editor>,
    source: PreviewSource,
    /// 편집기의 포커스 핸들을 함께 써서, 모드를 바꿔도 초점이 유지되고 키 입력이 같은 곳으로 간다.
    focus_handle: FocusHandle,
    pages: Vec<PreviewPage>,
    /// 마지막 컴파일의 오류 (성공하면 비어 있다)
    errors: Vec<Diagnostic>,
    warning_count: usize,
    /// 컴파일 중인지 여부
    compiling: bool,
    list_state: ListState,
    /// 앱이 가진 글꼴 파일 내용(D2Coding). Typst 컴파일에 넘긴다.
    font_data: Arc<Vec<Vec<u8>>>,
    compile_task: Option<Task<()>>,
    _subscription: Subscription,
}

impl TypstPreviewView {
    pub fn new(
        editor: Entity<Editor>,
        source: PreviewSource,
        font_data: Arc<Vec<Vec<u8>>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let shared_focus_handle = editor.read(cx).focus_handle(cx);
        cx.new(|cx| {
            // 편집 내용이 바뀌면 다시 컴파일한다
            let subscription = cx.subscribe_in(
                &editor,
                window,
                |this: &mut Self, _, event: &EditorEvent, window, cx| {
                    if matches!(
                        event,
                        EditorEvent::Edited { .. }
                            | EditorEvent::BufferEdited
                            | EditorEvent::DirtyChanged
                            | EditorEvent::ExcerptsEdited { .. }
                            | EditorEvent::Reparsed(_)
                    ) {
                        this.recompile(true, window, cx);
                    }
                },
            );
            let mut this = Self {
                editor: editor.downgrade(),
                source,
                focus_handle: shared_focus_handle,
                pages: Vec::new(),
                errors: Vec::new(),
                warning_count: 0,
                compiling: false,
                list_state: ListState::new(0, ListAlignment::Top, px(1500.)),
                font_data,
                compile_task: None,
                _subscription: subscription,
            };
            this.recompile(false, window, cx);
            this
        })
    }

    /// 마지막 컴파일의 오류 목록
    pub fn errors(&self) -> &[Diagnostic] {
        &self.errors
    }

    /// 원본 편집기에서 문서 글자와 문서가 있는 폴더를 읽는다.
    fn read_source(&self, cx: &App) -> Option<(String, Option<std::path::PathBuf>)> {
        let editor = self.editor.upgrade()?;
        let editor = editor.read(cx);
        let text = editor.buffer().read(cx).snapshot(cx).text();
        let root = editor
            .buffer()
            .read(cx)
            .as_singleton()
            .and_then(|buffer| {
                buffer
                    .read(cx)
                    .file()
                    .and_then(|file| file.as_local())
                    .map(|file| file.abs_path(cx))
            })
            .and_then(|path| path.parent().map(|parent| parent.to_path_buf()));
        Some((text, root))
    }

    /// 문서를 다시 컴파일한다. `debounce` 이면 입력이 멈출 때까지 잠깐 기다린다. 진행 중이던 컴파일은 버린다.
    fn recompile(&mut self, debounce: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.compiling = true;
        let renderer = cx.svg_renderer();
        let scale_factor = window.scale_factor();
        let source_kind = self.source;
        let font_data = self.font_data.clone();
        cx.notify();

        self.compile_task = Some(cx.spawn_in(window, async move |this, cx| {
            if debounce {
                cx.background_executor().timer(RECOMPILE_DEBOUNCE).await;
            }
            let Some((text, root)) = this.update(cx, |this, cx| this.read_source(cx)).ok().flatten() else {
                return;
            };

            // 컴파일과 쪽 그림 만들기는 모두 백그라운드에서 한다
            let outcome = cx
                .background_spawn(async move {
                    let compiled = match source_kind {
                        PreviewSource::Typst => don_typst::compile(text, root, &font_data),
                        PreviewSource::Markdown => don_typst::compile_markdown(&text, root, &font_data),
                    };
                    match compiled {
                        Ok(document) => {
                            let warning_count = document.warnings.len();
                            let sizes = document.page_sizes();
                            let mut pages = Vec::new();
                            // 쪽마다 SVG 를 그림으로 바꾼다. 화면 배율을 곱해 선명하게 만든다
                            for (svg, (width_pt, height_pt)) in document.pages_svg().iter().zip(sizes) {
                                match renderer.render_single_frame(
                                    svg.as_bytes(),
                                    scale_factor * POINTS_TO_PIXELS,
                                    true,
                                ) {
                                    Ok(image) => {
                                        pages.push((image, width_pt * POINTS_TO_PIXELS, height_pt * POINTS_TO_PIXELS))
                                    }
                                    Err(error) => {
                                        return CompileOutcome {
                                            pages: Err(vec![Diagnostic {
                                                is_error: true,
                                                message: format!("쪽 그림을 만들지 못했습니다: {error}"),
                                                line: None,
                                                column: None,
                                                hints: Vec::new(),
                                            }]),
                                            warning_count,
                                        };
                                    }
                                }
                            }
                            CompileOutcome {
                                pages: Ok(pages),
                                warning_count,
                            }
                        }
                        Err(errors) => CompileOutcome {
                            pages: Err(errors),
                            warning_count: 0,
                        },
                    }
                })
                .await;

            this.update_in(cx, |this, window, cx| this.apply_outcome(outcome, window, cx))
                .ok();
        }));
    }

    /// 컴파일 결과를 화면에 반영한다. 성공하면 쪽 그림을 바꾸고, 실패하면 오류만 갱신하고 이전 쪽은 그대로 둔다.
    fn apply_outcome(&mut self, outcome: CompileOutcome, window: &mut Window, cx: &mut Context<Self>) {
        self.compiling = false;
        self.compile_task = None;
        match outcome.pages {
            Ok(pages) => {
                // 이전 쪽 그림은 GPU 자원을 반납한다
                for page in self.pages.drain(..) {
                    window.drop_image(page.image);
                }
                let scroll_top = self.list_state.logical_scroll_top();
                self.pages = pages
                    .into_iter()
                    .map(|(image, width, height)| PreviewPage { image, width, height })
                    .collect();
                self.errors.clear();
                self.warning_count = outcome.warning_count;
                self.list_state.reset(self.pages.len());
                // 편집 중에도 보던 쪽에서 멈춰 있도록 스크롤 위치를 유지한다
                self.list_state.scroll_to(scroll_top);
            }
            Err(errors) => {
                self.errors = errors;
                self.warning_count = 0;
            }
        }
        cx.notify();
    }

    /// 화면 위쪽 오류 알림 줄
    fn render_errors(&self, cx: &App) -> Option<impl IntoElement> {
        if self.errors.is_empty() {
            return None;
        }
        let colors = cx.theme().colors();
        // 오류가 많아도 위쪽을 차지하지 않도록 처음 몇 개만 보여 준다
        const SHOWN_ERRORS: usize = 5;
        let lines = self.errors.iter().take(SHOWN_ERRORS).map(|error| {
            let location = match (error.line, error.column) {
                (Some(line), Some(column)) => i18n::t!("typst_preview.error_location", line = line, column = column),
                _ => String::new(),
            };
            let mut text = format!("{location}{}", error.message);
            if let Some(hint) = error.hints.first() {
                text.push_str(&format!(" ({hint})"));
            }
            h_flex()
                .gap_2()
                .items_start()
                .child(Icon::new(IconName::XCircle).size(IconSize::Small).color(Color::Error))
                .child(Label::new(text).size(LabelSize::Small))
        });
        let hidden = self.errors.len().saturating_sub(SHOWN_ERRORS);
        Some(
            v_flex()
                .w_full()
                .px_3()
                .py_2()
                .gap_1()
                .bg(colors.element_background)
                .border_b_1()
                .border_color(colors.border_variant)
                .child(
                    // 성공한 쪽이 아직 없으면 "마지막으로 성공한 미리보기" 문구를 쓰지 않는다
                    Label::new(if self.pages.is_empty() {
                        i18n::t!("typst_preview.compile_failed_no_preview")
                    } else {
                        i18n::t!("typst_preview.compile_failed")
                    })
                    .size(LabelSize::Small)
                    .color(Color::Error),
                )
                .children(lines)
                .when(hidden > 0, |this| {
                    this.child(
                        Label::new(i18n::t!("typst_preview.more_errors", count = hidden))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                }),
        )
    }
}

/// 키보드로 한 번에 움직이는 작은 스크롤 거리 (위/아래 화살표)
const SMALL_SCROLL: f32 = 48.0;

impl TypstPreviewView {
    /// 한 화면만큼 위로 스크롤한다 (PageUp, Alt+위)
    fn scroll_page_up(&mut self, _: &ScrollPageUp, _window: &mut Window, cx: &mut Context<Self>) {
        let height = self.list_state.viewport_bounds().size.height;
        if !height.is_zero() {
            self.list_state.scroll_by(-height);
            cx.notify();
        }
    }

    /// 한 화면만큼 아래로 스크롤한다 (PageDown, Alt+아래)
    fn scroll_page_down(&mut self, _: &ScrollPageDown, _window: &mut Window, cx: &mut Context<Self>) {
        let height = self.list_state.viewport_bounds().size.height;
        if !height.is_zero() {
            self.list_state.scroll_by(height);
            cx.notify();
        }
    }

    fn scroll_up(&mut self, _: &ScrollUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.scroll_by(-px(SMALL_SCROLL));
        cx.notify();
    }

    fn scroll_down(&mut self, _: &ScrollDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.scroll_by(px(SMALL_SCROLL));
        cx.notify();
    }

    /// 첫 쪽의 맨 위로 간다 (Ctrl+Home)
    fn scroll_to_top(&mut self, _: &ScrollToTop, _window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.scroll_to(ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        cx.notify();
    }

    /// 마지막 쪽의 맨 위로 간다 (Ctrl+End)
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
}

impl Focusable for TypstPreviewView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for TypstPreviewView {}

impl Render for TypstPreviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let background = cx.theme().colors().editor_background;
        let status = if self.compiling && self.pages.is_empty() {
            Some(i18n::t!("typst_preview.compiling"))
        } else {
            None
        };

        v_flex()
            .id("TypstPreview")
            .key_context("TypstPreview")
            .track_focus(&self.focus_handle)
            // 키보드로 쪽을 넘기는 동작 (Markdown 미리보기와 같은 액션·키 바인딩을 쓴다)
            .on_action(cx.listener(Self::scroll_page_up))
            .on_action(cx.listener(Self::scroll_page_down))
            .on_action(cx.listener(Self::scroll_up))
            .on_action(cx.listener(Self::scroll_down))
            .on_action(cx.listener(Self::scroll_to_top))
            .on_action(cx.listener(Self::scroll_to_bottom))
            .size_full()
            .bg(background)
            .children(self.render_errors(cx))
            .when_some(status, |this, status| {
                this.child(div().p_4().child(Label::new(status).color(Color::Muted)))
            })
            .child(
                div().flex_grow().size_full().child(
                    list(
                        self.list_state.clone(),
                        cx.processor(|this, ix: usize, _window, cx| {
                            let Some(page) = this.pages.get(ix) else {
                                return div().into_any_element();
                            };
                            // 쪽은 가운데에 두고 그림자 대신 테두리로 구분한다. 창이 좁으면 비율을 유지한 채 줄어든다.
                            let border = cx.theme().colors().border_variant;
                            h_flex()
                                .w_full()
                                .justify_center()
                                .py_3()
                                .px_3()
                                .child(
                                    div().border_1().border_color(border).child(
                                        img(page.image.clone())
                                            .w(px(page.width))
                                            .h(px(page.height))
                                            .max_w_full()
                                            .object_fit(gpui::ObjectFit::Contain),
                                    ),
                                )
                                .into_any_element()
                        }),
                    )
                    .size_full(),
                ),
            )
            .vertical_scrollbar_for(&self.list_state, window, cx)
    }
}
