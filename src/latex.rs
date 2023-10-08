use katex::{self, Opts};

pub fn render_inline_latex(latex: &str) -> Result<String, katex::Error> {
    let opts = katex::Opts::builder()
        .display_mode(false)
        .build()
        .unwrap();
    render_latex(latex, opts)
}

pub fn render_display_latex(latex: &str) -> Result<String, katex::Error> {
    let opts = katex::Opts::builder()
        .display_mode(true)
        .build()
        .unwrap();
    render_latex(latex, opts)
}

fn render_latex(latex: &str, opts: Opts) -> Result<String, katex::Error> {
    katex::render_with_opts(latex, opts)
}
