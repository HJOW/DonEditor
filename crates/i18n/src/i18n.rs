//! [DonEditor] 다국어(i18n) 지원 (요구사항 R1: 한국어 UI).
//!
//! - 문자열 리소스는 `assets/locales/en.json`, `assets/locales/ko.json` 에 키-문자열 형태로 있고 바이너리에 내장된다.
//! - `t!("menu.file.open")` 처럼 키로 문자열을 얻는다. 현재 언어에 키가 없으면 영어, 영어에도 없으면 키 이름을 돌려준다.
//! - 자리 표시자는 `t!("dialog.save_changes", name = file_name)` 처럼 `{name}` 에 값을 채운다.
//! - 표시 언어는 설정 `ui_language` (`"auto"`, `"ko"`, `"en"`)로 정한다. `auto` 는 OS 언어를 따른다.
//!
//! 번역된 문자열은 프로세스가 끝날 때까지 살아 있는 `&'static str` 이므로
//! `Panel::icon_tooltip` 처럼 `&'static str` 을 요구하는 곳에도 그대로 쓸 수 있다.

use gpui::App;
use settings::{RegisterSetting, Settings, SettingsContent, SettingsStore, UiLanguageContent};
use std::{
    cell::Cell,
    collections::HashMap,
    sync::{
        OnceLock, RwLock,
        atomic::{AtomicU8, Ordering},
    },
};

/// 영어 문자열 리소스 (원본 UI 문자열과 같은 내용)
const ENGLISH_JSON: &str = include_str!("../../../assets/locales/en.json");
/// 한국어 문자열 리소스
const KOREAN_JSON: &str = include_str!("../../../assets/locales/ko.json");

/// 지원하는 표시 언어
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Korean,
}

impl Language {
    /// 지원하는 모든 언어
    pub const ALL: [Language; 2] = [Language::English, Language::Korean];

    /// 설정 파일과 로케일 이름에 쓰는 언어 코드
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Korean => "ko",
        }
    }

    /// `ko`, `ko-KR`, `ko_KR.UTF-8` 같은 로케일 이름에서 언어를 찾는다. 지원하지 않는 언어는 `None`.
    pub fn from_locale(locale: &str) -> Option<Language> {
        // 언어 부분만 비교한다 (지역·인코딩 표기는 무시)
        let language_part = locale
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        // 지원하는 언어 중 코드가 같은 것을 찾는다
        Language::ALL
            .into_iter()
            .find(|language| language.code() == language_part)
    }

    /// OS 의 표시 언어를 따라 정한다. 알 수 없거나 지원하지 않는 언어이면 영어.
    pub fn detect_system() -> Language {
        sys_locale::get_locale()
            .as_deref()
            .and_then(Language::from_locale)
            .unwrap_or(Language::English)
    }
}

/// 키 → 문자열 표 (값은 프로세스 수명 동안 유지된다)
type Table = HashMap<&'static str, &'static str>;

/// 언어별 문자열 표
struct Catalog {
    english: Table,
    korean: Table,
}

/// 내장된 JSON 리소스를 읽어 표로 만든다. 문자열은 한 번만 만들어 프로세스가 끝날 때까지 유지한다.
fn parse_table(name: &str, json: &'static str) -> Table {
    let entries: HashMap<String, String> =
        serde_json::from_str(json).unwrap_or_else(|error| panic!("로케일 리소스 {name} 을 읽지 못했습니다: {error}"));
    entries
        .into_iter()
        .map(|(key, value)| {
            // 표가 프로세스 수명 동안 쓰이므로 누출시켜 'static 으로 만든다 (언어당 한 번만 실행됨)
            let key: &'static str = Box::leak(key.into_boxed_str());
            let value: &'static str = Box::leak(value.into_boxed_str());
            (key, value)
        })
        .collect()
}

/// 문자열 표를 처음 쓸 때 한 번만 읽는다
fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| Catalog {
        english: parse_table("en", ENGLISH_JSON),
        korean: parse_table("ko", KOREAN_JSON),
    })
}

/// 지정한 언어의 문자열 표 (검증 테스트용)
pub fn table(language: Language) -> &'static HashMap<&'static str, &'static str> {
    match language {
        Language::English => &catalog().english,
        Language::Korean => &catalog().korean,
    }
}

/// 현재 표시 언어 (0 = 영어, 1 = 한국어). `init` 을 부르기 전과 테스트에서는 영어이다.
static ACTIVE_LANGUAGE: AtomicU8 = AtomicU8::new(0);

thread_local! {
    /// 이 스레드에서만 쓰는 임시 표시 언어. `with_language` 가 설정한다.
    static LANGUAGE_OVERRIDE: Cell<Option<Language>> = const { Cell::new(None) };
}

/// 현재 표시 언어. `with_language` 안이면 그 스레드의 임시 언어를 먼저 따른다.
pub fn language() -> Language {
    if let Some(language) = LANGUAGE_OVERRIDE.with(Cell::get) {
        return language;
    }
    match ACTIVE_LANGUAGE.load(Ordering::Relaxed) {
        1 => Language::Korean,
        _ => Language::English,
    }
}

/// 지정한 언어로 `f` 를 실행한다. **현재 스레드에서만** 적용되고 끝나면 원래대로 돌아오므로,
/// 같은 프로세스에서 병렬로 도는 다른 테스트의 표시 언어에 영향을 주지 않는다. (테스트와 도구용)
pub fn with_language<R>(language: Language, f: impl FnOnce() -> R) -> R {
    /// 패닉이 나도 이전 값을 되돌리는 정리 도구
    struct Restore(Option<Language>);
    impl Drop for Restore {
        fn drop(&mut self) {
            LANGUAGE_OVERRIDE.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(LANGUAGE_OVERRIDE.with(|slot| slot.replace(Some(language))));
    f()
}

/// 표시 언어를 바꾼다. 이미 그려진 화면은 다시 그려져야 바뀐다 (`init` 이 설정 변경 시 창을 새로 그린다).
pub fn set_language(language: Language) {
    let value = match language {
        Language::English => 0,
        Language::Korean => 1,
    };
    ACTIVE_LANGUAGE.store(value, Ordering::Relaxed);
}

/// 키에 해당하는 현재 언어의 문자열을 돌려준다.
/// 현재 언어에 없으면 영어, 영어에도 없으면 키 이름 그대로 돌려준다.
pub fn t(key: &'static str) -> &'static str {
    let catalog = catalog();
    let current = match language() {
        Language::English => &catalog.english,
        Language::Korean => &catalog.korean,
    };
    current
        .get(key)
        .or_else(|| catalog.english.get(key))
        .copied()
        .unwrap_or(key)
}

/// `t` 와 같지만 문자열 안의 `{이름}` 자리 표시자를 `args` 의 값으로 바꾼다.
/// 값이 주어지지 않은 자리 표시자는 그대로 둔다.
pub fn t_with(key: &'static str, args: &[(&str, String)]) -> String {
    let template = t(key);
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    // 문자열을 한 번만 훑으며 `{이름}` 을 값으로 바꾼다.
    // 바꿔 넣은 값은 다시 훑지 않으므로, 값에 `{다른이름}` 같은 글자가 있어도(파일 이름 등) 치환되지 않는다.
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('}') {
            Some(end) => {
                let name = &after[..end];
                match args.iter().find(|(arg_name, _)| *arg_name == name) {
                    Some((_, value)) => result.push_str(value),
                    // 값이 주어지지 않은 자리 표시자는 그대로 둔다
                    None => {
                        result.push('{');
                        result.push_str(name);
                        result.push('}');
                    }
                }
                rest = &after[end + 1..];
            }
            // 닫는 중괄호가 없으면 나머지를 그대로 붙인다
            None => {
                result.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    result.push_str(rest);
    result
}

/// 액션 이름(예: `workspace::Open`) → 현재 언어의 표시 이름. 메뉴를 만들 때 등록한다.
fn action_labels() -> &'static RwLock<HashMap<String, String>> {
    static LABELS: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();
    LABELS.get_or_init(Default::default)
}

/// 액션에 현재 언어의 표시 이름을 연결한다. 같은 액션을 다시 등록하면 마지막 값이 쓰인다.
pub fn set_action_label(action_name: &str, label: &str) {
    if let Ok(mut labels) = action_labels().write() {
        labels.insert(action_name.to_string(), label.to_string());
    }
}

/// 명령 팔레트에 보여 줄 이름을 돌려준다.
/// 한국어이고 액션에 표시 이름이 등록되어 있으면 `"파일 열기 (workspace: open)"` 처럼 한국어와 영어 이름을 함께 돌려주어
/// 두 언어 모두로 검색할 수 있게 한다. 그 밖에는 영어 이름 그대로이다.
pub fn action_display_name(action_name: &str, english_name: &str) -> String {
    if language() != Language::Korean {
        return english_name.to_string();
    }
    let label = action_labels()
        .read()
        .ok()
        .and_then(|labels| labels.get(action_name).cloned());
    match label {
        Some(label) if !label.is_empty() => format!("{label} ({english_name})"),
        _ => english_name.to_string(),
    }
}

/// 번역 문자열을 얻는 매크로.
///
/// ```ignore
/// let label = i18n::t!("menu.file.open");
/// let message = i18n::t!("dialog.save_changes", name = file_name);
/// ```
#[macro_export]
macro_rules! t {
    ($key:literal) => {
        $crate::t($key)
    };
    ($key:literal, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::t_with($key, &[$((stringify!($name), ($value).to_string())),+])
    };
}

/// 설정 `ui_language` 를 읽어 표시 언어를 정하는 설정 항목
#[derive(RegisterSetting)]
pub struct UiLanguageSetting(pub UiLanguageContent);

impl Settings for UiLanguageSetting {
    fn from_settings(content: &SettingsContent) -> Self {
        Self(content.ui_language.unwrap_or_default())
    }
}

impl UiLanguageSetting {
    /// 설정값(`auto`/`ko`/`en`)을 실제 표시 언어로 바꾼다
    pub fn resolve(&self) -> Language {
        match self.0 {
            UiLanguageContent::Auto => Language::detect_system(),
            UiLanguageContent::Ko => Language::Korean,
            UiLanguageContent::En => Language::English,
        }
    }
}

/// 설정에 따라 표시 언어를 적용한다. 바뀌었으면 열려 있는 창을 다시 그리게 한다.
fn apply_setting(cx: &mut App) {
    let resolved = UiLanguageSetting::get_global(cx).resolve();
    if resolved != language() {
        set_language(resolved);
        cx.refresh_windows();
    }
}

/// 설정을 읽어 표시 언어를 정하고, 이후 설정이 바뀔 때마다 다시 적용한다.
/// 메뉴처럼 시작할 때 한 번 만드는 UI 보다 먼저 호출해야 한다.
pub fn init(cx: &mut App) {
    apply_setting(cx);
    cx.observe_global::<SettingsStore>(apply_setting).detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::BTreeSet,
        path::{Path, PathBuf},
    };

    /// 전역 표시 언어를 바꾸는 테스트끼리 서로 간섭하지 않도록 직렬화하는 잠금
    static LANGUAGE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 문자열 안의 `{이름}` 자리 표시자 이름 목록
    fn placeholders(text: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        let mut rest = text;
        // 여는 중괄호를 찾을 때마다 닫는 중괄호까지를 이름으로 모은다
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            match after.find('}') {
                Some(end) => {
                    names.insert(after[..end].to_string());
                    rest = &after[end + 1..];
                }
                None => break,
            }
        }
        names
    }

    #[test]
    fn locale_names_are_mapped_to_languages() {
        assert_eq!(Language::from_locale("ko"), Some(Language::Korean));
        assert_eq!(Language::from_locale("ko-KR"), Some(Language::Korean));
        assert_eq!(Language::from_locale("ko_KR.UTF-8"), Some(Language::Korean));
        assert_eq!(Language::from_locale("KO"), Some(Language::Korean));
        assert_eq!(Language::from_locale("en-US"), Some(Language::English));
        assert_eq!(Language::from_locale("en"), Some(Language::English));
        // 지원하지 않는 언어와 비슷한 이름은 매칭되지 않는다
        assert_eq!(Language::from_locale("ja-JP"), None);
        assert_eq!(Language::from_locale("kok-IN"), None);
        assert_eq!(Language::from_locale(""), None);
    }

    #[test]
    fn english_and_korean_have_the_same_keys() {
        let english: BTreeSet<_> = table(Language::English).keys().copied().collect();
        let korean: BTreeSet<_> = table(Language::Korean).keys().copied().collect();
        let only_english: Vec<_> = english.difference(&korean).collect();
        let only_korean: Vec<_> = korean.difference(&english).collect();
        assert!(
            only_english.is_empty() && only_korean.is_empty(),
            "en.json 에만 있는 키: {only_english:?}\nko.json 에만 있는 키: {only_korean:?}"
        );
    }

    #[test]
    fn placeholders_match_between_languages() {
        let mut mismatched = Vec::new();
        // 모든 키에서 영어와 한국어의 자리 표시자 이름이 같아야 한다
        for (key, english) in table(Language::English) {
            let Some(korean) = table(Language::Korean).get(key) else {
                continue;
            };
            if placeholders(english) != placeholders(korean) {
                mismatched.push(*key);
            }
        }
        assert!(mismatched.is_empty(), "자리 표시자가 다른 키: {mismatched:?}");
    }

    #[test]
    fn no_value_is_empty() {
        for language in Language::ALL {
            for (key, value) in table(language) {
                assert!(!value.trim().is_empty(), "{} 의 {key} 값이 비어 있음", language.code());
            }
        }
    }

    #[test]
    fn missing_key_falls_back_to_the_key_name() {
        assert_eq!(t("this.key.does.not.exist"), "this.key.does.not.exist");
    }

    #[test]
    fn placeholders_are_filled() {
        // 키가 없으면 키 이름이 본문이 되고, 그 본문에도 치환이 적용된다
        let text = t_with("hello {name}, {name}! {other}", &[("name", "Don".to_string())]);
        assert_eq!(text, "hello Don, Don! {other}");

        // 실제 리소스에서 자리 표시자를 쓰는 키 하나를 골라, 모든 자리 표시자를 채우면 중괄호가 남지 않는다
        let (key, english) = table(Language::English)
            .iter()
            .find(|(_, value)| !placeholders(value).is_empty())
            .expect("자리 표시자를 쓰는 키가 하나는 있어야 함");
        let args: Vec<(String, String)> = placeholders(english)
            .into_iter()
            .map(|name| (name, "값".to_string()))
            .collect();
        let args: Vec<(&str, String)> = args
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
            .collect();
        let filled = t_with(key, &args);
        assert!(filled.contains("값") && !filled.contains('{'), "{key}: {filled}");
    }

    #[test]
    fn substituted_values_are_not_substituted_again() {
        // 값에 다른 자리 표시자 모양의 글자가 있어도 다시 치환되지 않아야 한다 (예: 파일 이름이 "{b}")
        let text = t_with("{a} and {b}", &[("a", "{b}".to_string()), ("b", "B".to_string())]);
        assert_eq!(text, "{b} and B");
        // 닫히지 않은 중괄호와 값이 없는 자리 표시자는 그대로 둔다
        assert_eq!(t_with("x {open", &[]), "x {open");
        assert_eq!(t_with("{missing}", &[]), "{missing}");
    }

    #[test]
    fn palette_names_show_both_languages_only_in_korean() {
        let _guard = LANGUAGE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        set_action_label("test_ns::OpenThing", "물건 열기");
        let original = language();
        set_language(Language::English);
        let english = action_display_name("test_ns::OpenThing", "test ns: open thing");
        set_language(Language::Korean);
        let korean = action_display_name("test_ns::OpenThing", "test ns: open thing");
        // 표시 이름이 등록되지 않은 액션은 한국어에서도 영어 이름 그대로이다
        let unregistered = action_display_name("test_ns::Unknown", "test ns: unknown");
        set_language(original);
        assert_eq!(english, "test ns: open thing");
        assert_eq!(korean, "물건 열기 (test ns: open thing)");
        assert_eq!(unregistered, "test ns: unknown");
    }

    #[test]
    fn with_language_only_changes_the_current_thread_and_restores() {
        // 다른 스레드가 읽는 전역 언어가 이 테스트 동안 바뀌지 않도록 잠근다
        let _guard = LANGUAGE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = "menu.file.save";
        let korean = table(Language::Korean)[key];
        let english = table(Language::English)[key];
        // 다른 스레드는 영향을 받지 않는다
        let other_thread = with_language(Language::Korean, || {
            assert_eq!(language(), Language::Korean);
            assert_eq!(t(key), korean);
            std::thread::spawn(|| t("menu.file.save")).join().unwrap()
        });
        assert_eq!(
            other_thread,
            table(Language::English)[key],
            "다른 스레드는 전역 언어(영어)를 따라야 함"
        );
        // 중첩 호출은 바깥 값으로 돌아온다
        with_language(Language::Korean, || {
            with_language(Language::English, || assert_eq!(t(key), english));
            assert_eq!(t(key), korean);
        });
    }

    #[test]
    fn language_switch_changes_the_returned_text() {
        let _guard = LANGUAGE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        // 전역 상태를 바꾸므로 이 테스트 안에서만 바꾸고 원래대로 돌려놓는다
        let key = "menu.file.save";
        assert_ne!(table(Language::English)[key], table(Language::Korean)[key]);
        let original = language();
        set_language(Language::Korean);
        let korean = t(key);
        set_language(Language::English);
        let english = t(key);
        set_language(original);
        assert_eq!(korean, table(Language::Korean)[key]);
        assert_eq!(english, table(Language::English)[key]);
    }

    /// crates 폴더 아래의 모든 .rs 파일 경로 (i18n crate 자신은 제외)
    #[allow(clippy::disallowed_methods)]
    fn source_files() -> Vec<PathBuf> {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            // 폴더를 재귀적으로 훑으며 .rs 파일을 모은다
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().is_some_and(|name| name != "target" && name != "i18n") {
                        walk(&path, out);
                    }
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    out.push(path);
                }
            }
        }
        let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = Vec::new();
        walk(&crates_dir, &mut files);
        files
    }

    /// 소스에서 `t!("키")`, `i18n::t("키")`, `t_with("키"` 로 쓰인 키를 모은다.
    /// `format!("`, `assert!("` 처럼 이름이 `t!("` 로 끝나는 다른 매크로는 앞 글자를 보고 제외한다.
    fn referenced_keys(source: &str) -> BTreeSet<String> {
        let mut keys = BTreeSet::new();
        // 호출 형태마다 첫 번째 문자열 리터럴을 키로 본다
        for pattern in ["t!(\"", "t(\"", "t_with(\""] {
            // 패턴이 나타나는 모든 위치를 훑는다
            for (start, _) in source.match_indices(pattern) {
                // 앞 글자가 식별자 문자이면 다른 이름의 일부이므로 건너뛴다 (예: format!(" 의 끝 t!(")
                let previous = source[..start].chars().next_back();
                if previous.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let after = &source[start + pattern.len()..];
                if let Some(end) = after.find('"') {
                    keys.insert(after[..end].to_string());
                }
            }
        }
        keys
    }

    /// 소스가 쓰는 키는 모두 en.json 에 있어야 하고, en.json 의 키는 모두 어딘가에서 쓰여야 한다.
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn keys_used_in_source_exist_and_catalog_has_no_dead_keys() {
        let mut all_source = String::new();
        // 모든 소스 파일을 한 문자열로 합친다 (키 사용 여부는 따옴표 친 키 문자열이 있는지로 판단)
        for path in source_files() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                all_source.push_str(&text);
                all_source.push('\n');
            }
        }
        let english = table(Language::English);

        let missing: Vec<_> = referenced_keys(&all_source)
            .into_iter()
            // `t!("...")` 형태가 아닌 일반 문자열(예: 매크로 설명 주석의 예시)은 제외한다
            .filter(|key| key.contains('.') && !key.contains(' ') && !english.contains_key(key.as_str()))
            .collect();
        assert!(missing.is_empty(), "소스가 쓰지만 en.json 에 없는 키: {missing:?}");

        let mut dead: Vec<_> = english
            .keys()
            .filter(|key| !all_source.contains(&format!("\"{key}\"")))
            .collect();
        dead.sort();
        assert!(dead.is_empty(), "en.json 에 있지만 소스에서 쓰이지 않는 키: {dead:?}");
    }
}
