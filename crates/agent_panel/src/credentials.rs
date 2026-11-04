//! [DonEditor] API 키를 OS 자격 증명 저장소에만 기록한다.

use crate::provider::Provider;
use anyhow::{Context as _, Result, ensure};

/// 다른 앱의 자격 증명과 구분하는 서비스 이름이다.
const SERVICE: &str = "DonEditor.Agent";

/// OS 보안 저장소에서 읽고, 항목이 없거나 저장소가 잠겼으면 환경 변수로 폴백한다.
pub fn read(provider: Provider) -> Option<String> {
    let stored = keyring::Entry::new(SERVICE, provider.name()).and_then(|entry| entry.get_password());
    if let Ok(key) = stored {
        if !key.trim().is_empty() {
            return Some(key);
        }
    }
    if let Ok(key) = std::env::var(provider.environment_key()) {
        if !key.trim().is_empty() {
            return Some(key);
        }
    }
    None
}

/// 빈 키는 거절하고 제공자별 항목으로 OS 보안 저장소에 기록한다.
pub fn save(provider: Provider, key: &str) -> Result<()> {
    ensure!(!key.trim().is_empty(), i18n::t!("agent.error.empty_key"));
    keyring::Entry::new(SERVICE, provider.name())
        .and_then(|entry| entry.set_password(key.trim()))
        .context(i18n::t!("agent.error.credential_save"))
}

/// 저장된 키를 지운다. 환경 변수에 지정된 키는 변경하지 않는다.
pub fn delete(provider: Provider) -> Result<()> {
    match keyring::Entry::new(SERVICE, provider.name()).and_then(|entry| entry.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error).context(i18n::t!("agent.error.credential_delete")),
    }
}
