use pulldown_cmark::{html, Options, Parser};

pub fn render_markdown(input: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Sanitize HTML to prevent XSS (strips <script>, event handlers, etc.)
    ammonia::clean(&html_output)
}
