// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! A renderer for HTML documents.
//!
//! *Requires the `html` feature (enabled per default).*
//!
//! This module provides the [`Renderer`][] struct, a renderer that uses [`html2text`][] to render
//! HTML documents.  You can custommize the renderer by settting a [`TextDecorator`][] and a
//! [`Converter`][].  The [`TextDecorator`][] is used by [`html2text`][] to convert the HTML DOM
//! into annotated strings.  The [`Converter`][] is used by the renderer to interpret the
//! annotations and to extract the text format and links.
//!
//! [`html2text`]: https://docs.rs/html2text/latest/html2text/
//! [`TextDecorator`]: https://docs.rs/html2text/latest/html2text/render/text_renderer/trait.TextDecorator.html
//! [`Renderer`]: struct.Renderer.html
//! [`Converter`]: trait.Converter.html

use cursive::theme;
use html2text::render::text_renderer;

use crate::{Element, RenderedDocument};

/// A renderer for HTML documents that uses the default rich text decorator and converter.
pub type RichRenderer = Renderer<text_renderer::RichDecorator, RichConverter>;

/// A renderer for HTML documents.
///
/// This renderer uses [`html2text`][] to parse and render an HTML document.  The provided document
/// is only parsed once: when the instance is constructed.  Then it is rendered every time the
/// width of the view changes.
///
/// You can custommize the renderer by settting a custom [`TextDecorator`][] and [`Converter`][].
/// The [`TextDecorator`][] is used by [`html2text`][] to convert the HTML DOM into annotated
/// strings.  The [`Converter`][] is used by the renderer to interpret the annotations and to
/// extract the text format and links.
///
/// Per default, the renderer uses the [`RichDecorator`][] and the [`RichConverter`][].
///
/// [`html2text`]: https://docs.rs/html2text/latest/html2text/
/// [`TextDecorator`]: https://docs.rs/html2text/latest/html2text/render/text_renderer/trait.TextDecorator.html
/// [`RichDecorator`]: https://docs.rs/html2text/latest/html2text/render/text_renderer/trait.RichDecorator.html
/// [`Converter`]: trait.Converter.html
/// [`RichConverter`]: trait.RichConverter.html
pub struct Renderer<D: text_renderer::TextDecorator + Clone, C: Converter<D::Annotation>> {
    render_tree: html2text::RenderTree,
    decorator: D,
    converter: C,
}

/// A converter for HTML annotations.
///
/// This trait extracts the text formatting and links from the annotations created by a
/// [`TextDecorator`][].
///
/// [`TextDecorator`]: https://docs.rs/html2text/latest/html2text/render/text_renderer/trait.TextDecorator.html
pub trait Converter<A> {
    /// Returns the style for the given annotation (if any).
    fn get_style(&self, annotation: &A) -> Option<theme::Style>;

    /// Returns the link target for the given annotation (if any).
    fn get_link<'a>(&self, annotation: &'a A) -> Option<&'a str>;
}

/// A converter for [`RichAnnotation`][].
///
/// Besides the straightforward mappings of links and text effects, this converter styles links
/// with the underline effect and code snippets with the secondary palette color.
///
/// [`RichAnnotation`]: https://docs.rs/html2text/latest/html2text/render/text_renderer/enum.RichAnnotation.html
pub struct RichConverter;

impl Renderer<text_renderer::RichDecorator, RichConverter> {
    /// Creates a new renderer for the given HTML document using the default settings.
    pub fn new(html: &str) -> Renderer<text_renderer::RichDecorator, RichConverter> {
        Renderer::custom(html, text_renderer::RichDecorator::new(), RichConverter)
    }
}

impl<D: text_renderer::TextDecorator + Clone, C: Converter<D::Annotation>> Renderer<D, C> {
    /// Creates a new renderer for the given HTML document using a custom decorator and converter.
    pub fn custom(html: &str, decorator: D, converter: C) -> Renderer<D, C> {
        Renderer {
            render_tree: html2text::parse(html.as_bytes()),
            decorator,
            converter,
        }
    }
}

impl<D: text_renderer::TextDecorator + Clone, C: Converter<D::Annotation>> super::Renderer
    for Renderer<D, C>
{
    fn render(&self, constraint: cursive::XY<usize>) -> RenderedDocument {
        let mut doc = RenderedDocument::new(constraint);

        let lines = self
            .render_tree
            .clone()
            .render(std::cmp::max(5, constraint.x), self.decorator.clone())
            .into_lines();
        for line in lines {
            let mut elements = Vec::new();
            for element in line.iter() {
                if let text_renderer::TaggedLineElement::Str(ts) = element {
                    let styles: Vec<_> = ts
                        .tag
                        .iter()
                        .filter_map(|a| self.converter.get_style(a))
                        .collect();
                    let link_target = ts
                        .tag
                        .iter()
                        .find_map(|a| self.converter.get_link(a))
                        .map(ToOwned::to_owned);
                    elements.push(Element::new(
                        ts.s.clone(),
                        theme::Style::merge(&styles),
                        link_target,
                    ));
                }
            }
            doc.push_line(elements);
        }

        doc
    }
}

impl Converter<text_renderer::RichAnnotation> for RichConverter {
    fn get_style(&self, annotation: &text_renderer::RichAnnotation) -> Option<theme::Style> {
        use text_renderer::RichAnnotation;
        match annotation {
            RichAnnotation::Default => None,
            RichAnnotation::Link(_) => Some(theme::Effect::Underline.into()),
            RichAnnotation::Image => None,
            RichAnnotation::Emphasis => Some(theme::Effect::Italic.into()),
            RichAnnotation::Strong => Some(theme::Effect::Bold.into()),
            RichAnnotation::Strikeout => Some(theme::Effect::Strikethrough.into()),
            RichAnnotation::Code => Some(theme::PaletteColor::Secondary.into()),
            RichAnnotation::Preformat(_) => None,
        }
    }

    fn get_link<'a>(&self, annotation: &'a text_renderer::RichAnnotation) -> Option<&'a str> {
        if let text_renderer::RichAnnotation::Link(target) = annotation {
            Some(target)
        } else {
            None
        }
    }
}
