pub fn decrypt_tsc(buf: &mut [u8]) {
    let half = buf.len() / 2;
    let key = if let Some(0) = buf.get(half) { 0xf9 } else { (-(*buf.get(half).unwrap() as isize)) as u8 };
    log::info!("Decrypting TSC using key {:#x}", key);

    for (idx, byte) in buf.iter_mut().enumerate() {
        if idx == half {
            continue;
        }

        *byte = byte.wrapping_add(key);
    }
}
