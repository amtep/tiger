":" @punctuation

(identifier) @variable
(string) @string
(macro) @constant
(expression) @function

(format "#" @punctuation.bracket (format_tag) @attribute "#!" @punctuation.bracket)

(comment) @comment
