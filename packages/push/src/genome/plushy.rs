use easy_cast::ConvApprox;
use ec_core::{
    distributions::{choices::ChoicesDistribution, collection::CollectionGenerator},
    genome::Genome,
};
use ec_linear::genome::Linear;
use rand::{prelude::Distribution, Rng};

use crate::instruction::PushInstruction;

#[derive(Clone, Eq, PartialEq)]
pub enum PushGene {
    Close,
    Instruction(PushInstruction),
}

impl<T> From<T> for PushGene
where
    T: Into<PushInstruction>,
{
    fn from(instruction: T) -> Self {
        Self::Instruction(instruction.into())
    }
}

impl std::fmt::Debug for PushGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Close => write!(f, "Close"),
            Self::Instruction(instruction) => instruction.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneGenerator<T>
where
    T: Distribution<PushInstruction>,
{
    close_probability: f32,
    instructions_distribution: T,
}

impl<T> GeneGenerator<T>
where
    T: Distribution<PushInstruction>,
{
    #[must_use]
    pub const fn new(close_probability: f32, instructions_distribution: T) -> Self {
        Self {
            close_probability,
            instructions_distribution,
        }
    }
}
impl<T> GeneGenerator<T>
where
    T: Distribution<PushInstruction> + ChoicesDistribution,
{
    /// Create a generator where the close tag is just as likely as any passed
    /// instruction
    ///
    /// # Errors
    /// - [`EmptySlice`] if the passed in slice is empty
    pub fn with_uniform_close_probability(instructions_distribution: T) -> Self {
        Self::new(
            1.0 / f32::conv_approx(instructions_distribution.num_choices().get() + 1),
            instructions_distribution,
        )
    }
}

pub trait ConvertToGeneGenerator
where
    Self: Distribution<PushInstruction>,
{
    fn into_gene_generator_with_close_probability(
        self,
        close_probability: f32,
    ) -> GeneGenerator<Self>
    where
        Self: Sized;

    fn to_gene_generator_with_close_probability(
        &self,
        close_probability: f32,
    ) -> GeneGenerator<&Self>;

    /// This creates a new gene generator, defaulting to a close probability
    /// that is uniform with the instructions distribution, eg. (1/(n+1)).
    fn into_gene_generator(self) -> GeneGenerator<Self>
    where
        Self: Sized + ChoicesDistribution;

    /// This creates a new gene generator by borrowing from self, defaulting to
    /// a close probability that is uniform with the instructions distribution,
    /// eg. (1/(n+1)).
    fn to_gene_generator(&self) -> GeneGenerator<&Self>
    where
        Self: ChoicesDistribution;
}

impl<T> ConvertToGeneGenerator for T
where
    T: Distribution<PushInstruction> + ?Sized,
{
    fn into_gene_generator_with_close_probability(
        self,
        close_probability: f32,
    ) -> GeneGenerator<Self>
    where
        Self: Sized,
    {
        GeneGenerator::new(close_probability, self)
    }

    fn to_gene_generator_with_close_probability(
        &self,
        close_probability: f32,
    ) -> GeneGenerator<&Self> {
        GeneGenerator::new(close_probability, self)
    }

    fn into_gene_generator(self) -> GeneGenerator<Self>
    where
        Self: Sized + ChoicesDistribution,
    {
        GeneGenerator::with_uniform_close_probability(self)
    }

    fn to_gene_generator(&self) -> GeneGenerator<&Self>
    where
        Self: ChoicesDistribution,
    {
        GeneGenerator::with_uniform_close_probability(self)
    }
}

impl<T> Distribution<PushGene> for GeneGenerator<T>
where
    T: Distribution<PushInstruction>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PushGene {
        if rng.gen::<f32>() < self.close_probability {
            PushGene::Close
        } else {
            // this is safe since we check that the slice is not empty in the constructor
            PushGene::Instruction(self.instructions_distribution.sample(rng))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Plushy {
    genes: Vec<PushGene>,
}

// TODO: We might want to implement some sort of `Into`
// trait instead of just having a getter. Having something
// like `to_instructions()` since we're cloning?
impl Plushy {
    pub fn new(iterable: impl IntoIterator<Item = PushGene>) -> Self {
        Self {
            genes: iterable.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn get_genes(&self) -> Vec<PushGene> {
        self.genes.clone()
    }
}

impl Genome for Plushy {
    type Gene = PushGene;
}

impl Linear for Plushy {
    fn size(&self) -> usize {
        self.genes.len()
    }

    fn gene_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        self.genes.get_mut(index)
    }
}

impl<GG> Distribution<Plushy> for CollectionGenerator<GG>
where
    GG: Distribution<PushGene>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Plushy {
        Plushy {
            genes: rng.sample(self),
        }
    }
}

impl IntoIterator for Plushy {
    type Item = PushGene;

    type IntoIter = std::vec::IntoIter<PushGene>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

impl FromIterator<PushGene> for Plushy {
    fn from_iter<T: IntoIterator<Item = PushGene>>(iterable: T) -> Self {
        Self {
            genes: iterable.into_iter().collect(),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use ec_core::{
        distributions::collection::ConvertToCollectionGenerator, operator::mutator::Mutator,
        uniform_distribution_of,
    };
    use ec_linear::mutator::umad::Umad;
    use rand::thread_rng;

    use super::*;
    use crate::{
        instruction::{variable_name::VariableName, BoolInstruction, IntInstruction},
        list_into::vec_into,
    };

    #[ignore = "this has about a 2.665% chance on failing at least once across the three test \
                runners in ci"]
    #[test]
    #[allow(clippy::unwrap_used)]
    fn generator() {
        let mut rng = thread_rng();
        let plushy: Plushy = uniform_distribution_of![<PushInstruction>
            IntInstruction::Add,
            IntInstruction::Subtract,
            IntInstruction::Multiply,
            IntInstruction::ProtectedDivide,
        ]
        .into_gene_generator()
        .into_collection_generator(10)
        .sample(&mut rng);

        assert_eq!(10, plushy.genes.len());
    }

    #[test]
    fn umad() {
        let mut rng = thread_rng();

        let instruction_options = uniform_distribution_of![<PushGene> VariableName::from("x")];

        let umad = Umad::new(0.3, 0.3, instruction_options);

        let parent = Plushy {
            genes: vec_into![
                IntInstruction::Add,
                BoolInstruction::And,
                BoolInstruction::Or,
                IntInstruction::Multiply,
                IntInstruction::Add,
                BoolInstruction::And,
                BoolInstruction::Or,
                IntInstruction::Multiply,
                IntInstruction::Add,
                BoolInstruction::And,
                BoolInstruction::Or,
                IntInstruction::Multiply,
                IntInstruction::Add,
                BoolInstruction::And,
                BoolInstruction::Or,
                IntInstruction::Multiply,
                IntInstruction::Add,
                BoolInstruction::And,
                BoolInstruction::Or,
                IntInstruction::Multiply,
            ],
        };

        let child = umad.mutate(parent, &mut rng);

        #[allow(clippy::unwrap_used)]
        let num_inputs = child
            .unwrap()
            .genes
            .iter()
            .filter(|c| matches!(c, PushGene::Instruction(PushInstruction::InputVar(v)) if v == &VariableName::from("x")))
            .count();
        assert!(
            num_inputs > 0,
            "Expected at least one input instruction to be added, but none were."
        );
    }

    // TODO: Test that `Umad` works here on Plushy genomes.
}
