use std::collections::HashMap;

use pyo3::prelude::*;

#[pyclass]
pub struct Aig {
    inner: crate::aig::Aig,
}

#[pymethods]
impl Aig {
    #[staticmethod]
    pub fn from_file(path: &str) -> eyre::Result<Self> {
        let aig = crate::aig::Aig::from_file(path)?;
        Ok(Aig { inner: aig })
    }

    #[staticmethod]
    pub fn from_string(s: &str) -> eyre::Result<Self> {
        let aig = crate::aig::Aig::from_reader(s.as_bytes())?;
        Ok(Aig { inner: aig })
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

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn layers_input(&self) -> Vec<Vec<u32>> {
        self.inner.layers_input().collect()
    }

    pub fn layers_output(&self) -> Vec<Vec<u32>> {
        self.inner.layers_output().collect()
    }
}
