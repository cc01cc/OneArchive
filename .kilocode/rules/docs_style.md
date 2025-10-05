# 文档规范

## 通用

1. 文档不要使用 Emoji 符号，请使用合适的 ascii 符号或组合，或者文字描述
2. 文档的层级不要包含标题在内，不要超过 4 级，以 markdown 为例，不要超过 `####`
3. 当文档层级过多，或者文档内容过长时，请将文档拆分，新文件命名，参考已有的文件
4. 文档的语言风格参考已经存在的文档，应当尽量书面化，严谨，简洁，易懂

## Markdown

1. 语法格式要符合 markdownlint 要求 <https://github.com/DavidAnson/markdownlint?tab=readme-ov-file#rules--aliases>
   1. 其中关于 MD033 - Inline HTML, 可以放宽，例如 img 可以使用 inline HTML
2. 中文文档格式要求符合 autoCorrect 要求 <https://github.com/huacnlee/autocorrect>
3. markdown 的一些可多选的样式，参考已经存在的文档，使用统一的样式

### AutoCorrect 补充

语法示例

```txt
# Config rules
rules:
  # Auto add spacing between CJK (Chinese, Japanese, Korean) and English words.
  # 0 - off, 1 - error, 2 - warning
  space-word: 1
  # Add space between some punctuations.
  space-punctuation: 1
  # Add space between brackets (), [] when near the CJK.
  space-bracket: 1
  # Add space between ``, when near the CJK.
  space-backticks: 1
  # Add space between dash `-`
  space-dash: 0
  # Add space between dollar $ when near the CJK.
  space-dollar: 0
  # Convert to fullwidth.
  fullwidth: 1
  # To remove space near the fullwidth.
  no-space-fullwidth: 1
  # Fullwidth alphanumeric characters to halfwidth.
  halfwidth-word: 1
  # Fullwidth punctuations to halfwidth in english.
  halfwidth-punctuation: 1
  # Spellcheck
  spellcheck: 2
```

### Markdownlint 补充

语法示例

```txt
MD001 heading-increment - Heading levels should only increment by one level at a time
MD003 heading-style - Heading style
MD004 ul-style - Unordered list style
MD005 list-indent - Inconsistent indentation for list items at the same level
MD007 ul-indent - Unordered list indentation
MD009 no-trailing-spaces - Trailing spaces
MD010 no-hard-tabs - Hard tabs
MD011 no-reversed-links - Reversed link syntax
MD012 no-multiple-blanks - Multiple consecutive blank lines
MD013 line-length - Line length
MD014 commands-show-output - Dollar signs used before commands without showing output
MD018 no-missing-space-atx - No space after hash on atx style heading
MD019 no-multiple-space-atx - Multiple spaces after hash on atx style heading
MD020 no-missing-space-closed-atx - No space inside hashes on closed atx style heading
MD021 no-multiple-space-closed-atx - Multiple spaces inside hashes on closed atx style heading
MD022 blanks-around-headings - Headings should be surrounded by blank lines
MD023 heading-start-left - Headings must start at the beginning of the line
MD024 no-duplicate-heading - Multiple headings with the same content
MD025 single-title/single-h1 - Multiple top-level headings in the same document
MD026 no-trailing-punctuation - Trailing punctuation in heading
MD027 no-multiple-space-blockquote - Multiple spaces after blockquote symbol
MD028 no-blanks-blockquote - Blank line inside blockquote
MD029 ol-prefix - Ordered list item prefix
MD030 list-marker-space - Spaces after list markers
MD031 blanks-around-fences - Fenced code blocks should be surrounded by blank lines
MD032 blanks-around-lists - Lists should be surrounded by blank lines
[optional] MD033 no-inline-html - Inline HTML
MD034 no-bare-urls - Bare URL used
MD035 hr-style - Horizontal rule style
MD036 no-emphasis-as-heading - Emphasis used instead of a heading
MD037 no-space-in-emphasis - Spaces inside emphasis markers
MD038 no-space-in-code - Spaces inside code span elements
MD039 no-space-in-links - Spaces inside link text
MD040 fenced-code-language - Fenced code blocks should have a language specified
MD041 first-line-heading/first-line-h1 - First line in a file should be a top-level heading
MD042 no-empty-links - No empty links
MD043 required-headings - Required heading structure
MD044 proper-names - Proper names should have the correct capitalization
MD045 no-alt-text - Images should have alternate text (alt text)
MD046 code-block-style - Code block style
MD047 single-trailing-newline - Files should end with a single newline character
MD048 code-fence-style - Code fence style
MD049 emphasis-style - Emphasis style
MD050 strong-style - Strong style
MD051 link-fragments - Link fragments should be valid
MD052 reference-links-images - Reference links and images should use a label that is defined
MD053 link-image-reference-definitions - Link and image reference definitions should be needed
MD054 link-image-style - Link and image style
MD055 table-pipe-style - Table pipe style
MD056 table-column-count - Table column count
MD058 blanks-around-tables - Tables should be surrounded by blank lines
MD059 descriptive-link-text - Link text should be descriptive
```
