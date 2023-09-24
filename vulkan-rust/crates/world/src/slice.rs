use crate::traits::Data3D;

pub type QuadSlice<T, const N: usize> = [[T; N]; N];

pub struct CubeSlice<T, const N: usize>([[[T; N]; N]; N]);

impl<T, const N: usize> CubeSlice<T, N> {
    #[inline]
    pub fn new(data: [[[T; N]; N]; N]) -> Self {
        Self(data)
    }
}

impl<T, const N: usize> Default for CubeSlice<T, N>
where
    T: Default + Copy,
{
    #[inline]
    fn default() -> Self {
        Self([[[Default::default(); N]; N]; N])
    }
}

#[inline]
pub fn is_inside<T>(x: T, y: T, z: T, x1: T, x2: T, y1: T, y2: T, z1: T, z2: T) -> bool
where
    T: PartialOrd,
{
    x >= x1 && x < x2 && y >= y1 && y < y2 && z >= z1 && z < z2
}

impl<T, const N: usize> Data3D<T> for CubeSlice<T, N>
where
    T: Default + Copy,
{
    #[inline]
    fn set(&mut self, x: usize, y: usize, z: usize, value: T) {
        if is_inside(x, y, z, 0, N, 0, N, 0, N) {
            self.0[x][y][z] = value;
        }
    }

    #[inline]
    fn get(&self, x: usize, y: usize, z: usize) -> T {
        if is_inside(x, y, z, 0, N, 0, N, 0, N) {
            self.0[x][y][z]
        } else {
            Default::default()
        }
    }
}

pub struct Slice3<T> {
    dimensions: [usize; 3],
    data: Vec<T>,
}

impl<T> Slice3<T> {
    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        let [size_x, size_y, ..] = self.dimensions;
        let index = z * size_x * size_y + y * size_x + x;

        debug_assert!(
            index < self.data.len(),
            "Slice to small. Size was {}. Index was {} ({},{},{})",
            self.data.len(),
            index,
            x,
            y,
            z
        );

        index
    }
}

impl<T> Slice3<T>
where
    T: Default + Clone + PartialEq,
{
    #[inline]
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self {
            dimensions: [x, y, z],
            data: vec![Default::default(); x * y * z],
        }
    }

    pub fn write_into<D>(&self, dest: &mut D, offset_x: isize, offset_y: isize, offset_z: isize)
    where
        D: Data3D<T>,
    {
        let [end_x, end_y, end_z] = self.dimensions;
        let start_x = 0;
        let start_y = 0;
        let start_z = 0;

        // TODO: could improve by calculating the box better;
        let mut index = 0;
        for z in start_z..end_z {
            for y in start_y..end_y {
                for x in start_x..end_x {
                    let value = self.data.get(index).unwrap();
                    if *value != T::default() {
                        let dest_x = offset_x + x as isize;
                        let dest_y = offset_y + y as isize;
                        let dest_z = offset_z + z as isize;

                        dest.set(
                            dest_x as usize,
                            dest_y as usize,
                            dest_z as usize,
                            value.clone(),
                        );
                    }

                    index += 1;
                }
            }
        }
    }
}

impl<T> Data3D<T> for Slice3<T>
where
    T: Default + Copy,
{
    fn set(&mut self, x: usize, y: usize, z: usize, value: T) {
        let index = self.index(x, y, z);
        if let Some(item) = self.data.get_mut(index) {
            *item = value;
        }
    }

    fn get(&self, x: usize, y: usize, z: usize) -> T {
        let index = self.index(x, y, z);
        self.data.get(index).map_or(Default::default(), |x| *x)
    }
}
