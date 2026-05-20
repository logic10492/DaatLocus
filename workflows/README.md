# Builtin SOP Primitive Specs

This directory stores Daat Locus builtin SOP primitive specification assets.

Rules:

- One Markdown file per builtin SOP primitive spec.
- The file name is the primitive id used by the primitive runtime tools, and it may contain only lowercase `a-z` and `-`.
- Markdown content is unrestricted by the primitive id; any legacy frontmatter is ignored for identity.
- Runtime writes primitive specs as Markdown bodies without frontmatter.
- These primitive specs are compiled into the program by `build.rs` and belong to builtin baseline capabilities.
- Builtin primitive specs are read-only and are never written back by `create_primitive_spec`, sleep patch, or sleep merge.
- Runtime-evolvable SOP primitive specs only live under `~/daat-locus-workspace/workflows`.

To add a builtin primitive later, add the corresponding `*.md` file directly under this directory.
