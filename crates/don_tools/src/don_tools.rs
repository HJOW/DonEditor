//! [DonEditor] MCP와 Agent의 공용 편집기 도구. 모든 UI 접근은 GPUI 스레드에서 실행한다.
mod approval;
pub mod registry;
pub use registry::definitions;

use anyhow::{Context as _, Result, bail, ensure};
use editor::{Editor, MultiBufferOffset, SelectionEffects, ToOffset as _};
use futures::channel::oneshot;
use gpui::{App, Context, Entity, Focusable as _, Global, ReadGlobal as _, Task, Window};
use serde_json::{Value, json};
use settings::{SettingsContent, SettingsStore};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use workspace::{OpenOptions, OpenVisible, Workspace, pane::SaveIntent};

/// Agent 패널이 등록하는 읽기 전용 대화 기록 조회 함수이다.
struct AgentHistoryProvider(fn(&Workspace, &App) -> Value);
impl Global for AgentHistoryProvider {}

/// 순환 의존성 없이 Agent 패널의 대화 기록을 공용 도구에 연결한다.
pub fn register_agent_history(cx: &mut App, provider: fn(&Workspace, &App) -> Value) {
    cx.set_global(AgentHistoryProvider(provider));
}

/// 현재 편집기를 요구하는 도구에서 공통으로 오류를 반환한다.
fn active_editor(workspace: &Workspace, cx: &App) -> Result<Entity<Editor>> {
    workspace
        .active_item_as::<Editor>(cx)
        .context("활성 텍스트 편집기가 없습니다")
}

/// 탭 경로와 제목을 사용자 데이터 그대로 반환한다.
fn tabs(workspace: &Workspace, cx: &App) -> Vec<Value> {
    workspace
        .items(cx)
        .map(|item| {
            let path = item
                .project_path(cx)
                .and_then(|path| workspace.project().read(cx).absolute_path(&path, cx));
            json!({"title":item.tab_content_text(0,cx),"path":path,"dirty":item.is_dirty(cx)})
        })
        .collect()
}

/// 활성 파일·텍스트·선택·탭을 Agent 첨부와 MCP 리소스에서 공유한다.
pub fn active_context(workspace: &Workspace, cx: &App) -> Value {
    let editor = workspace.active_item_as::<Editor>(cx);
    let active = editor.map(|editor| {
        let editor = editor.read(cx);
        // 표시 맵을 갱신하지 않고 버퍼 스냅샷으로 선택 앵커를 해석한다.
        let snapshot = editor.buffer().read(cx).snapshot(cx);
        let selections: Vec<_> = editor
            .selections
            .disjoint_anchors()
            .iter()
            .chain(editor.selections.pending_anchor())
            .map(|selection| json!({"start":selection.start.to_offset(&snapshot).0,"end":selection.end.to_offset(&snapshot).0}))
            .collect();
        let path = editor
            .project_path(cx)
            .and_then(|path| workspace.project().read(cx).absolute_path(&path, cx));
        json!({"title":editor.title(cx),"path":path,"text":editor.text(cx),"selections":selections})
    });
    json!({"active":active,"tabs":tabs(workspace,cx)})
}

/// 무시되지 않은 프로젝트 파일의 절대 경로를 일정 개수까지 모은다.
fn project_files(workspace: &Workspace, limit: usize, cx: &App) -> Vec<PathBuf> {
    workspace
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .flat_map(|worktree| {
            let snapshot = worktree.read(cx).snapshot();
            snapshot
                .files(false, 0)
                .map(|entry| snapshot.abs_path().join(entry.path.as_std_path()))
                .collect::<Vec<_>>()
        })
        .take(limit)
        .collect()
}

/// 승인창을 띄운다. 모달이 닫히거나 워크스페이스가 사라지면 실행하지 않는다.
fn approve(
    detail: String,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> oneshot::Receiver<bool> {
    let (sender, receiver) = oneshot::channel();
    workspace.toggle_modal(window, cx, move |window, cx| {
        approval::ToolApproval::new(detail, sender, window, cx)
    });
    receiver
}

/// 변경 전후 전체 내용을 누락 없이 미리보기에 넣는다.
fn diff_preview(title: &str, old: &str, new: &str) -> String {
    let mut preview = format!(
        "{title}\n--- {}\n+++ {}\n",
        i18n::t!("tools.diff.before"),
        i18n::t!("tools.diff.after")
    );
    // 삭제와 추가 줄을 원래 순서대로 보여 준다.
    for (prefix, text) in [("-", old), ("+", new)] {
        for line in text.split_inclusive('\n') {
            preview.push_str(prefix);
            preview.push_str(line);
        }
        if !text.ends_with('\n') {
            preview.push('\n');
        }
    }
    preview
}

/// 중첩 객체만 재귀 병합하고 나머지 값은 주어진 값으로 교체한다.
fn merge_json(target: &mut Value, patch: Value) {
    if let (Some(target), Some(patch)) = (target.as_object_mut(), patch.as_object()) {
        // 지정된 설정만 덮어쓰며 지정하지 않은 설정은 유지한다.
        for (key, value) in patch {
            merge_json(target.entry(key.clone()).or_insert(Value::Null), value.clone());
        }
    } else {
        *target = patch;
    }
}

/// JSON 스키마를 검사하고, 수정 작업을 승인한 뒤 결과를 돌려준다.
pub fn execute(
    name: &str,
    arguments: Value,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Task<Result<Value>> {
    execute_cancellable(name, arguments, workspace, window, cx, Arc::new(AtomicBool::new(false)))
}

/// MCP에서 취소된 요청은 사용자 승인 직후에도 적용되지 않도록 토큰을 검사한다.
pub fn execute_cancellable(
    name: &str,
    arguments: Value,
    _workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
    cancelled: Arc<AtomicBool>,
) -> Task<Result<Value>> {
    if let Err(error) = registry::validate(name, &arguments) {
        return Task::ready(Err(error));
    }
    let name = name.to_owned();
    cx.spawn_in(window, async move |workspace, cx| {
        ensure!(!cancelled.load(Ordering::Acquire), "도구 요청이 취소되었습니다");
        let args = arguments;
        match name.as_str() {
            "open_file" | "open_folder" => {
                let path = PathBuf::from(args["path"].as_str().unwrap());
                ensure!(path.is_absolute(), "경로는 절대 경로여야 합니다");
                if name == "open_folder" {
                    let fs = workspace.update(cx, |workspace, cx| workspace.project().read(cx).fs().clone())?;
                    ensure!(fs.metadata(&path).await?.is_some_and(|metadata| metadata.is_dir), "폴더가 존재하지 않습니다");
                    let results = workspace.update_in(cx, |workspace, window, cx| workspace.open_paths(vec![path], OpenOptions {visible: Some(OpenVisible::All), ..Default::default()}, None, window, cx))?.await;
                    // 폴더는 편집기 탭을 만들지 않으므로 None 결과가 정상이다.
                    for result in results.into_iter().flatten() {result?;}
                    return Ok(json!({"opened": true}));
                }
                workspace.update_in(cx, |workspace, window, cx| workspace.open_abs_path(path, OpenOptions::default(), window, cx))?.await?;
                return Ok(json!({"opened":true}));
            }
            "save" => {
                require_approval(&workspace, &name, &args, cx, &cancelled).await?;
                workspace.update_in(cx, |workspace, window, cx| workspace.save_active_item(SaveIntent::Save, window, cx))?.await?;
                return Ok(json!({"saved":true}));
            }
            "save_all" => {
                require_approval(&workspace, &name, &args, cx, &cancelled).await?;
                let saved = workspace.update_in(cx, |workspace, window, cx| workspace.save_all_for_tools(window,cx))?.await?;
                ensure!(saved, "저장이 취소되었습니다");
                return Ok(json!({"saved":true}));
            }
            "set_buffer_text" | "insert_text" | "replace_range" => {
                let (editor, old, title) = workspace.update(cx, |workspace,cx| {
                    let editor = active_editor(workspace,cx)?;
                    ensure!(editor.read(cx).buffer().read(cx).is_singleton(), "발췌 편집기에서는 지원하지 않습니다. 원본 파일 탭을 열어 주세요");
                    let title = editor.read(cx).project_path(cx)
                        .and_then(|path| workspace.project().read(cx).absolute_path(&path, cx))
                        .map(|path| path.display().to_string()).unwrap_or_else(|| editor.read(cx).title(cx).to_string());
                    Ok::<_,anyhow::Error>((editor.clone(),editor.read(cx).text(cx),title))
                })??;
                let (start,end) = match name.as_str() {
                    "set_buffer_text" => (0,old.len()),
                    "insert_text" => {let offset=args["offset"].as_u64().unwrap() as usize; (offset,offset)},
                    _ => (args["start"].as_u64().unwrap() as usize,args["end"].as_u64().unwrap() as usize),
                };
                let new = registry::replace_text(&old,start,end,args["text"].as_str().unwrap())?;
                let preview = diff_preview(&format!("{name} — {title}"),&old,&new);
                let accepted = workspace.update_in(cx, |workspace,window,cx| approve(preview,workspace,window,cx))?;
                ensure!(accepted.await.unwrap_or(false), "사용자가 편집을 거절했습니다");
                editor.update_in(cx, |editor,_,cx| {
                    ensure!(!cancelled.load(Ordering::Acquire), "도구 요청이 취소되었습니다");
                    ensure!(editor.text(cx)==old,"승인 대기 중 파일이 변경되었습니다. 새 변경안을 요청하세요");
                    ensure!(!editor.read_only(cx),"읽기 전용 편집기입니다");
                    editor.edit([(MultiBufferOffset(start)..MultiBufferOffset(end),args["text"].as_str().unwrap().to_owned())],cx);
                    Ok::<_,anyhow::Error>(())
                })??;
                return Ok(json!({"edited":true,"saved":false}));
            }
            "project_search" | "replace_in_project" => {
                let query = args["query"].as_str().unwrap();
                ensure!(!query.is_empty(),"검색어가 비어 있습니다");
                let (paths,fs) = workspace.update(cx,|workspace,cx| (project_files(workspace,10000,cx),workspace.project().read(cx).fs().clone()))?;
                let mut matches = Vec::new();
                let mut edits = Vec::new();
                let mut changed_bytes = 0;
                // 디스크의 텍스트와 열린 버퍼를 비교해 미저장 편집을 보호한다.
                for path in paths {
                    if !fs.metadata(&path).await?.is_some_and(|metadata| metadata.len <= 2 * 1024 * 1024) {continue;}
                    let text = match fs.load(&path).await { Ok(text) if text.len()<=2*1024*1024 && !text.contains('\0') => text, _=>continue };
                    let ranges=registry::find_text(&text,query)?;
                    if ranges.is_empty() {continue;}
                    matches.push(json!({"path":path,"ranges":ranges}));
                    if name=="replace_in_project" {
                        changed_bytes += text.len();
                        ensure!(changed_bytes <= 8 * 1024 * 1024, "변경안이 8 MiB를 초과합니다. 검색 범위를 줄여 주세요");
                        edits.push((path,text.clone(),text.replace(query,args["replacement"].as_str().unwrap())));
                    }
                    if matches.len()>=1000 {break;}
                }
                if name=="project_search" {return Ok(json!({"matches":matches,"limits":{"files":10000,"matching_files":1000,"bytes_per_file":2097152}}));}
                let preview=edits.iter().map(|(path,old,new)| diff_preview(&path.display().to_string(),old,new)).collect::<Vec<_>>().join("\n");
                if edits.is_empty() {return Ok(json!({"changed_files":0}));}
                let accepted=workspace.update_in(cx,|workspace,window,cx| approve(preview,workspace,window,cx))?;
                ensure!(accepted.await.unwrap_or(false),"사용자가 프로젝트 편집을 거절했습니다");
                ensure!(!cancelled.load(Ordering::Acquire), "도구 요청이 취소되었습니다");
                let count=edits.len();
                // 파일을 먼저 전부 열고 검증해 승인 중 변경된 버퍼에 덮어쓰지 않는다.
                let mut prepared=Vec::new();
                for (path,old,new) in edits {
                    let item=workspace.update_in(cx,|workspace,window,cx| workspace.open_abs_path(path,OpenOptions::default(),window,cx))?.await?;
                    let editor=item.downcast::<Editor>().context("텍스트 파일이 아닙니다")?;
                    editor.update(cx,|editor,cx| {ensure!(editor.text(cx)==old,"파일이 변경됐거나 미저장 수정이 있습니다");ensure!(!editor.read_only(cx),"읽기 전용 파일입니다");Ok::<_,anyhow::Error>(())})??;
                    prepared.push((editor,old,new));
                }
                // 검증 완료된 버퍼를 한 UI 갱신에서 수정한다.
                workspace.update_in(cx,|_,_,cx| {
                    ensure!(!cancelled.load(Ordering::Acquire), "도구 요청이 취소되었습니다");
                    for (editor,old,_) in &prepared {
                        ensure!(editor.read(cx).text(cx)==*old,"승인 후 파일이 변경되었습니다");
                        ensure!(!editor.read(cx).read_only(cx),"읽기 전용 파일입니다");
                        ensure!(editor.read(cx).buffer().read(cx).is_singleton(),"원본 파일 탭이 아닙니다");
                    }
                    for (editor,old,new) in prepared {editor.update(cx,|editor,cx| editor.edit([(MultiBufferOffset(0)..MultiBufferOffset(old.len()),new)],cx));}
                    Ok::<_,anyhow::Error>(())
                })??;
                return Ok(json!({"changed_files":count,"saved":false}));
            }
            "set_buffer_language" => {
                require_approval(&workspace, &name, &args, cx, &cancelled).await?;
                let languages=workspace.update(cx,|workspace,_| workspace.app_state().languages.clone())?;
                let language=languages.language_for_name(args["language"].as_str().unwrap()).await?;
                return workspace.update(cx,|workspace,cx| {
                    ensure!(!cancelled.load(Ordering::Acquire), "도구 요청이 취소되었습니다");
                    let editor=active_editor(workspace,cx)?;
                    let buffers=editor.read(cx).buffer().read(cx).all_buffers();
                    for buffer in buffers {buffer.update(cx,|buffer,cx| buffer.set_language(Some(language.clone()),cx));}
                    Ok(json!({"language":language.name()}))
                })?;
            }
            "run_in_terminal" => {
                require_approval(&workspace, &name, &args, cx, &cancelled).await?;
                let task=workspace.update_in(cx,|workspace,window,cx| {
                    let panel=workspace.panel::<terminal_view::terminal_panel::TerminalPanel>(cx).context("터미널 패널이 없습니다")?;
                    let command=args["command"].as_str().unwrap().to_owned();
                    let task=task::SpawnInTerminal{ id:task::TaskId("don-agent".into()),label:"Don Agent".into(),full_label:command.clone(),command_label:command.clone(),command:Some(command),use_new_terminal:true,allow_concurrent_runs:true,..Default::default()};
                    Ok::<_,anyhow::Error>(panel.update(cx,|panel,cx| panel.spawn_task(&task,window,cx)))
                })??;
                task.await?;
                return Ok(json!({"started":true}));
            }
            _ => {}
        }
        let read_only=definitions().into_iter().find(|tool| tool["name"]==name).is_some_and(|tool| tool["annotations"]["readOnlyHint"]==true);
        if !read_only { require_approval(&workspace, &name, &args, cx, &cancelled).await?; }
        workspace.update_in(cx,|workspace,window,cx| execute_sync(&name,args,workspace,window,cx))?
    })
}

/// 모든 수정 경로에서 공용 승인 모달을 사용한다.
async fn require_approval(
    workspace: &gpui::WeakEntity<Workspace>,
    name: &str,
    args: &Value,
    cx: &mut gpui::AsyncWindowContext,
    cancelled: &AtomicBool,
) -> Result<()> {
    // 승인 대상 탭이 바뀌면 새 대상으로 명령을 실행하지 않는다.
    let item = workspace.update(cx, |workspace, cx| workspace.active_item(cx).map(|item| item.item_id()))?;
    let preview = format!("{name}\n{}", serde_json::to_string_pretty(args)?);
    let receiver = workspace.update_in(cx, |workspace, window, cx| approve(preview, workspace, window, cx))?;
    ensure!(receiver.await.unwrap_or(false), "사용자가 도구 실행을 거절했습니다");
    ensure!(!cancelled.load(Ordering::Acquire), "도구 요청이 취소되었습니다");
    let current = workspace.update(cx, |workspace, cx| workspace.active_item(cx).map(|item| item.item_id()))?;
    ensure!(
        item == current,
        "승인 대기 중 활성 탭이 변경되었습니다. 다시 요청해 주세요"
    );
    Ok(())
}

/// UI 스레드에서 즉시 끝나는 조회·선택·액션·설정 도구를 실행한다.
fn execute_sync(
    name: &str,
    args: Value,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<Value> {
    match name {
        "list_actions" => {
            let mut generator = schemars::SchemaGenerator::default();
            let actions: Vec<_> = cx
                .action_schemas(&mut generator)
                .into_iter()
                .map(|(name, schema)| json!({"name":name,"inputSchema":schema}))
                .collect();
            Ok(json!({"actions":actions,"$defs":generator.definitions()}))
        }
        "list_open_tabs" => Ok(json!({"tabs":tabs(workspace,cx)})),
        "list_agent_conversations" => {
            let provider = cx
                .try_global::<AgentHistoryProvider>()
                .context("Agent 패널이 초기화되지 않았습니다")?;
            Ok((provider.0)(workspace, cx))
        }
        "list_project_files" => {
            Ok(json!({"files":project_files(workspace,args["limit"].as_u64().unwrap_or(10000) as usize,cx)}))
        }
        "get_buffer_text" | "get_selection" => {
            let active = active_context(workspace, cx)["active"].clone();
            ensure!(active.is_object(), "활성 텍스트 편집기가 없습니다");
            if name == "get_selection" {
                Ok(json!({"selections":active["selections"]}))
            } else {
                Ok(active)
            }
        }
        "find_in_buffer" => Ok(
            json!({"ranges":registry::find_text(&active_editor(workspace,cx)?.read(cx).text(cx),args["query"].as_str().unwrap())?}),
        ),
        "set_selection" | "go_to_line" => {
            let editor = active_editor(workspace, cx)?;
            let text = editor.read(cx).text(cx);
            let (start, end) = if name == "set_selection" {
                (
                    args["start"].as_u64().unwrap() as usize,
                    args["end"].as_u64().unwrap() as usize,
                )
            } else {
                let row = args["line"].as_u64().unwrap() as usize;
                let column = args["column"].as_u64().unwrap_or(1) as usize;
                let mut offset = 0;
                let line = text.split('\n').nth(row - 1).context("줄 번호가 범위를 벗어났습니다")?;
                ensure!(column - 1 <= line.len(), "열 번호가 범위를 벗어났습니다");
                for preceding in text.split_inclusive('\n').take(row - 1) {
                    offset += preceding.len();
                }
                offset += column - 1;
                (offset, offset)
            };
            registry::replace_text(&text, start, end, "")?;
            editor.update(cx, |editor, cx| {
                editor.change_selections(SelectionEffects::default(), window, cx, |selections| {
                    selections.select_ranges([MultiBufferOffset(start)..MultiBufferOffset(end)])
                })
            });
            Ok(json!({"start":start,"end":end}))
        }
        "get_outline" => {
            let editor = active_editor(workspace, cx)?;
            let snapshot = editor.read(cx).buffer().read(cx).snapshot(cx);
            let symbols = snapshot
                .outline(None)
                .map(|outline| {
                    outline
                        .items
                        .into_iter()
                        .map(|item| json!({"text":item.text,"depth":item.depth}))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok(json!({"symbols":symbols}))
        }
        "get_diagnostics" => {
            let editor = active_editor(workspace, cx)?;
            let snapshot = editor.read(cx).buffer().read(cx).snapshot(cx);
            let diagnostics:Vec<_>=snapshot.diagnostics_in_range(MultiBufferOffset(0)..snapshot.len()).map(|entry|json!({"start":entry.range.start.0,"end":entry.range.end.0,"message":entry.diagnostic.message,"severity":format!("{:?}",entry.diagnostic.severity)})).collect();
            Ok(json!({"diagnostics":diagnostics}))
        }
        "get_settings" => {
            let store = SettingsStore::global(cx);
            Ok(json!({"defaults":store.raw_default_settings(),"user":store.raw_user_settings()}))
        }
        "update_settings" | "set_ui_language" | "set_theme" => {
            let patch = match name {
                "set_ui_language" => json!({"ui_language":args["language"]}),
                "set_theme" => json!({"theme":args["name"]}),
                _ => args["settings"].clone(),
            };
            let mut candidate = serde_json::to_value(
                SettingsStore::global(cx)
                    .raw_user_settings()
                    .map(|s| s.content.as_ref())
                    .cloned()
                    .unwrap_or_default(),
            )?;
            merge_json(&mut candidate, patch.clone());
            let _: SettingsContent = serde_json::from_value(candidate)?;
            settings::update_settings_file(
                workspace.project().read(cx).fs().clone(),
                cx,
                Box::new(move |settings, _| {
                    let mut value = serde_json::to_value(&*settings).unwrap_or_default();
                    merge_json(&mut value, patch);
                    if let Ok(updated) = serde_json::from_value(value) {
                        *settings = updated;
                    }
                }),
            );
            Ok(json!({"queued":true}))
        }
        "undo" | "redo" => {
            active_editor(workspace, cx)?.update(cx, |editor, cx| {
                if name == "undo" {
                    editor.undo(&editor::actions::Undo, window, cx)
                } else {
                    editor.redo(&editor::actions::Redo, window, cx)
                }
            });
            Ok(json!({"executed":true}))
        }
        _ => {
            let action_name = match name {
                "dispatch_action" => args["name"].as_str().unwrap(),
                "toggle_panel" => args["action"].as_str().unwrap(),
                "close_tab" => "pane::CloseActiveItem",
                "format_buffer" => "editor::Format",
                "split_pane" => match args["direction"].as_str().unwrap() {
                    "up" => "workspace::SplitUp",
                    "down" => "workspace::SplitDown",
                    "left" => "workspace::SplitLeft",
                    _ => "workspace::SplitRight",
                },
                _ => bail!("도구 구현을 찾을 수 없습니다: {name}"),
            };
            let action = cx.build_action(action_name, args.get("args").cloned())?;
            // 범용 액션은 패널의 현재 포커스를 보존해야 삭제·탐색 등도 제어할 수 있다.
            if !matches!(name, "dispatch_action" | "toggle_panel") {
                if let Ok(editor) = active_editor(workspace, cx) {
                    window.focus(&editor.focus_handle(cx), cx);
                }
            }
            window.dispatch_action(action, cx);
            Ok(json!({"dispatched":action_name}))
        }
    }
}
