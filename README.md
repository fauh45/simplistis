# simplistis

Small weekend-ish (for now!) project written in rust, to fulfill the needs of hosting my own personal site. Is there already more well-written tools out there? Of course, but it’s more fun to learn how to make it ourselves isn’t it?

## What is it?

Simplistis comes from Indonesian word that means, simplistic, simple enough. It’s a program written in Rust that will read a folder system, and CommonMark (`.md`) then generate a simple static web out of it.

## How?

So, from what I think there’s going to be two major components to simplistis. One would be focused on getting the content of the blog itself (I think I’m going to focus on blog use case first for now). The other one is putting the content to the web template. Then ta-da! You got yourself a static site generated from a static template and content!

Here’s a more detailed explanation of the components of simplistis.

### Content Subsystem

This component will scour the predefined (or configured) root folder to get a list of content that needs to be generated. From what I think, here’s what the folder structure should look like.

```
contents (root content folder) ->
	[slug].md
	…
```

That’s it! Each of the markdown (`.md`) files will result in a page generated with the content put inside of it. Each of the markdown files should follow CommonMark Spec, and have the following header on the top.

```
—
# Any valid TOML file
title = “simplistis”
author = “fauh45”
summary = “Super simplistic static website generator”
—
```

Web Template Subsystem

This components use the information from the content subsystem and generates pages based on another file system.

```
(root) ->
	pages ->
		content.html (post html template)
		index.html (main page template)
		… (could be other templates here, but I’m not sure for now)
	contents -> (as described on the previous subsystem)
		[slug].md 
	config.toml (main configuration file)
```

This subsystem will get all the content data, generate it to its content.html, make a list of all the contents, then generate all the pages including the post files. Ta-da, that’s your static site!

