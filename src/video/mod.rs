use crate::fast_mem::FastAllocator;
use crate::file::SeekFrom;
use crate::util::get_timer;
use crate::{println, RomFile};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::global_asm;
use core::ptr;
use core::slice;
use core::iter;

mod bititer;
use bititer::BitIter;

fn write_dispcnt(value: u16) {
    unsafe {
        ptr::write_volatile(0x4000000 as *mut u16, value);
    }
}

#[derive(Debug)]
enum HuffmanTreeEntry {
    Leaf(u16),
    Internal {
        left: Box<Self, FastAllocator>,
        right: Box<Self, FastAllocator>,
    },
}

impl HuffmanTreeEntry {
    fn from_code_lengths(lengths: &[u8]) -> HuffmanTreeEntry {
        let mut root = HuffmanTreeEntry::Leaf(0);
        let mut length_count = [0; 16];
        for length in lengths {
            length_count[*length as usize] += 1;
        }
        length_count[0] = 0;

        let mut current_code = 0;
        let mut next_code_length = [0; 16];
        for bit in 1..16 {
            current_code = (current_code + length_count[bit - 1]) << 1;
            next_code_length[bit] = current_code;
        }

        for (i, length) in lengths.iter().enumerate() {
            if *length == 0 {
                continue;
            }
            root.replace_entry(next_code_length[*length as usize], *length, i as u16);
            next_code_length[*length as usize] += 1;
        }

        root
    }

    fn next_bit(&self, bit: bool) -> &HuffmanTreeEntry {
        match self {
            HuffmanTreeEntry::Leaf(_) => panic!("Invalid huffman code"),
            HuffmanTreeEntry::Internal { left, right } => {
                if !bit {
                    left
                } else {
                    right
                }
            }
        }
    }

    #[link_section = ".fast_text"]
    #[export_name = "hufman_tree_decode_from_bits"]
    extern "C" fn decode_from_bits(&self, bits: &mut BitIter) -> u16 {
        let mut decoder = self;

        while let HuffmanTreeEntry::Internal { .. } = decoder {
            decoder = decoder.next_bit(bits.next().unwrap());
        }

        *match decoder {
            HuffmanTreeEntry::Leaf(x) => x,
            _ => unreachable!(),
        }
    }

    fn replace_entry(&mut self, code: u16, in_code_len: u8, value: u16) {
        let mut current_entry = self;
        for code_len in (0..in_code_len).rev() {
            let bit = (code >> code_len) & 1;

            if matches!(current_entry, HuffmanTreeEntry::Leaf(_)) {
                *current_entry = HuffmanTreeEntry::Internal {
                    left: Box::new_in(HuffmanTreeEntry::Leaf(0), FastAllocator::new()),
                    right: Box::new_in(HuffmanTreeEntry::Leaf(0), FastAllocator::new()),
                };
            }

            match current_entry {
                HuffmanTreeEntry::Internal { left, right } => {
                    if bit == 0 {
                        current_entry = left;
                    } else {
                        current_entry = right;
                    }
                }
                _ => unreachable!(),
            }
        }

        *current_entry = HuffmanTreeEntry::Leaf(value)
    }
}

#[repr(C)]
struct BlockDecodeState {
    literal_code_lengths: Vec<u8>,
    distance_code_lengths: Vec<u8>,
    vram_addr: *mut u16,
    vram_data: u16,
    vram_parity: bool,
    last_block: bool,
}


extern "C" {
    // These pointers are treated as blackboxes in the asm
    #[allow(improper_ctypes)]
    fn decode_hufman_block_loop(state: &mut BlockDecodeState, bits: &mut BitIter, literal_tree: &HuffmanTreeEntry, distance_tree: &HuffmanTreeEntry);
}

global_asm!(".section \".fast_text\"",
            ".arm",
            ".type decode_hufman_block_loop, %function",
            ".align 4",
            ".global decode_hufman_block_loop",
            "decode_hufman_block_loop:",
            //R4 is state, R5 is bits, R6 is literal_tree, R7 is distance tree, 
            //R8 is the const 265, R9 is scratch, R10 is huffman_tree_decode, R11 is vram_write
            "PUSH {{R4, R5, R6, R7, R8, R9, R10, R11, R14}}",
            "MOV R4, R0",
            "MOV R5, R1",
            "MOV R6, R2",
            "MOV R7, R3",
            "MOV R8, #256",
            "ADD R8, R8, 9",
            "LDR R10, hufman_decode_loc",
            "LDR R11, vram_write_loc",
            "main_loop:",
            "MOV R0, R6",
            "MOV R1, R5",
            "ADR R14, after_decode",
            "BX R10",
            "after_decode:",
            "TST R0, #0xFF00",
            "BNE not_literal",
            "MOV R1, R0",
            "MOV R0, R4",
            "ADR R14, main_loop",
            "BX R11",
            "not_literal:",
            "SUBS R0, R0, R8",
            "BMI edge_case",
            "CMP R0, #20",
            "BEQ big_case",
            "MOV R1, R0,LSR#2",
            "ADD R1, #1",
            "MOV R2, R1",
            "ADD R2, #2",
            "MOV R3, #1",
            "MOV R9, R3, LSL R2",
            "AND R0, #0x3",
            "ADD R9, R0, LSL R1",
            "MOV R0, R5",
            "ADR R14, after_bititer",
            "LDR R3, bititer_loc",
            "BX R3",
            "after_bititer:",
            "ADD R3, R9, R0",
            "ADD R3, #3",
            "MOV R0, R4",
            "MOV R1, R5",
            "MOV R2, R7",
            "ADR R14, main_loop",
            "LDR R9, lz77_copy_loc",
            "BX R9",
            "edge_case:",
            "CMN R0, #9",
            "BEQ end",
            "ADD R3, R0, #11",
            "MOV R0, R4",
            "MOV R1, R5",
            "MOV R2, R7",
            "ADR R14, main_loop",
            "LDR R9, lz77_copy_loc",
            "BX R9",
            "big_case:",
            "ADD R3, R0, #238",
            "MOV R0, R4",
            "MOV R1, R5",
            "MOV R2, R7",
            "ADR R14, main_loop",
            "LDR R9, lz77_copy_loc",
            "BX R9",
            "end:",
            "POP {{R4, R5, R6, R7, R8, R9, R10, R11, R14}}",
            "BX R14",
            "hufman_decode_loc:",
            ".word hufman_tree_decode_from_bits + 1",
            "vram_write_loc:",
            ".word write_to_vram + 1",
            "lz77_copy_loc:",
            ".word lz77_copy + 1",
            "bititer_loc:",
            ".word bititer_take_into_u16 + 1");

#[link_section = ".fast_text"]
fn decode_hufman_block(state: &mut BlockDecodeState, bits: &mut BitIter) {
    let literal_hufman_tree = HuffmanTreeEntry::from_code_lengths(&state.literal_code_lengths);
    let distance_hufman_tree = HuffmanTreeEntry::from_code_lengths(&state.distance_code_lengths);

    unsafe { decode_hufman_block_loop(state, bits, &literal_hufman_tree, &distance_hufman_tree); }
}

#[link_section = ".fast_text"]
#[no_mangle]
extern "C" fn lz77_copy(
    state: &mut BlockDecodeState,
    bits: &mut BitIter,
    distance_tree: &HuffmanTreeEntry,
    length: u16,
) {
    let distance_sym = distance_tree.decode_from_bits(bits);

    let distance = if (0..=3).contains(&distance_sym) {
        distance_sym
    } else {
        let offset_sym = distance_sym - 4;
        let extra_bits_len = (offset_sym >> 1) + 1;
        let extra_bits = bits.take_into_u16(extra_bits_len as u8);
        let start_distance = 1 << (extra_bits_len + 1);

        let distance = (offset_sym & 1) << (extra_bits_len);
        distance | extra_bits | start_distance
    } + 1;

    // Due to the need to write two bytes at a time distance of 1 needs special handeling
    if distance == 1 {
        let rept_byte = if state.vram_parity {
            state.vram_data as u8
        } else {
            unsafe {
                let loc = (state.vram_addr as *const u8).sub(1);
                ptr::read_volatile(loc)
            }
        };

        for _ in 0..length {
            write_to_vram(state, rept_byte);
        }
    } else {
        for _ in 0..length {
            unsafe {
                let real_distance = (distance as usize) - (state.vram_parity as usize);
                let loc = (state.vram_addr as *const u8).sub(real_distance);
                let value = ptr::read_volatile(loc);
                write_to_vram(state, value);
            }
        }
    }
}

#[link_section = ".fast_text"]
#[no_mangle]
extern "C" fn write_to_vram(state: &mut BlockDecodeState, in_value: u8) {
    let value = in_value as u16;
    if !state.vram_parity {
        state.vram_data = value;
    } else {
        state.vram_data |= value << 8;
        unsafe {
            ptr::write_volatile(state.vram_addr, state.vram_data);
            state.vram_addr = state.vram_addr.offset(1)
        }
    }
    state.vram_parity = !state.vram_parity;
}

static CODE_LENGTH_LENGTH_ORDER: &[u8] = &[
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn handle_len_len(bits: &mut BitIter, len_tree: &HuffmanTreeEntry, dest: &mut Vec<u8>) {
    let len_symbol = len_tree.decode_from_bits(bits) as u8;

    match len_symbol {
        0..=15 => dest.push(len_symbol),
        16 => {
            let last_len = *dest.last().unwrap();

            let repeat_count = bits.take_into_u8(2) + 3;

            for _ in 0..repeat_count {
                dest.push(last_len);
            }
        }
        17 => {
            let repeat_count = bits.take_into_u8(3) + 3;

            for _ in 0..repeat_count {
                dest.push(0);
            }
        }
        18 => {
            let repeat_count = bits.take_into_u8(7) + 11;

            for _ in 0..repeat_count {
                dest.push(0);
            }
        }
        _ => unreachable!(),
    }
}

pub fn display_bitmap_file(mut file: RomFile) {
    write_dispcnt(0xF03);
    let begin_time = get_timer();

    let mut state = BlockDecodeState {
        vram_data: 0,
        vram_parity: false,
        vram_addr: 0x6000000 as *mut u16,
        last_block: false,
        literal_code_lengths: Vec::with_capacity(288),
        distance_code_lengths: Vec::with_capacity(32),
    };

    file.seek(SeekFrom::Start(0)).unwrap();
    let file_bytes = file.as_bytes();

    let mut bits = unsafe { BitIter::new(file_bytes as *const _ as *const u8 as *const u32) };

    while !state.last_block {
        state.literal_code_lengths.clear();
        state.distance_code_lengths.clear();

        state.last_block = bits.next().unwrap();
        let block_type = bits.take_into_u8(2);

        match block_type {
            0 => {
                bits.skip_to_byte_start();
                let length = bits.take_into_usize(16);
                bits.take_into_usize(16);

                for _ in 0..length {
                    let next_byte = bits.take_into_u8(8);
                    write_to_vram(&mut state, next_byte);
                }
            }
            1 => {
                for _ in 0..=143 {
                    state.literal_code_lengths.push(8);
                }

                for _ in 144..=255 {
                    state.literal_code_lengths.push(9);
                }

                for _ in 256..=279 {
                    state.literal_code_lengths.push(7);
                }

                for _ in 280..=287 {
                    state.literal_code_lengths.push(8);
                }

                for _ in 0..=31 {
                    state.distance_code_lengths.push(5);
                }

                decode_hufman_block(&mut state, &mut bits);
            }
            2 => {
                let literal_len = bits.take_into_usize(5) + 257;
                let distance_len = bits.take_into_usize(5) + 1;
                let len_len = bits.take_into_usize(4) + 4;

                let mut len_code_len = [0; 19];
                for index in CODE_LENGTH_LENGTH_ORDER.iter().take(len_len) {
                    let len = bits.take_into_u8(3);

                    len_code_len[*index as usize] = len;
                }

                let len_tree = HuffmanTreeEntry::from_code_lengths(&len_code_len);

                while state.literal_code_lengths.len() != literal_len {
                    handle_len_len(&mut bits, &len_tree, &mut state.literal_code_lengths);
                }

                while state.distance_code_lengths.len() != distance_len {
                    handle_len_len(&mut bits, &len_tree, &mut state.distance_code_lengths);
                }

                decode_hufman_block(&mut state, &mut bits);
            }
            _ => panic!("Unknown deflate block type"),
        }
    }

    let end_time = get_timer();

    println!("Took {} timer overflows", end_time - begin_time);
}
