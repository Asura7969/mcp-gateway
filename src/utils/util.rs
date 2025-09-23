/// 提取请求路径中endpoint
pub fn extract_endpoint_id(url: &str) -> Option<String> {
    let (_sse, star_tag, end_tag) = stream_or_sse(url);
    let start = url.find(star_tag)?;
    let value_start = start + star_tag.len();

    // 从值开始位置查找下一个 '&' / '?' 或字符串结束位置
    let end = url[value_start..]
        .find(end_tag)
        .map(|pos| value_start + pos)
        .unwrap_or(url.len());
    Some(url[value_start..end].to_string())
}

fn stream_or_sse(url: &str) -> (bool, &str, &str) {
    if url.contains("/sse") {
        // .../sse
        (false, "endpointId=", "&")
    } else {
        // .../stream/... , 实际路径只能截取到stream后面的路径, 所以此处只返回'/'
        (true, "/", "?")
    }
}
