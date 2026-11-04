//! [DonEditor] 문자 인코딩(캐릭터셋) 공용 처리. GPL-3.0-or-later.
//!
//! - 선택 화면에 보여 줄 지원 인코딩 목록과 언어 계열 묶음
//! - 자동 감지 결과나 BOM 과 관계없이 지정한 인코딩으로 바이트를 **강제로 해독**하는 함수 (깨진 글자 복원용)
//! - 대상 인코딩으로 표현할 수 없는 문자가 있으면 **조용히 바꾸지 않고 알려 주는 인코딩** 함수 (저장용)
//!
//! GUI 에 의존하지 않으므로 단위 테스트를 빠르게 실행할 수 있다.

use encoding_rs::{
    BIG5, EUC_JP, EUC_KR, Encoding, GB18030, GBK, IBM866, ISO_2022_JP, ISO_8859_2, ISO_8859_3, ISO_8859_4, ISO_8859_5,
    ISO_8859_6, ISO_8859_7, ISO_8859_8, ISO_8859_10, ISO_8859_13, ISO_8859_14, ISO_8859_15, ISO_8859_16, KOI8_R,
    KOI8_U, MACINTOSH, SHIFT_JIS, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_874, WINDOWS_1250, WINDOWS_1251, WINDOWS_1252,
    WINDOWS_1253, WINDOWS_1254, WINDOWS_1255, WINDOWS_1256, WINDOWS_1257, WINDOWS_1258, X_MAC_CYRILLIC,
};

/// 선택 화면에서 인코딩을 묶어 보여 줄 언어 계열
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharsetGroup {
    /// UTF-8, UTF-16
    Unicode,
    Korean,
    Japanese,
    SimplifiedChinese,
    TraditionalChinese,
    WesternEuropean,
    CentralEuropean,
    SouthEuropean,
    Baltic,
    Cyrillic,
    Greek,
    Turkish,
    Hebrew,
    Arabic,
    Thai,
    Vietnamese,
}

impl CharsetGroup {
    /// 계열 이름을 i18n 카탈로그에서 찾을 때 쓰는 키
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Unicode => "charset.group.unicode",
            Self::Korean => "charset.group.korean",
            Self::Japanese => "charset.group.japanese",
            Self::SimplifiedChinese => "charset.group.simplified_chinese",
            Self::TraditionalChinese => "charset.group.traditional_chinese",
            Self::WesternEuropean => "charset.group.western_european",
            Self::CentralEuropean => "charset.group.central_european",
            Self::SouthEuropean => "charset.group.south_european",
            Self::Baltic => "charset.group.baltic",
            Self::Cyrillic => "charset.group.cyrillic",
            Self::Greek => "charset.group.greek",
            Self::Turkish => "charset.group.turkish",
            Self::Hebrew => "charset.group.hebrew",
            Self::Arabic => "charset.group.arabic",
            Self::Thai => "charset.group.thai",
            Self::Vietnamese => "charset.group.vietnamese",
        }
    }
}

/// 선택 화면에 보여 줄 인코딩 한 항목
#[derive(Clone, Copy, Debug)]
pub struct CharsetEntry {
    /// encoding_rs 의 인코딩
    pub encoding: &'static Encoding,
    /// 어느 언어 계열인지
    pub group: CharsetGroup,
}

/// 선택 화면에 보여 줄 인코딩 목록. 한국어 사용자가 자주 쓰는 순서(유니코드 → 한국어 → 일본어 → 중국어 → 그 밖)로 둔다.
/// 모두 encoding_rs 가 해독할 수 있고, 저장(인코딩)도 가능한 것만 넣는다.
pub fn supported_charsets() -> &'static [CharsetEntry] {
    // 목록은 상수이므로 한 번만 만들어 둔다
    const fn entry(encoding: &'static Encoding, group: CharsetGroup) -> CharsetEntry {
        CharsetEntry { encoding, group }
    }
    static CHARSETS: &[CharsetEntry] = &[
        entry(UTF_8, CharsetGroup::Unicode),
        entry(UTF_16LE, CharsetGroup::Unicode),
        entry(UTF_16BE, CharsetGroup::Unicode),
        // EUC-KR 은 encoding_rs 에서 windows-949(CP949) 확장을 포함한다
        entry(EUC_KR, CharsetGroup::Korean),
        entry(SHIFT_JIS, CharsetGroup::Japanese),
        entry(EUC_JP, CharsetGroup::Japanese),
        entry(ISO_2022_JP, CharsetGroup::Japanese),
        entry(GBK, CharsetGroup::SimplifiedChinese),
        entry(GB18030, CharsetGroup::SimplifiedChinese),
        entry(BIG5, CharsetGroup::TraditionalChinese),
        entry(WINDOWS_1252, CharsetGroup::WesternEuropean),
        entry(ISO_8859_15, CharsetGroup::WesternEuropean),
        entry(ISO_8859_14, CharsetGroup::WesternEuropean),
        entry(MACINTOSH, CharsetGroup::WesternEuropean),
        entry(WINDOWS_1250, CharsetGroup::CentralEuropean),
        entry(ISO_8859_2, CharsetGroup::CentralEuropean),
        entry(ISO_8859_16, CharsetGroup::CentralEuropean),
        entry(ISO_8859_3, CharsetGroup::SouthEuropean),
        entry(WINDOWS_1257, CharsetGroup::Baltic),
        entry(ISO_8859_4, CharsetGroup::Baltic),
        entry(ISO_8859_10, CharsetGroup::Baltic),
        entry(ISO_8859_13, CharsetGroup::Baltic),
        entry(WINDOWS_1251, CharsetGroup::Cyrillic),
        entry(KOI8_R, CharsetGroup::Cyrillic),
        entry(KOI8_U, CharsetGroup::Cyrillic),
        entry(ISO_8859_5, CharsetGroup::Cyrillic),
        entry(IBM866, CharsetGroup::Cyrillic),
        entry(X_MAC_CYRILLIC, CharsetGroup::Cyrillic),
        entry(WINDOWS_1253, CharsetGroup::Greek),
        entry(ISO_8859_7, CharsetGroup::Greek),
        entry(WINDOWS_1254, CharsetGroup::Turkish),
        entry(WINDOWS_1255, CharsetGroup::Hebrew),
        entry(ISO_8859_8, CharsetGroup::Hebrew),
        entry(WINDOWS_1256, CharsetGroup::Arabic),
        entry(ISO_8859_6, CharsetGroup::Arabic),
        entry(WINDOWS_874, CharsetGroup::Thai),
        entry(WINDOWS_1258, CharsetGroup::Vietnamese),
    ];
    CHARSETS
}

/// 이름(`EUC-KR`, `euc-kr`, `utf8` 등 WHATWG 레이블)으로 인코딩을 찾는다. 모르는 이름이면 `None`.
pub fn encoding_for_label(label: &str) -> Option<&'static Encoding> {
    Encoding::for_label(label.trim().as_bytes())
}

/// 강제 해독 결과
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    /// 해독한 글자. 해독할 수 없는 바이트는 대체 문자(U+FFFD)로 들어 있다.
    pub text: String,
    /// 파일이 지정한 인코딩과 같은 종류의 BOM 으로 시작했는지 (BOM 은 `text` 에 포함하지 않는다)
    pub has_bom: bool,
    /// 해독할 수 없는 바이트가 있어 대체 문자가 들어갔는지
    pub had_errors: bool,
}

/// 파일 맨 앞의 BOM 이 지정한 인코딩과 같은 종류이면 떼고 `(나머지, true)`, 아니면 `(그대로, false)` 를 돌려준다.
/// 지정한 인코딩과 다른 BOM(예: EUC-KR 로 읽는데 UTF-8 BOM)은 일반 바이트로 보고 건드리지 않는다.
fn strip_matching_bom<'a>(bytes: &'a [u8], encoding: &'static Encoding) -> (&'a [u8], bool) {
    // 인코딩마다 그 인코딩의 BOM 바이트열
    let bom: &[u8] = if encoding == UTF_8 {
        &[0xEF, 0xBB, 0xBF]
    } else if encoding == UTF_16LE {
        &[0xFF, 0xFE]
    } else if encoding == UTF_16BE {
        &[0xFE, 0xFF]
    } else {
        return (bytes, false);
    };
    match bytes.strip_prefix(bom) {
        Some(rest) => (rest, true),
        None => (bytes, false),
    }
}

/// 바이트를 자동 감지나 BOM 과 관계없이 **지정한 인코딩으로 강제 해독**한다.
/// 글자가 이미 깨져 보일 때 다른 인코딩으로 다시 읽어 보는 용도다. 해독할 수 없는 바이트는 오류로 막지 않고
/// 대체 문자(U+FFFD)로 두며 `had_errors` 로 알린다.
pub fn decode_forced(bytes: &[u8], encoding: &'static Encoding) -> Decoded {
    let (body, has_bom) = strip_matching_bom(bytes, encoding);
    let (text, had_errors) = encoding.decode_without_bom_handling(body);
    Decoded {
        text: text.into_owned(),
        has_bom,
        had_errors,
    }
}

/// 지정한 인코딩으로 표현할 수 없는 문자
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnmappableCharacter {
    /// 표현할 수 없는 첫 문자
    pub character: char,
    /// 그 문자가 있는 줄(1부터)
    pub line: usize,
}

/// 글자를 지정한 인코딩의 바이트로 바꾼다. `has_bom` 이면 인코딩에 맞는 BOM 을 앞에 붙인다(UTF-8/UTF-16 만 해당).
///
/// encoding_rs 의 기본 동작은 표현할 수 없는 문자를 `&#128512;` 같은 HTML 숫자 참조로 **조용히 바꿔 쓴다.**
/// 저장 중에 내용이 바뀌는 것을 막기 위해, 표현할 수 없는 문자가 있으면 바꾸지 않고 [`UnmappableCharacter`] 를 돌려준다.
pub fn encode_checked(text: &str, encoding: &'static Encoding, has_bom: bool) -> Result<Vec<u8>, UnmappableCharacter> {
    if encoding == UTF_8 {
        let mut bytes = Vec::with_capacity(text.len() + 3);
        if has_bom {
            bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        }
        bytes.extend_from_slice(text.as_bytes());
        return Ok(bytes);
    }

    // encoding_rs 는 UTF-16 을 인코딩하지 못하고 UTF-8 로 바꿔 버리므로 직접 변환한다
    if encoding == UTF_16LE || encoding == UTF_16BE {
        let little_endian = encoding == UTF_16LE;
        let mut bytes = Vec::with_capacity(text.len() * 2 + 2);
        if has_bom {
            bytes.extend_from_slice(if little_endian { &[0xFF, 0xFE] } else { &[0xFE, 0xFF] });
        }
        // 코드 단위(u16)마다 지정한 바이트 순서로 쓴다
        for unit in text.encode_utf16() {
            let unit_bytes = if little_endian {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            };
            bytes.extend_from_slice(&unit_bytes);
        }
        return Ok(bytes);
    }

    let (bytes, _, had_unmappable) = encoding.encode(text);
    if !had_unmappable {
        return Ok(bytes.into_owned());
    }
    Err(first_unmappable(text, encoding))
}

/// 인코딩할 수 없는 첫 문자를 찾는다. 실패 경로에서만 호출하므로 문자마다 인코딩해 보는 느린 방식을 쓴다.
fn first_unmappable(text: &str, encoding: &'static Encoding) -> UnmappableCharacter {
    let mut line = 1;
    let mut buffer = [0u8; 4];
    // 문자를 하나씩 인코딩해 보며 처음 실패하는 문자를 찾는다
    for character in text.chars() {
        if character == '\n' {
            line += 1;
        }
        let (_, _, had_unmappable) = encoding.encode(character.encode_utf8(&mut buffer));
        if had_unmappable {
            return UnmappableCharacter { character, line };
        }
    }
    // 전체는 실패했는데 문자별로는 성공하는 경우(상태가 있는 인코딩의 경계 문제)는 첫 문자로 보고한다
    UnmappableCharacter {
        character: text.chars().next().unwrap_or('\u{FFFD}'),
        line: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "한글 테스트" 의 EUC-KR(CP949) 바이트
    fn korean_euc_kr() -> Vec<u8> {
        EUC_KR.encode("한글 테스트").0.into_owned()
    }

    #[test]
    fn forced_decode_uses_the_given_encoding() {
        let bytes = korean_euc_kr();
        let decoded = decode_forced(&bytes, EUC_KR);
        assert_eq!(decoded.text, "한글 테스트");
        assert!(!decoded.had_errors && !decoded.has_bom);
    }

    #[test]
    fn forced_decode_with_the_wrong_encoding_gives_replacement_characters_and_reports_errors() {
        let bytes = korean_euc_kr();
        // EUC-KR 바이트를 UTF-8 로 읽으면 깨진다 (깨진 글자 복원 상황)
        let decoded = decode_forced(&bytes, UTF_8);
        assert!(decoded.had_errors);
        assert!(decoded.text.contains('\u{FFFD}'));
        // 다시 올바른 인코딩으로 읽으면 원래대로 돌아온다 (디스크 바이트는 그대로이므로 부분 복원이 가능)
        assert_eq!(decode_forced(&bytes, EUC_KR).text, "한글 테스트");
    }

    #[test]
    fn matching_bom_is_removed_and_recorded() {
        let utf8_bom = [&[0xEF, 0xBB, 0xBF][..], "가나".as_bytes()].concat();
        let decoded = decode_forced(&utf8_bom, UTF_8);
        assert_eq!((decoded.text.as_str(), decoded.has_bom), ("가나", true));

        let utf16le_bom = [
            &[0xFF, 0xFE][..],
            &"가".encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>(),
        ]
        .concat();
        let decoded = decode_forced(&utf16le_bom, UTF_16LE);
        assert_eq!((decoded.text.as_str(), decoded.has_bom), ("가", true));
    }

    #[test]
    fn a_bom_of_another_encoding_is_not_stripped() {
        // UTF-8 BOM 이 붙은 파일을 EUC-KR 로 강제로 읽으면 BOM 바이트도 그대로 일반 글자가 된다 (자동 감지 결과를 덮어씀)
        let utf8_bom = [&[0xEF, 0xBB, 0xBF][..], b"abc"].concat();
        let decoded = decode_forced(&utf8_bom, EUC_KR);
        assert!(!decoded.has_bom);
        assert_ne!(decoded.text, "abc");
        assert!(!decoded.text.starts_with('\u{FEFF}'));
        // UTF-16LE BOM 을 UTF-8 로 읽으면 BOM 이 아니라 해독 불가 바이트로 취급한다
        assert!(!decode_forced(&[0xFF, 0xFE, b'a'], UTF_8).has_bom);
    }

    #[test]
    fn encoding_round_trips_for_every_supported_charset_with_ascii() {
        // 모든 지원 인코딩에서 ASCII 글자는 저장 후 강제 해독해도 같아야 한다
        for entry in supported_charsets() {
            let bytes = encode_checked("abc\nXYZ 123\n", entry.encoding, false).unwrap();
            assert_eq!(
                decode_forced(&bytes, entry.encoding).text,
                "abc\nXYZ 123\n",
                "{}",
                entry.encoding.name()
            );
        }
    }

    #[test]
    fn korean_text_round_trips_through_euc_kr_and_utf16() {
        for encoding in [EUC_KR, UTF_8, UTF_16LE, UTF_16BE] {
            let bytes = encode_checked("한글 저장 — 왕복 확인", encoding, false);
            // EUC-KR(CP949)은 "—"(EM DASH)를 표현할 수 있지만 환경마다 다를 수 있으므로 한글만 비교한다
            if let Ok(bytes) = bytes {
                assert!(
                    decode_forced(&bytes, encoding).text.starts_with("한글 저장"),
                    "{}",
                    encoding.name()
                );
            }
        }
        let bytes = encode_checked("한글", UTF_16LE, true).unwrap();
        assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
        assert_eq!(
            decode_forced(&bytes, UTF_16LE),
            Decoded {
                text: "한글".into(),
                has_bom: true,
                had_errors: false
            }
        );
        let bytes = encode_checked("한글", UTF_8, true).unwrap();
        assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn unmappable_characters_are_reported_instead_of_silently_replaced() {
        // 이모지는 EUC-KR 로 표현할 수 없다. encoding_rs 기본 동작은 "&#128512;" 로 바꿔 쓰지만 여기서는 오류로 알려야 한다
        let error = encode_checked("첫 줄\n둘째 줄 😀 끝", EUC_KR, false).unwrap_err();
        assert_eq!(
            error,
            UnmappableCharacter {
                character: '😀',
                line: 2
            }
        );
        // UTF-8 과 UTF-16 은 모든 글자를 표현할 수 있다
        assert!(encode_checked("😀", UTF_8, false).is_ok());
        assert!(encode_checked("😀", UTF_16BE, false).is_ok());
    }

    #[test]
    fn labels_are_resolved() {
        assert_eq!(encoding_for_label("euc-kr"), Some(EUC_KR));
        assert_eq!(encoding_for_label(" UTF-8 "), Some(UTF_8));
        // WHATWG 표준에서 한국어 레이블은 모두 EUC-KR(CP949 확장 포함)로 연결된다. "cp949" 는 표준 레이블이 아니다
        assert_eq!(encoding_for_label("windows-949"), Some(EUC_KR));
        assert_eq!(encoding_for_label("ks_c_5601-1987"), Some(EUC_KR));
        assert_eq!(encoding_for_label("cp949"), None);
        assert_eq!(encoding_for_label("not-an-encoding"), None);
    }

    #[test]
    fn the_supported_list_has_no_duplicates_and_starts_with_utf8() {
        let charsets = supported_charsets();
        assert_eq!(charsets[0].encoding, UTF_8);
        let mut names: Vec<_> = charsets.iter().map(|entry| entry.encoding.name()).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "지원 목록에 같은 인코딩이 두 번 들어 있다");
    }
}
