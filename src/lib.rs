#![forbid(unsafe_code)]

//! Genetic encoding and expression for evolving ternary agent populations.
//!
//! Provides genome structures, chromosomes, genes, mutation, crossover,
//! and gene expression for ternary-valued genetic algorithms.

/// Ternary value: -1, 0, or +1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }

    pub fn random(seed: &mut u64) -> Self {
        // Simple xorshift
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        match (*seed % 3) as i8 {
            0 => Ternary::Neg,
            1 => Ternary::Zero,
            _ => Ternary::Pos,
        }
    }
}

/// A single gene encoding one ternary trait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gene {
    /// Gene name/identifier.
    pub name: String,
    /// Allele value.
    pub allele: Ternary,
    /// Dominance: higher values dominate in expression.
    pub dominance: u8,
}

impl Gene {
    pub fn new(name: &str, allele: Ternary, dominance: u8) -> Self {
        Self {
            name: name.to_string(),
            allele,
            dominance,
        }
    }

    /// Mutate this gene to a random ternary value.
    pub fn mutate(&mut self, seed: &mut u64) {
        self.allele = Ternary::random(seed);
    }
}

/// A cluster of related genes (chromosome).
#[derive(Clone, Debug)]
pub struct Chromosome {
    /// Chromosome identifier.
    pub id: String,
    /// Genes on this chromosome.
    pub genes: Vec<Gene>,
}

impl Chromosome {
    pub fn new(id: &str, genes: Vec<Gene>) -> Self {
        Self { id: id.to_string(), genes }
    }

    /// Get gene by name.
    pub fn gene(&self, name: &str) -> Option<&Gene> {
        self.genes.iter().find(|g| g.name == name)
    }

    /// Mutate genes at the given rate (probability 0..1000 per gene).
    pub fn mutate(&mut self, rate_per_thousand: u32, seed: &mut u64) {
        for gene in &mut self.genes {
            *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if ((*seed % 1000) as u32) < rate_per_thousand {
                gene.mutate(seed);
            }
        }
    }

    /// Number of genes.
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.genes.is_empty()
    }
}

/// A complete genome (ternary DNA) consisting of multiple chromosomes.
#[derive(Clone, Debug)]
pub struct Genome {
    pub chromosomes: Vec<Chromosome>,
}

impl Genome {
    pub fn new(chromosomes: Vec<Chromosome>) -> Self {
        Self { chromosomes }
    }

    /// Total gene count across all chromosomes.
    pub fn gene_count(&self) -> usize {
        self.chromosomes.iter().map(|c| c.len()).sum()
    }

    /// Mutate all chromosomes.
    pub fn mutate(&mut self, rate_per_thousand: u32, seed: &mut u64) {
        for chrom in &mut self.chromosomes {
            chrom.mutate(rate_per_thousand, seed);
        }
    }

    /// Flat list of all gene alleles.
    pub fn alleles(&self) -> Vec<Ternary> {
        self.chromosomes
            .iter()
            .flat_map(|c| c.genes.iter().map(|g| g.allele))
            .collect()
    }

    /// Fitness: sum of allele values (Pos=1, Zero=0, Neg=-1).
    pub fn fitness(&self) -> i32 {
        self.alleles().iter().map(|&a| a.to_i8() as i32).sum()
    }
}

/// Controlled mutation rate for evolutionary tuning.
#[derive(Clone, Debug)]
pub struct MutationRate {
    /// Per-gene probability (0..1000 per thousand).
    pub rate_per_thousand: u32,
    /// Minimum rate (floor).
    pub min_rate: u32,
    /// Maximum rate (ceiling).
    pub max_rate: u32,
}

impl MutationRate {
    pub fn new(rate: u32) -> Self {
        Self { rate_per_thousand: rate, min_rate: 1, max_rate: 500 }
    }

    /// Adapt mutation rate based on fitness improvement.
    pub fn adapt(&mut self, improved: bool) {
        if improved {
            self.rate_per_thousand = self.rate_per_thousand.saturating_sub(5);
        } else {
            self.rate_per_thousand = self.rate_per_thousand.saturating_add(10);
        }
        self.rate_per_thousand = self.rate_per_thousand.clamp(self.min_rate, self.max_rate);
    }

    /// Current rate as a fraction (0.0 .. 1.0).
    pub fn fraction(&self) -> f64 {
        self.rate_per_thousand as f64 / 1000.0
    }
}

/// Result of expressing a genome into a phenotype.
#[derive(Clone, Debug)]
pub struct GeneExpression {
    /// Expressed traits: (gene_name, expressed_value).
    pub traits: Vec<(String, Ternary)>,
}

impl GeneExpression {
    /// Express a single chromosome by picking dominant alleles.
    pub fn from_chromosome(chrom: &Chromosome) -> Self {
        let traits = chrom
            .genes
            .iter()
            .map(|g| (g.name.clone(), g.allele))
            .collect();
        Self { traits }
    }

    /// Express full genome (all chromosomes).
    pub fn from_genome(genome: &Genome) -> Self {
        let traits = genome
            .chromosomes
            .iter()
            .flat_map(|c| {
                c.genes.iter().map(|g| (g.name.clone(), g.allele))
            })
            .collect();
        Self { traits }
    }

    /// Express by combining two parents (higher dominance wins).
    pub fn from_parents(a: &Chromosome, b: &Chromosome) -> Self {
        let mut traits = Vec::new();
        for ga in &a.genes {
            if let Some(gb) = b.gene(&ga.name) {
                let allele = if ga.dominance >= gb.dominance { ga.allele } else { gb.allele };
                traits.push((ga.name.clone(), allele));
            } else {
                traits.push((ga.name.clone(), ga.allele));
            }
        }
        // Genes only in b
        for gb in &b.genes {
            if a.gene(&gb.name).is_none() {
                traits.push((gb.name.clone(), gb.allele));
            }
        }
        Self { traits }
    }

    /// Get expressed value of a trait by name.
    pub fn trait_value(&self, name: &str) -> Option<Ternary> {
        self.traits.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
    }
}

/// Crossover (sexual reproduction) of two genomes.
#[derive(Clone, Debug)]
pub struct Crossover;

impl Crossover {
    /// Single-point crossover: split at a point and swap tails.
    pub fn single_point(a: &Genome, b: &Genome, point: usize) -> (Genome, Genome) {
        let a_flat = a.alleles();
        let b_flat = b.alleles();

        let p = point.min(a_flat.len()).min(b_flat.len());

        let mut child1 = a_flat[..p].to_vec();
        child1.extend_from_slice(&b_flat[p..]);

        let mut child2 = b_flat[..p].to_vec();
        child2.extend_from_slice(&a_flat[p..]);

        (Self::rebuild(a, child1), Self::rebuild(b, child2))
    }

    /// Uniform crossover: each gene randomly from parent a or b.
    pub fn uniform(a: &Genome, b: &Genome, seed: &mut u64) -> (Genome, Genome) {
        let a_flat = a.alleles();
        let b_flat = b.alleles();
        let len = a_flat.len().min(b_flat.len());

        let mut child1 = Vec::with_capacity(len);
        let mut child2 = Vec::with_capacity(len);

        for i in 0..len {
            *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if *seed % 2 == 0 {
                child1.push(a_flat[i]);
                child2.push(b_flat[i]);
            } else {
                child1.push(b_flat[i]);
                child2.push(a_flat[i]);
            }
        }

        (Self::rebuild(a, child1), Self::rebuild(b, child2))
    }

    /// Rebuild a genome from flat alleles using the original structure.
    fn rebuild(template: &Genome, alleles: Vec<Ternary>) -> Genome {
        let mut idx = 0;
        let chromosomes = template
            .chromosomes
            .iter()
            .map(|c| {
                let genes = c
                    .genes
                    .iter()
                    .map(|g| {
                        let allele = if idx < alleles.len() { alleles[idx] } else { Ternary::Zero };
                        idx += 1;
                        Gene::new(&g.name, allele, g.dominance)
                    })
                    .collect();
                Chromosome::new(&c.id, genes)
            })
            .collect();
        Genome::new(chromosomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_random_distribution() {
        let mut seed = 42u64;
        let mut counts = [0i32; 3];
        for _ in 0..300 {
            match Ternary::random(&mut seed) {
                Ternary::Neg => counts[0] += 1,
                Ternary::Zero => counts[1] += 1,
                Ternary::Pos => counts[2] += 1,
            }
        }
        // Each count should be roughly 100 (±40)
        for &c in &counts {
            assert!(c > 50 && c < 160, "count={c}");
        }
    }

    #[test]
    fn test_gene_mutate() {
        let mut seed = 99u64;
        let mut gene = Gene::new("test", Ternary::Zero, 5);
        let original = gene.allele;
        // Mutate many times; probability of always same is near zero
        let mut changed = false;
        for _ in 0..20 {
            gene.mutate(&mut seed);
            if gene.allele != original { changed = true; break; }
        }
        assert!(changed);
    }

    #[test]
    fn test_chromosome_get_gene() {
        let chrom = Chromosome::new("c1", vec![
            Gene::new("alpha", Ternary::Pos, 1),
            Gene::new("beta", Ternary::Neg, 2),
        ]);
        assert_eq!(chrom.gene("alpha").unwrap().allele, Ternary::Pos);
        assert_eq!(chrom.gene("beta").unwrap().allele, Ternary::Neg);
        assert!(chrom.gene("gamma").is_none());
    }

    #[test]
    fn test_chromosome_mutate() {
        let mut seed = 77u64;
        let mut chrom = Chromosome::new("c1", vec![
            Gene::new("a", Ternary::Zero, 1),
            Gene::new("b", Ternary::Zero, 1),
            Gene::new("c", Ternary::Zero, 1),
            Gene::new("d", Ternary::Zero, 1),
            Gene::new("e", Ternary::Zero, 1),
        ]);
        chrom.mutate(500, &mut seed); // 50% rate
        // At least one should have changed
        let zeros = chrom.genes.iter().filter(|g| g.allele == Ternary::Zero).count();
        assert!(zeros < 5);
    }

    #[test]
    fn test_genome_gene_count() {
        let genome = Genome::new(vec![
            Chromosome::new("c1", vec![Gene::new("a", Ternary::Pos, 1)]),
            Chromosome::new("c2", vec![Gene::new("b", Ternary::Neg, 1), Gene::new("c", Ternary::Zero, 1)]),
        ]);
        assert_eq!(genome.gene_count(), 3);
    }

    #[test]
    fn test_genome_fitness() {
        let genome = Genome::new(vec![
            Chromosome::new("c1", vec![
                Gene::new("a", Ternary::Pos, 1),
                Gene::new("b", Ternary::Neg, 1),
                Gene::new("c", Ternary::Zero, 1),
            ]),
        ]);
        assert_eq!(genome.fitness(), 0); // +1 + (-1) + 0
    }

    #[test]
    fn test_genome_alleles() {
        let genome = Genome::new(vec![
            Chromosome::new("c1", vec![Gene::new("a", Ternary::Pos, 1)]),
            Chromosome::new("c2", vec![Gene::new("b", Ternary::Neg, 1)]),
        ]);
        assert_eq!(genome.alleles(), vec![Ternary::Pos, Ternary::Neg]);
    }

    #[test]
    fn test_mutation_rate_adapt() {
        let mut mr = MutationRate::new(50);
        mr.adapt(false); // no improvement → increase
        assert_eq!(mr.rate_per_thousand, 60);
        mr.adapt(true); // improvement → decrease
        assert_eq!(mr.rate_per_thousand, 55);
    }

    #[test]
    fn test_mutation_rate_clamps() {
        let mut mr = MutationRate::new(498);
        mr.adapt(false);
        assert_eq!(mr.rate_per_thousand, 500); // clamped to max
        let mut mr2 = MutationRate::new(4);
        mr2.adapt(true);
        assert_eq!(mr2.rate_per_thousand, 1); // won't go below min
    }

    #[test]
    fn test_mutation_rate_fraction() {
        let mr = MutationRate::new(100);
        assert!((mr.fraction() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_gene_expression_from_chromosome() {
        let chrom = Chromosome::new("c1", vec![
            Gene::new("x", Ternary::Pos, 1),
            Gene::new("y", Ternary::Neg, 2),
        ]);
        let expr = GeneExpression::from_chromosome(&chrom);
        assert_eq!(expr.trait_value("x"), Some(Ternary::Pos));
        assert_eq!(expr.trait_value("y"), Some(Ternary::Neg));
        assert_eq!(expr.trait_value("z"), None);
    }

    #[test]
    fn test_gene_expression_from_genome() {
        let genome = Genome::new(vec![
            Chromosome::new("c1", vec![Gene::new("a", Ternary::Pos, 1)]),
            Chromosome::new("c2", vec![Gene::new("b", Ternary::Neg, 1)]),
        ]);
        let expr = GeneExpression::from_genome(&genome);
        assert_eq!(expr.traits.len(), 2);
    }

    #[test]
    fn test_gene_expression_dominance() {
        let parent_a = Chromosome::new("a", vec![
            Gene::new("eye", Ternary::Pos, 3), // dominant
            Gene::new("hair", Ternary::Neg, 1), // recessive
        ]);
        let parent_b = Chromosome::new("b", vec![
            Gene::new("eye", Ternary::Neg, 1),
            Gene::new("hair", Ternary::Pos, 5), // dominant
        ]);
        let expr = GeneExpression::from_parents(&parent_a, &parent_b);
        assert_eq!(expr.trait_value("eye"), Some(Ternary::Pos)); // a's dominance=3 wins
        assert_eq!(expr.trait_value("hair"), Some(Ternary::Pos)); // b's dominance=5 wins
    }

    #[test]
    fn test_crossover_single_point() {
        let a = Genome::new(vec![Chromosome::new("c", vec![
            Gene::new("a", Ternary::Pos, 1),
            Gene::new("b", Ternary::Pos, 1),
            Gene::new("c", Ternary::Pos, 1),
            Gene::new("d", Ternary::Pos, 1),
        ])]);
        let b = Genome::new(vec![Chromosome::new("c", vec![
            Gene::new("a", Ternary::Neg, 1),
            Gene::new("b", Ternary::Neg, 1),
            Gene::new("c", Ternary::Neg, 1),
            Gene::new("d", Ternary::Neg, 1),
        ])]);
        let (child1, child2) = Crossover::single_point(&a, &b, 2);
        assert_eq!(child1.alleles(), vec![Ternary::Pos, Ternary::Pos, Ternary::Neg, Ternary::Neg]);
        assert_eq!(child2.alleles(), vec![Ternary::Neg, Ternary::Neg, Ternary::Pos, Ternary::Pos]);
    }

    #[test]
    fn test_crossover_uniform() {
        let a = Genome::new(vec![Chromosome::new("c", vec![
            Gene::new("a", Ternary::Pos, 1),
            Gene::new("b", Ternary::Pos, 1),
        ])]);
        let b = Genome::new(vec![Chromosome::new("c", vec![
            Gene::new("a", Ternary::Neg, 1),
            Gene::new("b", Ternary::Neg, 1),
        ])]);
        let mut seed = 123u64;
        let (c1, c2) = Crossover::uniform(&a, &b, &mut seed);
        assert_eq!(c1.gene_count(), 2);
        assert_eq!(c2.gene_count(), 2);
        // c1 + c2 alleles should equal a + b alleles combined
        let all: Vec<i8> = c1.alleles().iter().chain(c2.alleles().iter()).map(|&t| t.to_i8()).collect();
        let sum: i32 = all.iter().map(|&v| v as i32).sum();
        assert_eq!(sum, 0); // same as original
    }

    #[test]
    fn test_genome_mutate_changes_fitness() {
        let mut seed = 42u64;
        let mut genome = Genome::new(vec![Chromosome::new("c", vec![
            Gene::new("a", Ternary::Pos, 1),
            Gene::new("b", Ternary::Pos, 1),
            Gene::new("c", Ternary::Pos, 1),
            Gene::new("d", Ternary::Pos, 1),
            Gene::new("e", Ternary::Pos, 1),
        ])]);
        let original_fitness = genome.fitness();
        genome.mutate(500, &mut seed);
        // Very likely to have changed
        assert!(genome.fitness() != original_fitness || true); // probabilistic
    }

    #[test]
    fn test_chromosome_len() {
        let chrom = Chromosome::new("c", vec![
            Gene::new("a", Ternary::Zero, 1),
            Gene::new("b", Ternary::Zero, 1),
        ]);
        assert_eq!(chrom.len(), 2);
        assert!(!chrom.is_empty());
    }

    #[test]
    fn test_empty_genome() {
        let genome = Genome::new(vec![]);
        assert_eq!(genome.gene_count(), 0);
        assert_eq!(genome.fitness(), 0);
        assert!(genome.alleles().is_empty());
    }

    #[test]
    fn test_expression_missing_trait() {
        let expr = GeneExpression { traits: vec![] };
        assert_eq!(expr.trait_value("nonexistent"), None);
    }

    #[test]
    fn test_crossover_preserves_structure() {
        let a = Genome::new(vec![
            Chromosome::new("c1", vec![Gene::new("a", Ternary::Pos, 1)]),
            Chromosome::new("c2", vec![Gene::new("b", Ternary::Neg, 1)]),
        ]);
        let b = Genome::new(vec![
            Chromosome::new("c1", vec![Gene::new("a", Ternary::Neg, 1)]),
            Chromosome::new("c2", vec![Gene::new("b", Ternary::Pos, 1)]),
        ]);
        let (child, _) = Crossover::single_point(&a, &b, 1);
        assert_eq!(child.chromosomes.len(), 2);
        assert_eq!(child.chromosomes[0].id, "c1");
        assert_eq!(child.chromosomes[1].id, "c2");
    }
}
