//! [DonEditor] Markdown → Typst 변환. GPL-3.0-or-later.
//!
//! 변환 규칙은 nervosys/md2typ(Apache-2.0, <https://github.com/nervosys/md2typ>)의 대응 관계를 참고했다.
//! 제목 `=`, 강조 `#emph`/`#strong`/`#strike`, 링크 `#link`, 인용 `#quote`, 수평선 `#line`, 각주 `#footnote` 등이다.
//! 코드는 md2typ 의 소스를 가져온 것이 아니라 같은 규칙을 따라 새로 작성했으며, md2typ 와 다른 점은 다음과 같다.
//! - 표는 Typst 문법에 맞게 `#table(columns: N, align: (...), table.header(...), ...)` 로 만든다.
//! - Typst 마크업을 깨는 문자(`# $ * _ ` < > @ [ ] ~ \` 와 줄 맨 앞의 `= - + /`, `숫자.`)를 모두 이스케이프한다.
//! - 변환 결과가 Typst 로 컴파일되는지 이 crate 의 시험에서 실제로 컴파일해 확인한다.

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

/// 변환 옵션
#[derive(Clone, Debug)]
pub struct ConvertOptions {
    /// 결과 맨 앞에 용지·글꼴 기본 설정(`#set page/text`)을 붙일지 여부
    pub preamble: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self { preamble: true }
    }
}

/// 각주 본문을 임시로 가리키는 표지. 각주 정의가 본문보다 뒤에 있어도 참조 자리에 본문을 넣기 위해 쓴다.
const FOOTNOTE_OPEN: char = '\u{E000}';
const FOOTNOTE_CLOSE: char = '\u{E001}';

/// 목록 한 단계의 상태
enum ListKind {
    /// 글머리 기호 목록
    Bullet,
    /// 번호 목록과 다음에 쓸 번호
    Ordered(u64),
}

/// Markdown 문서를 Typst 마크업으로 바꾼다.
pub fn markdown_to_typst(markdown: &str, options: &ConvertOptions) -> String {
    let mut converter = Converter::default();
    let body = converter.convert(markdown);
    let mut output = String::new();
    if options.preamble {
        output.push_str(&preamble(markdown));
    }
    output.push_str(body.trim_end());
    output.push('\n');
    output
}

/// 용지·글꼴 기본 설정. 한글이 들어 있으면 문서 언어를 한국어로 한다.
fn preamble(markdown: &str) -> String {
    // 한글 음절이 하나라도 있으면 한국어 문서로 본다
    let has_hangul = markdown.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c));
    let lang = if has_hangul { "ko" } else { "en" };
    format!(
        "// Don Editor 가 Markdown 에서 변환한 Typst 문서\n\
         #set page(paper: \"a4\", margin: 2cm)\n\
         #set text(font: (\"Libertinus Serif\", \"D2Coding\"), size: 10.5pt, lang: \"{lang}\")\n\
         #set par(justify: false, leading: 0.8em)\n\
         #show raw: set text(font: (\"D2Coding\", \"DejaVu Sans Mono\"))\n\
         #show link: set text(fill: rgb(\"#0b5cad\"))\n\n"
    )
}

/// 변환 중 상태를 모아 둔 구조체
#[derive(Default)]
struct Converter {
    /// 지금 쓰고 있는 출력. 표 셀·각주 정의처럼 따로 모아야 할 때는 스택으로 바꿔 쓴다.
    stack: Vec<String>,
    list_stack: Vec<ListKind>,
    /// 표 상태: 열 정렬, 현재 행의 셀, 머리글 여부, 지금까지 만든 셀 목록
    table_alignments: Vec<Alignment>,
    in_table_head: bool,
    table_cells: Vec<String>,
    table_header_cells: Vec<String>,
    /// 코드 블록 안인지 여부와 모아 둔 코드
    code_block: Option<(String, String)>,
    /// 각주 이름 → 변환된 본문
    footnotes: HashMap<String, String>,
    footnote_stack: Vec<String>,
    /// 링크 안인지 여부 (이미지의 대체 글자 등을 위한 상태)
    image_alt: Option<(String, String)>,
    /// 새 줄 맨 앞인지 (줄 맨 앞에서만 의미가 있는 기호를 이스케이프하기 위한 상태)
    at_line_start: bool,
    /// 인용문 깊이
    quote_depth: usize,
    /// 방금 쓴 글자가 줄 맨 앞의 숫자뿐이었는지 (바로 뒤에 "." 이 오면 번호 목록이 되므로 점을 이스케이프해야 한다)
    digits_at_line_start: bool,
}

impl Converter {
    /// 지금 쓰는 출력 버퍼
    fn out(&mut self) -> &mut String {
        self.stack.last_mut().expect("출력 버퍼가 비어 있습니다")
    }

    fn push(&mut self, text: &str) {
        if !text.is_empty() {
            self.at_line_start = text.ends_with('\n');
            self.out().push_str(text);
        }
    }

    /// 새 출력 버퍼를 시작한다 (표 셀, 각주 정의, 인용문 본문 등)
    fn begin_capture(&mut self) {
        self.stack.push(String::new());
        self.at_line_start = true;
    }

    fn end_capture(&mut self) -> String {
        self.stack.pop().unwrap_or_default()
    }

    fn convert(&mut self, markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        self.stack.push(String::new());
        self.at_line_start = true;
        // 이벤트를 하나씩 처리한다
        for event in Parser::new_ext(markdown, options) {
            self.handle(event);
        }
        let body = self.end_capture();
        self.resolve_footnotes(body)
    }

    /// 본문에 넣어 둔 각주 표지를 실제 `#footnote[...]` 로 바꾼다
    fn resolve_footnotes(&self, body: String) -> String {
        let mut result = String::with_capacity(body.len());
        let mut rest = body.as_str();
        // 표지를 만날 때마다 이름을 읽어 각주 본문으로 바꾼다
        while let Some(start) = rest.find(FOOTNOTE_OPEN) {
            result.push_str(&rest[..start]);
            let after = &rest[start + FOOTNOTE_OPEN.len_utf8()..];
            let Some(end) = after.find(FOOTNOTE_CLOSE) else {
                rest = after;
                break;
            };
            let name = &after[..end];
            match self.footnotes.get(name) {
                Some(text) => {
                    result.push_str("#footnote[");
                    result.push_str(text.trim());
                    result.push(']');
                }
                None => {
                    result.push_str("#super[");
                    result.push_str(&escape_text(name, false));
                    result.push(']');
                }
            }
            rest = &after[end + FOOTNOTE_CLOSE.len_utf8()..];
        }
        result.push_str(rest);
        result
    }

    fn handle(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(code) => {
                if let Some((alt, _)) = &mut self.image_alt {
                    alt.push_str(&code);
                } else {
                    let raw = raw_inline(&code);
                    self.push(&raw);
                }
            }
            Event::SoftBreak => {
                if self.code_block.is_some() {
                    self.text("\n");
                } else if self.image_alt.is_none() {
                    // Markdown 의 줄 바꿈은 공백이다 (Typst 도 같은 문단 안의 줄 바꿈을 공백으로 본다)
                    self.push(" ");
                }
            }
            Event::HardBreak => self.push(" \\\n"),
            Event::Rule => self.push("#line(length: 100%)\n\n"),
            Event::Html(html) | Event::InlineHtml(html) => self.html(&html),
            Event::FootnoteReference(name) => {
                let marker = format!("{FOOTNOTE_OPEN}{name}{FOOTNOTE_CLOSE}");
                self.push(&marker);
            }
            Event::TaskListMarker(checked) => {
                // Typst 에는 체크박스가 없으므로 글자로 표시한다
                self.push(if checked { "\\[x\\] " } else { "\\[ \\] " });
            }
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading { level, .. } => {
                let depth = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let marker = format!("{} ", "=".repeat(depth));
                self.push(&marker);
                // 제목 맨 앞은 이미 표시를 썼으므로 줄 맨 앞 이스케이프는 필요 없다
                self.at_line_start = false;
            }
            Tag::Emphasis => self.push("#emph["),
            Tag::Strong => self.push("#strong["),
            Tag::Strikethrough => self.push("#strike["),
            Tag::Link { dest_url, .. } => {
                let open = format!("#link({})[", string_literal(&dest_url));
                self.push(&open);
            }
            Tag::Image { dest_url, .. } => {
                self.image_alt = Some((String::new(), dest_url.to_string()));
            }
            Tag::BlockQuote(_) => {
                self.quote_depth += 1;
                self.push("#quote(block: true)[\n");
            }
            Tag::List(start) => {
                // 목록 앞에는 빈 줄을 둔다 (이미 한 줄 비어 있지 않으면)
                if self.list_stack.is_empty() && !self.out().ends_with("\n\n") && !self.out().is_empty() {
                    self.push("\n");
                }
                self.list_stack.push(match start {
                    Some(number) => ListKind::Ordered(number),
                    None => ListKind::Bullet,
                });
            }
            Tag::Item => {
                let depth = self.list_stack.len().saturating_sub(1);
                if !self.out().is_empty() && !self.out().ends_with('\n') {
                    self.push("\n");
                }
                let indent = "  ".repeat(depth);
                let marker = match self.list_stack.last() {
                    Some(ListKind::Ordered(number)) => format!("{indent}{number}. "),
                    _ => format!("{indent}- "),
                };
                self.push(&marker);
                // 목록 표시 뒤는 줄 맨 앞이 아니다
                self.at_line_start = false;
            }
            Tag::CodeBlock(kind) => {
                let language = match kind {
                    CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or("").to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.code_block = Some((language, String::new()));
            }
            Tag::Table(alignments) => {
                self.table_alignments = alignments;
                self.table_cells.clear();
                self.table_header_cells.clear();
            }
            Tag::TableHead => self.in_table_head = true,
            Tag::TableRow => {}
            Tag::TableCell => self.begin_capture(),
            Tag::FootnoteDefinition(name) => {
                self.footnote_stack.push(name.to_string());
                self.begin_capture();
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                // 목록 항목 안의 문단은 줄만 바꾸고, 그 밖에는 빈 줄로 문단을 나눈다
                if self.list_stack.is_empty() {
                    self.push("\n\n");
                } else {
                    self.push("\n");
                }
            }
            TagEnd::Heading(_) => self.push("\n\n"),
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => self.push("]"),
            TagEnd::Image => {
                if let Some((alt, destination)) = self.image_alt.take() {
                    let image = image_markup(&alt, &destination);
                    self.push(&image);
                }
            }
            TagEnd::BlockQuote(_) => {
                self.quote_depth = self.quote_depth.saturating_sub(1);
                // 인용문 마지막 문단이 남긴 빈 줄을 모두 떼고 한 줄만 남긴 뒤 닫는다
                while self.out().ends_with('\n') {
                    self.out().pop();
                }
                self.push("\n]\n\n");
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.push("\n");
                }
            }
            TagEnd::Item => {
                if !self.out().ends_with('\n') {
                    self.push("\n");
                }
                if let Some(ListKind::Ordered(number)) = self.list_stack.last_mut() {
                    *number += 1;
                }
            }
            TagEnd::CodeBlock => {
                if let Some((language, code)) = self.code_block.take() {
                    let block = fenced_code(&language, &code);
                    self.push(&block);
                }
            }
            TagEnd::TableCell => {
                let cell = self.end_capture();
                let cell = cell.trim().to_string();
                if self.in_table_head {
                    self.table_header_cells.push(cell);
                } else {
                    self.table_cells.push(cell);
                }
            }
            TagEnd::TableHead => self.in_table_head = false,
            TagEnd::Table => {
                let table = self.table_markup();
                self.push(&table);
            }
            TagEnd::FootnoteDefinition => {
                let body = self.end_capture();
                if let Some(name) = self.footnote_stack.pop() {
                    self.footnotes.insert(name, body);
                }
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if let Some((alt, _)) = &mut self.image_alt {
            alt.push_str(text);
        } else if let Some((_, code)) = &mut self.code_block {
            code.push_str(text);
        } else {
            let at_start = self.at_line_start;
            let digits_before = std::mem::take(&mut self.digits_at_line_start);
            let mut escaped = escape_text(text, at_start);
            // 앞 글자가 줄 맨 앞의 숫자뿐이었고 이번 글자가 "." 로 시작하면 번호 목록으로 읽히므로 점을 이스케이프한다.
            // (CommonMark 는 `1\.` 을 "1" 과 ". ..." 두 글자 이벤트로 쪼갠다)
            if digits_before && escaped.starts_with('.') {
                escaped.insert(0, '\\');
            }
            self.push(&escaped);
            self.digits_at_line_start = at_start && !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
        }
    }

    fn html(&mut self, html: &str) {
        let trimmed = html.trim();
        if trimmed.starts_with("<!--") {
            // HTML 주석은 문서에 나타나지 않는다
            return;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower == "<br>" || lower == "<br/>" || lower == "<br />" {
            self.push(" \\\n");
            return;
        }
        // 그 밖의 HTML 은 변환할 수 없으므로 글자 그대로 보이게 한다
        let escaped = escape_text(html, false);
        self.push(&escaped);
    }

    /// 지금까지 모은 표 셀로 `#table(...)` 을 만든다
    fn table_markup(&mut self) -> String {
        let columns = self.table_alignments.len().max(self.table_header_cells.len()).max(1);
        // 열 정렬 목록: left / center / right / auto
        let aligns = (0..columns)
            .map(|index| match self.table_alignments.get(index) {
                Some(Alignment::Left) => "left",
                Some(Alignment::Center) => "center",
                Some(Alignment::Right) => "right",
                _ => "auto",
            })
            .collect::<Vec<_>>()
            .join(", ");
        let mut markup = format!("#table(\n  columns: {columns},\n  align: ({aligns},),\n");
        if !self.table_header_cells.is_empty() {
            markup.push_str("  table.header(");
            markup.push_str(
                &self
                    .table_header_cells
                    .iter()
                    .map(|cell| format!("[#strong[{cell}]]"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            markup.push_str("),\n");
        }
        // 본문 셀은 한 줄에 한 행씩 늘어놓는다 (행이 열 수보다 짧으면 Typst 가 빈 칸으로 채운다)
        for row in self.table_cells.chunks(columns) {
            markup.push_str("  ");
            markup.push_str(
                &row.iter()
                    .map(|cell| format!("[{cell}]"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            markup.push_str(",\n");
        }
        markup.push_str(")\n\n");
        markup
    }
}

/// Typst 마크업에서 특별한 뜻을 가진 문자를 이스케이프한다.
/// `at_line_start` 이면 줄 맨 앞에서 제목·목록·용어 목록으로 해석될 수 있는 시작 기호도 이스케이프한다.
fn escape_text(text: &str, at_line_start: bool) -> String {
    let mut escaped = String::with_capacity(text.len() + 8);
    for (index, character) in text.chars().enumerate() {
        match character {
            // 마크업·코드·수식·참조·레이블을 시작하는 문자
            '\\' | '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' | '[' | ']' | '~' => {
                escaped.push('\\');
                escaped.push(character);
            }
            // 줄 맨 앞의 `=`(제목), `-`·`+`(목록), `/`(용어 목록)
            '=' | '-' | '+' | '/' if at_line_start && index == 0 => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    // 줄 맨 앞의 "숫자." 는 번호 목록이 되므로 점을 이스케이프한다
    if at_line_start {
        let digits = escaped.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 && escaped[digits..].starts_with(". ") {
            escaped.insert(digits, '\\');
        }
    }
    escaped
}

/// 문자열 리터럴(`"..."`)로 만든다.
fn string_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for character in text.chars() {
        match character {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            _ => literal.push(character),
        }
    }
    literal.push('"');
    literal
}

/// 가장 긴 연속 백틱 수보다 하나 더 긴 울타리 문자열을 돌려준다 (최소 `min`개).
fn backtick_fence(text: &str, min: usize) -> String {
    let mut longest = 0;
    let mut current = 0;
    // 본문에 나오는 연속 백틱의 최대 길이를 센다
    for character in text.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    "`".repeat((longest + 1).max(min))
}

/// 인라인 코드 → Typst 원문(raw) 글자
fn raw_inline(code: &str) -> String {
    let fence = backtick_fence(code, 1);
    // 앞뒤가 백틱이면 공백을 넣어 울타리와 구분한다
    let pad = if code.starts_with('`') || code.ends_with('`') {
        " "
    } else {
        ""
    };
    if code.is_empty() {
        return "#raw(\"\")".to_string();
    }
    format!("{fence}{pad}{code}{pad}{fence}")
}

/// 코드 블록 → Typst 원문(raw) 블록
fn fenced_code(language: &str, code: &str) -> String {
    let fence = backtick_fence(code, 3);
    let mut block = String::new();
    block.push_str(&fence);
    block.push_str(language);
    block.push('\n');
    block.push_str(code);
    if !code.ends_with('\n') {
        block.push('\n');
    }
    block.push_str(&fence);
    block.push_str("\n\n");
    block
}

/// 이미지 → `#image(...)`. 네트워크 주소는 Typst 가 읽을 수 없으므로 링크로 바꾼다.
fn image_markup(alt: &str, destination: &str) -> String {
    let remote = ["http://", "https://", "data:"]
        .iter()
        .any(|prefix| destination.starts_with(prefix));
    if remote {
        let label = if alt.is_empty() { destination } else { alt };
        return format!("#link({})[{}]", string_literal(destination), escape_text(label, false));
    }
    if alt.is_empty() {
        format!("#image({})", string_literal(destination))
    } else {
        format!("#image({}, alt: {})", string_literal(destination), string_literal(alt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 서문 없이 변환한다
    fn convert(markdown: &str) -> String {
        markdown_to_typst(markdown, &ConvertOptions { preamble: false })
    }

    #[test]
    fn headings_paragraphs_and_inline_styles() {
        let typst = convert("# 제목\n\n본문 **굵게** 와 *기울임* 그리고 ~~취소~~ 입니다.\n\n## 소제목\n");
        assert_eq!(
            typst,
            "= 제목\n\n본문 #strong[굵게] 와 #emph[기울임] 그리고 #strike[취소] 입니다.\n\n== 소제목\n"
        );
    }

    #[test]
    fn special_characters_are_escaped() {
        let typst = convert("가격은 $5 #태그 *별* _밑줄_ <꺾쇠> user@example.com [대괄호] a~b `코드`\n");
        assert!(typst.contains("\\$5"));
        assert!(typst.contains("\\#태그"));
        assert!(typst.contains("\\<꺾쇠\\>"));
        assert!(typst.contains("user\\@example.com"));
        assert!(typst.contains("\\[대괄호\\]"));
        assert!(typst.contains("a\\~b"));
        // 인라인 코드는 이스케이프하지 않고 원문(raw) 그대로 둔다
        assert!(typst.contains("`코드`"));
    }

    #[test]
    fn line_start_markers_in_text_are_escaped() {
        let typst = convert("\\= 제목 아님\n\n\\- 목록 아님\n\n1\\. 번호 아님\n");
        assert!(typst.contains("\\= 제목 아님"));
        assert!(typst.contains("\\- 목록 아님"));
        assert!(typst.contains("1\\. 번호 아님"));
    }

    #[test]
    fn lists_keep_numbers_and_nesting() {
        let typst = convert("- 가\n- 나\n  - 나-1\n\n3. 셋\n4. 넷\n");
        assert!(typst.contains("- 가\n- 나\n  - 나-1\n"));
        assert!(typst.contains("3. 셋\n4. 넷\n"));
    }

    #[test]
    fn task_list_items_show_checkbox_text() {
        let typst = convert("- [x] 완료\n- [ ] 미완료\n");
        assert!(typst.contains("- \\[x\\] 완료"));
        assert!(typst.contains("- \\[ \\] 미완료"));
    }

    #[test]
    fn links_images_and_rules() {
        let typst = convert(
            "[사이트](https://example.com \"제목\") ![그림](img/a.png)\n\n![원격](https://e.com/x.png)\n\n---\n",
        );
        assert!(typst.contains("#link(\"https://example.com\")[사이트]"));
        assert!(typst.contains("#image(\"img/a.png\", alt: \"그림\")"));
        // 네트워크 이미지는 Typst 가 읽을 수 없으므로 링크로 바꾼다
        assert!(typst.contains("#link(\"https://e.com/x.png\")[원격]"));
        assert!(typst.contains("#line(length: 100%)"));
    }

    #[test]
    fn code_blocks_use_raw_fences_that_cannot_be_broken() {
        let typst = convert("```rust\nfn main() {}\n```\n\n````\n```\n안에 백틱 세 개\n```\n````\n");
        assert!(typst.contains("```rust\nfn main() {}\n```"));
        assert!(typst.contains("````\n```\n안에 백틱 세 개\n```\n````"));
    }

    #[test]
    fn tables_become_typst_tables_with_alignment() {
        let typst = convert("| 이름 | 값 |\n|:--|--:|\n| a | 1 |\n| b | 2 |\n");
        assert!(typst.contains("#table("));
        assert!(typst.contains("columns: 2"));
        assert!(typst.contains("align: (left, right,)"));
        assert!(typst.contains("table.header([#strong[이름]], [#strong[값]])"));
        assert!(typst.contains("[a], [1],"));
        assert!(typst.contains("[b], [2],"));
    }

    #[test]
    fn block_quotes_and_footnotes() {
        let typst = convert("> 인용문\n\n각주가 있는 문장[^1].\n\n[^1]: 각주 내용\n");
        assert!(typst.contains("#quote(block: true)[\n인용문\n]"));
        assert!(typst.contains("각주가 있는 문장#footnote[각주 내용]."));
    }

    #[test]
    fn html_comments_are_dropped_and_br_becomes_a_line_break() {
        let typst = convert("<!-- 주석 -->\n\n줄<br>바꿈\n");
        assert!(!typst.contains("주석"));
        assert!(typst.contains("줄 \\\n바꿈"));
    }

    #[test]
    fn preamble_sets_korean_language_only_for_hangul_documents() {
        assert!(markdown_to_typst("한글 문서", &ConvertOptions::default()).contains("lang: \"ko\""));
        assert!(markdown_to_typst("english only", &ConvertOptions::default()).contains("lang: \"en\""));
        assert!(markdown_to_typst("x", &ConvertOptions::default()).starts_with("// Don Editor"));
    }
}
