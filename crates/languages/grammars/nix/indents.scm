[
  (attrset_expression)
  (rec_attrset_expression)
  (let_attrset_expression)
  (list_expression)
  (parenthesized_expression)
  (formals)
] @indent

[
  "}"
  ")"
  "]"
] @outdent

(binding) @indent

(let_expression) @indent
(let_expression body: (_) @outdent)

(if_expression
  consequence: (_) @indent)
(if_expression
  alternative: (_) @indent)

(apply_expression) @indent
