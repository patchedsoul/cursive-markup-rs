// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: CC0-1.0

//! This example implements a basic web browser using `cursive-markup`, `html2text` and `ureq`.

fn parse_args(url: &mut String) {
    let mut parser = argparse::ArgumentParser::new();
    parser.set_description("A basic web browser");
    parser
        .refer(url)
        .add_argument("url", argparse::Store, "the URL to open");
    parser.parse_args_or_exit();
}

fn show_error(s: &mut cursive::Cursive, msg: impl Into<String>) {
    let mut dialog = cursive::views::Dialog::info(msg.into());
    dialog.set_title("Error");
    s.add_layer(dialog);
}

fn open_url(s: &mut cursive::Cursive, url: url::Url) {
    s.call_on_name("status", |t: &mut cursive::views::TextView| {
        t.set_content(format!("Opened URL: {}", url.as_str()));
    });

    // In a real application, we would have to implement proper error handling for the response.
    let response = ureq::get(url.as_str()).call();
    match response.content_type() {
        "text/html" => {
            let html = response.into_string().expect("Failed to download URL");
            open_view(s, cursive_markup::MarkupView::html(&html), url);
        }
        _ => show_error(
            s,
            format!("Unsupported content type: {}", response.content_type()),
        ),
    }
}

fn open_view<R: cursive_markup::Renderer + 'static>(
    s: &mut cursive::Cursive,
    mut view: cursive_markup::MarkupView<R>,
    url: url::Url,
) {
    use cursive::view::{Resizable, Scrollable};

    view.set_maximum_width(120);
    view.on_link_select(move |s, link_url| match url.join(link_url) {
        Ok(url) => open_url(s, url),
        Err(err) => show_error(s, format!("Failed to parse URL: {}", err)),
    });
    view.on_link_focus(move |s, link_url| {
        s.call_on_name("status", |t: &mut cursive::views::TextView| {
            t.set_content(format!("Link target: {}", link_url));
        });
    });

    s.call_on_name("content", |s: &mut cursive::views::StackView| {
        s.add_fullscreen_layer(view.scrollable().full_screen());
    });
}

fn create_cursive() -> cursive::Cursive {
    use cursive::traits::{Nameable, Resizable};

    let mut s = cursive::default();
    s.add_global_callback('q', |s| s.quit());
    s.add_global_callback(cursive::event::Key::Backspace, |s| {
        if s.screen().len() > 1 {
            s.pop_layer();
        }
    });

    let content = cursive::views::StackView::new().with_name("content");
    let status = cursive::views::TextView::new("").with_name("status");
    let layout = cursive::views::LinearLayout::vertical()
        .child(content.full_screen())
        .child(status.fixed_height(1));
    s.add_fullscreen_layer(layout.full_screen());

    s
}

fn main() {
    let mut url = String::new();
    parse_args(&mut url);
    let url = url::Url::parse(&url).expect("Could not parse URL");

    let mut s = create_cursive();
    open_url(&mut s, url);
    s.run();
}
