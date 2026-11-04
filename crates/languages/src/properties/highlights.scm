; Don Editor: derived from tree-sitter-properties 0.3.0 (MIT), queries/highlights.scm.
; [DonEditor] 수정 없음 (캡처 이름이 Gram 테마와 그대로 호환).

(comment) @comment

(key) @property

(value) @string

(value (escape) @string.escape)

((index) @number
  (#match? @number "^[0-9]+$"))

((substitution (key) @constant)
  (#match? @constant "^[A-Z0-9_]+"))

(substitution
  (key) @function
  "::" @punctuation.special
  (secret) @embedded)

(property [ "=" ":" ] @operator)

[ "${" "}" ] @punctuation.special

(substitution ":" @punctuation.special)

[ "[" "]" ] @punctuation.bracket

[ "." "\\" ] @punctuation.delimiter
