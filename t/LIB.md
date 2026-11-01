```rust
use ttfun::*;

// Render text to font image
let iosevka = FontArc::try_from_vec(IOSEVKA.to_vec()).expect("font");
let font_size = 18.0;
let padding = Padding::all(0);
let rescale = Rescale::parse("1").expect("rescale");
let img = text_to_image(
    "ttfun",
    &iosevka,
    font_size,
    &rescale,
    &padding,
)
.expect("image");

// Render font image to `full` style
let invert = false;
let threshold = 180;
let full_char = "\u{2588}";
let full = Style::Full.render(
    &img,
    invert,
    threshold,
    full_char,
)
.expect("render");

assert_eq!(
    full,
    "\
                                      
                                      
                                      
  ██      █       ██                  
  ██     ██      ████                 
  ██     ██      █                    
 █████  █████    █     █   █  █ ██    
 █████  █████  ██████ ██   █  █████   
  ██     ██      █    ██   █  █   █   
  ██     ██      █    ██   █  █   █   
  ██     ██      █    ██   █  █   █   
  ██  █  ██  █   █    ██   █  █   █   
  ██ ██   █ ██   █     █  ██  █   █   
   ███    ███    █     █████  █   █   
                                      
                                      
                                      
                                      
\
    ",
);
```

