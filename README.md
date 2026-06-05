# ternary-genome — Genetic encoding and expression for evolving ternary agent populations

Genome, chromosome, gene, mutation rate, crossover, and gene expression structures for ternary-valued genetic algorithms. Every allele is a balanced ternary value.

## Why This Exists

Ternary agent populations need to evolve. Standard binary genetic algorithms lose the "neutral" state that's central to ternary logic. This crate provides genetic structures where every gene carries {-1, 0, +1} alleles — enabling agents to encode "don't care" as a first-class genetic state, not just a coin flip between two extremes.

## Core Concepts

- **Balanced ternary** — Three values: Neg (-1), Zero (0), Pos (+1). Zero means "neutral" or "no preference" — a real genetic state, not an absence of information.
- **Gene** — One trait with a ternary allele and a dominance score. Higher dominance wins during expression when two parents contribute different alleles.
- **Chromosome** — A named cluster of genes. Supports mutation at a configurable rate (per-thousand per gene).
- **Genome** — The full genetic code: a collection of chromosomes. Provides flat allele access and a simple fitness metric (sum of allele values).
- **MutationRate** — Adaptive mutation rate that increases when fitness stagnates and decreases when fitness improves. Clamped between configurable min/max bounds.
- **GeneExpression** — Converts genotype to phenotype. Supports single-chromosome expression, full-genome expression, and parent-combined expression where dominance determines which allele wins.
- **Crossover** — Sexual reproduction: single-point crossover (split at a point, swap tails) and uniform crossover (each gene randomly from one parent).

## Quick Start

```toml
[dependencies]
ternary-genome = "0.1"
```

```rust
use ternary_genome::*;

// Build a genome with two chromosomes
let genome = Genome::new(vec![
    Chromosome::new("movement", vec![
        Gene::new("speed", Ternary::Pos, 3),
        Gene::new("caution", Ternary::Neg, 1),
    ]),
    Chromosome::new("perception", vec![
        Gene::new("range", Ternary::Pos, 2),
        Gene::new("focus", Ternary::Zero, 1),
    ]),
]);

// Express the genome into a phenotype
let phenotype = GeneExpression::from_genome(&genome);
assert_eq!(phenotype.trait_value("speed"), Some(Ternary::Pos));

// Crossover two genomes
let other = /* ... another genome ... */;
let (child_a, child_b) = Crossover::single_point(&genome, &other, 2);
```

## API Overview

| Type | Purpose |
|------|---------|
| `Ternary` | Core ternary value: Neg, Zero, Pos |
| `Gene` | One trait with ternary allele and dominance score |
| `Chromosome` | Named cluster of genes with mutation support |
| `Genome` | Full genetic code (multiple chromosomes) |
| `MutationRate` | Adaptive mutation rate with self-tuning |
| `GeneExpression` | Genotype-to-phenotype conversion with dominance |
| `Crossover` | Single-point and uniform crossover operators |

## How It Works

Each gene stores a ternary allele (-1, 0, or +1) and a dominance score. During expression, if two parents contribute the same gene, the parent with higher dominance wins. Mutation flips alleles to random ternary values at a configurable per-gene rate.

The `MutationRate` struct adapts automatically: call `adapt(true)` when fitness improved (rate decreases) or `adapt(false)` when it stagnated (rate increases). This implements a simple 1/5th-rule-inspired adaptation without requiring you to tune mutation rates by hand.

Crossover preserves genome structure — chromosome names, gene names, and dominance scores are inherited from templates, only alleles are shuffled.

## Known Limitations

- **No inversion or duplication operators** — Only point mutation and crossover are provided. Structural mutations (gene duplication, chromosome inversion) are not implemented.
- **Simple PRNG** — Uses a linear congruential generator (xorshift-based), not a cryptographic RNG. Do not use for security-sensitive applications.
- **Fitness is naive** — The built-in fitness function is just the sum of allele values. Real applications should compute domain-specific fitness externally.
- **No epistasis modeling** — Gene interactions are not modeled; each gene contributes independently.
- **Dominance is static** — Dominance scores are set at creation and never change during evolution.

## Use Cases

1. **Evolving room strategies** — Encode agent behavioral traits (aggression, exploration, cooperation) as ternary genes and evolve populations over generations.
2. **Ternary neural architecture search** — Use genes to encode layer sizes, activation choices, and connection patterns for ternary networks.
3. **Adaptive trading agents** — Evolve ternary strategies (buy/hold/sell) across multiple market conditions, with dominance expressing which strategy wins in conflicting signals.

## Ecosystem Context

Part of the SuperInstance ternary crate family. `ternary-genome` provides the genetic algorithm foundation that `ternary-ga` and `ternary-evolution-advanced` build upon. It feeds evolved genomes into `ternary-agent` for instantiation and `ternary-ecosystem` for population-level simulation.

## See Also

- **ternary-ga** — Genetic algorithms with ternary genomes
- **ternary-fitness** — Fitness landscape analysis for ternary strategies
- **ternary-popgen** — Population genetics for ternary agents
- **ternary-evolution-advanced** — Advanced evolutionary optimization
- **ternary-cell** — Cellular computing with ternary state machines

## License

MIT
