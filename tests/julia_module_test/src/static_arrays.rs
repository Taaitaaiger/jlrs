use jlrs::data::layout::static_arrays::{
    dims::Dims3D, m_array::MArray, m_matrix::MMatrix, m_vector::MVector, s_array::SArray,
    s_matrix::SMatrix, s_vector::SVector,
};

pub fn sum_svector(svector: SVector<f32, 3>) -> f32 {
    svector.data().iter().sum()
}

pub fn sum_svector_ref(svector: &SVector<f32, 3>) -> f32 {
    svector.data().iter().sum()
}

pub fn sum_mvector_ref(mvector: &MVector<f32, 3>) -> f32 {
    mvector.data().iter().sum()
}

pub fn reverse_mvector(mvector: &mut MVector<f32, 3>) {
    let data = mvector.data_mut();
    let temp = data[0];
    data[0] = data[2];
    data[2] = temp;
}

pub fn sum_smatrix(smatrix: SMatrix<f32, 3, 2>) -> f32 {
    smatrix.data().iter().flatten().sum()
}

pub fn sum_smatrix_ref(smatrix: &SMatrix<f32, 3, 2>) -> f32 {
    smatrix.data().iter().flatten().sum()
}

pub fn sum_mmatrix_ref(mmatrix: &MMatrix<f32, 3, 2>) -> f32 {
    mmatrix.data().iter().flatten().sum()
}

pub fn swap_mmatrix_cols(mmatrix: &mut MMatrix<f32, 3, 2>) {
    let data = mmatrix.data_mut();
    let temp = data[0];
    data[0] = data[1];
    data[1] = temp;
}

pub fn sum_sarray(sarray: SArray<f32, Dims3D<2, 2, 2>, 8, 3>) -> f32 {
    sarray.data().iter().sum()
}

pub fn sum_sarray_ref(sarray: &SArray<f32, Dims3D<2, 2, 2>, 8, 3>) -> f32 {
    sarray.data().iter().sum()
}

pub fn sum_marray_ref(marray: &MArray<f32, Dims3D<2, 2, 2>, 8, 3>) -> f32 {
    marray.data().iter().sum()
}

pub fn swap_marray_blocks(marray: &mut MArray<f32, Dims3D<2, 2, 2>, 8, 3>) {
    let data = marray.data_mut();
    for i in 0..4 {
        let tmp = data[i];
        data[i] = data[i + 4];
        data[i + 4] = tmp;
    }
}
