//! [DonEditor] 요구사항 R2(문법 강조 26개 언어) 검증 테스트.
//!
//! `tests/samples/` 의 샘플 파일을 각 언어로 파싱하여 다음을 확인한다.
//! 1. 언어 설정(config.toml)과 쿼리(*.scm)가 로드·컴파일된다.
//! 2. 샘플에 파싱 오류가 없다.
//! 3. 대표 토큰이 기대한 테마 키로 강조된다 (실제 편집기와 같은 BufferChunks 경로를 사용한다).
//! 4. 샘플 파일 이름(확장자)으로 기대한 언어가 감지된다.
//!
//! 강조 결과를 사람이 눈으로 확인하려면 환경 변수 `DON_DUMP_HIGHLIGHTS=1` 을 주고
//! `cargo test -p languages --features load-grammars -- --nocapture language_support` 로 실행한다.

use crate::{Language, LanguageDir, LanguageRegistry, Rope, language, load_config, register_language};
use gpui::{Hsla, TestAppContext};
use std::{collections::BTreeMap, ops::Range, path::Path, sync::Arc};
use theme::SyntaxTheme;

/// Gram 에 번들된 테마들이 쓰는 syntax 키의 합집합.
/// 캡처 이름이 이 키들 중 어느 것과 매칭되는지(= 실제로 색이 입혀지는지)를 시험하기 위한 가짜 테마를 만든다.
const THEME_KEYS: &[&str] = &[
    "@function.special.definition",
    "attribute",
    "boolean",
    "character",
    "character.special",
    "comment",
    "comment.doc",
    "comment.documentation",
    "comment.error",
    "comment.hint",
    "comment.note",
    "comment.todo",
    "comment.unused",
    "comment.warning",
    "concept",
    "constant",
    "constant.builtin",
    "constant.macro",
    "constructor",
    "declaration",
    "diff.minus",
    "diff.plus",
    "embedded",
    "emphasis",
    "emphasis.strong",
    "enum",
    "field",
    "float",
    "function",
    "function.builtin",
    "function.call",
    "function.decorator",
    "function.definition",
    "function.macro",
    "function.method",
    "function.method.call",
    "function.special.definition",
    "hint",
    "keyword",
    "keyword.conditional",
    "keyword.conditional.ternary",
    "keyword.coroutine",
    "keyword.debug",
    "keyword.directive",
    "keyword.directive.define",
    "keyword.exception",
    "keyword.export",
    "keyword.function",
    "keyword.import",
    "keyword.modifier",
    "keyword.operator",
    "keyword.preproc",
    "keyword.repeat",
    "keyword.return",
    "keyword.type",
    "label",
    "link_text",
    "link_uri",
    "module",
    "namespace",
    "number",
    "number.float",
    "operator",
    "parameter",
    "parent",
    "predictive",
    "preproc",
    "primary",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.list_marker",
    "punctuation.markup",
    "punctuation.special",
    "punctuation.special.symbol",
    "selector",
    "selector.pseudo",
    "string",
    "string.doc",
    "string.documentation",
    "string.escape",
    "string.regex",
    "string.regexp",
    "string.special",
    "string.special.path",
    "string.special.symbol",
    "string.special.url",
    "symbol",
    "tag",
    "tag.attribute",
    "tag.delimiter",
    "tag.doctype",
    "text",
    "text.literal",
    "title",
    "type",
    "type.builtin",
    "type.class",
    "type.class.definition",
    "type.definition",
    "type.enum",
    "type.enum.definition",
    "type.enum.member",
    "type.event",
    "type.interface",
    "type.interface.definition",
    "type.parameter",
    "type.parameter.definition",
    "type.struct",
    "type.struct.definition",
    "type.super",
    "type.unit",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
    "variable.special",
    "variant",
];

/// 문법 한계 때문에 샘플에서 파싱 오류가 나는 것을 알고 있는 파일.
/// sample.xhtml: HTML 문법(tree-sitter-html)이 XML 선언 `<?xml ... ?>` 을 처리하지 못해 그 줄만 오류 노드가 된다.
/// 나머지(태그, 속성, DOCTYPE)는 정상 강조된다. XML 문법으로 바꿀지는 사용자 결정 사항이다 (PROGRESS.md 참고).
const KNOWN_PARSE_ERRORS: &[&str] = &["sample.xhtml"];

/// 샘플 파일 내용을 읽어 오는 매크로 (tests/samples/<파일>)
macro_rules! sample {
    ($file:literal) => {
        include_str!(concat!("../../../tests/samples/", $file))
    };
}

/// 검증 대상 하나: 언어 설정 디렉터리, 문법, 샘플 파일과 기대 강조
struct Case {
    /// 언어 설정 디렉터리 이름 (languages/src/<dir>)
    dir: &'static str,
    /// 감지되어야 하는 언어 이름 (config.toml 의 name)
    language_name: &'static str,
    grammar: tree_sitter::Language,
    /// 샘플 파일 이름 (확장자 감지 검증에도 쓴다)
    file: &'static str,
    text: &'static str,
    /// (문맥 문자열, 검사할 토큰, 기대하는 테마 키). 문맥 문자열의 첫 위치에서 토큰의 첫 글자를 검사한다.
    expect: &'static [(&'static str, &'static str, &'static str)],
}

/// 26개 언어의 검증 대상 목록. javascript(mjs, cjs)와 yaml(yml, yaml)처럼 확장자가 여럿이면 파일마다 한 항목을 둔다.
fn cases() -> Vec<Case> {
    vec![
        // ---------- 원본에 내장된 언어 ----------
        Case {
            dir: "c",
            language_name: "C",
            grammar: tree_sitter_c::LANGUAGE.into(),
            file: "sample.c",
            text: sample!("sample.c"),
            expect: &[
                ("#include <stdio.h>", "#include", "preproc"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("typedef struct", "typedef", "keyword"),
                ("int add(int a", "add", "function"),
                ("0xFFUL", "0xFFUL", "number"),
                ("\"안녕, C\"", "\"안녕, C\"", "string"),
                ("MAX_SIZE 16", "MAX_SIZE", "constant"),
                ("Point p", "Point", "type"),
                ("\\n\"", "\\n", "string.escape"),
            ],
        },
        Case {
            dir: "cpp",
            language_name: "C++",
            grammar: tree_sitter_cpp::LANGUAGE.into(),
            file: "sample.cpp",
            text: sample!("sample.cpp"),
            expect: &[
                ("#include <iostream>", "#include", "preproc"),
                ("namespace demo {", "namespace", "keyword"),
                ("class Stack", "class", "keyword"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("demo::Stack<std::string>", "demo", "namespace"),
                ("\"안녕\"", "\"안녕\"", "string"),
                ("void push", "void", "type.builtin"),
                ("T pop()", "pop", "function"),
                ("while (!stack", "while", "keyword"),
            ],
        },
        Case {
            dir: "sql",
            language_name: "SQL",
            grammar: tree_sitter_sql::LANGUAGE.into(),
            file: "sample.sql",
            text: sample!("sample.sql"),
            expect: &[
                ("-- 한 줄 주석", "-- 한 줄 주석", "comment"),
                ("CREATE TABLE", "CREATE", "keyword"),
                ("'홍길동'", "'홍길동'", "string"),
                ("INTEGER PRIMARY", "INTEGER", "type.builtin"),
                ("COUNT(o.id)", "COUNT", "function.call"),
                ("LIMIT 10", "10", "number"),
                ("NOT NULL", "NOT", "keyword.operator"),
            ],
        },
        Case {
            dir: "rust",
            language_name: "Rust",
            grammar: tree_sitter_rust::LANGUAGE.into(),
            file: "sample.rs",
            text: sample!("sample.rs"),
            expect: &[
                ("use std::collections", "use", "keyword"),
                ("/// 문서 주석", "/// 문서 주석", "comment.doc"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("derive(Debug", "derive", "attribute"),
                ("struct Point<T>", "struct", "keyword"),
                ("fn sum", "sum", "function.definition"),
                ("Circle(f64)", "f64", "type.builtin"),
                ("\"키\"", "\"키\"", "string"),
                ("42);", "42", "number"),
                ("&self", "self", "variable.special"),
                ("println!(\"합계", "println!", "function"),
            ],
        },
        Case {
            dir: "javascript",
            language_name: "JavaScript",
            grammar: tree_sitter_typescript::LANGUAGE_TSX.into(),
            file: "sample.js",
            text: sample!("sample.js"),
            expect: &[
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("import fs", "import", "keyword.import"),
                ("class Counter", "class", "keyword"),
                ("/** JSDoc 주석 */", "/** JSDoc 주석 */", "comment"),
                ("async (name)", "async", "keyword"),
                ("`안녕하세요", "`안녕하세요", "string"),
                ("0xff", "0xff", "number"),
                ("true)", "true", "boolean"),
                ("/ab+c/gi", "/ab+c/", "string.regex"),
                ("increment(step", "increment", "function.method"),
                ("#count = 0", "#count", "property"),
                ("this.#count", "this", "variable.special"),
            ],
        },
        Case {
            dir: "javascript",
            language_name: "JavaScript",
            grammar: tree_sitter_typescript::LANGUAGE_TSX.into(),
            file: "sample.mjs",
            text: sample!("sample.mjs"),
            expect: &[
                ("// ES 모듈", "// ES 모듈", "comment"),
                ("import {", "import", "keyword.import"),
                ("export const load", "export", "keyword.import"),
                ("\"utf8\"", "\"utf8\"", "string"),
                ("\\n\"", "\\n", "string.escape"),
            ],
        },
        Case {
            dir: "javascript",
            language_name: "JavaScript",
            grammar: tree_sitter_typescript::LANGUAGE_TSX.into(),
            file: "sample.cjs",
            text: sample!("sample.cjs"),
            expect: &[
                ("// CommonJS", "// CommonJS", "comment"),
                ("require(", "require", "function"),
                ("function join", "function", "keyword"),
                ("module.exports =", "exports", "property"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
            ],
        },
        Case {
            dir: "typescript",
            language_name: "TypeScript",
            grammar: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            file: "sample.ts",
            text: sample!("sample.ts"),
            expect: &[
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("interface User", "interface", "keyword"),
                ("enum Level", "enum", "keyword"),
                ("class Repository", "class", "keyword"),
                ("export class", "export", "keyword.import"),
                ("\"홍길동\"", "\"홍길동\"", "string"),
                ("id: number;", "number", "type.builtin"),
                ("type Result<T>", "Result", "type"),
                ("this.items.set", "this", "variable.special"),
            ],
        },
        Case {
            dir: "python",
            language_name: "Python",
            grammar: tree_sitter_python::LANGUAGE.into(),
            file: "sample.py",
            text: sample!("sample.py"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("import os", "import", "keyword"),
                ("@dataclass", "@dataclass", "function.decorator"),
                ("class Point", "class", "keyword"),
                ("\"\"\"독스트링 주석\"\"\"", "\"\"\"독스트링 주석\"\"\"", "string.doc"),
                ("def distance", "distance", "function.definition"),
                ("x: float", "float", "type.builtin"),
                ("is not None", "None", "constant.builtin"),
                ("= 0.0", "0.0", "number"),
                ("self.x", "self", "variable.special"),
                ("len(argv)", "len", "function.builtin"),
            ],
        },
        Case {
            dir: "json",
            language_name: "JSON",
            grammar: tree_sitter_json::LANGUAGE.into(),
            file: "sample.json",
            text: sample!("sample.json"),
            expect: &[
                ("\"name\"", "\"name\"", "property"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
                ("true", "true", "boolean"),
                ("null", "null", "constant.builtin"),
                ("3.14", "3.14", "number"),
                ("{\n  \"name\"", "{", "punctuation.bracket"),
            ],
        },
        Case {
            dir: "yaml",
            language_name: "YAML",
            grammar: tree_sitter_yaml::LANGUAGE.into(),
            file: "sample.yml",
            text: sample!("sample.yml"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("name: Don", "name", "property"),
                ("Don Editor", "Don Editor", "string"),
                ("true", "true", "boolean"),
                ("null", "null", "constant.builtin"),
                ("3.14", "3.14", "number"),
                ("&base", "&", "punctuation.special"),
                ("\"한글\"", "\"한글\"", "string"),
            ],
        },
        Case {
            dir: "yaml",
            language_name: "YAML",
            grammar: tree_sitter_yaml::LANGUAGE.into(),
            file: "sample.yaml",
            text: sample!("sample.yaml"),
            expect: &[
                ("# .yaml 확장자 샘플", "# .yaml 확장자 샘플", "comment"),
                ("services:", "services", "property"),
                ("\"nginx:latest\"", "\"nginx:latest\"", "string"),
            ],
        },
        Case {
            dir: "xml",
            language_name: "XML",
            grammar: tree_sitter_xml::LANGUAGE_XML.into(),
            file: "sample.xml",
            text: sample!("sample.xml"),
            expect: &[
                ("<!-- 주석 -->", "<!-- 주석 -->", "comment"),
                ("<project", "project", "tag"),
                ("attr=", "attr", "property"),
                ("&amp;", "&amp;", "constant.builtin"),
                ("<?xml", "xml", "keyword"),
                ("version=\"1.0\"", "version", "property"),
            ],
        },
        Case {
            dir: "html",
            language_name: "HTML",
            grammar: tree_sitter_html::LANGUAGE.into(),
            file: "sample.html",
            text: sample!("sample.html"),
            expect: &[
                ("<!-- 주석 -->", "<!-- 주석 -->", "comment"),
                ("<h1", "h1", "tag"),
                ("lang=\"ko\"", "lang", "attribute"),
                ("\"utf-8\"", "\"utf-8\"", "string"),
                ("DOCTYPE html", "DOCTYPE html", "tag.doctype"),
            ],
        },
        Case {
            dir: "html",
            language_name: "HTML",
            grammar: tree_sitter_html::LANGUAGE.into(),
            file: "sample.xhtml",
            text: sample!("sample.xhtml"),
            expect: &[
                ("<title>", "title", "tag"),
                ("xmlns=", "xmlns", "attribute"),
                ("class=\"intro\"", "class", "attribute"),
                ("DOCTYPE html PUBLIC", "DOCTYPE", "tag.doctype"),
            ],
        },
        Case {
            dir: "markdown",
            language_name: "Markdown",
            grammar: tree_sitter_md::LANGUAGE.into(),
            file: "sample.md",
            text: sample!("sample.md"),
            expect: &[
                ("# 제목", "# 제목", "title"),
                ("- 목록 1", "-", "punctuation.markup"),
                ("> 인용문", ">", "punctuation.markup"),
            ],
        },
        Case {
            dir: "bash",
            language_name: "Shell Script",
            grammar: tree_sitter_bash::LANGUAGE.into(),
            file: "sample.sh",
            text: sample!("sample.sh"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("#!/usr/bin/env bash", "#!", "keyword.directive"),
                ("greet() {", "greet", "function"),
                ("for i in", "for", "keyword"),
                ("then", "then", "keyword"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
                ("NAME=", "NAME", "variable"),
                ("COUNT=3", "3", "number"),
                ("[[ -n", "[[", "punctuation.bracket"),
            ],
        },
        Case {
            dir: "go",
            language_name: "Go",
            grammar: tree_sitter_go::LANGUAGE.into(),
            file: "sample.go",
            text: sample!("sample.go"),
            expect: &[
                ("// Shape 는", "// Shape 는", "comment"),
                ("package main", "package", "keyword"),
                ("type Shape interface", "interface", "keyword"),
                ("func main", "main", "function"),
                ("Radius float64", "float64", "type.builtin"),
                ("3.14159", "3.14159", "number"),
                ("\"fmt\"", "\"fmt\"", "string"),
                ("Radius float64", "Radius", "property"),
                ("chan int", "chan", "keyword"),
                ("make(chan", "make", "function.builtin"),
                ("type Circle", "Circle", "type.definition"),
            ],
        },
        // ---------- 새로 추가한 언어 11개 ----------
        Case {
            dir: "java",
            language_name: "Java",
            grammar: tree_sitter_java::LANGUAGE.into(),
            file: "Sample.java",
            text: sample!("Sample.java"),
            expect: &[
                ("package com", "package", "keyword"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("@Override", "Override", "attribute"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
                ("3.14f", "3.14f", "number"),
                ("int i = 0", "int", "type.builtin"),
                ("MAX_COUNT = 10", "MAX_COUNT", "constant"),
                ("void run()", "run", "function.method"),
            ],
        },
        Case {
            dir: "csharp",
            language_name: "C#",
            grammar: tree_sitter_c_sharp::LANGUAGE.into(),
            file: "Sample.cs",
            text: sample!("Sample.cs"),
            expect: &[
                ("using System;", "using", "keyword"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("/// <summary>", "/// <summary>", "comment"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
                ("int count", "int", "type.builtin"),
                ("Task<int> RunAsync", "RunAsync", "function"),
                ("namespace Demo", "Demo", "module"),
                ("3.14", "3.14", "number"),
            ],
        },
        Case {
            dir: "ruby",
            language_name: "Ruby",
            grammar: tree_sitter_ruby::LANGUAGE.into(),
            file: "sample.rb",
            text: sample!("sample.rb"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("module Greeting", "module", "keyword"),
                ("class Person", "class", "keyword"),
                ("def initialize", "def", "keyword"),
                ("@name = name", "@name", "property"),
                ("\"홍길동\"", "\"홍길동\"", "string"),
                ("age = 20", "20", "number"),
                ("!= nil", "nil", "constant.builtin"),
                (":value", ":value", "string.special.symbol"),
            ],
        },
        Case {
            dir: "json5",
            language_name: "JSON5",
            grammar: tree_sitter_json5::LANGUAGE.into(),
            file: "sample.json5",
            text: sample!("sample.json5"),
            expect: &[
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("/* 블록 주석 */", "/* 블록 주석 */", "comment"),
                ("name: 'Don", "name", "property"),
                ("'Don Editor'", "'Don Editor'", "string"),
                ("+3.14", "+3.14", "number"),
                ("0xFF", "0xFF", "number"),
                ("enabled: true", "true", "boolean"),
                ("null,", "null", "constant.builtin"),
            ],
        },
        Case {
            dir: "properties",
            language_name: "Properties",
            grammar: tree_sitter_properties::LANGUAGE.into(),
            file: "sample.properties",
            text: sample!("sample.properties"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("! 느낌표 주석도 가능", "! 느낌표 주석도 가능", "comment"),
                ("app.name=", "app", "property"),
                ("name=Don Editor", "=", "operator"),
                ("Don Editor", "Don Editor", "string"),
                ("\\uD55C\\uAE00", "\\uD55C", "string.escape"),
            ],
        },
        Case {
            dir: "swift",
            language_name: "Swift",
            grammar: tree_sitter_swift::LANGUAGE.into(),
            file: "sample.swift",
            text: sample!("sample.swift"),
            expect: &[
                ("import Foundation", "import", "keyword.import"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("protocol Shape", "protocol", "keyword"),
                ("func area() -> Double {\n", "func", "keyword.function"),
                ("struct Circle", "struct", "keyword.type"),
                ("if shape.area()", "if", "keyword.conditional"),
                ("for shape in", "for", "keyword.repeat"),
                ("return Double.pi", "return", "keyword.return"),
                ("private var", "private", "keyword.modifier"),
                ("radius: 2.0", "2.0", "number.float"),
            ],
        },
        Case {
            dir: "scala",
            language_name: "Scala",
            grammar: tree_sitter_scala::LANGUAGE.into(),
            file: "sample.scala",
            text: sample!("sample.scala"),
            expect: &[
                ("package demo", "package", "keyword"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("trait Shape", "trait", "keyword"),
                ("def area: Double\n", "def", "keyword.function"),
                ("import scala.collection", "import", "keyword.import"),
                ("for (shape", "for", "keyword.repeat"),
                ("if shape.area > 10", "if", "keyword.conditional"),
                ("List(Circle(2.0)", "2.0", "float"),
                ("\"키\"", "\"키\"", "string"),
                ("val shapes: List[Shape]", "Shape", "type"),
            ],
        },
        Case {
            dir: "r",
            language_name: "R",
            grammar: tree_sitter_r::LANGUAGE.into(),
            file: "sample.r",
            text: sample!("sample.r"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("add <- function", "add", "function"),
                ("function(a", "function", "keyword.function"),
                ("if (length", "if", "keyword.conditional"),
                ("for (i in", "for", "keyword.repeat"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
                ("2.5", "2.5", "number"),
                ("add <- function", "<-", "operator"),
                ("\\n", "\\n", "string.escape"),
            ],
        },
        Case {
            dir: "bat",
            language_name: "Batch",
            grammar: tree_sitter_batch::LANGUAGE.into(),
            file: "sample.bat",
            text: sample!("sample.bat"),
            expect: &[
                ("rem 한 줄 주석", "rem 한 줄 주석", "comment"),
                (":: 다른 형태의 주석", ":: 다른 형태의 주석", "comment"),
                (":loop", ":loop", "label"),
                ("echo 반복", "echo", "function"),
                ("goto done", "goto", "keyword"),
                ("%NAME%", "%NAME%", "variable"),
            ],
        },
        Case {
            dir: "powershell",
            language_name: "PowerShell",
            grammar: tree_sitter_powershell::LANGUAGE.into(),
            file: "sample.ps1",
            text: sample!("sample.ps1"),
            expect: &[
                ("# 한 줄 주석", "# 한 줄 주석", "comment"),
                ("<# 블록 주석 #>", "<# 블록 주석 #>", "comment"),
                ("function Get-Greeting", "function", "keyword"),
                ("Get-Greeting {", "Get-Greeting", "function"),
                ("\"Don Editor\"", "\"Don Editor\"", "string"),
                ("$items = ", "$items", "variable"),
                ("-gt 1", "-gt", "operator"),
                ("3.5", "3.5", "number"),
                ("foreach (", "foreach", "keyword"),
                ("[string]$Name", "string", "type"),
            ],
        },
        Case {
            dir: "dart",
            language_name: "Dart",
            grammar: tree_sitter_dart::LANGUAGE.into(),
            file: "sample.dart",
            text: sample!("sample.dart"),
            expect: &[
                ("import 'dart:async'", "import", "keyword"),
                ("// 한 줄 주석", "// 한 줄 주석", "comment"),
                ("abstract class", "abstract", "keyword"),
                ("@override", "@override", "attribute"),
                ("3.14159", "3.14159", "number.float"),
                ("'dart:async'", "'dart:async'", "string"),
                ("double area", "double", "type.builtin"),
                ("&& shape", "&&", "operator"),
            ],
        },
    ]
}

/// 모든 테마 키가 있는 가짜 테마
fn test_theme() -> SyntaxTheme {
    SyntaxTheme::new_test(THEME_KEYS.iter().map(|key| (*key, Hsla::default())))
}

/// 샘플 전체의 강조 결과를 (구간, 테마 키) 목록으로 얻는다.
fn highlights_of(language: &Arc<Language>, theme: &SyntaxTheme, text: &str) -> Vec<(Range<usize>, String)> {
    language.set_theme(theme);
    let rope = Rope::from(text);
    language
        .highlight_text(&rope, 0..text.len())
        .into_iter()
        .filter_map(|(range, id)| id.name(theme).map(|name| (range, name.to_string())))
        .collect()
}

/// 주어진 위치를 덮는 강조의 테마 키를 돌려준다
fn key_at(highlights: &[(Range<usize>, String)], offset: usize) -> Option<&str> {
    highlights
        .iter()
        .find(|(range, _)| range.contains(&offset))
        .map(|(_, key)| key.as_str())
}

/// 강조 결과를 테마 키별 토큰 목록으로 요약해 출력한다 (눈으로 확인용)
fn dump(case: &Case, text: &str, highlights: &[(Range<usize>, String)]) {
    let mut by_key: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    // 같은 키가 이어지는 구간은 하나로 합친다
    let mut merged: Vec<(Range<usize>, &str)> = Vec::new();
    for (range, key) in highlights {
        match merged.last_mut() {
            Some((last, last_key)) if last.end == range.start && *last_key == key.as_str() => last.end = range.end,
            _ => merged.push((range.clone(), key.as_str())),
        }
    }
    for (range, key) in merged {
        let token: String = text[range].trim().chars().take(18).collect();
        let entry = by_key.entry(key).or_default();
        if !token.is_empty() && !entry.contains(&token) && entry.len() < 14 {
            entry.push(token);
        }
    }
    println!("==== {} ({})", case.language_name, case.file);
    for (key, tokens) in by_key {
        println!("    {key:<26} {}", tokens.join(" | "));
    }
}

/// 26개 언어 샘플: 설정·쿼리 로드, 파싱 오류, 대표 토큰의 강조 키를 검사한다
#[test]
fn language_support_highlights_samples() {
    let theme = test_theme();
    let dump_enabled = std::env::var("DON_DUMP_HIGHLIGHTS").is_ok();
    // 실패를 모아서 한 번에 보고한다 (하나가 실패해도 나머지 언어를 계속 검사한다)
    let mut failures: Vec<String> = Vec::new();

    for case in cases() {
        let label = format!("{} ({})", case.language_name, case.file);

        // 1) 파싱 오류가 없는지 확인한다
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&case.grammar)
            .unwrap_or_else(|e| panic!("{label}: 문법을 불러오지 못함: {e}"));
        let tree = parser.parse(case.text, None).expect("파싱 실패");
        if tree.root_node().has_error() && !KNOWN_PARSE_ERRORS.contains(&case.file) {
            failures.push(format!(
                "{label}: 샘플에 파싱 오류가 있음 ({})",
                tree.root_node().to_sexp().chars().take(120).collect::<String>()
            ));
        }

        // 2) 언어 설정과 쿼리를 불러온다 (쿼리 컴파일 오류가 있으면 여기서 패닉한다)
        let language = language(case.dir, case.grammar.clone());
        let highlights = highlights_of(&language, &theme, case.text);
        if highlights.is_empty() {
            failures.push(format!("{label}: 강조 결과가 비어 있음"));
        }
        if dump_enabled {
            dump(&case, case.text, &highlights);
        }

        // 3) 대표 토큰이 기대한 테마 키로 강조되는지 확인한다
        for (context, token, expected_key) in case.expect {
            let Some(context_start) = case.text.find(context) else {
                failures.push(format!("{label}: 샘플에서 문맥 문자열을 찾지 못함: {context:?}"));
                continue;
            };
            let token_start = context_start
                + context
                    .find(token)
                    .unwrap_or_else(|| panic!("{label}: {context:?} 안에 {token:?} 없음"));
            let actual = key_at(&highlights, token_start);
            if actual != Some(expected_key) {
                failures.push(format!(
                    "{label}: {token:?} 의 강조 키가 {actual:?} 임 (기대: {expected_key:?})"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "문법 강조 검증 실패 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// 샘플 파일 이름(확장자)으로 기대한 언어가 감지되는지 확인한다.
/// 등록 가능한 모든 언어 설정을 레지스트리에 올린 상태에서 검사하므로 확장자 충돌도 함께 드러난다.
#[gpui::test]
async fn language_support_detects_languages_by_extension(cx: &mut TestAppContext) {
    let registry = Arc::new(LanguageRegistry::test(cx.executor()));
    // 설정 파일(config.toml)이 있는 모든 언어 디렉터리를 올린다
    let mut dirs: Vec<String> = LanguageDir::iter()
        .filter_map(|path| path.strip_suffix("/config.toml").map(|dir| dir.to_string()))
        .collect();
    dirs.sort();
    for dir in &dirs {
        // register_language 는 이름이 'static 이어야 하므로 누출시켜 쓴다 (테스트 한정)
        let name: &'static str = Box::leak(dir.clone().into_boxed_str());
        register_language(&registry, name, Vec::new(), None, None, None);
    }

    let mut failures: Vec<String> = Vec::new();
    for case in cases() {
        match registry.language_for_file_path(Path::new(case.file)) {
            Some(available) if available.name().0.as_ref() == case.language_name => {}
            Some(available) => failures.push(format!(
                "{}: {} 로 감지됨 (기대: {})",
                case.file,
                available.name().0,
                case.language_name
            )),
            None => failures.push(format!(
                "{}: 감지된 언어가 없음 (기대: {})",
                case.file, case.language_name
            )),
        }
    }
    // 추가로, 확장자 없이 파일 이름만으로 감지되는 Ruby 파일과 대소문자 확장자도 확인한다
    for (file, expected) in [
        ("Gemfile", "Ruby"),
        ("Rakefile", "Ruby"),
        ("app.gemspec", "Ruby"),
        ("run.cmd", "Batch"),
        ("module.psm1", "PowerShell"),
        ("manifest.psd1", "PowerShell"),
        ("tool.pyw", "Python"),
        ("Program.csx", "C#"),
        ("build.sbt", "Scala"),
        ("script.sc", "Scala"),
        ("analysis.R", "R"),
        ("schema.xsd", "XML"),
        ("config.yaml", "YAML"),
        ("README.md", "Markdown"),
    ] {
        match registry.language_for_file_path(Path::new(file)) {
            Some(available) if available.name().0.as_ref() == expected => {}
            other => failures.push(format!(
                "{file}: {:?} 로 감지됨 (기대: {expected})",
                other.map(|a| a.name().0.to_string())
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "확장자 감지 검증 실패 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// 모든 언어 설정이 읽히고 name 이 비어 있지 않은지 확인한다 (설정 오타를 빨리 발견하기 위함)
#[test]
fn language_support_configs_load() {
    for path in LanguageDir::iter() {
        if let Some(dir) = path.strip_suffix("/config.toml") {
            let config = load_config(dir);
            assert!(!config.name.as_ref().is_empty(), "{dir}: name 이 비어 있음");
        }
    }
}
