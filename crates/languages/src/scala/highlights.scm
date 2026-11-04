; Don Editor: derived from tree-sitter-scala 0.26.3 (MIT), queries/highlights.scm.
; [DonEditor] Gram 테마가 모르는 nvim 방식 캡처 이름을 Gram 이름으로 바꿨다 (conditional→keyword.conditional, repeat→keyword.repeat, include→keyword.import,
; [DonEditor] exception→keyword.exception, storageclass→keyword.modifier, method→function.method). 스타일을 덮어써 지우는 @none, @spell 은 제거했다.

; CREDITS @stumash (stuart.mashaal@gmail.com)

(field_expression field: (identifier) @property)
(field_expression value: (identifier) @type
 (#match? @type "^[A-Z]"))

(type_identifier) @type

(class_definition
  name: (identifier) @type)

(enum_definition
  name: (identifier) @type)

(object_definition
  name: (identifier) @type)

(trait_definition
  name: (identifier) @type)

(full_enum_case
  name: (identifier) @type)

(simple_enum_case
  name: (identifier) @type)

;; variables

(class_parameter
  name: (identifier) @parameter)

(self_type (identifier) @parameter)


;; types

(type_definition
  name: (type_identifier) @type.definition)

;; val/var definitions/declarations

(val_definition
  pattern: (identifier) @variable)

(var_definition
  pattern: (identifier) @variable)

(val_declaration
  name: (identifier) @variable)

(var_declaration
  name: (identifier) @variable)

; imports/exports

(import_declaration
  path: (identifier) @namespace)
((stable_identifier (identifier) @namespace))

((import_declaration
  path: (identifier) @type) (#match? @type "^[A-Z]"))
((stable_identifier (identifier) @type) (#match? @type "^[A-Z]"))

(export_declaration
  path: (identifier) @namespace)
((stable_identifier (identifier) @namespace))

((export_declaration
  path: (identifier) @type) (#match? @type "^[A-Z]"))
((stable_identifier (identifier) @type) (#match? @type "^[A-Z]"))

((namespace_selectors (identifier) @type) (#match? @type "^[A-Z]"))

; method invocation

(call_expression
  function: (identifier) @function.call)

(call_expression
  function: (operator_identifier) @function.call)

(call_expression
  function: (field_expression
    field: (identifier) @function.method.call))

((call_expression
   function: (identifier) @constructor)
 (#match? @constructor "^[A-Z]"))

(generic_function
  function: (identifier) @function.call)

(interpolated_string_expression
  interpolator: (identifier) @function.call)

; function definitions

(function_definition
  name: (identifier) @function)

(parameter
  name: (identifier) @parameter)

(binding
  name: (identifier) @parameter)

; method definition

(function_declaration
      name: (identifier) @function.method)

(function_definition
      name: (identifier) @function.method)

; expressions

(infix_expression operator: (identifier) @operator)
(infix_expression operator: (operator_identifier) @operator)
; An operator taking a colon argument parses as a postfix expression call.
(call_expression
  function: (postfix_expression (identifier) @operator .)
  arguments: (colon_argument))
(infix_type operator: (operator_identifier) @operator)
(infix_type operator: (operator_identifier) @operator)

; literals

(boolean_literal) @boolean
(integer_literal) @number
(floating_point_literal) @float

[
  (string)
  (character_literal)
  (interpolated_string_expression)
] @string

(interpolation "$" @punctuation.special)

;; keywords

(opaque_modifier) @type.qualifier
(infix_modifier) @keyword
(transparent_modifier) @type.qualifier
(open_modifier) @type.qualifier

[
  "case"
  "class"
  "enum"
  "extends"
  "derives"
  "finally"
  "forSome"
;; `macro` not implemented yet
  "object"
  "override"
  "package"
  "trait"
  "type"
  "val"
  "var"
  "with"
  "given"
  "using"
  "implicit"
  "with"
] @keyword

; `end` is scanner-lexed, so the marker node is the only thing to match.
(end_marker) @keyword

; `extension` is a soft keyword. Highlight it only where it starts an
; extension definition, not when used as a plain identifier.
(extension_definition "extension" @keyword)

[
  "abstract"
  "final"
  "lazy"
  "sealed"
  "private"
  "protected"
] @type.qualifier

(inline_modifier) @keyword.modifier

(null_literal) @constant.builtin

(wildcard) @parameter

(annotation) @attribute

;; special keywords

"new" @keyword.operator

[
  "else"
  "if"
  "match"
  "then"
] @keyword.conditional

[
 "("
 ")"
 "["
 "]"
 "{"
 "}"
]  @punctuation.bracket

[
 "."
 ","
] @punctuation.delimiter

[
  "do"
  "for"
  "while"
  "yield"
] @keyword.repeat

"def" @keyword.function

[
 "=>"
 "<-"
 "@"
] @operator

["import" "export"] @keyword.import

[
  "try"
  "catch"
  "throw"
] @keyword.exception

"return" @keyword.return

(comment) @comment
(block_comment) @comment

;; `case` is a conditional keyword in case_block

(case_block
  (case_clause ("case") @keyword.conditional))
(indented_cases
  (case_clause ("case") @keyword.conditional))

(operator_identifier) @operator

((identifier) @type (#match? @type "^[A-Z]"))
((identifier) @variable.builtin
 (#match? @variable.builtin "^this$"))

(
  (identifier) @function.builtin
  (#match? @function.builtin "^super$")
)

;; Scala CLI using directives
(using_directive_key) @parameter
(using_directive_value) @string

;; XML literals
(xml_name) @tag
(xml_attribute key: (xml_name) @tag.attribute)
(xml_string) @string
(xml_comment) @comment
(xml_cdata) @string
(xml_processing_instruction) @keyword.directive

; [DonEditor] 문자열 보간 안의 식별자는 문자열 색이 아니라 변수 색으로 보이게 한다 (원본의 @none 대체)
(interpolation (identifier) @variable)
