# Seeds

Starting points drawn from Jared's lineage. Each seed has a lineage reference, a
technique, and the one rule that makes it that thing. Seeds are concepts, not
recipes. Offer one, hold it loosely, and drop it the moment Jared has his own
idea.

How to use this bank:

- Check `scripts/studio list` and the last session notes first. If the last
  pieces were 2D, pick from another section. Range is the point.
- Offer one seed, not a menu. If Jared shrugs, offer one from a different
  section.
- Say the rule out loud. The rule is what keeps the piece from turning into a
  cliché.
- Seeds marked **sound: monitor live; recording is Phase 2** can have sound
  while working, through `nannou_audio`. The hung piece is silent until
  recording lands, so the image must stand on its own. Keep `audio = false` in
  `piece.toml`.
- Seeds marked **needs material** read a source file from
  `/Users/jaredmcfarland/Developer/sketches-private/materials/`. If the folder
  has nothing fitting, say so and offer two ways on: a synthetic source made in
  code (named in the seed), or ask Jared to drop a file in and carry on with
  another seed while he finds one.
- Nannou's own examples show how each medium works in this version:
  `/Users/jaredmcfarland/Developer/nannou-sketches/examples/` (`draw/`,
  `bevy/`, `compute/`, `video/`, `audio/`, `wgpu/`). Read the matching one
  before writing the first shader, 3D, video or audio piece.

## 2D

- **Hairline field.** Lineage: Ryoji Ikeda, *test pattern*. Technique: a
  thousand one-pixel horizontal lines whose spacing is set by a single slowly
  changing function; nothing else moves. Rule: every mark is exactly one pixel
  wide, and the only variable is spacing.
- **Grid and its violation.** Lineage: Mark Fell, *Sentielle Objectif
  Actualité*; Swiss grid design. Technique: a strict integer grid of identical
  marks; one cell per beat breaks the grid by a fixed, exact offset and returns.
  Rule: the grid is never wrong except in one place at a time.
- **Moiré of two grids.** Lineage: Bridget Riley; Carsten Nicolai's
  interference work. Technique: two identical line grids, one rotated by a tiny
  angle that sweeps across the loop, so the beat pattern between them is the
  image. Rule: only two grids, both plain; everything you see is their
  interference.
- **Lissajous ratios.** Lineage: Jared's own Curves engine and his first AV
  piece. Technique: a single curve tracing a 3:4 or 5:7 ratio, drawn with a
  short decaying history of points, never a full trail. Rule: one curve, one
  ratio, and the loop length closes the figure exactly.
- **Stochastic score.** Lineage: Iannis Xenakis, *Pithoprakta* and the UPIC
  drawings. Technique: short line segments placed by a Poisson process in time
  and a Gaussian in pitch (vertical position), seeded so the loop repeats.
  Rule: the distribution is the composition; no hand-placed marks.

## 3D

- **Orthographic city of bars.** Lineage: Ikeda, *data.matrix*; Raster-Noton
  sleeves. Technique: a field of thin boxes in orthographic projection, heights
  driven by a data stream, lit only by value (no colored lights). Rule:
  orthographic camera, no perspective, no color.
- **One plane, turning.** Lineage: Donald Judd's stacks; Larry Bell's glass
  cubes. Technique: a single thin plane rotating slowly about an off-center
  axis, its edge catching a hard light once per loop. Rule: one object, one
  light, one event per loop.
- **Point cloud of a signal.** Lineage: Curtis Roads, *Microsound*; oscilloscope
  phase plots. Technique: plot a signal in three dimensions (x = t, y =
  value, z = value delayed by a few samples) as points; rotate the camera
  slowly around the attractor shape that forms. Rule: points only, no lines,
  no surfaces.
- **Depth as value.** Lineage: Nicolas Bernier, *frequencies*; film depth
  passes. Technique: a 3D scene rendered so that brightness is depth alone
  (near is white, far is black), with geometry that rearranges on a grid.
  Rule: value carries all the information; no shading model.

## Shaders

- **Quantized field.** Lineage: Mille Plateaux's *Clicks & Cuts*; bitcrushing.
  Technique: a fragment shader computes a smooth field (two interfering sine
  waves), then quantizes it to four levels with hard edges. Rule: exactly four
  values; the beauty is in the edges.
- **Interference and beating.** Lineage: Alvin Lucier, *Still and Moving
  Lines of Silence in Families of Hyperbolas*; acoustic beating. Technique: two
  radial wave sources at slightly different frequencies; the slow beat
  between them sweeps the frame once per loop. Rule: two sources, close
  frequencies, nothing else.
- **Cellular rule.** Lineage: Stephen Wolfram's elementary automata; Autechre's
  rule-based patterning. Technique: a compute or feedback shader runs a 1D rule
  (rule 30, 110) one row per frame, scrolling. Rule: the rule is fixed; only
  the starting row changes. Make the loop seamless by choosing the period, or
  hang it as a fixed duration.
- **Signed distance grid.** Lineage: Ikeda, *spectra*; technical drawing.
  Technique: an SDF of simple shapes rendered as contour lines only, one pixel
  wide, while the shapes drift on integer steps. Rule: contours only, never a
  filled shape.
- **Error as form.** Lineage: Oval, *94diskont*; Kim Cascone, "The Aesthetics
  of Failure." Technique: a clean shader image with a controlled precision
  error (half-float rounding, a deliberately truncated coordinate) that grows
  across the loop and resets. Rule: the error is the only thing that changes.

## Image and video material

- **Granular image.** Lineage: Curtis Roads, *Microsound*; granular samplers.
  **needs material** (synthetic fallback: a photograph-like gradient field from
  layered noise in grayscale). Technique: draw thousands of small grains, each
  a tiny rectangle sampled from a random region of the source, with grain size,
  density and spray as the parameters. Rule: the source is felt but never
  shown whole.
- **Slit-scan.** Lineage: Douglas Trumbull's *2001* stargate; Adam Magyar,
  *Stainless*. **needs material** (synthetic fallback: a rendered moving
  pattern stored as a ring buffer of frames). Technique: each column of the
  output comes from a different moment of the source, so time becomes an axis
  of space. Rule: one axis is time; never show an unprocessed frame.
- **Pixel sort to a threshold.** Lineage: Kim Asendorf's pixel sorting; Oval's
  skipping CDs. **needs material** (synthetic fallback: a high-contrast
  generated image of rectangles and grain). Technique: sort runs of pixels by
  brightness wherever they cross a threshold, and sweep the threshold through
  the loop. Rule: grayscale only; the threshold is the instrument.
- **Re-quantized frame.** Lineage: Alva Noto and Ikeda's bit-depth play;
  Raster-Noton sleeves. **needs material** (synthetic fallback: a generated
  smooth gradient field). Technique: reduce a source to 1-bit with ordered
  dithering (Bayer matrix), then animate the dither matrix's offset by whole
  pixels. Rule: two values only, black and white.
- **Frame as sample.** Lineage: Jared's own sampling practice; Akufen's
  microhouse cut-ups. **needs material** (synthetic fallback: a short
  rendered loop of moving geometry). Technique: chop a short clip into sixteen
  slices and resequence them to a rhythm, like chopping a break. Rule: the
  slices are re-ordered, never altered.

## Numerals and data

- **Timecode.** Lineage: Ikeda, *datamatics*; film post-production. Technique:
  a field of running timecodes and frame counters in mono type, each counting
  at a different rate, aligned to a grid. Rule: everything on screen is a true
  number derived from the clock.
- **Test card.** Lineage: Alva Noto, *Unitxt*; broadcast test patterns.
  Technique: a precise test pattern (bars, a crosshair, a gray ramp, a
  resolution wedge) where one element at a time slowly drifts out of
  calibration. Rule: built from real test-pattern elements only.
- **Data as material.** Lineage: Ikeda, *data.tron*. Technique: read a real
  data set (a file of numbers, the bytes of the piece's own source code, or a
  hash stream) and draw it as a dense raster of bits, scrolling. Rule: the data
  is real, and nothing decorative is added.
- **Prime sieve.** Lineage: Ikeda, *the transfinite*; Mark Fell's numeric
  patterning. Technique: integers laid on a grid, with multiples struck out in
  order as the loop runs, so the primes remain. Rule: every mark has an
  arithmetic reason.

## Time and rhythm

- **Polyrhythm clock.** Lineage: Autechre, *Gantz Graf*; Steve Reich's
  phasing. Technique: three or four rings of ticks rotating at 3:4:5:7, each
  tick lighting once when it passes a fixed line; the loop length is a common
  multiple, so all of them realign at the seam. Rule: one fixed line, and every
  event is a crossing. **sound: monitor live; recording is Phase 2** (each
  crossing can be a click).
- **Phase drift.** Lineage: Steve Reich, *Piano Phase*. Technique: two
  identical patterns, one running very slightly faster, so they slide out of
  phase and back in by the end of the loop. Rule: two identical patterns; the
  only difference is speed.
- **Hold and act.** Lineage: Mark Fell; Ikeda's silence before a hit.
  Technique: a frame that holds almost still for most of the loop, then one
  exact event happens on a frame boundary. Rule: one event per loop, landed on
  an exact frame.
- **Granular time.** Lineage: Curtis Roads; time-stretching. Technique: a
  simple motion (a line sweeping) is played back as overlapping grains of
  itself at different time offsets, so the motion smears and stutters. Rule:
  the underlying motion is simple enough to name in four words.
- **One signal, two faces.** Lineage: Ikeda and Alva Noto's audiovisual work;
  Jared's Curves principle. Technique: a single oscillator bank drives both the
  image (as a waveform or a vectorscope) and the audio output from the same
  phase. Rule: one signal and one clock, never synced after the fact.
  **sound: monitor live; recording is Phase 2.**
- **Clicks and cuts.** Lineage: Mille Plateaux, *Clicks & Cuts*; Pole.
  Technique: a black frame interrupted by short white impulses (a single
  frame, a single column, a single row) on a sparse, swung rhythm. Rule: an
  impulse is never longer than one frame. **sound: monitor live; recording is
  Phase 2** (each impulse is a click).

## Writing a seed of your own

Name something specific from Jared's world (a record, a label, a composer, a
grade, an instrument, a place), then answer three questions: what is the
technique, what is the medium (2D, 3D, shader, material, numerals, time), and
what is the one rule. If you cannot state the rule, the seed is not sharp enough
yet. Check it against the cliché list in `docs/aesthetic.md` before offering
it.
