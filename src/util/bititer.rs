pub struct BitIter<T: Iterator<Item = u8>> {
    front_byte: u8,
    front_byte_left: u8,
    iter: T,
}

macro_rules! take_into_impl {
    ( $name: ident, $type: ty) => {
        pub fn $name(&mut self, n: u8) -> Result<$type, usize> {
            assert!(n as u32 <= <$type>::BITS);

            let mut ret = 0;
            for i in 0..n {
                if let Some(bit) = self.next() {
                    ret |= (bit as $type) << i;
                } else {
                    return Err(i as usize);
                }
            }
            Ok(ret)
        }
    };
}

impl<T: Iterator<Item = u8>> BitIter<T> {
    pub fn new<I: IntoIterator<IntoIter = T>>(iter: I) -> Self {
        Self {
            front_byte: 0,
            front_byte_left: 0,
            iter: iter.into_iter(),
        }
    }

    pub fn skip_to_byte_start(&mut self) -> &mut T {
        self.front_byte_left = 0;
        &mut self.iter
    }

    take_into_impl!(take_into_u8, u8);
    take_into_impl!(take_into_u16, u16);
    take_into_impl!(take_into_usize, usize);
}

impl<T: Iterator<Item = u8>> Iterator for BitIter<T> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.front_byte_left == 0 {
            if let Some(byte) = self.iter.next() {
                self.front_byte_left = 8;
                self.front_byte = byte;
            } else {
                return None;
            }
        }

        let ret = self.front_byte & 1 == 1;
        self.front_byte_left -= 1;
        self.front_byte >>= 1;
        Some(ret)
    }
}

