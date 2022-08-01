
pub(crate) fn arr_fill(dst: &mut [u8], from: usize, to: usize, val: u8) {
    for i in from..to {
        dst[i] = val;
    }
}

pub(crate) fn arr_copy(src: &Vec<u8>, src_pos: usize, dst: &mut [u8], dst_pos: usize, len: usize) {
    if dst.len() < dst_pos + len {
        // dst.resize(dst_pos + len, 0);
    }
    for i in 0..len {
        let val: u8;
        if src.len() < src_pos + i {
            val = 0;
        } else {
            val = src[src_pos + i];
        }

        dst[dst_pos + i] = val;
    }
}
