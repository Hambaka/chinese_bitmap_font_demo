#![warn(clippy::all)]

use std::{
  collections::HashSet,
  fs::{self},
  path::PathBuf,
};

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use anyhow::{Result, bail};
use chinese_bitmap_font_demo::config::Config;
use clap::Parser;
use image::{Rgb, RgbImage};

/// Fusion Pixel Font 10px = 9px + 1px padding
const CHAR_SIZE: f32 = 9.0;
/// https://baike.baidu.com/item/%E6%A0%87%E7%82%B9%E7%AC%A6%E5%8F%B7/588793
/// https://zh.wikipedia.org/wiki/%E6%A0%87%E7%82%B9%E7%AC%A6%E5%8F%B7
const CHINESE_PUNCTUATION_MARKS: [char; 32] = [
  '·', '—', '‘', '’', '“', '”', '…', '、', '。', '〈', '〉', '《', '》', '「', '」', '『', '』',
  '【', '】', '〔', '〕', '︰', '！', '（', '）', '，', '．', '：', '；', '？', '［', '］',
];
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  /// Game script/text file for generating bitmap font image.
  #[arg(short, long, value_name = "FILE")]
  text: PathBuf,
  /// Font file for generating bitmap font image.
  #[arg(short, long, value_name = "FILE")]
  font: PathBuf,
  /// Font size(px), only support 10px or 11px.
  #[arg(short, long, default_value_t = 10)]
  size: u32,
  /// Whether the font is zh-hant or zh-hans, for punctuation marks offset.
  #[arg(short, long, default_value_t = false)]
  is_zh_hant: bool,
  /// Output bitmap font image file (PNG only)
  #[arg(short, long, value_name = "FILE")]
  output: PathBuf,
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  // Check if game script file exists.
  let game_script = if cli.text.exists() {
    fs::read_to_string(&cli.text)?
  } else {
    bail!("[Error] Game script file not found!");
  };
  // Check if font file exists.
  let font_file = if cli.font.exists() {
    fs::read(&cli.font)?
  } else {
    bail!("[Error] Font file not found!");
  };
  // Check if font size is 10px or 11px.
  if cli.size != 10 && cli.size != 11 {
    bail!("[Error] Only support 10px or 11px!");
  }
  let font_size = cli.size;

  let is_zh_hant = cli.is_zh_hant;
  let output_file = cli.output;

  // Load config file.
  let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
  let config_file = exe_dir.join(CONFIG_FILE_NAME);
  let config = if config_file.exists() {
    if let Ok(config) = toml::from_str(&fs::read_to_string(&config_file)?) {
      config
    } else {
      println!("[Warning] Invalid config file, using default config.");
      Config::default()
    }
  } else {
    println!("[Warning] Config file not found, writing and using default config.");
    fs::write(&config_file, toml::to_string_pretty(&Config::default())?)?;
    Config::default()
  };

  let chars = get_unique_chinese_chars(&game_script);
  if chars.is_empty() {
    bail!("[Error] No chinese characters found in game script!");
  }

  let img_height = if chars.len() % config.chars_per_line == 0 {
    (chars.len() / config.chars_per_line) as u32 * font_size
  } else {
    (chars.len() / config.chars_per_line + 1) as u32 * font_size
  };
  let mut image: RgbImage = image::ImageBuffer::from_pixel(
    config.chars_per_line as u32 * font_size,
    img_height,
    Rgb(config.img_bg_color),
  );
  let font = FontRef::try_from_slice(&font_file)?;

  // 6.75 pt = 9 px
  // 6.75 * 2 = 13.5
  let scale = PxScale::from(CHAR_SIZE * 0.75 * 2.0);
  let scaled_font = font.as_scaled(scale);

  let loop_count = if font_size == 10 { 1 } else { 2 };

  for i in 0..loop_count {
    let mut x_offset = 0;
    let mut y_offset = 0;

    for (j, c) in chars.iter().enumerate() {
      let glyph_id = font.glyph_id(*c);
      if glyph_id.0 == 0 {
        if i == 0 {
          println!(
            "[Warning] The glyph for '{}' (U+{:04X}) is not found! (index: {})",
            *c, *c as u32, j
          );
        }
      } else {
        let glyph = glyph_id.with_scale(scale);
        if let Some(outlined_glyph) = scaled_font.outline_glyph(glyph) {
          outlined_glyph.draw(|x, y, v| {
            if v > 0.5 {
              let (x_pos, y_pos);

              if CHINESE_PUNCTUATION_MARKS.contains(c) {
                let (h_side_bearing, v_side_bearing) =
                  get_chinese_punctuation_offset(*c, is_zh_hant);

                x_pos = x + x_offset + h_side_bearing;
                y_pos = y + y_offset + v_side_bearing;
              } else {
                let h_side_bearing = scaled_font.h_side_bearing(scaled_font.glyph_id(*c));
                let v_side_bearing = scaled_font.v_side_bearing(scaled_font.glyph_id(*c));

                let bounds = outlined_glyph.px_bounds();
                let char_width = bounds.width();
                let char_height = bounds.height();

                // At least it works...
                x_pos = if char_width + h_side_bearing.ceil() > CHAR_SIZE {
                  // 极少数字符的边距+本体宽会超出9px边界的，因此直接舍弃边界值
                  x + x_offset
                } else if char_width < CHAR_SIZE && char_width + h_side_bearing.ceil() == CHAR_SIZE
                {
                  // 自、当、日、口、白、目……
                  // 对于比较瘦的字，尽可能靠左
                  x + (h_side_bearing.ceil() as u32) + x_offset - 1
                } else {
                  // 常见规格的方块字
                  x + (h_side_bearing.ceil() as u32) + x_offset
                };

                y_pos = if char_height + v_side_bearing.ceil() > CHAR_SIZE {
                  // 类似于水平方向的向左，这里尽可能靠近垂直向下方向。
                  y + y_offset + (CHAR_SIZE - char_height) as u32
                } else {
                  // 常见规格的方块字
                  y + (v_side_bearing.ceil() as u32) + y_offset
                };
              }

              if font_size == 10 {
                // Bottom shadow
                image.put_pixel(x_pos, y_pos + 1, Rgb(config.char_shadow_color));
                // Bottom-right shadow
                image.put_pixel(x_pos + 1, y_pos + 1, Rgb(config.char_shadow_color));
                // Right shadow
                image.put_pixel(x_pos + 1, y_pos, Rgb(config.char_shadow_color));
                // Character itself
                image.put_pixel(x_pos, y_pos, Rgb(config.char_color));
              } else {
                let (x_pos, y_pos) = (x_pos + 1, y_pos + 1);
                if i == 0 {
                  // Bottom shadow
                  image.put_pixel(x_pos, y_pos + 1, Rgb(config.char_shadow_color));
                  // Bottom-right shadow
                  image.put_pixel(x_pos + 1, y_pos + 1, Rgb(config.char_shadow_color));
                  // Right shadow
                  image.put_pixel(x_pos + 1, y_pos, Rgb(config.char_shadow_color));
                  // Top-right shadow
                  image.put_pixel(x_pos + 1, y_pos - 1, Rgb(config.char_shadow_color));
                  // Top shadow
                  image.put_pixel(x_pos, y_pos - 1, Rgb(config.char_shadow_color));
                  // Top-left shadow
                  image.put_pixel(x_pos - 1, y_pos - 1, Rgb(config.char_shadow_color));
                  // Left shadow
                  image.put_pixel(x_pos - 1, y_pos, Rgb(config.char_shadow_color));
                  // Bottom-left shadow
                  image.put_pixel(x_pos - 1, y_pos + 1, Rgb(config.char_shadow_color));
                } else {
                  // Character itself
                  image.put_pixel(x_pos, y_pos, Rgb(config.char_color));
                }
              }
            }
          });
        }
      }

      if (j + 1) % config.chars_per_line == 0 {
        x_offset = 0;
        y_offset += font_size;
      } else {
        x_offset += font_size;
      }
    }
  }

  image.save(output_file)?;

  Ok(())
}

fn get_unique_chinese_chars(game_script: &str) -> Vec<char> {
  let no_whitespace_chinese_script: String = game_script
    .chars()
    .filter(|c| {
      !c.is_whitespace()
        && (CHINESE_PUNCTUATION_MARKS.contains(c) || is_chinese::is_chinese(c.to_string().as_str()))
    })
    .collect();
  let unique_chars = no_whitespace_chinese_script.chars().collect::<HashSet<_>>();

  let mut sorted_chars = unique_chars.iter().copied().collect::<Vec<_>>();
  sorted_chars.sort_unstable();

  sorted_chars
}

/// FUSION PIXEL FONT 10PX ONLY
/// This is stupid, but it works.
fn get_chinese_punctuation_offset(c: char, is_zh_hant: bool) -> (u32, u32) {
  match c {
    '·' => (3, 4),
    '—' => (0, 4),
    '‘' => (5, 0),
    '’' => (0, 0),
    '“' => (2, 0),
    '”' => (0, 0),
    '…' => (0, 4),
    '、' => {
      if is_zh_hant {
        (3, 3)
      } else {
        (0, 6)
      }
    }
    '。' => {
      if is_zh_hant {
        (2, 3)
      } else {
        (0, 5)
      }
    }
    '〈' => (4, 0),
    '〉' => (0, 0),
    '《' => (1, 0),
    '》' => (0, 0),
    '「' => (4, 0),
    '」' => (0, 2),
    '『' => (2, 0),
    '』' => (0, 2),
    '【' => (3, 0),
    '】' => (0, 0),
    '〔' => (4, 0),
    '〕' => (0, 0),
    '︰' => (3, 1),
    '！' => {
      if is_zh_hant {
        (3, 0)
      } else {
        (1, 0)
      }
    }
    '（' => (4, 0),
    '）' => (0, 0),
    '，' => {
      if is_zh_hant {
        (3, 3)
      } else {
        (0, 5)
      }
    }
    '．' => {
      if is_zh_hant {
        (3, 4)
      } else {
        (0, 6)
      }
    }
    '：' => {
      if is_zh_hant {
        (3, 1)
      } else {
        (0, 1)
      }
    }
    '；' => {
      if is_zh_hant {
        (3, 1)
      } else {
        (0, 1)
      }
    }
    '？' => {
      if is_zh_hant {
        (1, 0)
      } else {
        (0, 0)
      }
    }
    '［' => (4, 0),
    '］' => (0, 0),
    _ => unreachable!(),
  }
}
