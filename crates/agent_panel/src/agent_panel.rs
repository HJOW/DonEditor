//! [DonEditor] 우측 Agent 패널. GPL-3.0-or-later; GramEditor/Zed 기반 UI를 사용한다.

use crate::{credentials, provider};

use anyhow::{Result, anyhow};
use db::kvp::KEY_VALUE_STORE;
use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};
use gpui::{
    Action, App, AppContext as _, AsyncWindowContext, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement, Pixels, Render, ScrollHandle, Styled, Task, WeakEntity, Window, actions,
};
use http_client::HttpClient;
use markdown::{Markdown, MarkdownElement, style::MarkdownStyle};
use provider::{ApiError, Message, Provider};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use ui::{Button, ButtonStyle, IconButton, Tooltip, prelude::*};
use ui_input::InputField;
use workspace::{
    Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};

/// 대화·환경 설정을 앱 데이터 DB에 저장하는 키이다. API 키는 이 DB에 넣지 않는다.
const STORAGE_KEY: &str = "DonEditor.AgentPanel.v1";
/// 한 응답에서 자동으로 이어갈 도구 사용 횟수이다.
const MAX_TOOL_ROUNDS: usize = 12;
/// 최근 대화 보관 개수를 제한해 DB 크기가 무한히 늘지 않게 한다.
const MAX_CONVERSATIONS: usize = 20;

/// 전송 계층의 오류를 화면의 표시 언어로 바꾼다.
fn localized_error(error: &anyhow::Error) -> String {
    match error.downcast_ref::<ApiError>() {
        Some(ApiError::Connection) => i18n::t!("agent.error.connect").into(),
        Some(ApiError::Http(status @ (401 | 403))) => i18n::t!("agent.error.auth", status = status),
        Some(ApiError::Http(429)) => i18n::t!("agent.error.rate").into(),
        Some(ApiError::Http(status)) => i18n::t!("agent.error.http", status = status),
        None => error.to_string(),
    }
}

actions!(
    agent_panel,
    [
        /// Agent 패널을 표시하거나 패널과 편집기 사이의 초점을 전환한다.
        ToggleFocus,
        /// 현재 작성한 메시지를 Agent에게 보낸다.
        Send,
        /// 실행 중인 Agent 응답과 후속 도구 호출을 중단한다.
        Stop,
        /// 현재 파일·선택 영역·탭 정보를 다음 메시지에 첨부한다.
        AttachContext,
        /// 이전 대화를 저장하고 새 대화를 시작한다.
        NewChat,
        /// 저장된 대화 기록을 표시한다.
        ShowHistory,
    ]
);

/// MCP 등 외부 제어에서 텍스트를 지정하여 Agent 메시지를 전송한다.
#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = agent_panel)]
#[serde(deny_unknown_fields)]
pub struct SendPrompt {
    pub text: String,
}

/// 키를 노출하지 않고 제공자와 모델만 변경하는 공개 액션이다.
#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = agent_panel)]
#[serde(deny_unknown_fields)]
pub struct Configure {
    #[serde(default)]
    pub provider: Option<Provider>,
    #[serde(default)]
    pub model: Option<String>,
}

/// 대화 한 개의 제목과 공급자 중립적인 메시지 기록이다.
#[derive(Clone, Default, Serialize, Deserialize)]
struct Conversation {
    id: String,
    title: String,
    messages: Vec<Message>,
}

/// 비밀이 아닌 UI 상태와 대화 기록만 DB에 보존한다.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredState {
    provider: Provider,
    model: String,
    width: Option<f32>,
    left_dock: bool,
    conversations: Vec<Conversation>,
}

/// 숨겨도 도크 아이콘이 남는 Agent 패널이다.
pub struct AgentPanel {
    workspace: WeakEntity<Workspace>,
    storage_key: String,
    client: Arc<dyn HttpClient>,
    input: Entity<InputField>,
    model: Entity<InputField>,
    api_key: Entity<InputField>,
    focus: FocusHandle,
    provider: Provider,
    language: i18n::Language,
    width: Option<Pixels>,
    position: DockPosition,
    current: Conversation,
    conversations: Vec<Conversation>,
    rendered: Vec<Entity<Markdown>>,
    scroll: ScrollHandle,
    attached: Option<Value>,
    status: Option<String>,
    show_history: bool,
    running: bool,
    task: Option<Task<()>>,
    persistence: Option<Shared<BoxFuture<'static, ()>>>,
    sizing_task: Option<Task<()>>,
}

/// 모든 새 워크스페이스에 패널 토글 액션을 등록한다. 패널은 앱의 기존 load 흐름에서 추가한다.
pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &ToggleFocus, window, cx| {
            workspace.toggle_panel_focus::<AgentPanel>(window, cx);
        });
        workspace.register_action(|workspace, _: &Send, window, cx| {
            if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                panel.update(cx, |panel, cx| panel.send(&Send, window, cx));
            }
        });
        workspace.register_action(|workspace, _: &Stop, window, cx| {
            if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                panel.update(cx, |panel, cx| panel.stop(&Stop, window, cx));
            }
        });
        workspace.register_action(|workspace, prompt: &SendPrompt, window, cx| {
            if let Some(panel) = workspace.focus_panel::<AgentPanel>(window, cx) {
                panel.update(cx, |panel, cx| {
                    if !panel.running {
                        panel
                            .input
                            .update(cx, |input, cx| input.set_text(prompt.text.clone(), window, cx));
                        panel.send(&Send, window, cx);
                    }
                });
            }
        });
        workspace.register_action(|workspace, config: &Configure, window, cx| {
            if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                panel.update(cx, |panel, cx| panel.configure(config, window, cx));
            }
        });
        workspace.register_action(|workspace, _: &AttachContext, _, cx| {
            let context = don_tools::active_context(workspace, cx);
            if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                panel.update(cx, |panel, cx| {
                    panel.attached = Some(context);
                    panel.status = Some(i18n::t!("agent.context_attached").into());
                    cx.notify();
                });
            }
        });
        workspace.register_action(|workspace, _: &NewChat, _, cx| {
            if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                panel.update(cx, |panel, cx| panel.new_chat(cx));
            }
        });
        workspace.register_action(|workspace, _: &ShowHistory, window, cx| {
            if let Some(panel) = workspace.focus_panel::<AgentPanel>(window, cx) {
                panel.update(cx, |panel, cx| {
                    panel.show_history = true;
                    cx.notify();
                });
            }
        });
    })
    .detach();
}

impl AgentPanel {
    /// 기존 패널과 같은 비동기 로딩 규약을 사용해 UI 생성 전에 DB를 읽는다.
    pub async fn load(workspace: WeakEntity<Workspace>, mut cx: AsyncWindowContext) -> Result<Entity<Self>> {
        let storage_key = workspace.read_with(&cx, |workspace, _| {
            workspace
                .database_id()
                .map(|id| i64::from(id).to_string())
                .or(workspace.session_id())
                .map_or_else(|| STORAGE_KEY.to_string(), |id| format!("{STORAGE_KEY}-{id}"))
        })?;
        let key_for_load = storage_key.clone();
        let state = cx
            .background_spawn(async move {
                KEY_VALUE_STORE
                    .read_kvp(&key_for_load)
                    .ok()
                    .flatten()
                    .and_then(|text| serde_json::from_str::<StoredState>(&text).ok())
                    .unwrap_or_default()
            })
            .await;
        workspace.update_in(&mut cx, |workspace, window, cx| {
            let handle = cx.entity().downgrade();
            let client: Arc<dyn HttpClient> = workspace.client().http_client();
            cx.new(|cx| Self::new(handle, storage_key, client, state, window, cx))
        })
    }

    /// 입력란과 표시 상태를 만든다. 저장된 대화는 기록 버튼으로 연다.
    fn new(
        workspace: WeakEntity<Workspace>,
        storage_key: String,
        client: Arc<dyn HttpClient>,
        state: StoredState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputField::new(window, cx, i18n::t!("agent.prompt")));
        let model = cx.new(|cx| InputField::new(window, cx, i18n::t!("agent.model")));
        model.update(cx, |input, cx| {
            input.set_text(
                if state.model.is_empty() {
                    state.provider.default_model().into()
                } else {
                    state.model
                },
                window,
                cx,
            )
        });
        let api_key = Self::key_input(window, cx);
        Self {
            focus: input.focus_handle(cx),
            input,
            model,
            api_key,
            workspace,
            storage_key,
            client,
            provider: state.provider,
            language: i18n::language(),
            width: state.width.map(px),
            position: if state.left_dock {
                DockPosition::Left
            } else {
                DockPosition::Right
            },
            current: Self::empty_conversation(),
            conversations: state.conversations,
            rendered: vec![],
            scroll: ScrollHandle::new(),
            attached: None,
            status: None,
            show_history: false,
            running: false,
            task: None,
            persistence: None,
            sizing_task: None,
        }
    }

    /// 비밀 입력은 마스킹하며 저장 후 편집기를 교체해 실행 취소 기록도 없앤다.
    fn key_input(window: &mut Window, cx: &mut App) -> Entity<InputField> {
        cx.new(|cx| {
            let input = InputField::new(window, cx, i18n::t!("agent.api_key"));
            input.editor.update(cx, |editor, cx| editor.set_masked(true, cx));
            input
        })
    }

    /// 새 대화에 현재 시각에서 얻은 식별자를 부여한다.
    fn empty_conversation() -> Conversation {
        Conversation {
            id: format!("{:?}", std::time::SystemTime::now()),
            ..Default::default()
        }
    }

    /// 화면의 각 메시지에 대응하는 Markdown 엔티티를 갱신한다.
    fn rebuild_messages(&mut self, cx: &mut Context<Self>) {
        self.rendered = self
            .current
            .messages
            .iter()
            .map(|message| cx.new(|cx| Markdown::new(message.text.clone().into(), None, None, cx)))
            .collect();
        cx.notify();
    }

    /// 스트리밍으로 늘어난 마지막 메시지만 다시 분석한다.
    fn append_stream(&mut self, text: &str, cx: &mut Context<Self>) {
        if let Some(message) = self.current.messages.last_mut() {
            message.text.push_str(text);
            if let Some(markdown) = self.rendered.last() {
                markdown.update(cx, |markdown, cx| markdown.reset(message.text.clone().into(), cx));
            }
        }
        self.scroll.scroll_to_bottom();
        cx.notify();
    }

    /// 현재 대화를 최근 기록에 넣고 비밀을 제외한 상태만 비동기로 저장한다.
    fn persist(&mut self, cx: &mut Context<Self>) {
        if !self.current.messages.is_empty() {
            self.conversations
                .retain(|conversation| conversation.id != self.current.id);
            self.conversations.insert(0, self.current.clone());
            self.conversations.truncate(MAX_CONVERSATIONS);
        }
        let state = StoredState {
            provider: self.provider,
            model: self.model.read(cx).text(cx),
            width: self.width.map(f32::from),
            left_dock: self.position == DockPosition::Left,
            conversations: self.conversations.clone(),
        };
        if let Ok(text) = serde_json::to_string(&state) {
            let storage_key = self.storage_key.clone();
            let previous = self.persistence.take();
            let write = async move {
                // 이전 저장을 먼저 끝내 새 대화나 삭제한 기록이 오래된 쓰기로 되돌아가지 않게 한다.
                if let Some(previous) = previous {
                    previous.await;
                }
                if let Err(error) = KEY_VALUE_STORE.write_kvp(storage_key, text).await {
                    log::error!("Agent 대화 저장 실패: {error}");
                }
            }
            .boxed()
            .shared();
            self.persistence = Some(write.clone());
            cx.spawn(async move |_, _| write.await).detach();
        }
    }

    /// 중단된 도구 호출에 취소 결과를 추가해 다음 API 요청의 형식을 유지한다.
    fn repair_interrupted_calls(&mut self) {
        provider::repair_interrupted_messages(&mut self.current.messages);
    }

    /// 응답을 중단하고 이미 받은 텍스트는 대화 기록에 남긴다.
    fn stop(&mut self, _: &Stop, _: &mut Window, cx: &mut Context<Self>) {
        self.task = None;
        self.running = false;
        self.repair_interrupted_calls();
        self.status = Some(i18n::t!("agent.stopped").into());
        self.rebuild_messages(cx);
        self.persist(cx);
    }

    /// 현재 파일·선택 영역·열린 탭을 전송 전에 명시적으로 첨부한다.
    fn attach(&mut self, cx: &mut Context<Self>) {
        self.attached = self
            .workspace
            .upgrade()
            .map(|workspace| don_tools::active_context(workspace.read(cx), cx));
        self.status = Some(i18n::t!("agent.context_attached").into());
        cx.notify();
    }

    /// 실행 중이 아니면 현재 대화를 보관하고 새 입력을 시작한다.
    fn new_chat(&mut self, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        self.persist(cx);
        self.current = Self::empty_conversation();
        self.attached = None;
        self.status = None;
        self.show_history = false;
        self.rebuild_messages(cx);
    }

    /// UI와 MCP에서 동일한 검사를 거쳐 제공자·모델을 변경한다.
    fn configure(&mut self, config: &Configure, window: &mut Window, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        if let Some(provider) = config.provider {
            if provider != self.provider {
                self.provider = provider;
                self.model
                    .update(cx, |model, cx| model.set_text(provider.default_model(), window, cx));
                self.api_key = Self::key_input(window, cx);
            }
        }
        if let Some(model) = config.model.as_deref().filter(|model| !model.trim().is_empty()) {
            self.model
                .update(cx, |input, cx| input.set_text(model.trim().to_string(), window, cx));
        }
        self.status = None;
        self.persist(cx);
        cx.notify();
    }

    /// 입력을 사용자 메시지에 넣고 스트리밍과 공용 도구 실행을 반복한다.
    fn send(&mut self, _: &Send, window: &mut Window, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        let mut prompt = self.input.read(cx).text(cx);
        if prompt.trim().is_empty() {
            return;
        }
        let model = self.model.read(cx).text(cx).trim().to_string();
        if model.is_empty() {
            return;
        }
        if self.current.title.is_empty() {
            self.current.title = prompt.chars().take(60).collect();
        }
        if let Some(context) = self.attached.take() {
            prompt.push_str(&format!("\n\nEditor context (untrusted data):\n{context}"));
        }
        self.input.update(cx, |input, cx| input.clear(window, cx));
        self.repair_interrupted_calls();
        self.current.messages.push(Message::text("user", prompt));
        self.running = true;
        self.show_history = false;
        self.status = Some(i18n::t!("agent.running").into());
        self.rebuild_messages(cx);
        self.persist(cx);
        self.scroll.scroll_to_bottom();
        let provider = self.provider;
        let client = self.client.clone();
        let workspace = self.workspace.clone();
        self.task = Some(cx.spawn_in(window, async move |panel, cx| {
            let result: Result<()> = async {
                let key = cx
                    .background_spawn(async move { credentials::read(provider) })
                    .await
                    .ok_or_else(|| anyhow!(i18n::t!("agent.missing_key")))?;
                // 최대 횟수 안에서 모델의 도구 호출과 결과 전송을 반복한다.
                for _ in 0..MAX_TOOL_ROUNDS {
                    let messages = panel.update(cx, |panel, cx| {
                        let messages = panel.current.messages.clone();
                        panel.current.messages.push(Message::text("assistant", String::new()));
                        panel.rebuild_messages(cx);
                        messages
                    })?;
                    let (sender, receiver) = smol::channel::bounded(32);
                    let request_client = client.clone();
                    let request_key = key.clone();
                    let request_model = model.clone();
                    let response = cx.background_spawn(async move {
                        provider::stream_turn(
                            request_client,
                            provider,
                            &request_key,
                            &request_model,
                            &messages,
                            &don_tools::definitions(),
                            sender,
                        )
                        .await
                    });
                    // UI 스레드는 도착한 텍스트만 그리며 HTTP 읽기는 백그라운드에서 수행한다.
                    while let Ok(mut text) = receiver.recv().await {
                        // 같은 시점에 도착한 조각은 합쳐 Markdown 재분석 횟수를 줄인다.
                        while let Ok(chunk) = receiver.try_recv() {
                            text.push_str(&chunk);
                        }
                        panel.update(cx, |panel, cx| panel.append_stream(&text, cx))?;
                    }
                    let turn = response.await?;
                    let calls = turn.calls.clone();
                    panel.update(cx, |panel, cx| {
                        if let Some(message) = panel.current.messages.last_mut() {
                            message.text = turn.text;
                            message.tool_calls = turn.calls;
                            message.provider_content = turn.provider_content;
                        }
                        panel.rebuild_messages(cx);
                    })?;
                    if calls.is_empty() {
                        return Ok(());
                    }
                    // 편집 차이 미리보기와 승인은 MCP와 같은 don_tools 실행기로 처리한다.
                    for call in calls {
                        panel.update(cx, |panel, cx| {
                            panel.status = Some(i18n::t!("agent.tool_result", name = call.name));
                            cx.notify();
                        })?;
                        let result = workspace
                            .update_in(cx, |workspace, window, cx| {
                                don_tools::execute(&call.name, call.arguments.clone(), workspace, window, cx)
                            })?
                            .await;
                        let result = match result {
                            Ok(value) => value,
                            Err(error) => json!({"error":error.to_string()}),
                        };
                        panel.update(cx, |panel, cx| {
                            panel.current.messages.push(Message {
                                role: "tool".into(),
                                text: result.to_string(),
                                tool_calls: vec![],
                                tool_call_id: Some(call.id),
                                provider_content: None,
                            });
                            panel.rebuild_messages(cx);
                        })?;
                    }
                }
                Err(anyhow!(i18n::t!("agent.tool_limit")))
            }
            .await;
            panel
                .update(cx, |panel, cx| {
                    panel.running = false;
                    panel.repair_interrupted_calls();
                    panel.status = result
                        .err()
                        .map(|error| i18n::t!("agent.error", error = localized_error(&error)));
                    panel.persist(cx);
                    panel.rebuild_messages(cx);
                })
                .ok();
        }));
    }

    /// 저장할 키를 백그라운드로 넘기고 입력란과 실행 취소 기록은 즉시 없앤다.
    fn save_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let key = self.api_key.read(cx).text(cx);
        self.api_key = Self::key_input(window, cx);
        let provider = self.provider;
        cx.spawn(async move |panel, cx| {
            let result = cx
                .background_spawn(async move { credentials::save(provider, &key) })
                .await;
            panel
                .update(cx, |panel, cx| {
                    panel.status = Some(match result {
                        Ok(()) => i18n::t!("agent.key_saved").into(),
                        Err(error) => i18n::t!("agent.error", error = error),
                    });
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }

    /// OS 자격 증명 저장소에서 선택한 제공자의 키만 삭제한다.
    fn delete_key(&mut self, cx: &mut Context<Self>) {
        let provider = self.provider;
        cx.spawn(async move |panel, cx| {
            let result = cx.background_spawn(async move { credentials::delete(provider) }).await;
            panel
                .update(cx, |panel, cx| {
                    panel.status = Some(match result {
                        Ok(()) => i18n::t!("agent.key_deleted").into(),
                        Err(error) => i18n::t!("agent.error", error = error),
                    });
                    cx.notify();
                })
                .ok();
        })
        .detach();
    }
}

impl Render for AgentPanel {
    /// 설정·스크롤 가능한 Markdown 대화·전송 입력란을 그린다.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.language != i18n::language() {
            self.language = i18n::language();
            // 언어를 바꾸면 기존 입력란의 안내 문구도 새 언어로 갱신한다.
            for (input, placeholder) in [
                (&self.input, i18n::t!("agent.prompt")),
                (&self.model, i18n::t!("agent.model")),
                (&self.api_key, i18n::t!("agent.api_key")),
            ] {
                input.update(cx, |input, cx| {
                    input
                        .editor
                        .update(cx, |editor, cx| editor.set_placeholder_text(placeholder, window, cx));
                });
            }
        }
        let colors = cx.theme().colors();
        let mut history = v_flex()
            .id("agent-history-list")
            .max_h(px(180.))
            .overflow_y_scroll()
            .gap_1();
        // 사용자가 기록을 펼친 경우에만 최근 대화를 선택할 수 있다.
        if self.show_history {
            if self.conversations.is_empty() {
                history = history.child(Label::new(i18n::t!("agent.no_history")));
            }
            // 최근 대화를 버튼 목록으로 만들며, 선택하면 해당 메시지를 복원한다.
            for (index, conversation) in self.conversations.iter().enumerate() {
                history = history.child(
                    Button::new(("agent-conversation", index), conversation.title.clone()).on_click(cx.listener(
                        move |panel, _, _, cx| {
                            if panel.running {
                                return;
                            }
                            panel.current = panel.conversations[index].clone();
                            panel.repair_interrupted_calls();
                            panel.show_history = false;
                            panel.rebuild_messages(cx);
                        },
                    )),
                );
            }
            history = history.child(
                Button::new("agent-clear-history", i18n::t!("agent.clear_history"))
                    .disabled(self.running)
                    .on_click(cx.listener(|panel, _, _, cx| {
                        panel.conversations.clear();
                        panel.current = Self::empty_conversation();
                        panel.rebuild_messages(cx);
                        panel.persist(cx);
                    })),
            );
        }
        let messages = self
            .current
            .messages
            .iter()
            .zip(&self.rendered)
            .enumerate()
            .map(|(index, (message, rendered))| {
                let role = if message.role == "user" {
                    i18n::t!("agent.you")
                } else {
                    i18n::t!("agent.assistant")
                };
                v_flex()
                    .id(("agent-message", index))
                    .gap_1()
                    .p_2()
                    .rounded_md()
                    .child(
                        Label::new(if message.role == "tool" {
                            i18n::t!(
                                "agent.tool_result",
                                name = message.tool_call_id.as_deref().unwrap_or_default()
                            )
                        } else {
                            role.into()
                        })
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                    .child(MarkdownElement::new(rendered.clone(), MarkdownStyle::hover(window, cx)))
            })
            .collect::<Vec<_>>();
        v_flex()
            .id("agent-panel")
            .key_context("AgentPanel")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::send))
            .on_action(cx.listener(Self::stop))
            .size_full()
            .bg(colors.panel_background)
            .gap_2()
            .p_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(Label::new(i18n::t!("agent.title")))
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                IconButton::new("agent-new", IconName::Plus)
                                    .disabled(self.running)
                                    .tooltip(Tooltip::text(i18n::t!("agent.new_chat")))
                                    .on_click(cx.listener(|panel, _, _, cx| {
                                        panel.new_chat(cx);
                                    })),
                            )
                            .child(
                                IconButton::new("agent-history", IconName::HistoryRerun)
                                    .tooltip(Tooltip::text(i18n::t!("agent.history")))
                                    .on_click(cx.listener(|panel, _, _, cx| {
                                        panel.show_history = !panel.show_history;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                IconButton::new("agent-hide", IconName::Close)
                                    .tooltip(Tooltip::text(i18n::t!("agent.hide")))
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        cx.emit(PanelEvent::Close);
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex().gap_2().child(Label::new(i18n::t!("agent.provider"))).child(
                    Button::new("agent-provider", self.provider.name())
                        .disabled(self.running)
                        .on_click(cx.listener(|panel, _, window, cx| {
                            let provider = if panel.provider == Provider::OpenAi {
                                Provider::Anthropic
                            } else {
                                Provider::OpenAi
                            };
                            panel.configure(
                                &Configure {
                                    provider: Some(provider),
                                    model: None,
                                },
                                window,
                                cx,
                            );
                        })),
                ),
            )
            .child(Label::new(i18n::t!("agent.model")).size(LabelSize::Small))
            .child(self.model.clone())
            .child(self.api_key.clone())
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("agent-save-key", i18n::t!("agent.save_key"))
                            .on_click(cx.listener(|panel, _, window, cx| panel.save_key(window, cx))),
                    )
                    .child(
                        Button::new("agent-delete-key", i18n::t!("agent.delete_key"))
                            .on_click(cx.listener(|panel, _, _, cx| panel.delete_key(cx))),
                    ),
            )
            .child(history)
            .child(
                v_flex()
                    .id("agent-messages")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .gap_3()
                    .children(messages),
            )
            .when_some(self.status.clone(), |this, status| {
                this.child(Label::new(status).size(LabelSize::Small))
            })
            .when(self.attached.is_some(), |this| {
                this.child(Label::new(i18n::t!("agent.context_label")).size(LabelSize::Small))
            })
            .child(self.input.clone())
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        Button::new("agent-attach", i18n::t!("agent.attach"))
                            .disabled(self.running)
                            .on_click(cx.listener(|panel, _, _, cx| panel.attach(cx))),
                    )
                    .child(if self.running {
                        Button::new("agent-stop", i18n::t!("agent.stop"))
                            .on_click(cx.listener(|panel, _, window, cx| panel.stop(&Stop, window, cx)))
                    } else {
                        Button::new("agent-send", i18n::t!("agent.send"))
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|panel, _, window, cx| panel.send(&Send, window, cx)))
                    }),
            )
    }
}

impl Focusable for AgentPanel {
    /// 패널을 열면 메시지 입력란으로 초점을 옮긴다.
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl EventEmitter<PanelEvent> for AgentPanel {}

impl Panel for AgentPanel {
    /// 저장용 식별 이름은 표시 언어가 바뀌어도 유지한다.
    fn persistent_name() -> &'static str {
        "Agent Panel"
    }
    /// 워크스페이스 안에서 쓰는 패널 식별자이다.
    fn panel_key() -> &'static str {
        "AgentPanel"
    }
    /// 기본 위치는 우측이며 사용자가 옮겼다면 저장된 쪽을 사용한다.
    fn position(&self, _: &Window, _: &App) -> DockPosition {
        self.position
    }
    /// 대화를 읽기 쉽도록 좌우 도크만 허용한다.
    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Left | DockPosition::Right)
    }
    /// 패널의 이동 위치를 저장하고 도크에 알린다.
    fn set_position(&mut self, position: DockPosition, window: &mut Window, cx: &mut Context<Self>) {
        if !self.position_is_valid(position) || self.position == position {
            return;
        }
        self.position = position;
        self.persist(cx);
        let panel = cx.entity();
        let workspace = self.workspace.clone();
        // 현재 패널의 변경이 끝난 뒤 도크를 옮겨 엔티티의 중복 대여를 피한다.
        window.defer(cx, move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    workspace.remove_panel(&panel, window, cx);
                    workspace.add_panel(panel, window, cx);
                    workspace.focus_panel::<AgentPanel>(window, cx);
                })
                .ok();
        });
        cx.notify();
    }
    /// 대화를 읽을 수 있는 기본 너비를 돌려준다.
    fn size(&self, _: &Window, _: &App) -> Pixels {
        self.width.unwrap_or(px(420.))
    }
    /// 사용자가 변경한 너비를 저장한다.
    fn set_size(&mut self, size: Option<Pixels>, _: &mut Window, cx: &mut Context<Self>) {
        self.width = size;
        // 크기를 드래그하는 동안 전체 대화 DB를 반복 저장하지 않고 마지막 너비만 보관한다.
        self.sizing_task = Some(cx.spawn(async move |panel, cx| {
            smol::Timer::after(std::time::Duration::from_millis(250)).await;
            panel.update(cx, |panel, cx| panel.persist(cx)).ok();
        }));
        cx.notify();
    }
    /// 숨긴 패널을 다시 열 수 있도록 아이콘은 항상 유지한다.
    fn icon(&self, _: &Window, _: &App) -> Option<IconName> {
        Some(IconName::Sparkle)
    }
    /// 도크 아이콘에 현재 표시 언어로 설명을 보여 준다.
    fn icon_tooltip(&self, _: &Window, _: &App) -> Option<&'static str> {
        Some(i18n::t!("agent.title"))
    }
    /// 메뉴와 도크에서 같은 토글 액션을 사용한다.
    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleFocus)
    }
    /// 처음 실행할 때 우측에 패널을 표시한다.
    fn starts_open(&self, _: &Window, _: &App) -> bool {
        true
    }
    /// 같은 도크에 여러 패널이 있을 때의 초기 선택 순위이다.
    fn activation_priority(&self) -> u32 {
        20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 대화 상태 직렬화에 API 자격 증명 항목이 생기지 않도록 검사한다.
    #[test]
    fn stored_state_contains_no_credentials() {
        let state = StoredState {
            provider: Provider::Anthropic,
            model: "custom-model".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(!json.contains("api_key"));
        let restored: StoredState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.provider, Provider::Anthropic);
        assert_eq!(restored.model, "custom-model");
    }
}
