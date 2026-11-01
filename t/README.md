# About

Render text and images to Unicode characters

# CLI

```bash
ttfun -h
```

```text
!run:../target/release/ttfun -h
```

```bash
ttfun --help
```

```text
!run:../target/release/ttfun --help
```

## Version

```bash
ttfun -V
```

```text
!run:../target/release/ttfun -V
```

## Examples

```bash
ttfun
```

```text
!run:../target/release/ttfun
```

### Font `eb-garamond` 36

```bash
ttfun -f eb-garamond -s 36
```

```text
!run:../target/release/ttfun -f eb-garamond -s 36
```

### Rescale `2x1`

```bash
ttfun -r 2x1
```

```text
!run:../target/release/ttfun -r 2x1
```

### Rescale `3x1`

```bash
ttfun -r 3x1
```

```text
!run:../target/release/ttfun -r 3x1
```

### Invert

```bash
ttfun -n
```

```text
!run:../target/release/ttfun -n
```

### Style `octant`

```bash
ttfun -S octant
```

```text
!run:../target/release/ttfun -S octant
```

### Style `octant` inverted

```bash
ttfun -S octant -n
```

```text
!run:../target/release/ttfun -S octant -n
```

### Style `octant-sep`

```bash
ttfun -S octant-sep
```

```text
!run:../target/release/ttfun -S octant-sep
```

### Style `octant-sep` inverted

```bash
ttfun -S octant-sep -n
```

```text
!run:../target/release/ttfun -S octant-sep -n
```

### Style `sextant`

```bash
ttfun -S sextant
```

```text
!run:../target/release/ttfun -S sextant
```

### Style `sextant` inverted

```bash
ttfun -S sextant -n
```

```text
!run:../target/release/ttfun -S sextant -n
```

### Style `sextant-sep`

```bash
ttfun -S sextant-sep
```

```text
!run:../target/release/ttfun -S sextant-sep
```

### Style `sextant-sep` inverted

```bash
ttfun -S sextant-sep -n
```

```text
!run:../target/release/ttfun -S sextant-sep -n
```

### Style `quadrant`

```bash
ttfun -S quadrant
```

```text
!run:../target/release/ttfun -S quadrant
```

### Style `quadrant` inverted

```bash
ttfun -S quadrant -n
```

```text
!run:../target/release/ttfun -S quadrant -n
```

### Style `quadrant-sep`

```bash
ttfun -S quadrant-sep
```

```text
!run:../target/release/ttfun -S quadrant-sep
```

### Style `quadrant-sep` inverted

```bash
ttfun -S quadrant-sep -n
```

```text
!run:../target/release/ttfun -S quadrant-sep -n
```

### Style `full`

```bash
ttfun -S full
```

```text
!run:../target/release/ttfun -S full
```

### Style `full` inverted

```bash
ttfun -S full -n
```

```text
!run:../target/release/ttfun -S full -n
```

### Style `full-color`

```bash
ttfun -S full-color
```

```text
!run:../target/release/ttfun -S full-color
```

### Style `full-color` inverted

```bash
ttfun -S full-color -n
```

```text
!run:../target/release/ttfun -S full-color -n
```

### Style `full-shade`

```bash
ttfun -S full-shade
```

```text
!run:../target/release/ttfun -S full-shade
```

### Style `full-shade` inverted

```bash
ttfun -S full-shade -n
```

```text
!run:../target/release/ttfun -S full-shade -n
```

### Demo

```bash
ttfun -d
```

```text
!run:../target/release/ttfun -d
```

### Render `img/tree.webp`

![](img/tree.webp)

---

```bash
ttfun -I img/tree.webp -r 0.25 -S octant-sep
```

```text
!run:../target/release/ttfun -I ../img/tree.webp -r 0.25 -S octant-sep
```

---

```bash
ttfun -I img/tree.webp -r 0.125x0.0625 -S full-color
```

```text
!run:../target/release/ttfun -I ../img/tree.webp -r 0.125x0.0625 -S full-color
```

---

```bash
ttfun -I img/tree.webp -r 0.125x0.0625 -S full-color -n
```

```text
!run:../target/release/ttfun -I ../img/tree.webp -r 0.125x0.0625 -S full-color -n
```

# Library

!inc:LIB.md

# Fonts

This crate includes the following unmodified fonts in its source code and embeds them in its binary
executable.

## EB Garamond

Copyright 2017 The EB Garamond Project Authors (<https://github.com/octaviopardo/EBGaramond12>)

This Font Software is licensed under the SIL Open Font License, Version 1.1.

## Iosevka

Copyright (c) 2015-2026, Renzhi Li (aka. Belleve Invis, <belleve@typeof.net>)

This Font Software is licensed under the SIL Open Font License, Version 1.1.

[Block Elements]:
https://www.unicode.org/Public/latest/charts/nameslist/2580/

[Braille Patterns]:
https://www.unicode.org/Public/latest/charts/nameslist/2800/

[Symbols for Legacy Computing]:
https://www.unicode.org/Public/latest/charts/nameslist/1fb00/

[Symbols for Legacy Computing Supplement]:
https://www.unicode.org/Public/latest/charts/nameslist/1cc00/

