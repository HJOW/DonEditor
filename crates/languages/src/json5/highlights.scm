; Don Editor: based on tree-sitter-json5-orchard 0.1.0 (MIT), queries/highlights.scm.
; [DonEditor] 숫자는 number, 불리언은 boolean, 객체 키는 property 로 구분하도록 확장했다.
; 같은 노드에 캡처가 여러 개면 나중 패턴이 이기므로 키(property) 패턴을 뒤에 둔다.

(string) @string

(identifier) @constant

(number) @number

(null) @constant.builtin

[
  (true)
  (false)
] @boolean

(comment) @comment

(member
  name: (identifier) @property)

(member
  name: (string) @property)
