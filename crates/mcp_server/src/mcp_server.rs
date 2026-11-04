//! [DonEditor] 로컬 MCP 서버와 GPUI 도구 실행을 연결한다.
pub mod limits;
pub mod protocol;
pub mod sessions;
pub mod transport;

use anyhow::Result;
use gpui::{App, AppContext as _, Global, PromptLevel, actions};
use serde_json::Value;
use settings::{RegisterSetting, Settings, SettingsContent, SettingsStore};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use workspace::Workspace;

actions!(
    mcp_server,
    [
        /// [DonEditor] 로컬 MCP 서버를 켜거나 끈다.
        ToggleServer,
        /// [DonEditor] 연결 정보 파일의 위치와 현재 상태를 표시한다.
        ShowConnectionInfo,
    ]
);

/// 설정 변경을 감지하는 MCP 전용 설정 값.
#[derive(RegisterSetting, Clone, Copy, PartialEq, Eq)]
struct McpSettings {
    enabled: bool,
    port: u16,
}
impl Settings for McpSettings {
    /// 설정이 없으면 비활성, 포트 자동 할당을 사용한다.
    fn from_settings(content: &SettingsContent) -> Self {
        Self {
            enabled: content.mcp_server.as_ref().and_then(|s| s.enabled).unwrap_or(false),
            port: content.mcp_server.as_ref().and_then(|s| s.port).unwrap_or(0),
        }
    }
}
struct Request {
    name: String,
    args: Value,
    reply: mpsc::SyncSender<Result<Value, String>>,
    deadline: Instant,
    cancelled: Arc<AtomicBool>,
}
struct UiBackend(smol::channel::Sender<Request>);
impl protocol::Backend for UiBackend {
    /// HTTP 작업 스레드에서 UI로 요청을 보내고 유한 시간만 기다린다.
    fn call(&self, name: &str, args: Value) -> Result<Value, String> {
        self.call_cancellable(name, args, Arc::new(AtomicBool::new(false)))
    }
    /// 세션·연결의 취소 플래그를 UI 실행기까지 보존한다.
    fn call_cancellable(&self, name: &str, args: Value, cancelled: Arc<AtomicBool>) -> Result<Value, String> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.0
            .try_send(Request {
                name: name.into(),
                args,
                reply,
                deadline: Instant::now() + Duration::from_secs(290),
                cancelled: cancelled.clone(),
            })
            .map_err(|_| "도구 요청이 너무 많거나 서버가 종료되었습니다".to_owned())?;
        let deadline = Instant::now() + Duration::from_secs(300);
        // UI가 일시적으로 바빠도 세션 종료에 묶여 HTTP 작업 스레드가 남지 않도록 한다.
        loop {
            if cancelled.load(Ordering::Acquire) {
                return Err("도구 요청이 취소되었습니다".into());
            }
            if Instant::now() >= deadline {
                return Err("도구 요청 시간이 만료되었습니다".into());
            }
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(result) => return result,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err("편집기 또는 MCP 서버가 닫혔습니다".into()),
            }
        }
    }
    /// Agent와 동일한 명세를 MCP에 게시한다.
    fn tools(&self) -> Vec<Value> {
        don_tools::definitions()
    }
}
struct McpState {
    settings: McpSettings,
    server: Option<transport::Server>,
    error: Option<String>,
    _executor: Option<gpui::Task<()>>,
    path: PathBuf,
}
impl Global for McpState {}

/// 설정 변경 시 서버를 중지/재시작하고 토큰을 새로 발급한다.
fn apply_settings(cx: &mut App) {
    let settings = *McpSettings::get_global(cx);
    if cx
        .try_global::<McpState>()
        .is_some_and(|state| state.settings == settings)
    {
        return;
    }
    // 이전 요청 채널을 닫아 중지된 서버의 작업이 실행되지 않도록 한다.
    if cx.has_global::<McpState>() {
        let previous = cx.global_mut::<McpState>();
        previous.server.take();
        previous._executor.take();
    }
    let path = transport::connection_path();
    let mut state = McpState {
        settings,
        server: None,
        error: None,
        _executor: None,
        path: path.clone(),
    };
    if settings.enabled {
        let (sender, receiver) = smol::channel::bounded::<Request>(8);
        match transport::Server::start(settings.port, path, Arc::new(UiBackend(sender))) {
            Ok(server) => {
                state.server = Some(server);
                state._executor = Some(cx.spawn(async move |cx| {
                    // 요청을 직렬로 실행해 승인 모달끼리 충돌하지 않게 한다.
                    while let Ok(request) = receiver.recv().await {
                        if request.deadline <= Instant::now() || request.cancelled.load(Ordering::Acquire) {
                            let _ = request
                                .reply
                                .send(Err("대기 중 도구 요청 시간이 만료되었습니다".into()));
                            continue;
                        }
                        let cancelled = request.cancelled.clone();
                        let deadline = request.deadline;
                        let task = cx.update(|cx| {
                            let window = cx
                                .active_window()
                                .and_then(|w| w.downcast::<Workspace>())
                                .or_else(|| cx.windows().into_iter().find_map(|w| w.downcast::<Workspace>()));
                            let window = window.ok_or_else(|| anyhow::anyhow!("열린 워크스페이스가 없습니다"))?;
                            window.update(cx, |workspace, window, cx| {
                                don_tools::execute_cancellable(
                                    &request.name,
                                    request.args,
                                    workspace,
                                    window,
                                    cx,
                                    request.cancelled.clone(),
                                )
                            })
                        });
                        let result = match task {
                            Ok(Ok(task)) => {
                                // HTTP가 시간 초과되기 전에 UI 작업을 취소해 뒤늦은 승인을 막는다.
                                match futures::future::select(
                                    Box::pin(task),
                                    Box::pin(async move {
                                        // 사용자 승인 중에도 외부 취소와 절대 마감 시각을 주기적으로 확인한다.
                                        loop {
                                            if cancelled.load(Ordering::Acquire) {
                                                break "도구 요청이 취소되었습니다";
                                            }
                                            if deadline <= Instant::now() {
                                                break "도구 승인/실행 시간이 만료되었습니다";
                                            }
                                            smol::Timer::after(Duration::from_millis(50)).await;
                                        }
                                    }),
                                )
                                .await
                                {
                                    futures::future::Either::Left((result, _)) => result.map_err(|e| e.to_string()),
                                    futures::future::Either::Right((message, task)) => {
                                        drop(task);
                                        let _ = cx.update(|cx| cx.refresh_windows());
                                        Err(message.to_owned())
                                    }
                                }
                            }
                            Ok(Err(e)) | Err(e) => Err(e.to_string()),
                        };
                        let _ = request.reply.send(result);
                    }
                }));
            }
            Err(error) => {
                log::error!("MCP 서버 시작 실패: {error}");
                state.error = Some(error.to_string());
            }
        }
    }
    cx.set_global(state);
    cx.refresh_windows();
}

/// 워크스페이스 메뉴 액션과 설정 감시를 등록한다.
pub fn init(cx: &mut App) {
    apply_settings(cx);
    cx.observe_global::<SettingsStore>(apply_settings).detach();
    cx.observe_new::<Workspace>(|workspace, _, _| {
        workspace.register_action(|workspace, _: &ToggleServer, _, cx| {
            let enabled = !McpSettings::get_global(cx).enabled;
            settings::update_settings_file(
                workspace.project().read(cx).fs().clone(),
                cx,
                Box::new(move |settings, _| {
                    settings.mcp_server.get_or_insert_default().enabled = Some(enabled);
                }),
            );
        });
        workspace.register_action(|_, _: &ShowConnectionInfo, window, cx| {
            let state = cx.global::<McpState>();
            let detail = if let Some(error) = &state.error {
                format!("{error}\n{}", state.path.display())
            } else {
                format!(
                    "{}\n{}",
                    if state.server.is_some() {
                        i18n::t!("mcp.enabled")
                    } else {
                        i18n::t!("mcp.disabled")
                    },
                    state.path.display()
                )
            };
            let _ = window.prompt(
                PromptLevel::Info,
                i18n::t!("mcp.connection"),
                Some(&detail),
                &[i18n::t!("common.ok")],
                cx,
            );
        });
    })
    .detach();
}
