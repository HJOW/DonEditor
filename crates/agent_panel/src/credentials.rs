//! [DonEditor] API 키를 OS 자격 증명 저장소에만 기록한다.
//! 설정 화면과 Agent 패널은 이 모듈의 전역 상태를 함께 사용해 키 등록 여부를 즉시 반영한다.

use crate::provider::Provider;
use anyhow::{Context as _, Result, ensure};
use gpui::{App, AppContext as _, Global, Task};
use std::collections::HashMap;

/// 다른 앱의 자격 증명과 구분하는 서비스 이름이다.
const SERVICE: &str = "DonEditor.Agent";

/// 제공자별 API 키를 어디에서 얻는지 나타낸다. 키 값 자체는 담지 않는다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeySource {
    /// 아직 OS 저장소를 조회하지 않았다.
    Unknown,
    /// 기본 설정(OS 자격 증명 저장소)에 저장된 키가 있다.
    Stored,
    /// 저장된 키는 없지만 환경 변수로 키가 지정되어 있다.
    Environment,
    /// 키가 없어 먼저 입력해야 한다.
    Missing,
}

impl KeySource {
    /// 요청을 보낼 수 있는 키가 있는지 돌려준다.
    pub fn is_available(self) -> bool {
        matches!(self, Self::Stored | Self::Environment)
    }
}

/// 앱 전체에서 공유하는 키 등록 상태이다. 키 값은 보관하지 않는다.
#[derive(Default)]
struct KeyStatus {
    sources: HashMap<Provider, KeySource>,
    /// 저장·삭제할 때마다 늘려 입력란을 새로 만들고 실행 취소 기록에 키가 남지 않게 한다.
    generation: usize,
}

impl Global for KeyStatus {}

/// 비어 있지 않은 문자열만 키로 인정한다.
fn non_empty(key: String) -> Option<String> {
    (!key.trim().is_empty()).then_some(key)
}

/// OS 저장소에 기록된 키만 읽는다.
fn read_stored(provider: Provider) -> Option<String> {
    keyring::Entry::new(SERVICE, provider.name())
        .and_then(|entry| entry.get_password())
        .ok()
        .and_then(non_empty)
}

/// 환경 변수로 지정된 키만 읽는다.
fn read_environment(provider: Provider) -> Option<String> {
    std::env::var(provider.environment_key()).ok().and_then(non_empty)
}

/// OS 보안 저장소에서 읽고, 항목이 없거나 저장소가 잠겼으면 환경 변수로 폴백한다.
pub fn read(provider: Provider) -> Option<String> {
    read_stored(provider).or_else(|| read_environment(provider))
}

/// 키 값을 노출하지 않고 출처만 확인한다. OS 호출이 있으므로 백그라운드에서 부른다.
fn source(provider: Provider) -> KeySource {
    if read_stored(provider).is_some() {
        KeySource::Stored
    } else if read_environment(provider).is_some() {
        KeySource::Environment
    } else {
        KeySource::Missing
    }
}

/// 빈 키는 거절하고 제공자별 항목으로 OS 보안 저장소에 기록한다.
fn save_blocking(provider: Provider, key: &str) -> Result<()> {
    ensure!(!key.trim().is_empty(), i18n::t!("agent.error.empty_key"));
    keyring::Entry::new(SERVICE, provider.name())
        .and_then(|entry| entry.set_password(key.trim()))
        .context(i18n::t!("agent.error.credential_save"))
}

/// 저장된 키를 지운다. 환경 변수에 지정된 키는 변경하지 않는다.
fn delete_blocking(provider: Provider) -> Result<()> {
    match keyring::Entry::new(SERVICE, provider.name()).and_then(|entry| entry.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error).context(i18n::t!("agent.error.credential_delete")),
    }
}

/// 상태를 바꾸고 모든 창(설정 화면·Agent 패널)을 다시 그린다.
fn set_source(provider: Provider, source: KeySource, bump_generation: bool, cx: &mut App) {
    let status = cx.default_global::<KeyStatus>();
    status.sources.insert(provider, source);
    if bump_generation {
        status.generation += 1;
    }
    cx.refresh_windows();
}

/// 앱 시작 시 모든 제공자의 키 출처를 백그라운드에서 한 번 조회한다.
pub fn init(cx: &mut App) {
    cx.default_global::<KeyStatus>();
    // 제공자마다 따로 조회해 한 저장소의 지연이 다른 제공자 표시를 막지 않게 한다.
    for provider in Provider::ALL {
        refresh(provider, cx);
    }
}

/// 한 제공자의 키 출처를 다시 조회한다(예: 패널에서 제공자를 바꿀 때).
pub fn refresh(provider: Provider, cx: &mut App) {
    cx.spawn(async move |cx| {
        let source = cx.background_spawn(async move { source(provider) }).await;
        cx.update(|cx| set_source(provider, source, false, cx)).ok();
    })
    .detach();
}

/// 현재 알려진 키 출처를 돌려준다. 조회 전이면 `Unknown`이다.
pub fn key_source(provider: Provider, cx: &App) -> KeySource {
    cx.try_global::<KeyStatus>()
        .and_then(|status| status.sources.get(&provider).copied())
        .unwrap_or(KeySource::Unknown)
}

/// 입력란을 새로 만들 때 쓰는 세대 번호이다. 저장·삭제 후 값이 바뀐다.
pub fn generation(cx: &App) -> usize {
    cx.try_global::<KeyStatus>().map_or(0, |status| status.generation)
}

/// 키를 OS 저장소에 저장하고 성공하면 즉시 모든 화면에 반영한다.
pub fn save(provider: Provider, key: String, cx: &mut App) -> Task<Result<()>> {
    cx.spawn(async move |cx| {
        cx.background_spawn(async move { save_blocking(provider, &key) })
            .await?;
        cx.update(|cx| set_source(provider, KeySource::Stored, true, cx))?;
        Ok(())
    })
}

/// 저장된 키를 지우고, 환경 변수 키가 남아 있는지 다시 확인해 반영한다.
pub fn delete(provider: Provider, cx: &mut App) -> Task<Result<()>> {
    cx.spawn(async move |cx| {
        let source = cx
            .background_spawn(async move {
                delete_blocking(provider)?;
                anyhow::Ok(source(provider))
            })
            .await?;
        cx.update(|cx| set_source(provider, source, true, cx))?;
        Ok(())
    })
}
