use crate::lexer::ExternalLink;

const YOUTUBE_WATCH_URL: &str = "https://www.youtube.com/watch?v=";
fn try_show_youtube(url: &str) -> Option<String> {
    if !url.starts_with(YOUTUBE_WATCH_URL) {
        return None;
    }

    let id = &url[YOUTUBE_WATCH_URL.len()..];

    Some(format!("{{{{< youtube id=\"{id}\" >}}}}"))
}

pub fn format_external_link(link: &ExternalLink) -> String {
    // Normal links go unchanged
    if let ExternalLink {
        render: false, url, ..
    } = link
    {
        return format!("[{}]({})", link.label(), url);
    }

    let url = link.url.as_str();

    if let Some(s) = try_show_youtube(url) {
        return s;
    }

    // Assume that the link is an image
    return format!("![{}]({})", link.label(), url);
}
