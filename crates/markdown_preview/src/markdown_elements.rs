use gpui::{FontStyle, FontWeight, HighlightStyle, SharedString, StrikethroughStyle, UnderlineStyle, px};
use language::HighlightId;
use std::{fmt::Display, ops::Range, path::PathBuf};
use urlencoding;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParsedMarkdownElement {
    Heading(ParsedMarkdownHeading),
    ListItem(ParsedMarkdownListItem),
    Table(ParsedMarkdownTable),
    BlockQuote(ParsedMarkdownBlockQuote),
    CodeBlock(ParsedMarkdownCodeBlock),
    MermaidDiagram(ParsedMarkdownMermaidDiagram),
    /// A paragraph of text and other inline elements.
    Paragraph(MarkdownParagraph),
    HorizontalRule(Range<usize>),
    Image(Image),
}

impl ParsedMarkdownElement {
    pub fn source_range(&self) -> Option<Range<usize>> {
        Some(match self {
            Self::Heading(heading) => heading.source_range.clone(),
            Self::ListItem(list_item) => list_item.source_range.clone(),
            Self::Table(table) => table.source_range.clone(),
            Self::BlockQuote(block_quote) => block_quote.source_range.clone(),
            Self::CodeBlock(code_block) => code_block.source_range.clone(),
            Self::MermaidDiagram(mermaid) => mermaid.source_range.clone(),
            Self::Paragraph(text) => match text.first()? {
                MarkdownParagraphChunk::Text(t) => t.source_range.clone(),
                MarkdownParagraphChunk::Image(image) => image.source_range.clone(),
            },
            Self::HorizontalRule(range) => range.clone(),
            Self::Image(image) => image.source_range.clone(),
        })
    }

    pub fn is_list_item(&self) -> bool {
        matches!(self, Self::ListItem(_))
    }
}

pub type MarkdownParagraph = Vec<MarkdownParagraphChunk>;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum MarkdownParagraphChunk {
    Text(ParsedMarkdownText),
    Image(Image),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdown {
    pub children: Vec<ParsedMarkdownElement>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdownListItem {
    pub source_range: Range<usize>,
    /// How many indentations deep this item is.
    pub depth: u16,
    pub item_type: ParsedMarkdownListItemType,
    pub content: Vec<ParsedMarkdownElement>,
    /// Whether we can expect nested list items inside of this items `content`.
    pub nested: bool,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParsedMarkdownListItemType {
    Ordered(u64),
    Task(bool, Range<usize>),
    Unordered,
}

// [DonEditor] 테스트용 비교는 `source_map` 을 빼고 비교하도록 markdown_parser 테스트 모듈에서 직접 구현한다.
#[derive(Debug)]
pub struct ParsedMarkdownCodeBlock {
    pub source_range: Range<usize>,
    pub language: Option<String>,
    pub contents: SharedString,
    pub highlights: Option<Vec<(Range<usize>, HighlightId)>>,
    /// [DonEditor] `contents` 의 글자 범위와 원본 문서 범위의 대응 (미리보기 검색 일치 강조용)
    pub source_map: Vec<SourceMapping>,
}

/// [DonEditor] 미리보기에 그려지는 글자 범위와, 그 글자를 만든 원본 Markdown 범위의 대응.
/// 검색은 원본 문서에서 하므로, 원본 기준 일치 범위를 미리보기 글자 위치로 옮겨 강조할 때 쓴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapping {
    /// 렌더링된 글자(`contents`) 안의 바이트 범위
    pub rendered: Range<usize>,
    /// 원본 Markdown 문서 안의 바이트 범위
    pub source: Range<usize>,
    /// 두 범위의 글자가 똑같아 바이트 단위로 1:1 대응하면 `true`.
    /// HTML 엔티티·줄 바꿈이 들어간 코드 조각처럼 글자가 달라지면 `false` 이고, 이때는 범위 전체끼리만 대응한다.
    pub exact: bool,
}

impl SourceMapping {
    /// 원본 이벤트 범위 `source_range` 가 렌더링 글자 `rendered_text`(렌더링 범위 `rendered`)가 되었을 때의 대응을 만든다.
    /// 원본 조각 안에서 렌더링 글자를 그대로 찾을 수 있으면(코드 조각의 백틱, 이스케이프의 역슬래시 등을 뺀 부분)
    /// 그 부분과 1:1 로 대응시키고, 찾을 수 없으면 범위 전체끼리 대응시킨다.
    pub fn new(source: &str, rendered: Range<usize>, rendered_text: &str, source_range: Range<usize>) -> Self {
        let original = source.get(source_range.clone()).unwrap_or_default();
        if !rendered_text.is_empty()
            && let Some(position) = original.find(rendered_text)
        {
            let start = source_range.start + position;
            return Self {
                rendered,
                source: start..start + rendered_text.len(),
                exact: true,
            };
        }
        Self {
            rendered,
            source: source_range,
            exact: false,
        }
    }

    /// 원본 범위 `matched` 중 이 대응에 걸친 부분을 렌더링 글자 범위로 바꾼다. 겹치지 않으면 `None`.
    pub fn rendered_range_for(&self, matched: &Range<usize>) -> Option<Range<usize>> {
        let start = matched.start.max(self.source.start);
        let end = matched.end.min(self.source.end);
        if start >= end {
            return None;
        }
        if !self.exact {
            return Some(self.rendered.clone());
        }
        let offset = self.rendered.start;
        Some(offset + (start - self.source.start)..offset + (end - self.source.start))
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdownMermaidDiagram {
    pub source_range: Range<usize>,
    pub contents: ParsedMarkdownMermaidDiagramContents,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParsedMarkdownMermaidDiagramContents {
    pub contents: SharedString,
    pub scale: u32,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdownHeading {
    pub source_range: Range<usize>,
    pub level: HeadingLevel,
    pub contents: MarkdownParagraph,
}

#[derive(Debug, PartialEq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Debug)]
pub struct ParsedMarkdownTable {
    pub source_range: Range<usize>,
    pub header: Vec<ParsedMarkdownTableRow>,
    pub body: Vec<ParsedMarkdownTableRow>,
    pub caption: Option<MarkdownParagraph>,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParsedMarkdownTableAlignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdownTableColumn {
    pub is_header: bool,
    pub children: MarkdownParagraph,
    pub alignment: ParsedMarkdownTableAlignment,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdownTableRow {
    pub columns: Vec<ParsedMarkdownTableColumn>,
}

impl Default for ParsedMarkdownTableRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsedMarkdownTableRow {
    pub fn new() -> Self {
        Self { columns: Vec::new() }
    }

    pub fn with_columns(columns: Vec<ParsedMarkdownTableColumn>) -> Self {
        Self { columns }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedMarkdownBlockQuote {
    pub source_range: Range<usize>,
    pub children: Vec<ParsedMarkdownElement>,
}

#[derive(Debug, Clone)]
pub struct ParsedMarkdownText {
    /// Where the text is located in the source Markdown document.
    pub source_range: Range<usize>,
    /// The text content stripped of any formatting symbols.
    pub contents: SharedString,
    /// The list of highlights contained in the Markdown document.
    pub highlights: Vec<(Range<usize>, MarkdownHighlight)>,
    /// The regions of the Markdown document.
    pub regions: Vec<(Range<usize>, ParsedRegion)>,
    /// [DonEditor] `contents` 의 글자 범위와 원본 문서 범위의 대응 (미리보기 검색 일치 강조용)
    pub source_map: Vec<SourceMapping>,
}

/// A run of highlighted Markdown text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownHighlight {
    /// A styled Markdown highlight.
    Style(MarkdownHighlightStyle),
    /// A highlighted code block.
    Code(HighlightId),
}

impl MarkdownHighlight {
    /// Converts this [`MarkdownHighlight`] to a [`HighlightStyle`].
    pub fn to_highlight_style(&self, theme: &theme::SyntaxTheme) -> Option<HighlightStyle> {
        match self {
            MarkdownHighlight::Style(style) => {
                let mut highlight = HighlightStyle::default();

                if style.italic {
                    highlight.font_style = Some(FontStyle::Italic);
                }

                if style.underline {
                    highlight.underline = Some(UnderlineStyle {
                        thickness: px(1.),
                        ..Default::default()
                    });
                }

                if style.strikethrough {
                    highlight.strikethrough = Some(StrikethroughStyle {
                        thickness: px(1.),
                        ..Default::default()
                    });
                }

                if style.weight != FontWeight::default() {
                    highlight.font_weight = Some(style.weight);
                }

                if style.link {
                    highlight.underline = Some(UnderlineStyle {
                        thickness: px(1.),
                        ..Default::default()
                    });
                }

                if style.oblique {
                    highlight.font_style = Some(FontStyle::Oblique)
                }

                Some(highlight)
            }

            MarkdownHighlight::Code(id) => id.style(theme),
        }
    }
}

/// The style for a Markdown highlight.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkdownHighlightStyle {
    /// Whether the text should be italicized.
    pub italic: bool,
    /// Whether the text should be underlined.
    pub underline: bool,
    /// Whether the text should be struck through.
    pub strikethrough: bool,
    /// The weight of the text.
    pub weight: FontWeight,
    /// Whether the text should be stylized as link.
    pub link: bool,
    // Whether the text should be obliqued.
    pub oblique: bool,
}

/// A parsed region in a Markdown document.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParsedRegion {
    /// Whether the region is a code block.
    pub code: bool,
    /// The link contained in this region, if it has one.
    pub link: Option<Link>,
}

/// A Markdown link.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Link {
    /// A link to a webpage.
    Web {
        /// The URL of the webpage.
        url: String,
    },
    /// A link to a fragment in the document
    Fragment {
        /// The fragment identifier, including the '#'
        id: String,
    },
    /// A link to a path on the filesystem.
    Path {
        /// The path as provided in the Markdown document.
        display_path: PathBuf,
        /// The absolute path to the item.
        path: PathBuf,
    },
}

impl Link {
    pub fn identify(file_location_directory: Option<PathBuf>, text: String) -> Option<Link> {
        if text.starts_with("http") {
            return Some(Link::Web { url: text });
        }
        if text.starts_with('#') {
            return Some(Link::Fragment { id: text });
        }

        // URL decode the text to handle spaces and other special characters
        let decoded_text = urlencoding::decode(&text).map(|s| s.into_owned()).unwrap_or(text);

        let path = PathBuf::from(&decoded_text);
        if path.is_absolute() && path.exists() {
            return Some(Link::Path {
                display_path: path.clone(),
                path,
            });
        }

        if let Some(file_location_directory) = file_location_directory {
            let display_path = path;
            let path = file_location_directory.join(decoded_text);
            return Some(Link::Path { display_path, path });
        }

        None
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Link::Web { url } => write!(f, "{}", url),
            Link::Fragment { id } => write!(f, "#{}", id),
            Link::Path { display_path, .. } => write!(f, "{}", display_path.display()),
        }
    }
}

/// A Markdown Image
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Image {
    pub link: Link,
    pub source_range: Range<usize>,
    pub alt_text: Option<SharedString>,
}

impl Image {
    pub fn identify(
        text: String,
        source_range: Range<usize>,
        file_location_directory: Option<PathBuf>,
    ) -> Option<Self> {
        let link = Link::identify(file_location_directory, text)?;
        Some(Self {
            source_range,
            link,
            alt_text: None,
        })
    }

    pub fn set_alt_text(&mut self, alt_text: SharedString) {
        self.alt_text = Some(alt_text);
    }
}
