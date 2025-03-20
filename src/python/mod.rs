use pyo3::prelude::*;

mod aig;

#[pymodule]
pub fn aigerox(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<aig::Aig>()?;
    Ok(())
}
