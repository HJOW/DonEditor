//! [DonEditor] 메뉴 기능: Markdown→Typst 변환, PDF 내보내기, 인쇄. GPL-3.0-or-later.
//!
//! - **Markdown→Typst 변환:** 메뉴를 고르면 곧바로 파일 저장 창을 띄워 변환 결과를 `.typ` 파일로 즉시 저장한다.
//! - **PDF 내보내기·인쇄:** Markdown 문서는 자동으로 Typst 로 바꾼 뒤 컴파일한다. 이때 중간 Typst 결과는 **파일로 저장하지 않고**
//!   메모리에서만 쓴다. PDF 내보내기는 결과 PDF 만 저장하고, 인쇄는 임시 PDF 를 만들어 운영체제의 인쇄 기능으로 넘긴다.
//! - 모든 기능은 액션이라 MCP `dispatch_action` 으로도 실행할 수 있다. `path` 를 주면 저장 창 없이 그 경로에 쓴다.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context as _, Result};
use don_typst::{ConvertOptions, Diagnostic};
use editor::Editor;
use gpui::{Action, App, AppContext as _, Context, PromptLevel, Window, actions};
use markdown_preview::markdown_mode::is_markdown_editor;
use schemars::JsonSchema;
use serde::Deserialize;
use workspace::{Toast, Workspace, notifications::NotificationId};

use crate::typst_mode::{is_typst_editor, preview_font_data};

actions!(
    typst,
    [
        /// Toggles the current Typst file tab between preview mode and edit mode.
        TogglePreviewMode,
        /// Prints the current Markdown or Typst document (Markdown is converted to Typst in memory first).
        Print
    ]
);

/// 현재 Markdown 문서를 Typst 로 변환해 파일로 저장한다. `path` 를 주면 저장 창 없이 그 경로에 쓴다.
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = typst)]
#[serde(deny_unknown_fields)]
pub struct ConvertMarkdownToTypst {
    /// 저장할 `.typ` 파일 경로. 비우면 저장 창을 띄운다.
    #[serde(default)]
    pub path: Option<String>,
}

/// 현재 Markdown 또는 Typst 문서를 PDF 로 저장한다(Markdown 은 메모리에서 Typst 로 바꾼 뒤 컴파일). `path` 를 주면 저장 창 없이 그 경로에 쓴다.
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = typst)]
#[serde(deny_unknown_fields)]
pub struct ExportPdf {
    /// 저장할 `.pdf` 파일 경로. 비우면 저장 창을 띄운다.
    #[serde(default)]
    pub path: Option<String>,
}

/// 활성 문서의 종류
#[derive(Clone, Copy, PartialEq, Eq)]
enum DocumentKind {
    Markdown,
    Typst,
}

/// 메뉴 기능이 다루는 현재 문서의 정보
struct ActiveDocument {
    kind: DocumentKind,
    /// 편집기 버퍼의 글자(저장하지 않은 편집 내용 포함)
    text: String,
    /// 디스크에 저장된 파일의 경로 (저장하지 않은 문서는 `None`)
    file_path: Option<PathBuf>,
    /// 새 파일 이름을 제안할 때 쓰는 문서 이름(확장자 없음)
    stem: String,
}

/// 작업 공간마다 메뉴 액션을 등록한다.
pub(crate) fn register(workspace: &mut Workspace) {
    workspace.register_action(|workspace, action: &ConvertMarkdownToTypst, window, cx| {
        convert_markdown_to_typst(workspace, action.path.clone(), window, cx);
    });
    workspace.register_action(|workspace, action: &ExportPdf, window, cx| {
        export_pdf(workspace, action.path.clone(), window, cx);
    });
    workspace.register_action(|workspace, _: &Print, window, cx| {
        print_document(workspace, window, cx);
    });
}

/// 활성 편집기가 Markdown 또는 Typst 문서이면 그 정보를 돌려준다.
fn active_document(workspace: &Workspace, cx: &App) -> Option<ActiveDocument> {
    let editor = workspace.active_item(cx)?.act_as::<Editor>(cx)?;
    let editor = editor.read(cx);
    let kind = if is_markdown_editor(editor, cx) {
        DocumentKind::Markdown
    } else if is_typst_editor(editor, cx) {
        DocumentKind::Typst
    } else {
        return None;
    };
    let buffer = editor.buffer().read(cx);
    let text = buffer.snapshot(cx).text();
    let file = buffer.as_singleton().and_then(|buffer| buffer.read(cx).file().cloned());
    let file_path = file
        .as_ref()
        .and_then(|file| file.as_local())
        .map(|file| file.abs_path(cx));
    let stem = file_path
        .as_ref()
        .and_then(|path| path.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    Some(ActiveDocument {
        kind,
        text,
        file_path,
        stem,
    })
}

/// 작업 공간 모서리에 잠깐 뜨는 안내 메시지를 보여 준다.
fn show_toast(workspace: &gpui::WeakEntity<Workspace>, message: String, cx: &mut App) {
    struct TypstToast;
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<TypstToast>(), message).autohide(),
                cx,
            );
        })
        .ok();
}

/// 저장 창에서 보여 줄 기본 폴더: 문서가 있는 폴더, 없으면 홈 폴더
fn default_directory(document: &ActiveDocument) -> PathBuf {
    document
        .file_path
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(dirs_home)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

/// 경로에 확장자가 없으면 `extension` 을 붙인다.
fn with_extension_if_missing(path: PathBuf, extension: &str) -> PathBuf {
    if path.extension().is_some() {
        path
    } else {
        path.with_extension(extension)
    }
}

/// 저장할 경로를 정한다. `explicit` 이 있으면 그대로 쓰고, 없으면 저장 창을 띄운다(취소하면 `None`).
async fn choose_save_path(
    explicit: Option<String>,
    directory: PathBuf,
    suggested_name: String,
    extension: &'static str,
    cx: &mut gpui::AsyncWindowContext,
) -> Result<Option<PathBuf>> {
    if let Some(path) = explicit.filter(|path| !path.trim().is_empty()) {
        return Ok(Some(with_extension_if_missing(PathBuf::from(path), extension)));
    }
    let receiver = cx.update(|_, cx| cx.prompt_for_new_path(&directory, Some(&suggested_name)))?;
    let chosen = receiver.await.context("저장 창이 닫혔습니다")??;
    Ok(chosen.map(|path| with_extension_if_missing(path, extension)))
}

/// 파일을 쓴다 (UI 를 막지 않도록 백그라운드에서).
async fn write_file(path: PathBuf, bytes: Vec<u8>, cx: &mut gpui::AsyncWindowContext) -> Result<()> {
    cx.background_spawn(async move { std::fs::write(&path, bytes).with_context(|| path.display().to_string()) })
        .await
}

/// 컴파일 진단을 사람이 읽을 글로 바꾼다.
fn describe_errors(errors: &[Diagnostic]) -> String {
    // 오류가 많아도 창이 너무 커지지 않도록 처음 몇 개만 보여 준다
    const SHOWN: usize = 8;
    let mut lines: Vec<String> = errors
        .iter()
        .take(SHOWN)
        .map(|error| {
            let location = match (error.line, error.column) {
                (Some(line), Some(column)) => i18n::t!("typst_preview.error_location", line = line, column = column),
                _ => String::new(),
            };
            format!("{location}{}", error.message)
        })
        .collect();
    if errors.len() > SHOWN {
        lines.push(i18n::t!("typst_preview.more_errors", count = errors.len() - SHOWN));
    }
    lines.join("\n")
}

/// 컴파일 실패를 알리는 오류 창을 띄운다.
fn show_compile_error(window: &mut Window, errors: &[Diagnostic], cx: &mut App) {
    let detail = describe_errors(errors);
    let receiver = window.prompt(
        PromptLevel::Critical,
        i18n::t!("typst_preview.error.compile_failed_title"),
        Some(&detail),
        &[i18n::t!("common.ok")],
        cx,
    );
    // 창이 닫힐 때까지 응답을 받아 둔다 (받는 쪽이 사라지면 창이 바로 취소될 수 있다)
    window
        .spawn(cx, async move |_| {
            receiver.await.ok();
        })
        .detach();
}

/// 문서를 PDF 바이트로 만든다 (Markdown 은 메모리에서 Typst 로 바꿔 컴파일). 백그라운드에서 호출한다.
fn build_pdf(
    kind: DocumentKind,
    text: String,
    root: Option<PathBuf>,
    fonts: &[Vec<u8>],
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let compiled = match kind {
        DocumentKind::Markdown => don_typst::compile_markdown(&text, root, fonts),
        DocumentKind::Typst => don_typst::compile(text, root, fonts),
    }?;
    compiled.to_pdf().map_err(|message| {
        vec![Diagnostic {
            is_error: true,
            message,
            line: None,
            column: None,
            hints: Vec::new(),
        }]
    })
}

/// 메뉴: Markdown→Typst 변환. 곧바로 저장 창을 띄워 변환 결과를 즉시 저장한다.
fn convert_markdown_to_typst(
    workspace: &mut Workspace,
    explicit_path: Option<String>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let workspace_handle = workspace.weak_handle();
    let Some(document) = active_document(workspace, cx) else {
        show_toast(
            &workspace_handle,
            i18n::t!("typst_preview.error.no_document").to_string(),
            cx,
        );
        return;
    };
    if document.kind != DocumentKind::Markdown {
        show_toast(
            &workspace_handle,
            i18n::t!("typst_preview.error.markdown_only").to_string(),
            cx,
        );
        return;
    }
    // 변환은 글자 처리만이라 즉시 끝난다
    let typst = don_typst::markdown_to_typst(&document.text, &ConvertOptions::default());
    let directory = default_directory(&document);
    let suggested = format!("{}.typ", document.stem);

    window
        .spawn(cx, async move |cx| -> Result<()> {
            let Some(path) = choose_save_path(explicit_path, directory, suggested, "typ", cx).await? else {
                // 저장 창에서 취소했으면 아무 파일도 만들지 않는다
                return Ok(());
            };
            let message = match write_file(path.clone(), typst.into_bytes(), cx).await {
                Ok(()) => i18n::t!("typst_preview.convert.done", path = path.display()),
                Err(error) => i18n::t!("typst_preview.error.write_failed", path = path.display(), error = error),
            };
            cx.update(|_, cx| show_toast(&workspace_handle, message, cx))?;
            Ok(())
        })
        .detach_and_log_err(cx);
}

/// 메뉴: PDF 내보내기. Markdown 은 메모리에서 Typst 로 바꾼 뒤 컴파일하고, 중간 Typst 파일은 저장하지 않는다.
fn export_pdf(
    workspace: &mut Workspace,
    explicit_path: Option<String>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let workspace_handle = workspace.weak_handle();
    let Some(document) = active_document(workspace, cx) else {
        show_toast(
            &workspace_handle,
            i18n::t!("typst_preview.error.no_document").to_string(),
            cx,
        );
        return;
    };
    let fonts = preview_font_data(cx);
    let directory = default_directory(&document);
    let suggested = format!("{}.pdf", document.stem);
    let root = document
        .file_path
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let kind = document.kind;
    let text = document.text;

    window
        .spawn(cx, async move |cx| -> Result<()> {
            let Some(path) = choose_save_path(explicit_path, directory, suggested, "pdf", cx).await? else {
                return Ok(());
            };
            // 컴파일은 백그라운드에서 한다 (큰 문서는 시간이 걸린다)
            let built = cx
                .background_spawn(async move { build_pdf(kind, text, root, &fonts) })
                .await;
            match built {
                Ok(pdf) => {
                    let message = match write_file(path.clone(), pdf, cx).await {
                        Ok(()) => i18n::t!("typst_preview.export.done", path = path.display()),
                        Err(error) => {
                            i18n::t!("typst_preview.error.write_failed", path = path.display(), error = error)
                        }
                    };
                    cx.update(|_, cx| show_toast(&workspace_handle, message, cx))?;
                }
                Err(errors) => cx.update(|window, cx| show_compile_error(window, &errors, cx))?,
            }
            Ok(())
        })
        .detach_and_log_err(cx);
}

/// 메뉴: 인쇄. 임시 PDF 를 만들어 운영체제의 인쇄 기능으로 넘긴다. 중간 Typst 결과는 저장하지 않는다.
fn print_document(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let workspace_handle = workspace.weak_handle();
    let Some(document) = active_document(workspace, cx) else {
        show_toast(
            &workspace_handle,
            i18n::t!("typst_preview.error.no_document").to_string(),
            cx,
        );
        return;
    };
    let fonts = preview_font_data(cx);
    let root = document
        .file_path
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let kind = document.kind;
    let text = document.text;
    let stem = document.stem;

    window
        .spawn(cx, async move |cx| -> Result<()> {
            let built = cx
                .background_spawn(async move {
                    let pdf = build_pdf(kind, text, root, &fonts)?;
                    // 임시 PDF 를 쓰고 인쇄를 요청한다. 쓰기·실행 실패는 진단 형태로 감싸 같은 경로로 알린다
                    send_to_printer(&stem, pdf).await.map_err(|error| {
                        vec![Diagnostic {
                            is_error: true,
                            message: i18n::t!("typst_preview.print.failed", error = error),
                            line: None,
                            column: None,
                            hints: Vec::new(),
                        }]
                    })
                })
                .await;
            match built {
                Ok(outcome) => {
                    let message = match outcome {
                        PrintOutcome::SentToPrinter(name) => i18n::t!("typst_preview.print.sent", name = name),
                        PrintOutcome::OpenedInViewer(name) => i18n::t!("typst_preview.print.opened", name = name),
                    };
                    cx.update(|_, cx| show_toast(&workspace_handle, message, cx))?;
                }
                Err(errors) => cx.update(|window, cx| show_compile_error(window, &errors, cx))?,
            }
            Ok(())
        })
        .detach_and_log_err(cx);
}

/// 인쇄 요청 결과
enum PrintOutcome {
    /// 인쇄 명령에 넘겼다
    SentToPrinter(String),
    /// 인쇄 명령을 쓸 수 없어 PDF 뷰어로 열었다 (뷰어에서 인쇄)
    OpenedInViewer(String),
}

/// 임시 PDF 파일을 보관하는 폴더
fn print_directory() -> PathBuf {
    std::env::temp_dir().join("doneditor-print")
}

/// 하루가 지난 임시 PDF 를 지운다 (인쇄 프로그램이 파일을 읽는 동안 지우지 않도록 바로 지우지 않고 나중에 정리한다).
fn remove_stale_print_files(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let limit = Duration::from_secs(24 * 60 * 60);
    // 폴더의 모든 파일을 훑으며 오래된 것만 지운다
    for entry in entries.flatten() {
        let old = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age > limit);
        if old {
            std::fs::remove_file(entry.path()).ok();
        }
    }
}

/// PDF 를 임시 파일로 쓰고 운영체제의 인쇄 기능으로 넘긴다.
async fn send_to_printer(stem: &str, pdf: Vec<u8>) -> Result<PrintOutcome> {
    let directory = print_directory();
    std::fs::create_dir_all(&directory).with_context(|| directory.display().to_string())?;
    remove_stale_print_files(&directory);

    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    // 파일 이름에 쓸 수 없는 글자는 밑줄로 바꾼다
    let safe_stem: String = stem
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let file_name = format!("{safe_stem}-{seconds}.pdf");
    let path = directory.join(&file_name);
    std::fs::write(&path, pdf).with_context(|| path.display().to_string())?;
    open_for_printing(&path, file_name).await
}

#[cfg(target_os = "windows")]
async fn open_for_printing(path: &Path, name: String) -> Result<PrintOutcome> {
    // 연결된 PDF 프로그램의 "인쇄" 동작으로 보낸다. 프로그램이 인쇄 동작을 지원하지 않으면(연결 없음 오류)
    // 같은 파일을 일반 "열기"로 열어 뷰어에서 인쇄하게 한다. 경로는 환경 변수로 넘겨 인용 문제를 피한다.
    let script = "$f = $env:DON_PRINT_FILE; try { Start-Process -FilePath $f -Verb Print -ErrorAction Stop; exit 0 }                   catch { try { Start-Process -FilePath $f -ErrorAction Stop; exit 2 } catch { exit 1 } }";
    // 프로젝트 표준 비동기 명령(Windows 에서 콘솔 창을 띄우지 않는다)을 쓴다
    let status = util::command::new_smol_command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            script,
        ])
        .env("DON_PRINT_FILE", path)
        .status()
        .await
        .context("powershell.exe")?;
    match status.code() {
        Some(0) => Ok(PrintOutcome::SentToPrinter(name)),
        Some(2) => Ok(PrintOutcome::OpenedInViewer(name)),
        _ => anyhow::bail!(i18n::t!("typst_preview.print.no_program")),
    }
}

#[cfg(not(target_os = "windows"))]
async fn open_for_printing(path: &Path, name: String) -> Result<PrintOutcome> {
    use smol::process::Stdio;
    // CUPS 의 lp 로 기본 프린터에 보낸다. lp 가 없거나 실패하면 PDF 뷰어로 연다.
    let printed = util::command::new_smol_command("lp")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .is_ok_and(|status| status.success());
    if printed {
        return Ok(PrintOutcome::SentToPrinter(name));
    }
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let opened = util::command::new_smol_command(opener)
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .is_ok_and(|status| status.success());
    if opened {
        Ok(PrintOutcome::OpenedInViewer(name))
    } else {
        anyhow::bail!(i18n::t!("typst_preview.print.no_program"))
    }
}
