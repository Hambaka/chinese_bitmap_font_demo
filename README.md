# chinese_bitmap_font_demo

A useless toy project (Why do I write this?)

## Introduction

Generate 10px(9px + drop shadows) or 11px(9px + stroke outline) Chinese characters bitmap font with [Fusion Pixel Font](https://github.com/TakWolf/fusion-pixel-font).

## Usage

```(text)
Usage: chinese_bitmap_font_demo.exe [OPTIONS] --text <FILE> --font <FILE> --output <FILE>

Options:
  -t, --text <FILE>    Game script/text file for generating bitmap font image
  -f, --font <FILE>    Font file for generating bitmap font image
  -s, --size <SIZE>    Font size(px), only support 10px or 11px [default: 10]
  -i, --is-zh-hant     Whether the font is zh-hant or zh-hans, for punctuation marks offset
  -o, --output <FILE>  Output bitmap font image file (PNG only)
  -h, --help           Print help
  -V, --version        Print version
```

## Example

Generate 10px Simplified Chinese bitmap font with the script file `script-zh_hans.txt` and the font file `fusion-pixel-10px-proportional-zh_hans.ttf`, then save the image to `zh_hans_image.png`  

```(bash)
chinese_bitmap_font_demo -t path\to\script-zh_hans.txt -f path\to\fusion-pixel-10px-proportional-zh_hans.ttf -s 10 -o path\to\zh_hans_image.png
```

Generate 11px Traditional Chinese bitmap font with the script file `script-zh_hant.txt` and the font file `fusion-pixel-11px-proportional-zh_hant.ttf`, then save the image to `zh_hant_image.png`. Don't forget to add `-i`/`--is-zh-hant` option!

```(bash)
chinese_bitmap_font_demo -t path\to\script-zh_hant.txt -f path\to\fusion-pixel-10px-proportional-zh_hant.ttf -s 11 -i -o path\to\zh_hant_image.png
```

## Config

Config file is `config.toml`, will be generated during the first run, and will be saved in the same directory as the executable file.

Default config:

```(toml)
img_bg_color = [
    45,
    45,
    45,
]
char_color = [
    250,
    250,
    245,
]
char_shadow_color = [
    110,
    110,
    110,
]
chars_per_line = 32
```
