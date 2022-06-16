#[repr(C)]
pub struct BitIter {
    front: u32,
    source: *const u32,
    front_left: u8,
}

macro_rules! take_into_impl {
    ( $name: ident, $c_name: expr, $type: ty) => {
        #[link_section = ".fast_text"]
        #[export_name = $c_name]
        pub extern "C" fn $name(&mut self, n: u8) -> $type {
            assert!(n as u32 <= <$type>::BITS);

            let mut ret = 0;
            for i in 0..n {
                let bit = self.next().unwrap();
                ret |= (bit as $type) << i;
            }
            ret
        }
    };
}

impl BitIter {
    pub unsafe fn new(source: *const u32) -> Self {
        Self {
            front: 0,
            source,
            front_left: 0,
        }
    }

    pub fn skip_to_byte_start(&mut self) {
        let rest_of_byte = self.front_left % 8;
        self.take_into_usize(rest_of_byte);
    }

    take_into_impl!(take_into_u8, "bititer_take_into_u8", u8);
    take_into_impl!(take_into_u16, "bititer_take_into_u16", u16);
    take_into_impl!(take_into_usize, "bititer_take_into_usize", usize);
}

impl Iterator for BitIter {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.front_left == 0 {
            unsafe {
                self.front = *self.source;
                self.source = self.source.offset(1);
            }
            self.front_left = 32;
        }

        let ret = self.front & 1 == 1;
        self.front_left -= 1;
        self.front >>= 1;
        Some(ret)
    }
}
