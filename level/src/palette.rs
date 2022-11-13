use crate::bitpack::PackedBits;
use std::collections::BTreeMap;

// TODO: Reduce code duplication (with macros?)

pub struct BiomePaletteContainer<const N: usize> {
    palette: BiomePalette<N>,
}

enum BiomePalette<const N: usize> {
    SingleValue(SingleValuePalette),
    Linear {
        palette: LinearPalette,
        data: PackedBits<N>,
    },
}

impl<const N: usize> BiomePaletteContainer<N> {
    #[inline]
    pub fn new(value: u64) -> Self {
        Self {
            palette: BiomePalette::SingleValue(SingleValuePalette(value)),
        }
    }

    pub fn with_bits(bits: usize, value: u64) -> Self {
        if bits > 3 {
            panic!("bits cannot exceed 3")
        }
        //SAFETY: This is safe because we just checked that bits is not greater than 3.
        unsafe { Self::with_bits_unchecked(bits, value) }
    }

    /// # Safety
    /// This method is safe as long as `bits` is not greater than 3.
    pub unsafe fn with_bits_unchecked(bits: usize, value: u64) -> Self {
        match bits {
            0 => Self::new(value),
            // Here we assume bits is 1, 2, or 3
            bits => {
                let mut values = Vec::new();
                values.reserve_exact(2usize.pow(bits as u32));
                let palette = LinearPalette { bits, values };
                Self {
                    palette: BiomePalette::Linear {
                        palette,
                        data: PackedBits::new_unchecked(bits),
                    },
                }
            }
        }
    }
}

impl<const N: usize> BiomePaletteContainer<N> {
    pub fn get(&mut self, i: usize) -> u64 {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we know i is in bounds.
        unsafe { self.get_unchecked(i) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    pub unsafe fn get_unchecked(&mut self, i: usize) -> u64 {
        match &mut self.palette {
            BiomePalette::SingleValue(v) => v.0,
            BiomePalette::Linear { palette, data } => palette.value(data.get_unchecked(i) as usize),
        }
    }

    pub fn set(&mut self, i: usize, v: u64) {
        if i >= N {
            panic!("out of bounds")
        }
        // SAFETY: This is sound because we just checked the bounds
        unsafe { self.set_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    pub unsafe fn set_unchecked(&mut self, i: usize, v: u64) {
        loop {
            match &mut self.palette {
                BiomePalette::SingleValue(val) => match val.index(v) {
                    IndexOrBits::Index(_) => return,
                    IndexOrBits::Bits(bits) => {
                        let mut values = Vec::new();
                        values.reserve_exact(2);
                        values.push(val.0);
                        let palette = BiomePalette::Linear {
                            palette: LinearPalette { bits, values },
                            data: PackedBits::new(1),
                        };
                        self.palette = palette
                    }
                },
                BiomePalette::Linear { palette, data } => match palette.index(v) {
                    IndexOrBits::Index(v) => return data.set_unchecked(i, v),
                    IndexOrBits::Bits(bits) => {
                        if bits > 3 {
                            panic!("bits cannot exceed 3")
                        }
                        let mut values = std::mem::take(&mut palette.values);
                        values.reserve_exact(values.capacity());
                        data.change_bits(bits);

                        let data = std::mem::take(data);

                        let palette = BiomePalette::Linear {
                            palette: LinearPalette { bits, values },
                            data,
                        };

                        self.palette = palette
                    }
                },
            }
        }
    }

    pub fn swap(&mut self, i: usize, v: u64) -> u64 {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we just checked the bounds.
        unsafe { self.swap_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds
    pub unsafe fn swap_unchecked(&mut self, i: usize, v: u64) -> u64 {
        let val = self.get_unchecked(i);
        self.set_unchecked(i, v);
        val
    }
}

pub struct StatePaletteContainer<const N: usize> {
    palette: StatePalette<N>,
}

enum StatePalette<const N: usize> {
    SingleValue(SingleValuePalette),
    Linear {
        palette: LinearPalette,
        data: PackedBits<N>,
    },
    Mapped {
        palette: MappedPalette,
        data: PackedBits<N>,
    },
    Global {
        data: PackedBits<N>,
    },
}

impl<const N: usize> StatePaletteContainer<N> {
    #[inline]
    pub fn new(value: u64) -> Self {
        Self {
            palette: StatePalette::SingleValue(SingleValuePalette(value)),
        }
    }

    pub fn with_bits(bits: usize, value: u64) -> Self {
        match bits {
            0 => Self::new(value),
            1..=4 => {
                let mut values = Vec::new();
                values.reserve_exact(2usize.pow(4));
                let palette = LinearPalette { bits: 4, values };
                Self {
                    palette: StatePalette::Linear {
                        palette,
                        data: PackedBits::new_unchecked(4),
                    },
                }
            }
            5..=8 => {
                let mut values = Vec::new();
                values.reserve_exact(2usize.pow(bits as u32));
                let palette = LinearPalette { bits, values };
                let palette = MappedPalette {
                    indices: BTreeMap::new(),
                    inner: palette,
                };
                Self {
                    palette: StatePalette::Mapped {
                        palette,
                        data: PackedBits::new_unchecked(bits),
                    },
                }
            }
            _ => Self {
                palette: StatePalette::Global {
                    data: PackedBits::new(bits),
                },
            },
        }
    }
}

impl<const N: usize> StatePaletteContainer<N> {
    pub fn get(&mut self, i: usize) -> u64 {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we know i is in bounds.
        unsafe { self.get_unchecked(i) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    pub unsafe fn get_unchecked(&mut self, i: usize) -> u64 {
        match &mut self.palette {
            StatePalette::SingleValue(v) => v.0,
            StatePalette::Linear { palette, data } => palette.value(data.get_unchecked(i) as usize),
            StatePalette::Mapped { palette, data } => palette.value(data.get_unchecked(i) as usize),
            StatePalette::Global { data } => u64::from(data.get_unchecked(i)),
        }
    }

    pub fn set(&mut self, i: usize, v: u64) {
        if i >= N {
            panic!("out of bounds")
        }
        // SAFETY: This is sound because we just checked the bounds
        unsafe { self.set_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    pub unsafe fn set_unchecked(&mut self, i: usize, v: u64) {
        loop {
            match &mut self.palette {
                StatePalette::SingleValue(val) => match val.index(v) {
                    IndexOrBits::Index(_) => return,
                    IndexOrBits::Bits(_) => {
                        let mut values = Vec::new();
                        values.reserve_exact(2usize.pow(4));
                        values.push(val.0);
                        let palette = StatePalette::Linear {
                            palette: LinearPalette { bits: 4, values },
                            data: PackedBits::new(4),
                        };
                        self.palette = palette;
                    }
                },
                StatePalette::Linear { palette, data } => match palette.index(v) {
                    IndexOrBits::Index(v) => return data.set(i, v),
                    IndexOrBits::Bits(bits) => {
                        debug_assert_eq!(bits, 5);
                        // We know bits will always be 5
                        data.change_bits(bits);
                        let data = std::mem::take(data);
                        let mut values = std::mem::take(&mut palette.values);
                        // Here we double the capacity so that it is equal to 2 to the power of 5
                        values.reserve_exact(2usize.pow(4)); // values.capacity() should be equal to 2usize.pow(4)
                        let palette = StatePalette::Mapped {
                            palette: MappedPalette {
                                indices: BTreeMap::new(),
                                inner: LinearPalette { values, bits: 5 },
                            },
                            data,
                        };

                        self.palette = palette;
                    }
                },
                StatePalette::Mapped { palette, data } => match palette.index(v) {
                    IndexOrBits::Index(v) => return data.set_unchecked(i, v),
                    IndexOrBits::Bits(bits) => {
                        let palette: StatePalette<N> = if bits == 9 {
                            let mut new_data = PackedBits::new(15);
                            for i in 0..N {
                                //SAFETY: This is fine because the for loop makes sure `i` stays in bounds
                                new_data.set_unchecked(i, self.get_unchecked(i));
                            }

                            StatePalette::Global { data: new_data }
                        } else {
                            data.change_bits(bits);
                            let data = std::mem::take(data);

                            let linear = LinearPalette {
                                values: std::mem::take(&mut palette.inner.values),
                                bits,
                            };
                            StatePalette::Mapped {
                                palette: MappedPalette {
                                    indices: std::mem::take(&mut palette.indices),
                                    inner: linear,
                                },
                                data,
                            }
                        };
                        self.palette = palette;
                    }
                },
                StatePalette::Global { data } => return data.set_unchecked(i, v.into()),
            }
        }
    }

    pub fn swap(&mut self, i: usize, v: u64) -> u64 {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we just checked the bounds.
        unsafe { self.swap_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds
    pub unsafe fn swap_unchecked(&mut self, i: usize, v: u64) -> u64 {
        let val = self.get_unchecked(i);
        self.set_unchecked(i, v);
        val
    }
}

trait Palette {
    fn index(&mut self, value: u64) -> IndexOrBits;
    fn value(&self, index: usize) -> u64;
}

// TODO: Rename?
enum IndexOrBits {
    Index(u64),
    Bits(usize),
}

#[derive(Copy, Clone)]
struct SingleValuePalette(u64);

impl Palette for SingleValuePalette {
    fn index(&mut self, state: u64) -> IndexOrBits {
        if self.0 == state {
            IndexOrBits::Index(0)
        } else {
            IndexOrBits::Bits(1)
        }
    }

    fn value(&self, index: usize) -> u64 {
        if index == 0 {
            self.0
        } else {
            panic!("index out of bounds")
        }
    }
}

struct LinearPalette {
    pub(crate) values: Vec<u64>,
    pub(crate) bits: usize,
}

impl Palette for LinearPalette {
    fn index(&mut self, state: u64) -> IndexOrBits {
        for i in 0..self.values.len() {
            // SAFETY: This is fine because i can only be in bounds due to the for loop.
            unsafe {
                if *self.values.get_unchecked(i) == state {
                    return IndexOrBits::Index(i as u64);
                }
            }
        }

        let len = self.values.len();
        if self.values.capacity() > len {
            debug_assert_eq!(self.values.capacity(), 2usize.pow(self.bits as u32));
            self.values.push(state);
            IndexOrBits::Index(len as u64)
        } else {
            IndexOrBits::Bits(self.bits + 1)
        }
    }

    #[inline]
    fn value(&self, index: usize) -> u64 {
        self.values[index]
    }
}

/// This makes the `index` method faster at the cost of memory usage.
struct MappedPalette {
    pub(crate) indices: BTreeMap<u64, usize>,
    pub(crate) inner: LinearPalette,
}

impl Palette for MappedPalette {
    fn index(&mut self, state: u64) -> IndexOrBits {
        match self.indices.get(&state) {
            Some(v) => IndexOrBits::Index(*v as u64),
            None => {
                let initial_len = self.inner.values.len();
                if self.inner.values.capacity() > initial_len {
                    debug_assert_eq!(
                        self.inner.values.capacity(),
                        2usize.pow(self.inner.bits as u32)
                    );
                    self.inner.values.push(state);
                    self.indices.insert(state, self.inner.values.len());
                    IndexOrBits::Index(initial_len as u64)
                } else {
                    IndexOrBits::Bits(self.inner.bits + 1)
                }
            }
        }
    }

    fn value(&self, index: usize) -> u64 {
        self.inner.value(index)
    }
}

#[cfg(test)]
mod tests {
    use super::{BiomePaletteContainer, StatePaletteContainer};

    #[test]
    fn state() {
        let mut data = Vec::new();
        for i in 0..512 {
            data.push(i)
        }
        data.reverse();
        let mut container = StatePaletteContainer::<512>::new(0);
        for i in 0..512 {
            container.set(i, data[i]);
            assert_eq!(container.get(i), data[i]);
            for j in 0..=i {
                assert_eq!(container.get(j), data[j])
            }
        }
    }

    #[test]
    fn biome() {
        let data = vec![7, 6, 5, 4, 3, 2, 1, 0];
        let mut container = BiomePaletteContainer::<8>::new(0);
        for i in 0..8 {
            container.set(i, data[i]);
            assert_eq!(container.get(i), data[i]);
            for j in 0..=i {
                assert_eq!(container.get(j), data[j])
            }
        }
    }
}
