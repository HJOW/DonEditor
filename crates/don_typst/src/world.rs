//! [DonEditor] Typst 컴파일러가 문서·글꼴·파일을 찾는 환경(`typst::World`). GPL-3.0-or-later.
//!
//! - 원본 문서는 편집기 버퍼의 글자(저장하지 않은 내용 포함)를 그대로 쓴다.
//! - `#image`·`#include` 같은 파일 참조는 문서가 있는 폴더 기준으로 읽는다(폴더 밖으로 나가는 경로는 막는다).
//! - 글꼴은 Typst 에 내장된 글꼴 + 호출하는 쪽이 넘긴 글꼴(D2Coding) + 시스템에 있는 한글 글꼴을 쓴다.
//! - 패키지(`@preview/...`) 내려받기는 지원하지 않는다. 네트워크 접근 없이 로컬 파일만 읽는다.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

/// 시스템에서 찾아 볼 한글 글꼴 파일 후보 (있는 것만 쓴다). 본문 한글이 고정폭 글꼴(D2Coding)로만 나오지 않게 한다.
fn system_korean_font_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    // Windows: 맑은 고딕
    if let Some(windir) = std::env::var_os("WINDIR") {
        let fonts = Path::new(&windir).join("Fonts");
        for name in ["malgun.ttf", "malgunbd.ttf"] {
            candidates.push(fonts.join(name));
        }
    }
    // macOS: Apple SD 고딕
    candidates.push(PathBuf::from("/System/Library/Fonts/AppleSDGothicNeo.ttc"));
    candidates.push(PathBuf::from("/System/Library/Fonts/Supplemental/AppleGothic.ttf"));
    // Linux: Noto Sans CJK, 나눔 글꼴이 흔한 위치
    for directory in [
        "/usr/share/fonts/opentype/noto",
        "/usr/share/fonts/noto-cjk",
        "/usr/share/fonts/truetype/nanum",
    ] {
        for name in [
            "NotoSansCJK-Regular.ttc",
            "NotoSansCJK-Bold.ttc",
            "NanumGothic.ttf",
            "NanumGothicBold.ttf",
        ] {
            candidates.push(Path::new(directory).join(name));
        }
    }
    candidates
}

/// 글꼴 모음. 글꼴 파일 해석은 비용이 크므로 프로세스에서 한 번만 한다.
#[derive(Clone)]
pub struct FontSet {
    book: Arc<LazyHash<FontBook>>,
    fonts: Arc<Vec<Font>>,
}

impl FontSet {
    /// 내장 글꼴 + `extra_font_data`(D2Coding 등) + 시스템 한글 글꼴로 글꼴 모음을 만든다.
    pub fn new(extra_font_data: &[Vec<u8>]) -> Self {
        let mut fonts: Vec<Font> = Vec::new();
        // Typst 에 내장된 글꼴 (본문 Libertinus Serif, 수식 New Computer Modern, 코드 DejaVu Sans Mono)
        for data in typst_assets::fonts() {
            fonts.extend(Font::iter(Bytes::new(data)));
        }
        // 호출하는 쪽이 준 글꼴. ttc 처럼 한 파일에 여러 글꼴이 있으면 모두 넣는다
        for data in extra_font_data {
            fonts.extend(Font::iter(Bytes::new(data.clone())));
        }
        // 시스템 한글 글꼴 (있을 때만)
        for path in system_korean_font_candidates() {
            if let Ok(data) = std::fs::read(&path) {
                fonts.extend(Font::iter(Bytes::new(data)));
            }
        }
        let book = FontBook::from_fonts(fonts.iter());
        Self {
            book: Arc::new(LazyHash::new(book)),
            fonts: Arc::new(fonts),
        }
    }
}

/// Typst 컴파일 환경
pub struct DonWorld {
    library: LazyHash<Library>,
    fonts: FontSet,
    main: FileId,
    source: Source,
    /// 문서가 있는 폴더. 파일 참조의 기준이며 `None` 이면(저장하지 않은 문서) 파일을 읽지 못한다.
    root: Option<PathBuf>,
}

/// 문서 이름. 진단 메시지에 나타난다.
const MAIN_FILE_NAME: &str = "main.typ";

impl DonWorld {
    pub fn new(source_text: String, root: Option<PathBuf>, fonts: FontSet) -> Self {
        let vpath = VirtualPath::new(MAIN_FILE_NAME).expect("고정된 파일 이름은 올바른 경로이다");
        let main = RootedPath::new(VirtualRoot::Project, vpath).intern();
        Self {
            library: LazyHash::new(Library::default()),
            fonts,
            main,
            source: Source::new(main, source_text),
            root,
        }
    }

    /// 파일 참조를 실제 경로로 바꾼다. 문서 폴더 밖으로 나가는 경로(`..` 등)나 패키지 파일은 거절한다.
    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if !matches!(id.root(), VirtualRoot::Project) {
            // 패키지는 네트워크에서 내려받아야 하므로 지원하지 않는다
            return Err(FileError::Other(Some("패키지(@...)는 지원하지 않습니다".into())));
        }
        let root = self
            .root
            .as_deref()
            .ok_or_else(|| FileError::Other(Some("저장하지 않은 문서는 다른 파일을 읽을 수 없습니다".into())))?;
        let path = id.vpath().realize(root).map_err(|_| FileError::AccessDenied)?;
        // 심볼릭 링크 등으로 문서 폴더 밖을 가리키면 거절한다
        let canonical_root = root.canonicalize().map_err(|error| FileError::from_io(error, root))?;
        let canonical = path.canonicalize().map_err(|error| FileError::from_io(error, &path))?;
        if !canonical.starts_with(&canonical_root) {
            return Err(FileError::AccessDenied);
        }
        Ok(canonical)
    }
}

impl World for DonWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.fonts.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            return Ok(self.source.clone());
        }
        let path = self.resolve(id)?;
        let text = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        let text = String::from_utf8(text).map_err(|_| FileError::InvalidUtf8)?;
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main {
            return Ok(Bytes::new(self.source.text().as_bytes().to_vec()));
        }
        let path = self.resolve(id)?;
        let data = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(Bytes::new(data))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        // 오늘 날짜(UTC 기준)를 시스템 시계로 구한다. 시간대 오프셋은 시(hour) 단위로만 반영한다.
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;
        let offset_seconds = offset.map(|duration| duration.seconds() as i64).unwrap_or(0);
        let days = (seconds + offset_seconds).div_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        Datetime::from_ymd(year as i32, month as u8, day as u8)
    }
}

/// 1970-01-01 부터의 날짜 수를 (년, 월, 일)로 바꾼다 (그레고리력, Howard Hinnant 의 civil_from_days 알고리즘).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// 프로세스 전체에서 한 번만 만드는 글꼴 모음 (첫 호출 때 `extra_font_data` 를 쓴다)
pub fn shared_fonts(extra_font_data: &[Vec<u8>]) -> FontSet {
    static FONTS: OnceLock<FontSet> = OnceLock::new();
    FONTS.get_or_init(|| FontSet::new(extra_font_data)).clone()
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    #[test]
    fn civil_dates_are_computed_correctly() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        // 윤년 2월 29일
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_427), (2025, 12, 5));
    }
}
