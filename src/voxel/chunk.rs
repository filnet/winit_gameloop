use std::slice;

#[derive(Copy, Clone)]
pub struct BlockType(u8);

#[derive(Copy, Clone)]
pub enum Voxel {
    Block(BlockType), // 1 byte payload
    Empty,            // 0 byte payload
}

const CHUNK_SIZE: u32 = 32;
const CHUNK_SIZE_SQUARED: u32 = CHUNK_SIZE * CHUNK_SIZE;
const CHUNK_SIZE_CUBED: u32 = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[inline(always)]
fn to_index(i: u32, j: u32, k: u32) -> u32 {
    assert!(i < CHUNK_SIZE && j < CHUNK_SIZE && k < CHUNK_SIZE);
    i + j * CHUNK_SIZE + k * CHUNK_SIZE_SQUARED
}

#[inline(always)]
fn to_coords(index: u32) -> (u32, u32, u32) {
    assert!(index < CHUNK_SIZE_CUBED);
    let i = index % CHUNK_SIZE;
    let j = (index / CHUNK_SIZE) % CHUNK_SIZE;
    let k = index / CHUNK_SIZE_SQUARED;
    (i, j, k)
}

pub struct Chunk {
    data: [Voxel; CHUNK_SIZE_CUBED as usize],
}

// https://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            data: [Voxel::Empty; CHUNK_SIZE_CUBED as usize],
        }
    }

    pub fn voxel(&self, i: u32, j: u32, k: u32) -> Voxel {
        self.data[to_index(i, j, k) as usize]
    }

    pub fn copy(&mut self, chunk: &Chunk) {
        self.data = chunk.data;
    }

    /*pub fn update(&mut self, iter: slice::Iter<'_, Voxel>) {
        let mut index = 0;
        for voxel in iter {
            assert!(index < self.data.len());
            self.data[index] = *voxel;
            index += 1;
        }
    }*/

    pub fn generate_from_index<F>(&mut self, generate: F)
    where
        F: Fn(u32) -> Voxel,
    {
        for index in 0..CHUNK_SIZE_CUBED {
            self.data[index as usize] = (generate)(index);
        }
    }

    pub fn generate_from_coords<F>(&mut self, generate: F)
    where
        F: Fn(u32, u32, u32) -> Voxel,
    {
        for index in 0..CHUNK_SIZE_CUBED {
            let (i, j, k) = to_coords(index);
            self.data[index as usize] = (generate)(i, j, k);
        }
    }
}

// Rust Iterator Cheat Sheet: https://danielkeep.github.io/itercheat_baked.html
impl<'a> IntoIterator for &'a Chunk {
    type Item = &'a Voxel;
    type IntoIter = slice::Iter<'a, Voxel>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

#[allow(dead_code)]
fn from_index(index: u32) -> Voxel {
    match index % 2 {
        0 => Voxel::Empty,
        _ => Voxel::Block(BlockType(0)),
    }
}

/*pub struct StrangeIterator {
    index: u32,
    //slice: &'a [Voxel],
    //iter: slice::Iter<'a, VoxelType>,
}

impl StrangeIterator {
    fn new() -> Self {
        StrangeIterator { index: 0 }
    }
}

impl Iterator for StrangeIterator {
    type Item = (u32, u32, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < CHUNK_SIZE_CUBED {
            let index = self.index;
            let (i, j, k) = to_coords(index);
            self.index += 1;
            Some((i, j, k, index))
        } else {
            None
        }
    }
}

impl ExactSizeIterator for StrangeIterator {
    fn len(&self) -> usize {
        CHUNK_SIZE_CUBED as usize
    }
}*/

/*

pub struct IndexedIterator<'a> {
    i: u32,
    j: u32,
    k: u32,
    index: u32,
    slice: &'a [Voxel],
    //iter: slice::Iter<'a, VoxelType>,
}

impl<'a> IndexedIterator<'a> {
    fn new(slice: &'a [Voxel]) -> Self {
        IndexedIterator {
            i: 0,
            j: 0,
            k: 0,
            index: 0,
            slice,
        }
    }
}

impl Iterator for IndexedIterator<'_> {
    type Item = (u32, u32, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.i, self.j, self.k, self.index))
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {
        let mut index = 0;
        for k in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for i in 0..CHUNK_SIZE {
                    assert!(index == to_index(i, j, k));
                    //println!("{:#02} {:#02} {:#02} - {}", i, j, k, index);
                    index += 1;
                }
            }
        }
    }

    #[test]
    fn internal() {
        println!("Hello World");
        let mut chunk = Chunk::new();

        //let g = MyGenerator {};//}::new();
        //chunk.generate(&mut g);
        chunk.generate_from_index(from_index);

        /*for (i, j, k, index) in g {
            println!("{:#02} {:#02} {:#02} - {}", i, j, k, index);
        }*/

        let mut c = 0;
        for _voxel in chunk.into_iter() {
            c += 1;
        }
        println!("{}", c);

        let mut cc = 0;
        for voxel in chunk.into_iter() {
            match voxel {
                Voxel::Block(_block_type) => {
                    cc += 1;
                }
                _ => (),
            }
        }
        println!("{}", cc);

        let other = Chunk::new();
        chunk.copy(&other);
        //chunk.update(other.into_iter());
    }
}
