use paste::paste;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::ptr::{self, NonNull};

/// 一个代表寄存器的类型。
///
/// # 地址范围
///
/// * `0x0000 ~ 0x1FFF` - 单比特值。
/// * `0x2000 ~ 0x7FFF` - 8 位整数值。
/// * `0x8000 ~ 0xBFFF` - 16 位整数值。
/// * `0xC000 ~ 0xDFFF` - 32 位整数、浮点值。
/// * `0xE000 ~ 0xFFFF` - 64 位整数、浮点值。
struct ModBusRegisters {
    mem: NonNull<u8>,
}

macro_rules! impl_bits {
    ($t:ty, $n:expr) => {
        paste! {
            pub fn [<get_ $t>](&self, reg: u16) -> $t {
                unsafe {
                    if reg < Self::[<BITS_ $n _REG_MIN>] || reg > Self::[<BITS_ $n _REG_MAX>] {
                        return 0;
                    }
                    let offset = reg - Self::[<BITS_ $n _REG_MIN>] + Self::[<BITS_ $n _REG_OFS>];
                    let val_ptr = self.mem.cast::<$t>().as_ptr().offset(offset as isize);
                    *val_ptr
                }
            }

            pub fn [<get_ $t _values>](&self, reg: u16, num: u16) -> &[$t] {
                unsafe {
                    if reg < Self::[<BITS_ $n _REG_MIN>] || reg > Self::[<BITS_ $n _REG_MAX>] || (reg + num) > Self::[<BITS_ $n _REG_MAX>] {
                        return &[];
                    }
                    let offset = reg - Self::[<BITS_ $n _REG_MIN>] + Self::[<BITS_ $n _REG_OFS>];
                    let val_ptr = self.mem.cast::<$t>().as_ptr().offset(offset as isize);
                    std::slice::from_raw_parts(val_ptr, num as usize)
                }
            }

            pub fn [<set_ $t>](&self, reg: u16, val: $t) {
                unsafe {
                    if reg >= Self::[<BITS_ $n _REG_MIN>] && reg <= Self::[<BITS_ $n _REG_MAX>] {
                        let offset = reg - Self::[<BITS_ $n _REG_MIN>] + Self::[<BITS_ $n _REG_OFS>];
                        let val_ptr = self.mem.cast::<$t>().as_ptr().offset(offset as isize);
                        *val_ptr = val;
                    }
                }
            }

            pub fn [<set_ $t _values>](&self, reg: u16, values: &[$t]) {
                unsafe {
                    if reg >= Self::[<BITS_ $n _REG_MIN>] && reg <= Self::[<BITS_ $n _REG_MAX>] && (reg + values.len() as u16) <= Self::[<BITS_ $n _REG_MAX>] {
                        let offset = reg - Self::[<BITS_ $n _REG_MIN>] + Self::[<BITS_ $n _REG_OFS>];
                        let val_ptr = self.mem.cast::<$t>().as_ptr().offset(offset as isize);
                        std::ptr::copy(values.as_ptr(), val_ptr, values.len());
                    }
                }
            }
        }
    }
}

impl Registers {
    pub const BIT_REG_MIN: u16 = 0x0000;
    pub const BIT_REG_MAX: u16 = 0x1FFF;
    pub const BIT_REG_NUM: u16 = 0x2000;

    pub const BITS_8_REG_MIN: u16 = 0x2000;
    pub const BITS_8_REG_MAX: u16 = 0x7FFF;
    pub const BITS_8_REG_NUM: u16 = 0x6000;

    pub const BITS_16_REG_MIN: u16 = 0x8000;
    pub const BITS_16_REG_MAX: u16 = 0xBFFF;
    pub const BITS_16_REG_NUM: u16 = 0x4000;

    pub const BITS_32_REG_MIN: u16 = 0xC000;
    pub const BITS_32_REG_MAX: u16 = 0xDFFF;
    pub const BITS_32_REG_NUM: u16 = 0x2000;

    pub const BITS_64_REG_MIN: u16 = 0xE000;
    pub const BITS_64_REG_MAX: u16 = 0xFFFF;
    pub const BITS_64_REG_NUM: u16 = 0x2000;

    const BITS_8_REG_OFS: u16 = Self::BIT_REG_NUM / 8;
    const BITS_16_REG_OFS: u16 = Self::BITS_8_REG_OFS + Self::BITS_8_REG_NUM;
    const BITS_32_REG_OFS: u16 = Self::BITS_16_REG_OFS + Self::BITS_16_REG_NUM;
    const BITS_64_REG_OFS: u16 = Self::BITS_32_REG_OFS + Self::BITS_32_REG_NUM;

    fn new() -> Self {
        unsafe {
            let layout = Layout::from_size_align_unchecked(1024 * 1024 * 1024, 4096);
            let ptr = alloc_zeroed(layout);
            Self {
                mem: NonNull::new_unchecked(ptr),
            }
        }
    }

    pub fn get_bit(&self, reg: u16) -> bool {
        unsafe {
            const N: u16 = std::mem::size_of::<usize>() as u16;
            if reg > Self::BIT_REG_MAX {
                return false;
            }
            let offset = reg / N;
            let val: usize = self
                .mem
                .cast::<usize>()
                .as_ptr()
                .offset(offset as isize)
                .read();
            val & (1 << (reg % N)) != 0
        }
    }

    pub fn clear_bit(&self, reg: u16) {
        unsafe {
            const N: u16 = std::mem::size_of::<usize>() as u16;
            if reg <= Self::BIT_REG_MAX {
                let offset = reg / N;
                let val_ptr = self.mem.cast::<usize>().as_ptr().offset(offset as isize);
                *val_ptr &= !(1 << (reg % N));
            }
        }
    }

    pub fn set_bit(&self, reg: u16) {
        unsafe {
            const N: u16 = std::mem::size_of::<usize>() as u16;
            if reg <= Self::BIT_REG_MAX {
                let offset = reg / N;
                let val_ptr = self.mem.cast::<usize>().as_ptr().offset(offset as isize);
                *val_ptr |= 1 << (reg % N);
            }
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

    // pub fn get_u8(&self, reg: u16) -> u8 {
    //     unsafe {
    //         if reg < Self::BITS_8_REG_MIN || reg > Self::BITS_8_REG_MAX {
    //             return 0;
    //         }
    //         let offset = reg - Self::BITS_8_REG_MIN + Self::BITS_8_REG_OFS;
    //         let val_ptr = self.mem.as_ptr().offset(offset as isize);
    //         *val_ptr
    //     }
    // }

    // pub fn set_u8(&self, reg: u16, val: u8) {
    //     unsafe {
    //         if reg >= Self::BITS_8_REG_MIN && reg <= Self::BITS_8_REG_MAX {
    //             let offset = reg - Self::BITS_8_REG_MIN + Self::BITS_8_REG_OFS;
    //             let val_ptr = self.mem.as_ptr().offset(offset as isize);
    //             *val_ptr = val;
    //         }
    //     }
    // }

    // pub fn get_u16(&self, reg: u16) -> u16 {
    //     unsafe {
    //         if reg < Self::BITS_16_REG_MIN || reg >= Self::BITS_16_REG_MAX {
    //             return 0;
    //         }
    //         let offset = reg - Self::BITS_16_REG_MIN + Self::BITS_16_REG_OFS;
    //         let val_ptr = self.mem.cast::<u16>().as_ptr().offset(offset as isize);
    //         *val_ptr
    //     }
    // }

    // pub fn set_u16(&self, reg: u16, val: u16) {
    //     unsafe {
    //         if reg >= Self::BITS_16_REG_MIN && reg < Self::BITS_16_REG_MAX {
    //             let offset = reg - Self::BITS_16_REG_MIN + Self::BITS_16_REG_OFS;
    //             let val_ptr = self.mem.cast::<u16>().as_ptr().offset(offset as isize);
    //             *val_ptr = val;
    //         }
    //     }
    // }

    // pub fn get_u32(&self, reg: u16) -> u32 {
    //     unsafe {
    //         if reg < Self::BITS_32_REG_MIN || reg >= Self::BITS_32_REG_MAX {
    //             return 0;
    //         }
    //         let offset = reg - Self::BITS_32_REG_MIN + Self::BITS_32_REG_OFS;
    //         let val_ptr = self.mem.cast::<u32>().as_ptr().offset(offset as isize);
    //         *val_ptr
    //     }
    // }

    // pub fn set_u32(&self, reg: u16, val: u32) {
    //     unsafe {
    //         if reg >= Self::BITS_32_REG_MIN && reg < Self::BITS_32_REG_MAX {
    //             let offset = reg - Self::BITS_32_REG_MIN + Self::BITS_32_REG_OFS;
    //             let val_ptr = self.mem.cast::<u32>().as_ptr().offset(offset as isize);
    //             *val_ptr = val;
    //         }
    //     }
    // }

    // pub fn get_u64(&self, reg: u16) -> u64 {
    //     unsafe {
    //         if reg < Self::BITS_64_REG_MIN || reg >= Self::BITS_64_REG_MAX {
    //             return 0;
    //         }
    //         let offset = reg - Self::BITS_64_REG_MIN + Self::BITS_64_REG_OFS;
    //         let val_ptr = self.mem.cast::<u64>().as_ptr().offset(offset as isize);
    //         *val_ptr
    //     }
    // }

    // pub fn set_u64(&self, reg: u16, val: u64) {
    //     unsafe {
    //         if reg >= Self::BITS_64_REG_MIN && reg < Self::BITS_64_REG_MAX {
    //             let offset = reg - Self::BITS_64_REG_MIN + Self::BITS_64_REG_OFS;
    //             let val_ptr = self.mem.cast::<u64>().as_ptr().offset(offset as isize);
    //             *val_ptr = val;
    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits() {
        let regs = Registers::new();
        for a in Registers::BIT_REG_MIN..=Registers::BIT_REG_MAX {
            regs.set_bit(a);
            assert_eq!(regs.get_bit(a), true);
            regs.clear_bit(a);
            assert_eq!(regs.get_bit(a), false);
        }
        for a in Registers::BITS_8_REG_MIN..=0xFFFF {
            regs.set_bit(a);
            assert_eq!(regs.get_bit(a), false);
        }
    }

    #[test]
    fn bits_8() {
        let regs = Registers::new();
        for a in Registers::BITS_8_REG_MIN..=Registers::BITS_8_REG_MAX {
            regs.set_u8(a, 0xAA);
            assert_eq!(regs.get_u8(a), 0xAA);
        }
        for a in 0x0000..Registers::BITS_8_REG_MIN {
            regs.set_u8(a, 0xAA);
            assert_eq!(regs.get_u8(a), 0x00);
        }
        for a in Registers::BITS_16_REG_MIN..0xFFFF {
            regs.set_u8(a, 0xAA);
            assert_eq!(regs.get_u8(a), 0x00);
        }
    }
}

fn main() {
    let regs = Registers::new();
    regs.set_bit(0x0000);
    println!("0x0000={}", regs.get_bit(0x0000));
    println!("0x0010={}", regs.get_bit(0x1000));
    regs.set_bit(0x010);
    println!("0x0010={}", regs.get_bit(0x1000));
    regs.set_i16_values(
        Registers::BITS_16_REG_MIN + 0x0010,
        &[1, 2, 3, 4, 5, 6, 7, 8, 9],
    );
    let s = regs.get_i16_values(Registers::BITS_16_REG_MIN + 0x0010, 16);
    println!("{:?}", s);
}
