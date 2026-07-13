# Git integration

Git commands use the configured provider and built-in prompt templates.

```console
pleiades git diff --staged
pleiades git commit
pleiades git review --staged
pleiades git summary --base origin/master --title 'Feature title'
```

Commit generation reads only staged changes. Review defaults to unstaged working-tree changes. PR summaries combine commit subjects and the selected revision range. Generated text is advisory; inspect it before publishing or committing.
