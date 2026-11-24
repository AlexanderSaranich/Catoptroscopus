# Catoptroscopus — Mirror Lab

An interactive playground for mirrors, chirality and enantiomorphs, rendered in Rust with [macroquad](https://github.com/not-fl3/macroquad). It has four modes: **2D shapes**, **3D models**, an animated **Hands** section and a **Theory** primer. Every mode except Theory has a live **Mathematics** panel showing the quantities behind what you see.

## Requirements

- Rust (stable) with Cargo, via [rustup](https://rustup.rs). On Windows: `winget install Rustlang.Rustup`.
- Windows: the MSVC C++ build tools (Visual Studio Build Tools).
- A GPU with OpenGL support; the Hands mode uses depth-buffered 3D meshes.

## Run

```sh
cargo run --release
```

Open a mode directly with optional start-up arguments:

```sh
cargo run --release -- <mode> [index] [seconds]
```

| Argument | Meaning |
| --- | --- |
| `mode` | `2d`, `3d`, `hands` or `theory` |
| `index` | shape number (2D), model number (3D) or scene number (Hands), counting from 0 |
| `seconds` | Hands only: start this far into the scene |

For example, `cargo run --release -- hands 3` opens the 4D scene, and `cargo run --release -- 3d 5` opens the trefoil knot.

The UI loads a system font with Greek and maths glyphs (Segoe UI on Windows, Arial Unicode on macOS, DejaVu Sans on Linux) and falls back to macroquad's built-in ASCII font. The default window is 1360 × 800 and resizable; the sidebar scrolls with the mouse wheel when it doesn't fit.

## 2D shapes (key `1`)

- **13 shapes.** Chiral: F pentomino, mitten, L tetromino, S tetromino, scalene triangle, parallelogram, pinwheel, triskelion. Achiral: isosceles triangle, T tetromino, arrow, square, star. Each is classified automatically from its symmetry group.
- **Mirrors.** Place and aim any number of mirror lines; reflections of reflections are drawn up to a chosen number of bounces. **Teal** = same hand (even number of reflections), **orange** = opposite hand (odd number). A curled arrow inside each copy reverses under reflection, making handedness visible.
- **Presets:** single mirror, 90° / 60° / 45° kaleidoscopes, parallel mirrors, triangle of mirrors.
- **Superimpose challenge (`C`).** Drag and rotate a free mirror-image copy onto the original. It snaps into place when it fits, which only happens for achiral shapes; for chiral ones the app explains why no slide or turn works.
- **Flip through 3D (`F`).** An animated flip squashes the shape through det = 0 and back, showing that flipping a 2D shape needs the third dimension.
- **Show own symmetry lines (`S`)** draws a shape's mirror lines and its rotation order.
- **Mathematics panel:**
  - the linear part A of the selected shape and det A (proper or improper)
  - the 2D symmetry group: Cn (rotations only, chiral) or Dn (n mirror lines, achiral)
  - the reflection matrix for a mirror at angle θ, and det = (−1)^k after k reflections
  - a vertex-based, Zabrodsky–Avnir-style **continuous chirality measure** S, exactly 0 for achiral shapes
  - with the copy on: det(A_copy) · det(A_orig), showing whether the two are enantiomorphs

## 3D models (key `2`)

- **7 models:** CHFClBr (chiral carbon), CH₂ClBr (achiral), a right-handed helix, a twisted propeller, a flat fan (achiral), a **trefoil knot** and a **figure-eight knot**.
- The original's reflection in a mirror plane updates live as you rotate it; the image can also be rotated on its own.
- **Try to superimpose (`Space`)** searches for the best rigid overlay and slides the image onto the original. Red dashes mark atoms that can't line up, and the best-fit RMS distance is reported.
- **Swap H and F on the image** (CHFClBr only) shows that swapping any two groups turns one enantiomer into the other.
- **Mathematics panel**, computed live from the coordinates:
  - the **point group** (C1, Cs, C2, C3, C3v, D3, D2, …), found by searching for every symmetry operation, with proper and improper operations counted separately
  - the signed volume χ = sign det[v₁−v₄, v₂−v₄, v₃−v₄] with **CIP R/S** labels (priorities Br > Cl > F > H)
  - helix **torsion** τ = b/(a² + b²), with its sign giving the handedness
  - knot **writhe** (discretised Gauss integral, which flips sign in the mirror) and the **Jones polynomials** V(t) and V(t⁻¹)
- The figure-eight shows the difference between rigid and topological chirality. As a rigid shape it is D2 (chiral as a solid), yet as a knot it is amphichiral: it can be deformed, not rotated, into its mirror image.

## Hands (key `3`)

A narrated, scrubbable animation in five scenes:

1. **A hand and its reflection.** A left hand moves in front of a mirror and a right hand follows. M = diag(−1, 1, 1), det M = −1.
2. **Try to superimpose.** The right hand slides onto the left and tries four 180° turns; each fixes one feature and breaks another. Red dashes show fingertips that don't match.
3. **The handedness triad.** Arrows for fingers (a), thumb (b) and palm normal (c). χ = sign((a × b) · c) stays −1 for the left hand and +1 for the right however they tumble.
4. **A detour through the 4th dimension.** A rotation in the (x, w) plane flattens the hand (det = 0) and brings it back mirrored, so it superimposes on the original. Colour shows the hidden w coordinate.
5. **Helicity.** Spinning particles and their mirror images: the spin reverses but the motion doesn't, so h = S·p/|p| flips sign. Covers parity violation (Wu, 1957) and γ⁵.

Sleeve colours show handedness live: teal for left (χ = −1), orange for right (χ = +1), grey when flattened (det = 0).

**How the hands are made.** Each hand is a smooth signed-distance model: a palm with curved sides that swell out from a narrow wrist, a domed, slightly cupped cross-section and an arched knuckle line, thenar and hypothenar pads, three-segment curled fingers with knuckles that each blend into the palm but stay separate from one another, a jointed thumb, a flattened wrist and a sleeve cuff. It is turned into a mesh once at start-up with surface nets. Each frame the mesh is lit per vertex (wrapped diffuse light, a warm subsurface tint, specular highlights and rim light), with the fingernails and sleeve picked out by region. It is drawn through a GPU camera that matches the app's software projection exactly, so the depth buffer handles overlaps and the 2D overlays still line up.

## Theory (key `4`)

A scrollable primer on the mathematics of chirality, in six sections:

1. Geometry and group theory: E(3), proper and improper isometries, point groups, Sohncke space groups, and why dimension matters
2. The sign of a determinant: signed volumes, CIP R/S, and 2ⁿ stereoisomers
3. Pseudoscalars and measuring chirality: continuous chirality measures and the chiral-zero theorem
4. Curves and helices: torsion
5. Topology: knot chirality, Jones polynomials, Lk = Tw + Wr, and chiral manifolds
6. Physics: parity, helicity, γ⁵, parity violation and optical activity

Each section has buttons that open the matching live demo, and the sidebar jumps straight to any section.

## Controls

| Mode | Control | Action |
| --- | --- | --- |
| 2D | Drag shape / wheel / `Q` `E` / drag knob | Move / rotate (Shift snaps to 15°) |
| 2D | Drag mirror ends or line | Aim / move mirror |
| 2D | `M` / `Del`, `+` / `-` | Add / remove mirror, change reflection bounces |
| 2D | `F`, `C`, `S`, `A`, `Tab` / arrows | Flip through 3D, copy, symmetry lines, handedness arrows, change shape |
| 3D | Left-drag left / right side | Rotate original / mirror image |
| 3D | Right-drag or arrow keys, wheel | Orbit, zoom |
| 3D | `Space`, `Tab` | Superimpose / separate, next model |
| Hands | `Space`, `←` `→`, click the bar | Play/pause, change scene, scrub |
| Hands | Drag, wheel | Orbit camera, zoom |
| Theory | Wheel, arrow keys, `PgUp` `PgDn` | Scroll |
| All | `1`–`4`, wheel over sidebar | Switch mode, scroll sidebar |

## Project layout

| File | Contents |
| --- | --- |
| `src/main.rs` | Window, mode switching, sidebar, start-up arguments |
| `src/ui.rs` | Immediate-mode UI widgets, font loading, info boxes |
| `src/geom.rs` | 2D isometries, reflections, triangulation, 2D symmetry groups |
| `src/shapes.rs` | 2D shape library and continuous chirality measure |
| `src/app2d.rs` | 2D mode |
| `src/render3d.rs` | Software 3D projection, shaded spheres, and a matching GPU camera |
| `src/symmetry3d.rs` | 3D point-group detection, signed volume, torsion, writhe |
| `src/app3d.rs` | 3D models mode |
| `src/handmesh.rs` | Signed-distance hand model and surface-nets mesher |
| `src/apphands.rs` | Hands animation |
| `src/theory.rs` | Theory primer |

## Tests

```sh
cargo test
```

Four tests check:

- every 2D shape's chirality, rotation order, mirror lines, triangulation and chirality measure (0 exactly for achiral shapes)
- every 3D model's point group and rigid-fit chirality
- that χ, torsion and writhe flip sign under reflection
- that the hand mesh builds with sensible size, fingernails and sleeve, the thumb on the correct side, and draw-call-sized chunks

## Changes

1. **First version.** 2D mirror lab (shapes, mirrors, kaleidoscopes, superimpose challenge) and 3D molecules with a mirror plane and a best-fit superimpose search.
2. **Mathematics and Hands.** Added the Hands animation, the Theory primer and live maths panels in every mode: det and symmetry groups and a chirality measure in 2D; point groups, R/S, torsion, and the trefoil and figure-eight knots with writhe and Jones polynomials in 3D. Also added the animated flip through 3D, a Unicode UI font, a scrollable sidebar, start-up arguments and a smaller default window. The 3D renderer moved to a shared right-handed camera so that displayed handedness matches the maths.
3. **Realistic hands.** Replaced the bead-chain hands with smooth anatomical meshes, lit per vertex and drawn with the GPU depth buffer.
4. **More natural hands.** Replaced the boxy palm with a curved outline and domed cross-section, and added webbing so the fingers meet the palm in rounded wedges.

## License

MIT
