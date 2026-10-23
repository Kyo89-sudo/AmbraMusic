pub fn is_youtube_stream(url: &str) -> bool {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .is_some_and(|host| host == "googlevideo.com" || host.ends_with(".googlevideo.com"))
}
