// This crate was essentially pulled out verbatim from main `gram` crate to avoid having to run RustEmbed macro whenever gram has to be rebuilt. It saves a second or two on an incremental build.

use anyhow::Context as _;
use gpui::{App, AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "fonts/**/*"]
#[include = "icons/**/*"]
#[include = "images/**/*"]
#[include = "themes/**/*"]
#[exclude = "themes/src/*"]
#[include = "sounds/**/*"]
#[include = "prompts/**/*"]
#[include = "*.toml"]
#[include = "*.md"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Self::get(path)
            .map(|f| Some(f.data))
            .with_context(|| format!("loading asset at path {path:?}"))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| if p.starts_with(path) { Some(p.into()) } else { None })
            .collect())
    }
}

// [DonEditor] 내장 글꼴로 읽어 들일 파일 확장자 목록.
// 원본은 `.ttf` 만 읽었으나, 기본 글꼴 D2Coding 이 TrueType 컬렉션(`.ttc`, 여러 글꼴이 한 파일)이라 함께 읽는다.
const EMBEDDED_FONT_EXTENSIONS: [&str; 2] = [".ttf", ".ttc"];

impl Assets {
    /// Populate the [`TextSystem`] of the given [`AppContext`] with all `.ttf` and `.ttc` fonts in the `fonts` directory.
    // [DonEditor] 글꼴 폴더의 .ttf, .ttc 파일을 모두 텍스트 시스템에 등록한다.
    pub fn load_fonts(&self, cx: &App) -> anyhow::Result<()> {
        let font_paths = self.list("fonts")?;
        let mut embedded_fonts = Vec::new();
        // 글꼴 파일 경로를 순회하며 지원하는 확장자만 골라 읽는다
        for font_path in font_paths {
            if EMBEDDED_FONT_EXTENSIONS.iter().any(|ext| font_path.ends_with(ext)) {
                let font_bytes = cx
                    .asset_source()
                    .load(&font_path)?
                    .expect("Assets should never return None");
                embedded_fonts.push(font_bytes);
            }
        }

        cx.text_system().add_fonts(embedded_fonts)
    }

    pub fn load_test_fonts(&self, cx: &App) {
        cx.text_system()
            .add_fonts(vec![self.load("fonts/myna/Myna-Regular.ttf").unwrap().unwrap()])
            .unwrap()
    }
}

// [DonEditor] 앱 내장 도움말(gram://docs/...)이 읽는 문서 폴더.
// 원본 GramEditor 문서를 루트 `docs/` 에서 `GramEditor-docs/docs/` 로 옮겼으므로 경로만 변경했다 (내용은 원본 그대로).
#[derive(RustEmbed)]
#[folder = "../../GramEditor-docs/docs"]
#[include = "*.md"]
#[exclude = "*.DS_Store"]
pub struct Docs;

// [DonEditor] 자체 한국어 사용 안내를 원본 문서와 별도 폴더에서 내장한다.
// 외부 경로에는 don/ 접두사를 붙여 기존 gram://docs/... 링크와 구분한다.
#[derive(RustEmbed)]
#[folder = "../../docs"]
#[include = "*.md"]
#[exclude = "*.DS_Store"]
pub struct DonDocs;

impl AssetSource for Docs {
    // [DonEditor] 원본과 자체 문서를 같은 도움말 조회 규약으로 읽는다.
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        lookup_docs(path)
            .map(|f| Some(f.data))
            .with_context(|| format!("loading docs at path {path:?}"))
    }

    // [DonEditor] 도움말 검색에 두 문서 모음을 모두 제공하되 경로 충돌을 피한다.
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let original = Self::iter().map(|entry| entry.to_string());
        let don = DonDocs::iter().map(|entry| format!("don/{entry}"));
        Ok(original
            .chain(don)
            .filter(|entry| entry.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}

// [DonEditor] don/ 문서는 자체 문서에서, 나머지는 보존한 원본에서 찾는다.
pub fn lookup_docs(path: &str) -> Option<rust_embed::EmbeddedFile> {
    if let Some(path) = path.strip_prefix("don/") {
        DonDocs::get(path).or_else(|| DonDocs::get(&format!("{path}.md")))
    } else {
        Docs::get(path).or_else(|| Docs::get(&format!("{path}.md")))
    }
}
