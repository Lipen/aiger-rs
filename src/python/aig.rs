use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use pyo3::prelude::*;
use crate::aig::Aig;

#[pyclass(name = "Aig", str)]
pub struct PyAig {
    inner: Aig,
}

impl Display for PyAig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[pymethods]
impl PyAig {
    #[staticmethod]
    pub fn from_file(path: &str) -> eyre::Result<Self> {
        let aig = Aig::from_file(path)?;
        Ok(PyAig { inner: aig })
    }

    #[staticmethod]
    pub fn from_string(s: &str) -> eyre::Result<Self> {
        let aig = Aig::from_reader(s.as_bytes())?;
        Ok(PyAig { inner: aig })
    }

    pub fn inputs(&self) -> Vec<u32> {
        self.inner.inputs().to_vec()
    }

    pub fn outputs(&self) -> Vec<i32> {
        self.inner.outputs().iter().map(|r| r.get()).collect()
    }

    pub fn nodes(&self) -> HashMap<u32, Vec<i32>> {
        self.inner.nodes().iter().map(|(&k, v)| (k, v.children().iter().map(|r| r.get()).collect())).collect()
    }

    pub fn children(&self, id: u32) -> Vec<i32> {
        let node = self.inner.node(id);
        node.children().iter().map(|r| r.get()).collect()
    }

    pub fn is_input(&self, id: u32) -> bool {
        self.inner.is_input(id)
    }

    pub fn is_gate(&self, id: u32) -> bool {
        self.inner.is_gate(id)
    }

    pub fn __contains__(&self, id: u32) -> bool {
        self.inner.contains(id)
    }

    pub fn layers_input(&self) -> Vec<Vec<u32>> {
        self.inner.layers_input().collect()
    }

    pub fn layers_output(&self) -> Vec<Vec<u32>> {
        self.inner.layers_output().collect()
    }
}
