pub fn get<'a>(url: &'a seed::Url, param: &str) -> Option<&'a str> {
    let mut parts = url.search.as_ref()?.split('&');
    parts.find_map(|part| {
        if let &[key, val] = &*part.split('=').collect::<Vec<_>>() {
            if key == param {
                return Some(val);
            }
        }
        None
    })
}
