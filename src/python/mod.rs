use pyo3::prelude::*;

mod aig;

#[pymodule]
pub fn aigerox(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_class::<aig::PyAig>()?;
    Ok(())
}
