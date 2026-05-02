use pepecore::ops::svec_ops::best_tile::BestTile;
use crate::structure::svec_traits::PySvec;
use pyo3::{PyResult, Python, pyclass, pymethods,PyAny,Bound};
#[pyclass(name ="BestTile")]
#[derive(Clone)]
pub struct PyBastTile{
     pub bt: BestTile,
}
#[pymethods]
impl PyBastTile {
    #[new]
    pub fn new( 
        py: Python<'_>,
        img: Bound<'_, PyAny>,
        tile_y:u16,
        tile_x:u16
    ) -> PyResult<PyBastTile> {
        
        let mut img = img.to_svec(py)?;
        let bt = py.detach(|| {
            img.as_f32();
            BestTile::new(&img, tile_y, tile_x)
        });
        Ok(PyBastTile { bt })
    }
    fn get_max_coords(&mut self,py: Python<'_>,) -> PyResult<Option<(usize,usize)>> {
        Ok(py.detach(|| {self.bt.get_max_coords()}))
    }
    #[pyo3(signature = ( n,threshold = None))]
    fn get_top_n(&mut self,py:Python<'_>,n:usize,threshold:Option<f32>)->PyResult<Vec<(usize,usize)>>{
        Ok(py.detach(|| {self.bt.get_top_n_non_overlapping(n,threshold)}))
    }
    #[pyo3(signature = ( n,threshold = None))]
    fn get_top_n_(&mut self,py:Python<'_>,n:usize,threshold:Option<f32>)->PyResult<Vec<(usize,usize)>>{
        Ok(py.detach(|| {self.bt.get_top_n_non_overlapping_(n,threshold)}))
    }
}