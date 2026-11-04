use gpui::{App, actions};
use workspace::Workspace;

pub mod markdown_elements;
// [DonEditor] Markdown 파일 탭의 미리보기/편집 모드 전환
pub mod markdown_mode;
pub mod markdown_parser;
pub mod markdown_preview_view;
pub mod markdown_renderer;

pub use app_actions::preview::markdown::{OpenPreview, OpenPreviewToTheSide};

actions!(
    markdown,
    [
        /// Scrolls up by one page in the markdown preview.
        #[action(deprecated_aliases = ["markdown::MovePageUp"])]
        ScrollPageUp,
        /// Scrolls down by one page in the markdown preview.
        #[action(deprecated_aliases = ["markdown::MovePageDown"])]
        ScrollPageDown,
        /// Scrolls up by approximately one visual line.
        ScrollUp,
        /// Scrolls down by approximately one visual line.
        ScrollDown,
        /// Scrolls up by one markdown element in the markdown preview
        ScrollUpByItem,
        /// Scrolls down by one markdown element in the markdown preview
        ScrollDownByItem,
        /// Scrolls to the top of the markdown preview.
        ScrollToTop,
        /// Scrolls to the bottom of the markdown preview.
        ScrollToBottom,
        /// Opens a following markdown preview that syncs with the editor.
        OpenFollowingPreview,
        /// [DonEditor] Toggles the current Markdown file tab between preview mode and edit mode.
        TogglePreviewMode
    ]
);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        markdown_preview_view::MarkdownPreviewView::register(workspace, window, cx);
        // [DonEditor] 미리보기/편집 모드 전환 액션과 "Markdown 파일은 미리보기로 열기" 처리
        markdown_mode::register(workspace, window, cx);
    })
    .detach();
}
