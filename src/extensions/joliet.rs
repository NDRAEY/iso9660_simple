pub(crate) fn parse_name(ucs2_name: &[u8]) -> Option<String> {
    if ucs2_name.len() == 1 {
        if ucs2_name[0] == 0 {
            return Some(String::from("."));
        } else if ucs2_name[0] == 1 {
            return Some(String::from(".."));
        }
    }

    debug_assert!(ucs2_name.len() % 2 == 0, "The length of UCS-2 name must be a multiple of two");

    let mut utf8_str_buf = vec![0u8; ucs2_name.len()];

    let utf16_buf: &[u16] = unsafe { core::slice::from_raw_parts(ucs2_name.as_ptr() as *const u16, ucs2_name.len() / 2) };

    ucs2::decode(utf16_buf, &mut utf8_str_buf).ok()?;

    str::from_utf8(&utf8_str_buf).map(|x| x.trim_end_matches('\0').to_owned()).ok()
}