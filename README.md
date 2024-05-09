# simplistis

Small weekend-ish (for now!) project written in rust, to fulfill the needs of
hosting my own personal site. Is there already more well-written tools out there?
Of course, but it’s more fun to learn how to make it ourselves isn't it?

## Folder Structure

There's a simple folder structure `simplistis` follows.

```text
(root) ->
  _index.md (index content file, required)
  template.hbs (index template file, required)

  content.hbs (content template file, optional, but required if there's file
other than _index.md)
  [slug].md (content file, optional)

  (sub-folder) ->
    _index.md (subfolder index content file, required)
    template.hbs (subfolder index template file, required)

    (the rest are basically the same as the root)
```

To see a valid example of this folder structure you can see the [`test_file`
folder](/test_files). This will contains a very simple (or as you can say
`simplistis`) homepage, and blog example.

## CLI

Currently `simplistis` only supports CLI interface, though it is very easy to
use. The only command you need to use and remember (though this maybe changing
in the future, but rest assured I'll be sure to make it compatible!).

```bash
simplistis [template directory root] [output directory root]
```

## Developer

This project is developed by the all the contributors. The list should show below.

- [fauh45](https://github.com/fauh45)
