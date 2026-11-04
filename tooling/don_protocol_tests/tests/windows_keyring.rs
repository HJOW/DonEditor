//! [DonEditor] 실제 API 키 없이 Windows 자격 증명 백엔드를 검증한다.
#![cfg(target_os = "windows")]

/// 테스트 종료 시 이 테스트가 만든 임시 자격 증명만 지운다.
struct TemporaryCredential {
    entry: keyring::Entry,
    created: bool,
}

impl Drop for TemporaryCredential {
    /// 패닉이 나더라도 임시 자격 증명을 남기지 않는다.
    fn drop(&mut self) {
        if self.created {
            let _ = self.entry.delete_credential();
        }
    }
}

/// 사용자의 제공자 키와 다른 고유한 서비스와 사용자 이름을 사용한다.
#[test]
#[ignore = "Windows 자격 증명 저장소에 임시 테스트 항목을 만들므로 명시적으로 실행한다"]
fn windows_credential_store_round_trip() {
    let unique = uuid::Uuid::new_v4().to_string();
    let entry = keyring::Entry::new("DonEditor.Agent.Test", &unique).unwrap();
    assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
    let mut temporary = TemporaryCredential { entry, created: false };
    temporary.entry.set_password("doneditor-test-not-an-api-key").unwrap();
    temporary.created = true;
    assert_eq!(temporary.entry.get_password().unwrap(), "doneditor-test-not-an-api-key");
    temporary.entry.delete_credential().unwrap();
    temporary.created = false;
    assert!(matches!(temporary.entry.get_password(), Err(keyring::Error::NoEntry)));
}
