<!---
Copyright (C) 2020 Robin Krahl <robin.krahl@ireas.org>
SPDX-License-Identifier: CC0-1.0
-->

# cursive-markup-rs

The `cursive-markup` crate provides a markup view for [`cursive`][] that can
render HTML.

[`cursive`]: https://lib.rs/cursive

[Documentation](https://docs.rs/cursive_markup/latest/cursive_markup)

For example, this page could be rendered like this:

<img alt="A screenshot of this readme rendered with cursive-markup"
     src="https://git.sr.ht/~ireas/cursive-markup-rs/blob/master/cursive-markup.jpg"/>

## Example

<!-- keep in sync with the crate documentation -->
```
// Create the markup view
let html = "<a href='https://rust-lang.org'>Rust</a>";
let mut view = cursive_markup::MarkupView::html(&html);
view.set_maximum_width(120);

// Set callbacks that are called if the link focus is changed and if a link is
// selected with the Enter key
view.on_link_focus(|s, url| {});
view.on_link_select(|s, url| {});

// Add the view to a Cursive instance
use cursive::view::{Resizable, Scrollable};
let mut s = cursive::dummy();
s.add_global_callback('q', |s| s.quit());
s.add_fullscreen_layer(view.scrollable().full_screen());
s.run();
```

For a complete example, see [`examples/browser.rs`][], a very simple browser
implementation.

[`examples/browser.rs`]: https://git.sr.ht/~ireas/cursive-markup-rs/tree/master/examples/browser.rs

## Features

- `html` (default): render HTML using [`html2text`][]

[`html2text`]: https://lib.rs/html2text

## Minimum Supported Rust Version

This crate supports Rust 1.45.0 or later.

## Contributing

Contributions to this project are welcome!  Please submit patches to the
mailing list [~ireas/public-inbox@lists.sr.ht][] ([archive][]) using the
`[PATCH cursive-markup-rs]` subject prefix.  For more information, see the
[Contributing Guide][].

[~ireas/public-inbox@lists.sr.ht]: mailto:~ireas/public-inbox@lists.sr.ht
[archive]: https://lists.sr.ht/~ireas/public-inbox
[Contributing Guide]: https://man.sr.ht/~ireas/guides/contributing.md

## Contact

For bug reports, feature requests and other messages, please send a mail to
[~ireas/public-inbox@lists.sr.ht][] ([archive][]) using the
`[cursive-markup-rs]` prefix in the subject.

## License

This project is dual-licensed under the [Apache-2.0][] and [MIT][] licenses.
The documentation and examples contained in this repository are licensed under
the [Creative Commons Zero][CC0] license.  You can find a copy of the license
texts in the `LICENSES` directory.

`cursive-markup-rs` complies with [version 3.0 of the REUSE
specification][reuse].

[Apache-2.0]: https://opensource.org/licenses/Apache-2.0
[MIT]: https://opensource.org/licenses/MIT
[CC0]: https://creativecommons.org/publicdomain/zero/1.0/
[reuse]: https://reuse.software/practices/3.0/
