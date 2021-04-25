# perfect-phylogeny-rs

Simple test for obtain perfect phylogeny tree from laminar matrix.

## Usage
```rust
use perfect_phylogeny_rs::PerfectPhylogeny;

fn main() {
    let per_phy = PerfectPhylogeny::from_file("input/matrix.txt");
    per_phy.get_dot("output/final.dot");
}
```
