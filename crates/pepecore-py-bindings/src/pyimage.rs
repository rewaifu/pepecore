// use numpy::{Element, PyReadonlyArray, PyReadonlyArray2, PyReadonlyArray3};
// use numpy::ndarray::Dimension;
// use pyo3::FromPyObject;
// use pepecore::array::svec::{SVec, Shape};
// use pepecore::enums::ImgData;
//
// #[derive(FromPyObject)]
// pub enum PyImage<'py> {
//     D2F32(PyReadonlyArray2<'py, f32>),
//     D3F32(PyReadonlyArray3<'py, f32>),
//     D2U8(PyReadonlyArray2<'py, u8>),
//     D3U8(PyReadonlyArray3<'py, u8>),
//     D2U16(PyReadonlyArray2<'py, u16>),
//     D3U16(PyReadonlyArray3<'py, u16>),
// }
//
// impl From<PyImage<'_>> for SVec {
//     fn from(value: PyImage<'_>) -> Self {
//         fn to_parts<D: Dimension, T: Element>(img: &PyReadonlyArray<T, D>) -> (Shape, Vec<T>) where T: Copy {
//             let data = img.as_array().iter().copied().collect();
//             (Shape::from(img.shape()), data)
//         }
//
//         match value {
//             PyImage::D2F32(arr) => {
//                 let parts = to_parts(&arr);
//                 SVec::new(parts.0, ImgData::F32(parts.1))
//             },
//             PyImage::D3F32(arr) => {
//                 let parts = to_parts(&arr);
//                 SVec::new(parts.0, ImgData::F32(parts.1))
//             },
//             PyImage::D2U8(arr) => {
//                 let parts = to_parts(&arr);
//                 SVec::new(parts.0, ImgData::U8(parts.1))
//             },
//             PyImage::D3U8(arr) => {
//                 let parts = to_parts(&arr);
//                 SVec::new(parts.0, ImgData::U8(parts.1))
//             },
//             PyImage::D2U16(arr) => {
//                 let parts = to_parts(&arr);
//                 SVec::new(parts.0, ImgData::U16(parts.1))
//             },
//             PyImage::D3U16(arr) => {
//                 let parts = to_parts(&arr);
//                 SVec::new(parts.0, ImgData::U16(parts.1))
//             },
//         }
//     }
// }
