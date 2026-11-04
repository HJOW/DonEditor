//! [DonEditor] Typst 지원: Markdown→Typst 변환, Typst 컴파일, PDF·SVG 내보내기. GPL-3.0-or-later.
//!
//! GUI 에 의존하지 않으므로 단위 테스트에서 실제로 컴파일해 볼 수 있다.
//! Typst(Apache-2.0, <https://github.com/typst/typst>)의 컴파일러와 PDF·SVG 내보내기를 라이브러리로 쓴다.
//! Markdown 변환 규칙은 md2typ(Apache-2.0, <https://github.com/nervosys/md2typ>)를 참고했다.

pub mod markdown;
mod world;

pub use markdown::{ConvertOptions, markdown_to_typst};
pub use world::{DonWorld, FontSet, shared_fonts};

use std::path::PathBuf;

use typst::WorldExt as _;
use typst::diag::{Severity, SourceDiagnostic};
use typst::syntax::{DiagSpanKind, Source};
use typst_layout::PagedDocument;

/// 컴파일 진단(오류·경고) 한 건
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// 오류이면 `true`, 경고이면 `false`
    pub is_error: bool,
    /// 사용자에게 보여 줄 메시지
    pub message: String,
    /// 문서 본문(원본)에서의 줄(1부터)과 열(1부터). 다른 파일이나 위치를 모르면 `None`
    pub line: Option<usize>,
    pub column: Option<usize>,
    /// 해결 방법 안내
    pub hints: Vec<String>,
}

/// 컴파일에 성공한 문서
pub struct CompiledDocument {
    document: PagedDocument,
    /// 컴파일은 성공했지만 알려 줄 경고
    pub warnings: Vec<Diagnostic>,
}

impl CompiledDocument {
    /// 쪽 수
    pub fn page_count(&self) -> usize {
        self.document.pages().len()
    }

    /// 각 쪽의 크기(포인트, 가로×세로)
    pub fn page_sizes(&self) -> Vec<(f32, f32)> {
        self.document
            .pages()
            .iter()
            .map(|page| {
                let size = page.frame.size();
                (size.x.to_pt() as f32, size.y.to_pt() as f32)
            })
            .collect()
    }

    /// 각 쪽을 SVG 글자로 만든다. 미리보기 화면에 쓴다. 글자는 윤곽선으로 들어 있어 글꼴 없이도 같게 보인다.
    pub fn pages_svg(&self) -> Vec<String> {
        let options = typst_svg::SvgOptions::default();
        self.document
            .pages()
            .iter()
            .map(|page| typst_svg::svg(page, &options))
            .collect()
    }

    /// 전체 문서를 PDF 바이트로 만든다.
    pub fn to_pdf(&self) -> Result<Vec<u8>, String> {
        typst_pdf::pdf(&self.document, &typst_pdf::PdfOptions::default()).map_err(|errors| {
            errors
                .iter()
                .map(|error| error.message.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })
    }
}

/// 진단을 사용자에게 보여 줄 형태로 바꾼다. `main_source` 는 문서 본문으로, 줄·열 계산에 쓴다.
fn to_diagnostic(diagnostic: &SourceDiagnostic, world: &DonWorld, main_source: &Source) -> Diagnostic {
    let in_main = match diagnostic.span.get() {
        DiagSpanKind::Number { id, .. } | DiagSpanKind::Range { id, .. } => id == typst::World::main(world),
        DiagSpanKind::Detached => false,
    };
    let (line, column) = if in_main {
        world
            .range(diagnostic.span)
            .and_then(|range| main_source.lines().byte_to_line_column(range.start))
            // 줄·열은 0부터 세므로 사람이 읽는 번호로 바꾼다
            .map_or((None, None), |(line, column)| (Some(line + 1), Some(column + 1)))
    } else {
        (None, None)
    };
    Diagnostic {
        is_error: diagnostic.severity == Severity::Error,
        message: diagnostic.message.to_string(),
        line,
        column,
        hints: diagnostic.hints.iter().map(|hint| hint.v.to_string()).collect(),
    }
}

/// Typst 문서를 컴파일한다.
///
/// - `source`: 문서 본문(저장하지 않은 편집 내용 포함)
/// - `root`: 문서가 있는 폴더. `#image`·`#include` 의 기준이다. 저장하지 않은 문서는 `None`
/// - `extra_fonts`: 앱이 가진 글꼴 파일 내용(D2Coding 등). 프로세스에서 처음 호출할 때만 쓰인다.
///
/// 성공하면 문서와 경고를, 실패하면 오류 목록을 돌려준다.
pub fn compile(
    source: String,
    root: Option<PathBuf>,
    extra_fonts: &[Vec<u8>],
) -> Result<CompiledDocument, Vec<Diagnostic>> {
    let fonts = shared_fonts(extra_fonts);
    let world = DonWorld::new(source.clone(), root, fonts);
    let main_source = Source::detached(source);
    let warned = typst::compile::<PagedDocument>(&world);
    let warnings: Vec<Diagnostic> = warned
        .warnings
        .iter()
        .map(|warning| to_diagnostic(warning, &world, &main_source))
        .collect();
    match warned.output {
        Ok(document) => Ok(CompiledDocument { document, warnings }),
        Err(errors) => Err(errors
            .iter()
            .map(|error| to_diagnostic(error, &world, &main_source))
            .collect()),
    }
}

/// Markdown 문서를 Typst 로 바꾼 뒤 컴파일한다 (인쇄·PDF 변환에서 중간 파일 없이 메모리에서만 쓴다).
pub fn compile_markdown(
    markdown: &str,
    root: Option<PathBuf>,
    extra_fonts: &[Vec<u8>],
) -> Result<CompiledDocument, Vec<Diagnostic>> {
    compile(
        markdown_to_typst(markdown, &ConvertOptions::default()),
        root,
        extra_fonts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_simple_document_compiles_to_one_page_pdf_and_svg() {
        let compiled =
            compile("= 제목\n\n본문 *굵게* 입니다. Hello, world!".into(), None, &[]).unwrap_or_else(|errors| {
                panic!("컴파일 실패: {errors:?}");
            });
        assert_eq!(compiled.page_count(), 1);
        let pdf = compiled.to_pdf().unwrap();
        assert!(pdf.starts_with(b"%PDF-"), "PDF 머리글이 아닙니다");
        let svg = compiled.pages_svg();
        assert_eq!(svg.len(), 1);
        assert!(svg[0].contains("<svg"));
        let (width, height) = compiled.page_sizes()[0];
        // 기본 용지(A4 = 595.28 × 841.89 pt)
        assert!(
            (width - 595.28).abs() < 1.0 && (height - 841.89).abs() < 1.0,
            "{width}x{height}"
        );
    }

    #[test]
    fn errors_report_the_line_and_column_in_the_document() {
        let errors = compile("첫 줄\n#unknown_function()\n".into(), None, &[])
            .err()
            .expect("오류여야 한다");
        assert!(errors[0].is_error);
        assert_eq!(errors[0].line, Some(2), "{errors:?}");
    }

    #[test]
    fn files_outside_the_document_folder_are_refused() {
        let directory = std::env::temp_dir().join("don_typst_test_folder");
        std::fs::create_dir_all(&directory).unwrap();
        let errors = compile("#image(\"../escape.png\")".into(), Some(directory), &[])
            .err()
            .expect("오류여야 한다");
        assert!(errors.iter().any(|error| error.is_error), "{errors:?}");
        // 저장하지 않은 문서(폴더 없음)는 파일을 읽지 못한다
        assert!(compile("#image(\"a.png\")".into(), None, &[]).is_err());
    }

    /// Markdown 변환 결과가 실제로 Typst 로 컴파일되는지 확인한다 (변환기의 이스케이프·표·각주 규칙 검증).
    #[test]
    fn converted_markdown_compiles_with_all_supported_constructs() {
        let markdown = "# 제목 $5 #태그\n\n\
            본문 **굵게** *기울임* ~~취소~~ `코드` [링크](https://example.com) user@example.com\n\n\
            - 목록 하나\n- 목록 둘\n  - 중첩\n\n\
            1. 첫째\n2. 둘째\n\n\
            - [x] 완료\n- [ ] 미완료\n\n\
            > 인용문 [대괄호] <꺾쇠>\n\n\
            | 이름 | 값 |\n|:--|--:|\n| a | 1 |\n| b | 2 |\n\n\
            각주가 있는 문장[^1].\n\n\
            ```rust\nfn main() { println!(\"안녕\"); }\n```\n\n\
            ---\n\n\
            마지막 문단입니다.\n\n[^1]: 각주 내용입니다.\n";
        let typst = markdown_to_typst(markdown, &ConvertOptions::default());
        let compiled = compile(typst.clone(), None, &[]).unwrap_or_else(|errors| {
            panic!("변환 결과가 컴파일되지 않았습니다: {errors:?}\n--- 변환 결과 ---\n{typst}");
        });
        assert!(compiled.page_count() >= 1);
        assert!(compiled.to_pdf().unwrap().starts_with(b"%PDF-"));
    }

    #[test]
    fn empty_markdown_still_compiles() {
        assert!(compile_markdown("", None, &[]).is_ok());
    }
}
