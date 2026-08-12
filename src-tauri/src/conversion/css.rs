use super::OutputOptions;

pub fn generate_css(options: &OutputOptions) -> String {
    let font_family = match options.font_family.as_str() {
        "serif" => "Georgia, \"Times New Roman\", serif",
        "sans-serif" => "\"Helvetica Neue\", Helvetica, Arial, sans-serif",
        "monospace" => "\"Courier New\", Courier, monospace",
        _ => "serif",
    };

    let text_align = match options.text_alignment.as_str() {
        "left" => "left",
        "right" => "right",
        _ => "justify",
    };

    format!(
        r#"body {{
    font-family: {font_family};
    font-size: {font_size}pt;
    line-height: {line_height};
    margin: {margins}em;
    text-align: {text_align};
    orphans: 2;
    widows: 2;
}}

h1 {{
    font-size: 1.6em;
    margin-top: 2em;
    margin-bottom: 0.5em;
    page-break-before: always;
    line-height: 1.2;
}}

h2 {{
    font-size: 1.3em;
    margin-top: 1.5em;
    margin-bottom: 0.4em;
    line-height: 1.2;
}}

h3 {{
    font-size: 1.1em;
    margin-top: 1.2em;
    margin-bottom: 0.3em;
    line-height: 1.3;
}}

h4, h5, h6 {{
    font-size: 1em;
    margin-top: 1em;
    margin-bottom: 0.3em;
}}

p {{
    margin-top: 0.3em;
    margin-bottom: 0.3em;
    text-indent: 1.5em;
}}

p:first-of-type {{
    text-indent: 0;
}}

ul, ol {{
    margin-left: 1.5em;
}}

li {{
    margin-bottom: 0.2em;
}}

img {{
    max-width: 100%;
    height: auto;
}}

hr.page-break {{
    page-break-before: always;
    border: none;
    margin: 0;
    padding: 0;
    height: 0;
}}"#,
        font_family = font_family,
        font_size = options.base_font_size,
        line_height = options.line_height,
        margins = options.margins,
        text_align = text_align,
    )
}
