# perfect-phylogeny-rs

Simple test for obtain perfect phylogeny tree from laminar matrix.

## Usage
```rust
use perfect_phylogeny_rs::PerfectPhylogeny;

fn main() {
    let per_phy = PerfectPhylogeny::from_file("input/matrix.txt", false);
    per_phy.get_dot("output/final.dot");
}
```

## Example
Input:
```
1   1   0   0   0
0   0   1   0   0
1   1   0   0   1
0   0   1   1   0
0   1   0   0   0
```
Output:

![](https://raw.githubusercontent.com/dlcgold/perfect-phylogeny-rs/main/output/final.png)

## TODO
- [ ] Documentation
