#![allow(dead_code)]

use paste::paste;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::ptr::{self, NonNull};

/// 一个代表多种值存储表的类型。
///
/// # Examples
///
/// ```
/// use xvt::ValueTable;
/// 
/// let vt = ValueTable::new();
/// vt.set_bit(0x1000);
/// assert_eq!(vt.get_bit(0x1000), true);
/// ```
pub struct ValueTable {
    mem: NonNull<u8>,
}

macro_rules! impl_bits {
    ($t:ty, $n:expr) => {
        paste! {
            #[doc = "获取指定地址 `addr` 类型为 `" $t "` 的值。"]
            pub fn [<get_ $t>](&self, addr: u16) -> $t {
                unsafe {
                    let ofs = Self::[<BITS_ $n _REG_OFS>] as isize;
                    let val_ptr = self.mem.as_ptr().offset(ofs).cast::<$t>().offset(addr as isize);
                    *val_ptr
                }
            }

            #[doc = "获取指定地址 `addr` 类型为 `" $t "` 的 `num` 个值。"]
            pub fn [<get_ $t s>](&self, addr: u16, num: u16) -> &[$t] {
                unsafe {
                    let ofs = Self::[<BITS_ $n _REG_OFS>] as isize;
                    let val_ptr = self.mem.as_ptr().offset(ofs).cast::<$t>().offset(addr as isize);
                    let m = 65536 - addr as usize;
                    std::slice::from_raw_parts(val_ptr, m.min(num as usize))
                }
            }

            #[doc = "设置指定地址 `addr` 类型为 `" $t "` 的值。"]
            pub fn [<set_ $t>](&self, addr: u16, val: $t) {
                unsafe {
                    let ofs = Self::[<BITS_ $n _REG_OFS>] as isize;
                    let val_ptr = self.mem.as_ptr().offset(ofs).cast::<$t>().offset(addr as isize);
                    *val_ptr = val;
                }
            }

            #[doc = "设置指定地址 `addr` 类型为 `" $t "` 的多个值。"]
            pub fn [<set_ $t s>](&self, addr: u16, vals: &[$t]) {
                unsafe {
                    let ofs = Self::[<BITS_ $n _REG_OFS>] as isize;
                    let val_ptr = self.mem.as_ptr().offset(ofs).cast::<$t>().offset(addr as isize);
                    let m = 65536 - addr as usize;
                    let n = vals.len();
                    std::ptr::copy(vals.as_ptr(), val_ptr, m.min(n));
                }
            }
        }
    };
}

impl ValueTable {
    const BITS_8_REG_OFS: usize = 8192;
    const BITS_16_REG_OFS: usize = Self::BITS_8_REG_OFS + 65536;
    const BITS_32_REG_OFS: usize = Self::BITS_16_REG_OFS + 131072;
    const BITS_64_REG_OFS: usize = Self::BITS_32_REG_OFS + 262144;

    /// 构建一个多种值存储表实例。
    pub fn new() -> Self {
        unsafe {
            let layout = Layout::from_size_align_unchecked(1024 * 1024, 4096);
            let ptr = alloc_zeroed(layout);
            Self {
                mem: NonNull::new_unchecked(ptr),
            }
        }
    }

    /// 获取指定地址 `addr` 的单比特值。
    pub fn get_bit(&self, addr: u16) -> bool {
        unsafe {
            const N: u16 = std::mem::size_of::<usize>() as u16;
            let offset = addr / N;
            let val: usize = self
                .mem
                .cast::<usize>()
                .as_ptr()
                .offset(offset as isize)
                .read();
            val & (1 << (addr % N)) != 0
        }
    }

    /// 清除指定地址 `addr` 的单比特值。
    pub fn clear_bit(&self, addr: u16) {
        unsafe {
            const N: u16 = std::mem::size_of::<usize>() as u16;
            let offset = addr / N;
            let val_ptr = self.mem.cast::<usize>().as_ptr().offset(offset as isize);
            *val_ptr &= !(1 << (addr % N));
        }
    }

    /// 设置指定地址 `addr` 的单比特值。
    pub fn set_bit(&self, addr: u16) {
        unsafe {
            const N: u16 = std::mem::size_of::<usize>() as u16;
            let offset = addr / N;
            let val_ptr = self.mem.cast::<usize>().as_ptr().offset(offset as isize);
            *val_ptr |= 1 << (addr % N);
        }
    }

    impl_bits!(i8, 8);
    impl_bits!(i16, 16);
    impl_bits!(i32, 32);
    impl_bits!(i64, 64);
    impl_bits!(u8, 8);
    impl_bits!(u16, 16);
    impl_bits!(u32, 32);
    impl_bits!(u64, 64);
}

impl Drop for ValueTable {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(1024 * 1024, 4096);
            dealloc(self.mem.as_ptr(), layout);
        }
    }
}

unsafe impl Send for ValueTable {}
unsafe impl Sync for ValueTable {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits() {
        let regs = ValueTable::new();
        for a in u16::MIN..=u16::MAX {
            regs.set_bit(a);
            assert_eq!(regs.get_bit(a), true);
            regs.clear_bit(a);
            assert_eq!(regs.get_bit(a), false);
        }
    }

    #[test]
    fn bits_8() {
        let regs = ValueTable::new();
        for a in u16::MIN..=u16::MAX {
            regs.set_u8(a, 0xAA);
            assert_eq!(regs.get_u8(a), 0xAA);
        }
    }
}

// fn main() {
//     let regs = Registers::new();
//     regs.set_bit(0x0000);
//     println!("0x0000={}", regs.get_bit(0x0000));
//     println!("0x0010={}", regs.get_bit(0x1000));
//     regs.set_bit(0x010);
//     println!("0x0010={}", regs.get_bit(0x1000));
//     regs.set_i16_values(
//         Registers::BITS_16_REG_MIN + 0x0010,
//         &[1, 2, 3, 4, 5, 6, 7, 8, 9],
//     );
//     let s = regs.get_i16_values(Registers::BITS_16_REG_MIN + 0x0010, 16);
//     println!("{:?}", s);
// }
