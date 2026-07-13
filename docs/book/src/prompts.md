# Prompt templates

Pleiades ships assistant, summarizer, review, commit, PR summary, diff explanation, refactoring, and test-generation templates. Templates substitute `{{name}}`; `{{name|default}}` supplies a fallback.

```console
pleiades prompt list
pleiades prompt show code-reviewer
pleiades prompt render explain-diff --var diff='...'
pleiades prompt save my-prompt 'Description' 'Hello {{name}}'
```
