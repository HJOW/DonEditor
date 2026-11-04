//! [DonEditor] MCP와 Agent가 공유하는 도구 명세와 순수 텍스트 연산.
use anyhow::{Result, bail, ensure};
use serde_json::{Value, json};

/// 도구의 입력 필드와 필수 필드를 JSON Schema로 만든다.
fn tool(name: &str, description: &str, properties: Value, required: &[&str], read_only: bool) -> Value {
    json!({"name":name,"description":description,"inputSchema":{
        "type":"object","properties":properties,"required":required,"additionalProperties":false
    },"annotations":{"readOnlyHint":read_only,"destructiveHint":!read_only}})
}

/// 외부 클라이언트와 모델에 제공할 전체 도구 목록을 돌려준다.
pub fn definitions() -> Vec<Value> {
    let string = json!({"type":"string"});
    let offset = json!({"type":"integer","minimum":0,"description":"UTF-8 byte offset"});
    vec![
        tool(
            "list_actions",
            "List every registered editor action; use dispatch_action to control any feature.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "dispatch_action",
            "Run a registered action in the editor window after approval. Completion means dispatched, not finished.",
            json!({"name":string,"args":{"type":"object"}}),
            &["name"],
            false,
        ),
        tool(
            "open_file",
            "Open an absolute file path in the current workspace.",
            json!({"path":string}),
            &["path"],
            true,
        ),
        tool(
            "open_folder",
            "Add an absolute folder path to the current workspace.",
            json!({"path":string}),
            &["path"],
            true,
        ),
        tool(
            "save",
            "Save the active editor, waiting for completion.",
            json!({}),
            &[],
            false,
        ),
        tool(
            "save_all",
            "Save all modified files, waiting for completion.",
            json!({}),
            &[],
            false,
        ),
        tool(
            "close_tab",
            "Close the active tab using the editor's unsaved-changes confirmation.",
            json!({}),
            &[],
            false,
        ),
        tool(
            "list_open_tabs",
            "List open editor tabs and the current editor.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "list_agent_conversations",
            "List saved Agent conversation titles and indexes for SelectConversation.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "list_project_files",
            "List visible project files (up to limit, default 10000).",
            json!({"limit":{"type":"integer","minimum":1,"maximum":100000}}),
            &[],
            true,
        ),
        tool(
            "get_buffer_text",
            "Read active editor text and selections.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "set_buffer_text",
            "Replace active editor text after showing an editable-file diff for approval. Does not save.",
            json!({"text":string}),
            &["text"],
            false,
        ),
        tool(
            "insert_text",
            "Insert text at a UTF-8 byte offset after diff approval. Does not save.",
            json!({"offset":offset,"text":string}),
            &["offset", "text"],
            false,
        ),
        tool(
            "replace_range",
            "Replace a UTF-8 byte range after diff approval. Does not save.",
            json!({"start":offset,"end":offset,"text":string}),
            &["start", "end", "text"],
            false,
        ),
        tool(
            "get_selection",
            "Read active editor selections as UTF-8 byte offsets.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "set_selection",
            "Select a UTF-8 byte range.",
            json!({"start":offset,"end":offset}),
            &["start", "end"],
            true,
        ),
        tool(
            "undo",
            "Undo in the active editor after approval.",
            json!({}),
            &[],
            false,
        ),
        tool(
            "redo",
            "Redo in the active editor after approval.",
            json!({}),
            &[],
            false,
        ),
        tool(
            "format_buffer",
            "Dispatch the active editor's format action after approval.",
            json!({}),
            &[],
            false,
        ),
        tool(
            "find_in_buffer",
            "Find literal, case-sensitive text in the active buffer. Returns UTF-8 byte ranges.",
            json!({"query":string}),
            &["query"],
            true,
        ),
        tool(
            "project_search",
            "Search visible non-ignored project text files for literal text. Limits: 10000 files, 2 MiB/file, 1000 matches.",
            json!({"query":string}),
            &["query"],
            true,
        ),
        tool(
            "replace_in_project",
            "Preview and approve literal replacements in project files; changes remain unsaved in editor tabs.",
            json!({"query":string,"replacement":string}),
            &["query", "replacement"],
            false,
        ),
        tool(
            "go_to_line",
            "Move to a 1-based line and UTF-8 byte column.",
            json!({"line":{"type":"integer","minimum":1},"column":{"type":"integer","minimum":1}}),
            &["line"],
            true,
        ),
        tool(
            "get_diagnostics",
            "Read active editor diagnostics.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "get_outline",
            "Read active editor document symbols.",
            json!({}),
            &[],
            true,
        ),
        tool(
            "toggle_panel",
            "Toggle a panel by its registered action name (e.g. agent_panel::ToggleFocus).",
            json!({"action":string}),
            &["action"],
            false,
        ),
        tool(
            "split_pane",
            "Split the active pane (up/down/left/right).",
            json!({"direction":{"type":"string","enum":["up","down","left","right"]}}),
            &["direction"],
            false,
        ),
        tool(
            "set_theme",
            "Persist a theme name after approval.",
            json!({"name":string}),
            &["name"],
            false,
        ),
        tool(
            "set_ui_language",
            "Persist UI language (auto/ko/en).",
            json!({"language":{"type":"string","enum":["auto","ko","en"]}}),
            &["language"],
            false,
        ),
        tool(
            "set_buffer_language",
            "Set the active buffer language by name.",
            json!({"language":string}),
            &["language"],
            false,
        ),
        tool(
            "get_settings",
            "Read effective global settings (may contain private user configuration).",
            json!({}),
            &[],
            true,
        ),
        tool(
            "update_settings",
            "Merge a JSON object into user settings after approval.",
            json!({"settings":{"type":"object"}}),
            &["settings"],
            false,
        ),
        tool(
            "run_in_terminal",
            "Run a shell command as a terminal task after approval.",
            json!({"command":string}),
            &["command"],
            false,
        ),
    ]
}

/// 필수 값, 타입, 알 수 없는 필드를 명세대로 검사한다.
pub fn validate(name: &str, args: &Value) -> Result<()> {
    let definition = definitions()
        .into_iter()
        .find(|tool| tool["name"] == name)
        .ok_or_else(|| anyhow::anyhow!("알 수 없는 도구: {name}"))?;
    let schema = &definition["inputSchema"];
    let values = args
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("도구 인자는 객체여야 합니다"))?;
    // 필수 인자의 누락을 검사한다.
    for key in schema["required"].as_array().unwrap() {
        ensure!(values.contains_key(key.as_str().unwrap()), "필수 인자 누락: {key}");
    }
    // 각 필드의 타입과 범위가 명세와 일치해야 한다.
    for (key, value) in values {
        let field = &schema["properties"][key];
        let valid = match field["type"].as_str() {
            Some("string") => value.is_string(),
            Some("integer") => value.as_u64().is_some(),
            Some("object") => value.is_object(),
            _ => false,
        };
        ensure!(valid, "지원하지 않는 인자 또는 잘못된 타입: {key}");
        if let Some(options) = field["enum"].as_array() {
            ensure!(options.contains(value), "허용되지 않는 값: {key}");
        }
        if let Some(minimum) = field["minimum"].as_u64() {
            ensure!(value.as_u64().unwrap() >= minimum, "최솟값 미달: {key}");
        }
        if let Some(maximum) = field["maximum"].as_u64() {
            ensure!(value.as_u64().unwrap() <= maximum, "최댓값 초과: {key}");
        }
    }
    Ok(())
}

/// 잘못된 UTF-8 경계나 역방향 범위를 거절하고 텍스트를 치환한다.
pub fn replace_text(original: &str, start: usize, end: usize, replacement: &str) -> Result<String> {
    ensure!(start <= end && end <= original.len(), "편집 범위를 벗어났습니다");
    ensure!(
        original.is_char_boundary(start) && original.is_char_boundary(end),
        "UTF-8 문자 경계가 아닙니다"
    );
    let mut text = original.to_owned();
    text.replace_range(start..end, replacement);
    Ok(text)
}

/// 빈 검색어는 거절하며 UTF-8 바이트 범위를 반환한다.
pub fn find_text(text: &str, query: &str) -> Result<Vec<(usize, usize)>> {
    if query.is_empty() {
        bail!("검색어가 비어 있습니다");
    }
    Ok(text
        .match_indices(query)
        .take(1000)
        .map(|(start, matched)| (start, start + matched.len()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 한글/이모지 경계와 범위 유효성의 회귀를 검사한다.
    #[test]
    fn unicode_edits_and_search() {
        assert_eq!(replace_text("한글🙂", 3, 6, "국").unwrap(), "한국🙂");
        assert!(replace_text("한글", 1, 3, "").is_err());
        assert!(replace_text("a", 2, 1, "").is_err());
        assert_eq!(find_text("한글 한글", "글").unwrap(), vec![(3, 6), (10, 13)]);
        assert!(find_text("abc", "").is_err());
    }
    /// 모든 도구 이름의 유일성과 입력 검사를 확인한다.
    #[test]
    fn schemas_validate_inputs() {
        let tools = definitions();
        let names: std::collections::BTreeSet<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(names.len(), tools.len());
        assert!(validate("replace_range", &json!({"start":0,"end":3,"text":"한"})).is_ok());
        assert!(validate("replace_range", &json!({"start":-1,"end":3,"text":""})).is_err());
        assert!(validate("save", &json!({"surprise":true})).is_err());
        assert!(validate("split_pane", &json!({"direction":"diagonal"})).is_err());
    }
}
