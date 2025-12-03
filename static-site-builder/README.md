### Wants

- A simpler process for building kaleidawave-github-io
- Something for publishing the new specification tests for ezno

and

- To use simple-markdown-parser with HTML extra add-ons like `include` etc
- A TSX based templating (not nunjucks)
  - May be more minimal with functions etc
- A single file transform?

### Included

- A static server for development
- RSS generation
- TSX execution
  - Partial runtime? Only supports certain constructs
- Code highlighting
  - Using `tree-sitter` grammars
- Warnings? (or seperate tool)
  - `#TODO` warnings

### Not included

- Formatter (use simple-markdown-parser formatter binary)
- Spell checker (use wip tool)
- Markdown checker (use simple-markdown-parser statistics tool)
- Publishing / deploy (use GitHub pages or other)
- Post view count
- GitHub comments
- CSS minifier

### Process

- New branch for kaleidawave-github-io

### Implementation details

- Have a cache of templates and things

Steps

- Parse YAML configuration
- Run through each path building files

### Libraries

- simple-markdown-parser (off improvements branch until know this is stable)
  - for parsing of markdown
  - for HTML emit
- ezno-parser?
  - maybe for templating
  - if we use the JSX feature we do not need the 
  - how do we evaluate the data?
- YAML parser (included in simple-markdown-parser)
  - for markdown frontmatter
  - for setup
- json-builder-macro
  - for RSS generation
- glob
  - for finding files
- tree-siiter and libloading
  - for loading languages for highlighting
- general-parser
  - for mathematics blocks

### YAML configuration

```yaml
name: Ben's engineering blog
host: ...
paths:
  - path: "/"
    page: "pages/index.tsx"
  - path: "/about"
    page: "pages/about.html"
  - path: "/posts/$identifier"
    page: "posts/**.md"
    ...
```

> Auto find? Distinguish between single and multiple pages? Auto assets

### Page frontmatter

> Maybe this inherits from the parent?

```yml
url: "/posts/..." # TODO this is specified in the main YAML configuration
identifier: ...
description: ...
banner: ...
publish_date: ...
start_date: ...
edits:
  - date: ...
    change: ...
template:
  path: ...
  arguments:
    x: ...
```

> `template: *path*` is allowed (arguments empty)

### Other

For blog

- `val.town` based analytics? `navigator.sendBeacon`
- replace discus comment section implementation
  - uses next.js 👎
- ???
