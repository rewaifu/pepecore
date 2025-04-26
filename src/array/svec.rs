use crate::enums::ImgData;
use std::any::TypeId;

#[derive(Debug)]
pub enum SVecError {
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
}
#[derive(Debug)]
pub struct Shape {
    height: usize,
    width: usize,
    channels: Option<usize>,
}
#[derive(Debug)]
pub struct SVec {
    shape: Shape,
    data: ImgData,
}
impl Shape {
    pub fn new(height: usize, width: usize, channels: Option<usize>) -> Self {
        Self {
            height,
            width,
            channels,
        }
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_channels(&self) -> Option<usize> {
        self.channels
    }
    pub fn get_shape(&self) -> (usize, usize, Option<usize>) {
        (self.height, self.width, self.channels)
    }
    pub fn get_ndims(&self) -> usize {
        if self.channels.is_some() { 3 } else { 2 }
    }
}
impl SVec {
    pub fn new(shape: Shape, data: ImgData) -> Self {
        SVec { shape, data }
    }
    pub fn shape(&self) -> (usize, usize, Option<usize>) {
        self.shape.get_shape()
    }
    pub fn get_len(&self) -> usize {
        let shape = self.shape.get_shape();
        shape.0 * shape.1 * shape.2.unwrap_or(1)
    }

    pub fn get_data<T: 'static>(&self) -> Result<&[T], SVecError> {
        match &self.data {
            ImgData::U8(data) => {
                if TypeId::of::<T>() == TypeId::of::<u8>() {
                    Ok(unsafe { std::mem::transmute::<&[u8], &[T]>(data) })
                } else {
                    Err(SVecError::TypeMismatch {
                        expected: "u8",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::U16(data) => {
                if TypeId::of::<T>() == TypeId::of::<u16>() {
                    Ok(unsafe { std::mem::transmute::<&[u16], &[T]>(data) })
                } else {
                    Err(SVecError::TypeMismatch {
                        expected: "u16",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::F32(data) => {
                if TypeId::of::<T>() == TypeId::of::<f32>() {
                    Ok(unsafe { std::mem::transmute::<&[f32], &[T]>(data) })
                } else {
                    Err(SVecError::TypeMismatch {
                        expected: "f32",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
        }
    }

    pub fn get_data_mut<T: 'static>(&mut self) -> Result<&mut [T], SVecError> {
        match &mut self.data {
            ImgData::U8(data) => {
                if TypeId::of::<T>() == TypeId::of::<u8>() {
                    Ok(unsafe { std::mem::transmute::<&mut [u8], &mut [T]>(data) })
                } else {
                    Err(SVecError::TypeMismatch {
                        expected: "u8",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::U16(data) => {
                if TypeId::of::<T>() == TypeId::of::<u16>() {
                    Ok(unsafe { std::mem::transmute::<&mut [u16], &mut [T]>(data) })
                } else {
                    Err(SVecError::TypeMismatch {
                        expected: "u16",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
            ImgData::F32(data) => {
                if TypeId::of::<T>() == TypeId::of::<f32>() {
                    Ok(unsafe { std::mem::transmute::<&mut [f32], &mut [T]>(data) })
                } else {
                    Err(SVecError::TypeMismatch {
                        expected: "f32",
                        actual: std::any::type_name::<T>(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_u8_svec() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U8(vec![1, 2, 3, 4]);
        let svec = SVec::new(shape, data);

        assert_eq!(svec.shape(), (2, 2, Some(1)));
        assert_eq!(svec.get_len(), 4);

        match svec.get_data::<u8>() {
            Ok(data) => assert_eq!(data, &[1, 2, 3, 4]),
            Err(e) => panic!("Ожидали данные типа u8, но получили ошибку: {:?}", e),
        }
    }

    #[test]
    fn test_create_u16_svec() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U16(vec![100, 200, 300, 400]);
        let svec = SVec::new(shape, data);

        assert_eq!(svec.shape(), (2, 2, Some(1)));
        assert_eq!(svec.get_len(), 4);

        match svec.get_data::<u16>() {
            Ok(data) => assert_eq!(data, &[100, 200, 300, 400]),
            Err(e) => panic!("Ожидали данные типа u16, но получили ошибку: {:?}", e),
        }
    }

    #[test]
    fn test_create_f32_svec() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::F32(vec![1.1, 2.2, 3.3, 4.4]);
        let svec = SVec::new(shape, data);

        assert_eq!(svec.shape(), (2, 2, Some(1)));
        assert_eq!(svec.get_len(), 4);

        match svec.get_data::<f32>() {
            Ok(data) => assert_eq!(data, &[1.1, 2.2, 3.3, 4.4]),
            Err(e) => panic!("Ожидали данные типа f32, но получили ошибку: {:?}", e),
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U8(vec![1, 2, 3, 4]);
        let svec = SVec::new(shape, data);
        // let result= svec.data;
        let result = svec.get_data::<u16>();
        // println!("{:?}",result);
        match result {
            Err(SVecError::TypeMismatch { expected, actual }) => {
                assert_eq!(expected, "u8");
                assert_eq!(actual, "u16");
            }
            _ => panic!("Ожидалась ошибка TypeMismatch, но получили что-то другое"),
        }
    }

    #[test]
    fn test_get_data_mut() {
        let shape = Shape::new(2, 2, Some(1));
        let data = ImgData::U8(vec![1, 2, 3, 4]);
        let mut svec = SVec::new(shape, data);

        match svec.get_data_mut::<u8>() {
            Ok(data_mut) => {
                data_mut[0] = 10;
            }
            Err(e) => panic!(
                "Ожидали мутабельные данные типа u8, но получили ошибку: {:?}",
                e
            ),
        }

        match svec.get_data::<u8>() {
            Ok(data) => assert_eq!(data, &[10, 2, 3, 4]),
            Err(e) => panic!("Ожидали данные типа u8, но получили ошибку: {:?}", e),
        }
    }
}
