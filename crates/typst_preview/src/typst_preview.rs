//! [DonEditor] Typst 지원의 화면 쪽: 쪽 미리보기, 미리보기/편집 모드 전환, 메뉴 기능(변환·PDF·인쇄). GPL-3.0-or-later.

mod typst_actions;
mod typst_mode;
pub mod typst_preview_view;

use gpui::App;
use workspace::Workspace;

pub use typst_actions::{ConvertMarkdownToTypst, ExportPdf, Print, TogglePreviewMode};
pub use typst_mode::{TypstModeToggle, is_preview_mode, is_typst_editor};

/// 앱 시작 때 작업 공간마다 미리보기 모드 전환과 메뉴 액션을 등록한다.
pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        typst_mode::register(workspace, window, cx);
        typst_actions::register(workspace);
    })
    .detach();
}
