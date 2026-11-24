# Catoptroscopus — Mirror Lab

An interactive playground for mirrors, chirality and enantiomorphs, rendered in Rust with [macroquad](https://github.com/not-fl3/macroquad). Four modes: 2D shapes, 3D models, an animated Hands section and a Theory primer. Every mode has a live **Mathematics** panel.

## Run

```sh
cargo run --release
cargo run --release -- hands 3      # open a mode directly: 2d|3d|hands|theory [index] [seconds]
```

The UI uses a system font with Greek and maths glyphs (Segoe UI on Windows, Arial Unicode on macOS, DejaVu Sans on Linux) and falls back to macroquad's built-in ASCII font.

## 2D shapes (key `1`)

- 13 shapes, chiral (F pentomino, mitten, L, S, scalene triangle, parallelogram, pinwheel, triskelion) and achiral (isosceles triangle, T, arrow, square, star).
- Place and aim any number of mirror lines; reflections of reflections are drawn up to a chosen number of bounces. **Teal** = same hand (even number of reflections), **orange** = opposite hand (odd number). A curled arrow inside each copy reverses under reflection.
- Presets: single mirror, 90° / 60° / 45° kaleidoscopes, parallel mirrors, triangle of mirrors.
- **Superimpose challenge (`C`)**: drag and rotate a free mirror-image copy onto the original; it snaps when it fits.
- **Flip through 3D (`F`)**: an animated flip that squashes the shape through det = 0, showing why a 2D enantiomer needs a third dimension.
- Maths panel: the linear part A and det A, the 2D point group (Cn = chiral, Dn = achiral), the reflection matrix, (−1)^k for k bounces, and a Zabrodsky–Avnir-style **continuous chirality measure** S (exactly 0 for achiral shapes).

## 3D models (key `2`)

- CHFClBr (chiral carbon), CH₂ClBr (achiral), helix, twisted propeller, flat fan, **trefoil knot**, **figure-eight knot**.
- The original's reflection in a mirror plane updates live; the image can also be rotated independently.
- **Try to superimpose (`Space`)** searches for the best overlay and slides the image onto the original, marking atoms that can't line up.
- Maths panel, computed live from the coordinates:
  - the **point group** (C1, Cs, C2, C3, C3v, D3, …), found by searching for all symmetry operations, with proper and improper operations counted separately
  - the signed volume χ = sign det[v₁−v₄, v₂−v₄, v₃−v₄] with **CIP R/S** labels (try swapping two groups)
  - helix **torsion** τ = b/(a² + b²) with its sign
  - knot **writhe** (discretised Gauss integral) and **Jones polynomials** V(t) and V(t⁻¹)

## Hands (key `3`)

A narrated, scrubbable animation in five scenes. The hands are smooth anatomical meshes: a signed-distance model (palm, thumb and pinky-side pads, three-jointed curled fingers, thumb, wrist and sleeve) polygonised with surface nets, lit per vertex and drawn with the GPU depth buffer.


1. **A hand and its reflection**: M = diag(−1, 1, 1), det M = −1.
2. **Try to superimpose**: the right hand tries four rotations; red dashes show fingertips that don't match.
3. **The handedness triad**: fingers × thumb · palm normal is a pseudoscalar that stays fixed while the hands tumble.
4. **A detour through the 4th dimension**: a rotation in the (x, w) plane flattens the hand (det = 0) and brings it back mirrored, so it superimposes. Colour shows w.
5. **Helicity**: spinning particles and their mirror images; h = S·p/|p| flips sign. Covers parity violation and γ⁵.

## Theory (key `4`)

A scrollable primer on the mathematics of chirality: isometries and E(3), point and space groups, signed volumes and R/S, pseudoscalars and the chiral-zero theorem, torsion, knots and Lk = Tw + Wr, and parity, helicity and γ⁵. Each section has buttons that open the matching live demo.

## Controls

| Mode | Control | Action |
| --- | --- | --- |
| 2D | Drag shape / wheel / `Q` `E` / knob | Move / rotate (Shift snaps 15°) |
| 2D | Drag mirror ends or line | Aim / move mirror |
| 2D | `M` / `Del`, `+` / `-` | Add / remove mirror, reflection bounces |
| 2D | `F`, `C`, `S`, `A`, `Tab` | Flip, copy, symmetry lines, arrows, next shape |
| 3D | Left-drag left / right side | Rotate original / mirror image |
| 3D | Right-drag or arrow keys, wheel | Orbit, zoom |
| 3D | `Space`, `Tab` | Superimpose, next model |
| Hands | `Space`, `←` `→`, click bar | Play/pause, scene, scrub |
| All | `1`–`4`, wheel over sidebar | Switch mode, scroll sidebar |

## Tests

```sh
cargo test
```

Checks every 2D shape's chirality, rotation order, mirror lines and chirality measure, and every 3D model's point group and rigid-fit chirality. Also checks that χ, torsion and writhe flip sign under reflection.

## License

MIT
