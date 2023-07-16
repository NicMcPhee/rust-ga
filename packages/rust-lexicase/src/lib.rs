#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::redundant_pub_crate)]

use std::iter::repeat_with;

use ec_core::{
    individual::Individual,
    operator::selector::{lexicase::Lexicase, Selector},
    test_results::{Error, TestResults},
};
use numpy::PyReadonlyArray1;
use pyo3::prelude::*;
use rand::thread_rng;

struct PyIndividual {
    individual: PyObject,
    errors: TestResults<Error>,
}

impl PyIndividual {
    fn new(py: Python<'_>, individual: PyObject) -> PyResult<Self> {
        let errors = individual
            .getattr(py, "error_vector")? // Python-style getattr, requires a GIL token (`py`).
            .extract::<PyReadonlyArray1<i64>>(py)? // Tell PyO3 what to convert the result to.
            .as_array()
            .into_iter()
            .copied()
            .map(|v| Error { error: v })
            .collect::<TestResults<Error>>();
        Ok(Self { individual, errors })
    }

    fn py_individual(&self) -> PyObject {
        self.individual.clone()
    }
}

impl Individual for PyIndividual {
    type Genome = Self;

    type TestResults = TestResults<Error>;

    fn genome(&self) -> &Self::Genome {
        unimplemented!()
    }

    fn test_results(&self) -> &Self::TestResults {
        &self.errors
    }
}

fn transform_population(py: Python<'_>, pop: Vec<PyObject>) -> PyResult<Vec<PyIndividual>> {
    pop.into_iter()
        .map(|i| PyIndividual::new(py, i))
        .collect::<PyResult<Vec<PyIndividual>>>()
}

#[pyfunction]
fn select_one(py: Python<'_>, pop: Vec<PyObject>) -> PyResult<PyObject> {
    let population = transform_population(py, pop)?;
    let num_cases = population[0].errors.results.len();
    let lexicase = Lexicase::new(num_cases);
    let mut rng = thread_rng();
    Ok(lexicase.select(&population, &mut rng)?.py_individual())
}

#[pyfunction]
fn select(py: Python<'_>, pop: Vec<PyObject>, n: usize) -> PyResult<Vec<PyObject>> {
    let population = transform_population(py, pop)?;
    let num_cases = population[0].errors.results.len();
    let lexicase = Lexicase::new(num_cases);
    let mut rng = thread_rng();
    Ok(repeat_with(|| lexicase.select(&population, &mut rng))
        .take(n)
        .map(|r| r.map(PyIndividual::py_individual))
        .collect::<Result<Vec<_>, _>>()?)
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_lexicase(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(select_one, m)?)?;
    m.add_function(wrap_pyfunction!(select, m)?)?;
    Ok(())
}
