# laminated
Laminated puzzle simulator.

## What is it
Consider a set $R$ of objects called _rays_ and a group $G$ that acts transitively on the set of rays. We can define an equivalence relation on rays where $r \sim r'$ whenever $`G_r = G_{r'}`$, where $`G_r`$ is the stabilizer of $r$ in $G$. The equivalence classes are then called _axes_. Then, given an axis, a _grip_ along that axis assigns a certain 'distance' along each ray in the axis. A group element $g$ applies to a grip to produce a new grip by transforming each axis in the corresponding ray. We can then create a finite set of grips closed under the group action, and construct the set of all _pieces_, where a piece has one grip along each axis. This is a _laminated puzzle_. The puzzle can be twisted by a group element $g$ and a grip whose axis is stabilized by $g$ by applying $g$ to each piece that contains that grip.

With this construction, it can be seen that laminated puzzles contain a subset of the pieces of complex puzzles. However, laminated puzzles are closer to real puzzles in that they have parallel layers that do not intersect. If the set of rays exists on a sphere, the laminated puzzle is the puzzle that contains all holding point pieces.

## Features
laminated supports laminated face-turning cubes, octahedra, dodecahedra, and rhombic dodecahedra.

## Running
Clone this repository and run `cargo run --release` in the directory.