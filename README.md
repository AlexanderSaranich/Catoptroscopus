# Catoptroscopus — Mirror Lab

An interactive playground for mirrors, chirality and enantiomorphs, rendered in Rust with [macroquad](https://github.com/not-fl3/macroquad).

## Run

```sh
cargo run --release
```

## 2D shapes (key `1`)

- 13 shapes, chiral (F pentomino, mitten, L, S, scalene triangle, parallelogram, pinwheel, triskelion) and achiral (isosceles triangle, T, arrow, square, star). Each is classified automatically from its symmetry group.
- Place and aim any number of mirror lines; reflections of reflections are drawn up to a chosen number of bounces.
  - **Teal** = same hand (even number of reflections), **orange** = opposite hand (odd number).
  - A curled arrow inside each copy reverses under reflection, making handedness visible.
- Presets: single mirror, 90° / 60° / 45° kaleidoscopes, parallel mirrors, triangle of mirrors.
- **Superimpose challenge (`C`)**: drag and rotate a free mirror-image copy onto the original. It snaps into place for achiral shapes; chiral ones never fit unless you flip (`F`), which needs a trip through 3D.

| Control | Action |
| --- | --- |
| Drag shape | Move |
| Wheel / `Q` `E` / drag knob | Rotate (Shift snaps 15°) |
| Drag mirror ends / line | Aim / move mirror |
| `M` / `Del` | Add / remove mirror |
| `+` / `-` | Reflection bounces |
| `F` | Flip selected shape |
| `Tab` / arrows | Change shape |
| `S` / `A` | Symmetry lines / handedness arrows |

## 3D molecules (key `2`)

- Ball-and-stick models: CHFClBr (chiral carbon), CH₂ClBr (achiral), helix, twisted propeller, flat fan (achiral).
- The original's reflection in the mirror plane updates live; the image can also be rotated independently.
- **Try to superimpose (`Space`)** searches for the best overlay and slides the image onto the original, reporting whether it fits and marking atoms that can't line up.
- **Swap H and F** on the image shows that swapping two groups converts one enantiomer into the other.

| Control | Action |
| --- | --- |
| Left-drag left side | Rotate original |
| Left-drag right side | Rotate mirror image |
| Right-drag / arrow keys | Orbit camera |
| Wheel | Zoom |
| `Tab` | Next model |

## Tests

```sh
cargo test
```

Checks the chirality, rotation order and mirror-line count of every 2D shape, and the chirality of every 3D model.

## License

MIT
