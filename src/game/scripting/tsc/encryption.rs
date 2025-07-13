pub fn decrypt_tsc(buf: &mut [u8]) {
    let half = buf.len() / 2;
    let key = if let Some(0) = buf.get(half) { 0x7 } else { *buf.get(half).unwrap() };
    log::debug!("Decrypting TSC using key {:#x}", key);

    for (idx, byte) in buf.iter_mut().enumerate() {
        if idx == half {
            continue;
        }

        *byte = byte.wrapping_sub(key);
    }
}
