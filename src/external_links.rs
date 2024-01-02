use std::collections::HashMap;

use crate::lexer::ExternalLink;

const YOUTUBE_WATCH_URL: &str = "https://www.youtube.com/watch?v=";
fn try_show_youtube(url: &str) -> Option<String> {
    if !url.starts_with(YOUTUBE_WATCH_URL) {
        return None;
    }

    let id_and_attrs = &url[YOUTUBE_WATCH_URL.len()..];

    let mut parts = id_and_attrs.split("&");

    let id = if let Some(id) = parts.next() {
        id
    } else {
        return None;
    };

    let mut attrs = HashMap::new();

    for attr_string in parts {
        if let Some((key, value)) = attr_string.split_once("=") {
            attrs.insert(key, value);
        }
    }

    Some(format!(
        "{{{{< youtube id=\"{}\" {} >}}}}",
        id,
        attrs
            .iter()
            .map(|(k, v)| format!("{}=\"{}\" ", k, v))
            .collect::<String>()
    ))
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
