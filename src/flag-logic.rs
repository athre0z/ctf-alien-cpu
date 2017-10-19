#![no_std]

#[inline(always)]
fn basic_checks(flag: &[u8]) -> bool {
    if flag.len() != 30 {
        return false
    }

    // '_' == '_' == '_'
    if flag[12] != flag[17] || flag[17] != flag[24] {
        return false
    }

    // 'l == 'l'
    if flag[20] != flag[19] {
        return false
    }

    // 'e' == 'e'
    if flag[8] != flag[10] || flag[10] != flag[14] || flag[14] != flag[15] {
        return false
    }

    // 'y' == 'y'
    if flag[5] != flag[28] {
        return false
    }

    /*// 'u' == 'u'
    if flag[23] != flag[7] {
        return false
    }*/

    // 'r' == 'r'
    if flag[7] != flag[11] || flag[11] != flag[25] {
        return false
    }

    true
}

// FLAG: FLAG{ysrever_mees_slliks_ruoy}
#[inline(always)]
fn check_flag(flag: &[u8]) -> bool {
    if !basic_checks(&flag) {
        return false
    }

    // '{' - '}' == 2
    if flag[29] - flag[4] != 2 {
        return false
    } 

    // "FLAG"
    for (idx, i) in [7, 13, 2, 8].iter().enumerate() {
        if flag[idx] != ('?' as u8) + i {
            return false
        }
    }

    // '_'
    if flag[24] > '`' as u8 {
        return false
    }

    // 'e' + 'l'
    if flag[14] + flag[19] - 59 != 150 || flag[19] - flag[14] != 7 {
        return false
    }

    // 's' == 's' == 's' == 's'
    if flag[6] ^ flag[16] != 0 || flag[18] & !flag[23] != 0 
    || flag[16] ^ !flag[18] != 255 || flag[18] - 4 != ('o' as u8) {
        return false
    }

    // 'v'
    if (2 * flag[9]) % (flag[23] as u8) != 6  {
        return false
    }

    // 'u'
    if (flag[26] + flag[9]) % ('e' as u8) != 33 {
        return false
    }

    // 'k', 'i', 'm'
    if 3 * (flag[22] as u32) + 2 * (flag[21] as u32) + flag[13] as u32 != 640 {
        return false
    }

    // 2 * 'o' % 'r' == 'l'
    if flag[27] % 35 != 6 || 2 * flag[27] % flag[7] != flag[19] {
        return false
    } 

    // use Fletcher's checksum, 5 6 Byte chunks
    for (idx, checksum) in [4206, 39207, 27243, 37899, 44301].iter().enumerate() {
        let mut x1 = 0u32;
        let mut x2 = 0u32;

        for c in &flag[idx*6..(idx*6+6)] {
            x1 = (x1 + *c as u32) % 255;
            x2 = (x2 + x1) % 255;
        }

        if *checksum != x1 * 256 + x2 {
            return false
        }
    }

    true
}

#[no_mangle]
pub unsafe extern fn upside_down_ep(flag: *const u8, size: usize) -> bool {
    check_flag(core::slice::from_raw_parts(flag, size))
}
