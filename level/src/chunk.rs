use std::marker::PhantomData;

use miners::encoding::{Encode, Decode};

/// A chunk column, not including heightmaps
/// N = amount of chunk sections (depends on world height)
/// SV = type used to represent the block states
/// BV = type used to represent the biomes
/// S = the data container used to store the states
/// B: the data container used to store the biomes
pub struct ChunkColumn<const N: usize, SV, BV, S: DataContainer<4096, SV>, B: DataContainer<64, BV>> {
    //pub motion_blocking: PackedBits<256>, // len = 256 bits = 9
    pub sections: [Option<ChunkSection<SV, BV, S, B>>; N],
}

/// A 16 * 16 * 16 section of a chunk.
pub struct ChunkSection<SV, BV, S: DataContainer<4096, SV>, B: DataContainer<64, BV>> {
    pub block_count: u16,
    pub states: S,
    pub biomes: B,
    __marker: PhantomData<SV>,
    __marker2: PhantomData<BV>
}

impl<SV, BV, S: DataContainer<4096, SV>, B: DataContainer<64, BV>> Encode for ChunkSection<SV, BV, S, B> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners::encoding::encode::Result<()> {
        self.block_count.encode(writer)?;
        self.states.encode(writer)?;
        self.biomes.encode(writer)
    }
}

impl<SV, BV, S: DataContainer<4096, SV>, B: DataContainer<64, BV>> Decode<'_> for ChunkSection<SV, BV, S, B> {
    fn decode(cursor: &mut std::io::Cursor<&'_ [u8]>) -> miners::encoding::decode::Result<Self> {
        Ok(
            Self {
                block_count: u16::decode(cursor)?,
                states: S::decode(cursor)?,
                biomes: B::decode(cursor)?,
                __marker: PhantomData,
                __marker2: PhantomData
            }
        )
    }
}


pub unsafe trait DataContainer<const N: usize, V>: Encode + for<'dec> Decode<'dec> {
    fn get(&self, i: usize) -> V {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we know i is in bounds.
        unsafe { self.get_unchecked(i) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    unsafe fn get_unchecked(&self, i: usize) -> V;

    fn set(&mut self, i: usize, v: V) {
        if i >= N {
            panic!("out of bounds")
        }
        // SAFETY: This is sound because we just checked the bounds
        unsafe { self.set_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    unsafe fn set_unchecked(&mut self, i: usize, v: V);

    fn swap(&mut self, i: usize, v: V) -> V {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we just checked the bounds.
        unsafe { self.swap_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds
    unsafe fn swap_unchecked(&mut self, i: usize, v: V) -> V {
        let val = self.get_unchecked(i);
        self.set_unchecked(i, v);
        val
    }
}

unsafe impl<const N: usize, T: super::palette::PaletteContainer<N> + Encode + for<'dec> Decode<'dec>> DataContainer<N, u16> for T {
    unsafe fn get_unchecked(&self, i: usize) -> u16 {
        self.get_unchecked(i)
    }
    unsafe fn set_unchecked(&mut self, i: usize, v: u16) {
        self.set_unchecked(i, v)
    }
}
