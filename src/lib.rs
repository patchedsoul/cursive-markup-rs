// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! `cursive-markup` provides the [`MarkupView`][] for [`cursive`][] that can render HTML or other
//! markup.
//!
//! # Quickstart
//!
//! To render an HTML document, create a [`MarkupView`][] with the [`html`][] method, configure the
//! maximum line width using the [`set_maximum_width`][] method and set callbacks for the links
//! using the [`on_link_select`][] and [`on_link_focus`][] methods.
//!
//! Typically, you’ll want to wrap the view in a [`ScrollView`][] and add it to a
//! [`Cursive`][`cursive::Cursive`] instance.
//!
//! ```
//! // Create the markup view
//! let html = "<a href='https://rust-lang.org'>Rust</a>";
//! let mut view = cursive_markup::MarkupView::html(&html);
//! view.set_maximum_width(120);
//!
//! // Set callbacks that are called if the link focus is changed and if a link is selected with
//! // the Enter key
//! view.on_link_focus(|s, url| {});
//! view.on_link_select(|s, url| {});
//!
//! // Add the view to a Cursive instance
//! use cursive::view::{Resizable, Scrollable};
//! let mut s = cursive::dummy();
//! s.add_global_callback('q', |s| s.quit());
//! s.add_fullscreen_layer(view.scrollable().full_screen());
//! s.run();
//! ```
//!
//! You can use the arrow keys to navigate between the links and press Enter to trigger the
//! [`on_link_select`][] callback.
//!
//! For a complete example, see [`examples/browser.rs`][], a very simple browser implementation.
//!
//! # Components
//!
//! The main component of the crate is [`MarkupView`][].  It is a [`cursive`][] view that displays
//! hypertext: a combination of formatted text and links.  You can use the arrow keys to navigate
//! between the links, and the Enter key to select a link.
//!
//! The displayed content is provided and rendered by a [`Renderer`][] instance.  If the `html`
//! feature is enabled (default), the [`html::Renderer`][] can be used to parse and render an HTML
//! document with [`html2text`][].  But you can also implement your own [`Renderer`][].
//! [`MarkupView`][] caches the rendered document ([`RenderedDocument`][]) and only invokes the
//! renderer if the width of the view has been changed.
//!
//! ## HTML rendering
//!
//! To customize the HTML rendering, you can change the [`TextDecorator`][] that is used by
//! [`html2text`][] to transform the HTML DOM into annotated strings.  Of course the renderer must
//! know how to interpret the annotations, so if you provide a custom decorator, you also have to
//! provide a [`Converter`][] that extracts formatting and links from the annotations.
//!
//! [`cursive`]: https://docs.rs/cursive/latest/cursive/
//! [`cursive::Cursive`]: https://docs.rs/cursive/latest/cursive/struct.Cursive.html
//! [`ScrollView`]: https://docs.rs/cursive/latest/cursive/views/struct.ScrollView.html
//! [`html2text`]: https://docs.rs/html2text/latest/html2text/
//! [`TextDecorator`]: https://docs.rs/html2text/latest/html2text/render/text_renderer/trait.TextDecorator.html
//! [`Converter`]: html/trait.Converter.html
//! [`MarkupView`]: struct.MarkupView.html
//! [`RenderedDocument`]: struct.RenderedDocument.html
//! [`Renderer`]: trait.Renderer.html
//! [`html`]: struct.MarkupView.html#method.html
//! [`set_maximum_width`]: struct.MarkupView.html#method.set_maximum_width
//! [`on_link_select`]: struct.MarkupView.html#method.on_link_select
//! [`on_link_focus`]: struct.MarkupView.html#method.on_link_focus
//! [`html::Renderer`]: html/struct.Renderer.html
//! [`examples/browser.rs`]: https://git.sr.ht/~ireas/cursive-markup-rs/tree/master/examples/browser.rs

#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "html")]
pub mod html;

use std::rc;

use cursive_core::theme;
use unicode_width::UnicodeWidthStr as _;

/// A view for hypertext that has been rendered by a [`Renderer`][].
///
/// This view displays hypertext (a combination of formatted text and links) that typically has
/// been parsed from a markup language.  You can use the arrow keys to navigate between the links,
/// and the Enter key to select a link.  If the focused link is changed, the [`on_link_focus`][]
/// callback is triggered.  If the focused link is selected using the Enter key, the
/// [`on_link_select`][] callback is triggered.
///
/// The displayed hypertext is created by a [`Renderer`][] implementation.  The `MarkupView` calls
/// the [`render`][] method with the size constraint provided by `cursive` and receives a
/// [`RenderedDocument`][] that contains the text and the links.  This document is cached until the
/// available width changes.
///
/// You can also limit the available width by setting a maximum line width with the
/// [`set_maximum_width`][] method.
///
/// [`RenderedDocument`]: struct.RenderedDocument.html
/// [`Renderer`]: trait.Renderer.html
/// [`render`]: trait.Renderer.html#method.render
/// [`on_link_select`]: #method.on_link_select
/// [`on_link_focus`]: #method.on_link_focus
/// [`set_maximum_width`]: #method.set_maximum_width
pub struct MarkupView<R: Renderer + 'static> {
    renderer: R,
    doc: Option<RenderedDocument>,
    on_link_focus: Option<rc::Rc<LinkCallback>>,
    on_link_select: Option<rc::Rc<LinkCallback>>,
    maximum_width: Option<usize>,
}

/// A callback that is triggered for a link.
///
/// The first argument is a mutable reference to the current [`Cursive`][] instance.  The second
/// argument is the target of the link, typically a URL.
///
/// [`Cursive`]: https://docs.rs/cursive/latest/cursive/struct.Cursive.html
pub type LinkCallback = dyn Fn(&mut cursive_core::Cursive, &str);

/// A renderer that produces a hypertext document.
pub trait Renderer {
    /// Renders this document within the given size constraint and returns the result.
    ///
    /// This method is called by [`MarkupView`][] every time the provided width changes.
    ///
    /// [`MarkupView`]: struct.MarkupView.html
    fn render(&self, constraint: cursive_core::XY<usize>) -> RenderedDocument;
}

/// A rendered hypertext document that consists of lines of formatted text and links.
#[derive(Clone, Debug)]
pub struct RenderedDocument {
    lines: Vec<Vec<RenderedElement>>,
    link_handler: LinkHandler,
    size: cursive_core::XY<usize>,
    constraint: cursive_core::XY<usize>,
}

/// A hypertext element: a formatted string with an optional link target.
#[derive(Clone, Debug, Default)]
pub struct Element {
    text: String,
    style: theme::Style,
    link_target: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct RenderedElement {
    text: String,
    style: theme::Style,
    link_idx: Option<usize>,
}

#[derive(Clone, Debug, Default)]
struct LinkHandler {
    links: Vec<Link>,
    focus: usize,
}

#[derive(Clone, Debug)]
struct Link {
    position: cursive_core::XY<usize>,
    width: usize,
    target: String,
}

#[cfg(feature = "html")]
impl MarkupView<html::RichRenderer> {
    /// Creates a new `MarkupView` that uses a rich text HTML renderer.
    ///
    /// *Requires the `html` feature (enabled per default).*
    pub fn html(html: &str) -> MarkupView<html::RichRenderer> {
        MarkupView::with_renderer(html::Renderer::new(html))
    }
}

impl<R: Renderer + 'static> MarkupView<R> {
    /// Creates a new `MarkupView` with the given renderer.
    pub fn with_renderer(renderer: R) -> MarkupView<R> {
        MarkupView {
            renderer,
            doc: None,
            on_link_focus: None,
            on_link_select: None,
            maximum_width: None,
        }
    }

    /// Sets the callback that is triggered if the link focus is changed.
    ///
    /// Note that this callback is only triggered if the link focus is changed with the arrow keys.
    /// It is not triggered if the view takes focus.  The callback will receive the target of the
    /// link as an argument.
    pub fn on_link_focus<F: Fn(&mut cursive_core::Cursive, &str) + 'static>(&mut self, f: F) {
        self.on_link_focus = Some(rc::Rc::new(f));
    }

    /// Sets the callback that is triggered if a link is selected.
    ///
    /// This callback is triggered if a link is focused and the users presses the Enter key.  The
    /// callback will receive the target of the link as an argument.
    pub fn on_link_select<F: Fn(&mut cursive_core::Cursive, &str) + 'static>(&mut self, f: F) {
        self.on_link_select = Some(rc::Rc::new(f));
    }

    /// Sets the maximum width of the view.
    ///
    /// This means that the width that is available for the renderer is limited to the given value.
    pub fn set_maximum_width(&mut self, width: usize) {
        self.maximum_width = Some(width);
    }

    fn render(&mut self, mut constraint: cursive_core::XY<usize>) -> cursive_core::XY<usize> {
        let mut last_focus = 0;

        if let Some(width) = self.maximum_width {
            constraint.x = std::cmp::min(width, constraint.x);
        }

        if let Some(doc) = &self.doc {
            if constraint.x == doc.constraint.x {
                return doc.size;
            }
            last_focus = doc.link_handler.focus;
        }

        let mut doc = self.renderer.render(constraint);

        // TODO: Rendering the document with a different width may lead to links being split up (or
        // previously split up links being no longer split up).  Ideally, we would adjust the focus
        // for these changes.
        if last_focus < doc.link_handler.links.len() {
            doc.link_handler.focus = last_focus;
        }
        let size = doc.size;
        self.doc = Some(doc);
        size
    }
}

impl<R: Renderer + 'static> cursive_core::View for MarkupView<R> {
    fn draw(&self, printer: &cursive_core::Printer<'_, '_>) {
        let doc = &self.doc.as_ref().expect("layout not called before draw");
        for (y, line) in doc.lines.iter().enumerate() {
            let mut x = 0;
            for element in line {
                let mut style = element.style;
                if let Some(link_idx) = element.link_idx {
                    if printer.focused && doc.link_handler.focus == link_idx {
                        style = style.combine(theme::PaletteColor::Highlight);
                    }
                }
                printer.with_style(style, |printer| printer.print((x, y), &element.text));
                x += element.text.width();
            }
        }
    }

    fn layout(&mut self, constraint: cursive_core::XY<usize>) {
        self.render(constraint);
    }

    fn required_size(&mut self, constraint: cursive_core::XY<usize>) -> cursive_core::XY<usize> {
        self.render(constraint)
    }

    fn take_focus(&mut self, direction: cursive_core::direction::Direction) -> bool {
        self.doc
            .as_mut()
            .map(|doc| doc.link_handler.take_focus(direction))
            .unwrap_or_default()
    }

    fn on_event(&mut self, event: cursive_core::event::Event) -> cursive_core::event::EventResult {
        use cursive_core::direction::Absolute;
        use cursive_core::event::{Callback, Event, EventResult, Key};

        let link_handler = if let Some(doc) = self.doc.as_mut() {
            if doc.link_handler.links.is_empty() {
                return EventResult::Ignored;
            } else {
                &mut doc.link_handler
            }
        } else {
            return EventResult::Ignored;
        };

        // TODO: implement mouse support

        let focus_changed = match event {
            Event::Key(Key::Left) => link_handler.move_focus(Absolute::Left),
            Event::Key(Key::Right) => link_handler.move_focus(Absolute::Right),
            Event::Key(Key::Up) => link_handler.move_focus(Absolute::Up),
            Event::Key(Key::Down) => link_handler.move_focus(Absolute::Down),
            _ => false,
        };

        if focus_changed {
            let target = link_handler.links[link_handler.focus].target.clone();
            EventResult::Consumed(
                self.on_link_focus
                    .clone()
                    .map(|f| Callback::from_fn(move |s| f(s, &target))),
            )
        } else if event == Event::Key(Key::Enter) {
            let target = link_handler.links[link_handler.focus].target.clone();
            EventResult::Consumed(
                self.on_link_select
                    .clone()
                    .map(|f| Callback::from_fn(move |s| f(s, &target))),
            )
        } else {
            EventResult::Ignored
        }
    }

    fn important_area(&self, _: cursive_core::XY<usize>) -> cursive_core::Rect {
        if let Some(doc) = &self.doc {
            doc.link_handler.important_area()
        } else {
            cursive_core::Rect::from((0, 0))
        }
    }
}

impl RenderedDocument {
    /// Creates a new rendered document with the given size constraint.
    ///
    /// The size constraint is used to check whether a cached document can be reused or whether it
    /// has to be rendered for the new constraint.  It is *not* enforced by this struct!
    pub fn new(constraint: cursive_core::XY<usize>) -> RenderedDocument {
        RenderedDocument {
            lines: Vec::new(),
            link_handler: Default::default(),
            size: (0, 0).into(),
            constraint,
        }
    }

    /// Appends a rendered line to the document.
    pub fn push_line<I: IntoIterator<Item = Element>>(&mut self, line: I) {
        let mut rendered_line = Vec::new();
        let y = self.lines.len();
        let mut x = 0;
        for element in line {
            let width = element.text.width();
            let link_idx = element.link_target.map(|target| {
                self.link_handler.push(Link {
                    position: (x, y).into(),
                    width,
                    target,
                })
            });
            x += width;
            rendered_line.push(RenderedElement {
                text: element.text,
                style: element.style,
                link_idx,
            });
        }
        self.lines.push(rendered_line);
        self.size = self.size.stack_vertical(&(x, 1).into());
    }
}

impl Element {
    /// Creates a new element with the given text, style and optional link target.
    pub fn new(text: String, style: theme::Style, link_target: Option<String>) -> Element {
        Element {
            text,
            style,
            link_target,
        }
    }

    /// Creates an element with the given text, with the default style and without a link target.
    pub fn plain(text: String) -> Element {
        Element {
            text,
            ..Default::default()
        }
    }

    /// Creates an element with the given text and style and without a link target.
    pub fn styled(text: String, style: theme::Style) -> Element {
        Element::new(text, style, None)
    }

    /// Creates an element with the given text, style and link target.
    pub fn link(text: String, style: theme::Style, target: String) -> Element {
        Element::new(text, style, Some(target))
    }
}

impl From<String> for Element {
    fn from(s: String) -> Element {
        Element::plain(s)
    }
}

impl From<Element> for RenderedElement {
    fn from(element: Element) -> RenderedElement {
        RenderedElement {
            text: element.text,
            style: element.style,
            link_idx: None,
        }
    }
}

impl LinkHandler {
    pub fn push(&mut self, link: Link) -> usize {
        self.links.push(link);
        self.links.len() - 1
    }

    pub fn take_focus(&mut self, direction: cursive_core::direction::Direction) -> bool {
        if self.links.is_empty() {
            false
        } else {
            use cursive_core::direction::{Absolute, Direction, Relative};
            let rel = match direction {
                Direction::Abs(abs) => match abs {
                    Absolute::Up | Absolute::Left | Absolute::None => Relative::Front,
                    Absolute::Down | Absolute::Right => Relative::Back,
                },
                Direction::Rel(rel) => rel,
            };
            self.focus = match rel {
                Relative::Front => 0,
                Relative::Back => self.links.len() - 1,
            };
            true
        }
    }

    pub fn move_focus(&mut self, direction: cursive_core::direction::Absolute) -> bool {
        use cursive_core::direction::{Absolute, Relative};

        match direction {
            Absolute::Left => self.move_focus_horizontal(Relative::Front),
            Absolute::Right => self.move_focus_horizontal(Relative::Back),
            Absolute::Up => self.move_focus_vertical(Relative::Front),
            Absolute::Down => self.move_focus_vertical(Relative::Back),
            Absolute::None => false,
        }
    }

    fn move_focus_horizontal(&mut self, direction: cursive_core::direction::Relative) -> bool {
        use cursive_core::direction::Relative;

        if self.links.is_empty() {
            return false;
        }

        let new_focus = match direction {
            Relative::Front => self.focus.checked_sub(1),
            Relative::Back => {
                if self.focus < self.links.len() - 1 {
                    Some(self.focus + 1)
                } else {
                    None
                }
            }
        };

        if let Some(new_focus) = new_focus {
            if self.links[self.focus].position.y == self.links[new_focus].position.y {
                self.focus = new_focus;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn move_focus_vertical(&mut self, direction: cursive_core::direction::Relative) -> bool {
        use cursive_core::direction::Relative;

        if self.links.is_empty() {
            return false;
        }

        // TODO: Currently, we select the first link on a different line.  We could instead select
        // the closest link on a different line (if there are multiple links on one line).

        let y = self.links[self.focus].position.y;
        let iter = self.links.iter().enumerate();
        let next = match direction {
            Relative::Front => iter
                .rev()
                .skip(self.links.len() - self.focus)
                .find(|(_, link)| link.position.y < y),
            Relative::Back => iter
                .skip(self.focus + 1)
                .find(|(_, link)| link.position.y > y),
        };

        if let Some((idx, _)) = next {
            self.focus = idx;
            true
        } else {
            false
        }
    }

    pub fn important_area(&self) -> cursive_core::Rect {
        if self.links.is_empty() {
            cursive_core::Rect::from((0, 0))
        } else {
            let link = &self.links[self.focus];
            cursive_core::Rect::from_size(link.position, (link.width, 1))
        }
    }
}
