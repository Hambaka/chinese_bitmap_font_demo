use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
  pub img_bg_color: [u8; 3],
  pub char_color: [u8; 3],
  pub char_shadow_color: [u8; 3],
  pub chars_per_line: usize,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      img_bg_color: [45, 45, 45],
      char_color: [250, 250, 245],
      char_shadow_color: [110, 110, 110],
      chars_per_line: 32,
    }
  }
}
