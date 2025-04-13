use {
  ab_glyph::{
    FontRef,
    PxScale
  },
  image::{
    ImageBuffer,
    ImageFormat,
    Rgb,
    Rgba,
    buffer::ConvertBuffer
  },
  imageproc::{
    drawing::{
      draw_filled_circle_mut,
      draw_filled_rect_mut,
      draw_hollow_rect_mut,
      draw_line_segment_mut,
      draw_text_mut
    },
    rect::Rect
  },
  std::io::Cursor
};

const CANVAS_WIDTH: u32 = 1500;
const CANVAS_HEIGHT: u32 = 750;
const INTERPOLATION_STEPS: usize = 100;

#[repr(align(16))]
pub struct Canvas {
  canvas:         ImageBuffer<Rgba<u8>, Vec<u8>>,
  palette:        Palette,
  line_thickness: i32,
  dot_thickness:  i32
}

struct Palette {
  odd_horizontal:  Rgba<u8>,
  even_horizontal: Rgba<u8>,
  background:      Rgba<u8>,
  text_color:      Rgba<u8>,
  red_line:        Rgba<u8>,
  yellow_line:     Rgba<u8>,
  green_line:      Rgba<u8>
}

struct DrawingBatch {
  lines:   Vec<(f32, f32, f32, f32, Rgba<u8>)>,
  circles: Vec<(i32, i32, i32, Rgba<u8>)>,
  rects:   Vec<(Rect, Rgba<u8>)>
}

impl DrawingBatch {
  fn new(data_len: usize) -> Self {
    Self {
      lines:   Vec::with_capacity(data_len * INTERPOLATION_STEPS),
      circles: Vec::with_capacity(data_len),
      rects:   Vec::with_capacity(data_len + 10) // +10 for horizontal lines
    }
  }

  /// Draw dots, points and lines in order at once
  fn exec_batch(
    &self,
    canvas: &mut Canvas
  ) {
    // Draw dot
    for &(rect, color) in &self.rects {
      draw_filled_rect_mut(&mut canvas.canvas, rect, color);
    }
    // Draw circle at each data point
    for &(x, y, radius, color) in &self.circles {
      draw_filled_circle_mut(&mut canvas.canvas, (x, y), radius, color);
    }
    // Draw lines
    for &(x1, y1, x2, y2, color) in &self.lines {
      canvas.draw_thick_line((x1, y1), (x2, y2), color, canvas.line_thickness);
    }
  }
}

impl Canvas {
  pub fn new() -> Self {
    let canvas = ImageBuffer::new(1500, 750);
    let empal = super::tasks::monica::EmbedPalette::new();
    let line_thickness = 5;
    let dot_thickness = 4;
    // Line thickness of 5 and dot thickness of 4 is the sweet spot for some reason...
    let palette = Palette {
      odd_horizontal:  Rgba([85, 91, 99, 255]),
      even_horizontal: Rgba([62, 66, 69, 255]),
      background:      Rgba([17, 17, 17, 255]),
      text_color:      Rgba([255, 255, 255, 255]),
      red_line:        empal.rgba(empal.red),
      yellow_line:     empal.rgba(empal.yellow),
      green_line:      empal.rgba(empal.green)
    };

    Self {
      canvas,
      palette,
      line_thickness,
      dot_thickness
    }
  }

  fn calculate_score(interval: f64) -> f64 {
    let interval_str = interval.to_string();
    let zero_count = interval_str.matches('0').count() as f64;
    zero_count / interval_str.len() as f64
  }

  fn calculate_multiplier(digit: char) -> f64 { if "124568".contains(digit) { 1.5 } else { 0.67 } }

  fn calculate_weighted_score(interval: f64) -> f64 {
    let digit = interval.to_string().chars().next().unwrap();
    let score = Self::calculate_score(interval);
    let multiplier = Self::calculate_multiplier(digit);
    score * multiplier
  }

  fn draw_thick_line(
    &mut self,
    start: (f32, f32),
    end: (f32, f32),
    color: Rgba<u8>,
    thickness: i32
  ) {
    for i in -thickness..=thickness {
      draw_line_segment_mut(&mut self.canvas, (start.0 + i as f32, start.1), (end.0 + i as f32, end.1), color);
      draw_line_segment_mut(&mut self.canvas, (start.0, start.1 + i as f32), (end.0, end.1 + i as f32), color);
    }
  }

  fn interpolate_color(
    start: Rgba<u8>,
    end: Rgba<u8>,
    factor: f32
  ) -> Rgba<u8> {
    let factor = factor.clamp(0.0, 1.0);

    let mut result = [0u8; 4];
    (0..4).for_each(|i| {
      result[i] = (start.0[i] as f32 + (end.0[i] as f32 - start.0[i] as f32) * factor) as u8;
    });
    Rgba(result)
  }

  pub fn render(
    &mut self,
    mut data: Vec<f64>
  ) {
    let data_len = data.len();
    let mut batch = DrawingBatch::new(data_len);

    // Handle negative values
    for i in 0..data_len {
      if data[i] < 0.0 {
        data[i] = data.get(i.wrapping_sub(1)).copied().unwrap_or(0.0);
      }
    }

    let top = 16.0;
    let text_size = 40.0;
    let origin = (15, 65);
    let size = (1300, 630);
    let csize = (CANVAS_WIDTH, CANVAS_HEIGHT);
    let node_width = size.0 as f64 / (data_len - 1) as f64;

    let relative_y = size.1 as f64 / top;
    let y_offset = origin.1 as f64;

    // Paint background
    draw_hollow_rect_mut(&mut self.canvas, Rect::at(0, 0).of_size(csize.0, csize.1), self.palette.background);

    // Interval calculation
    let interval_candidates: Vec<(f64, i32)> = (4..10)
      .map(|i| (top / i as f64, i))
      .filter(|&(interval, _)| interval.fract() == 0.0)
      .collect();

    let chosen_interval = interval_candidates
      .iter()
      .max_by(|&&(interval1, _), &&(interval2, _)| {
        Self::calculate_weighted_score(interval1)
          .partial_cmp(&Self::calculate_weighted_score(interval2))
          .unwrap()
      })
      .copied()
      .unwrap_or((1.0, 1));

    // Paint grey horizontal lines
    let interval_count = std::cmp::max((top / chosen_interval.0).ceil() as i32, 5);
    let interval_step = top / interval_count as f64;

    for i in 0..=chosen_interval.1 {
      let y = origin.1 + size.1 - ((i as f64 * interval_step) * relative_y) as i32;
      if y < origin.1 || y > origin.1 + size.1 {
        continue;
      }

      let color = if (i + 1) % 2 == 0 {
        self.palette.even_horizontal
      } else {
        self.palette.odd_horizontal
      };
      batch.rects.push((Rect::at(origin.0, y).of_size(size.0 as u32, 2), color));
    }

    // Load font
    let font_data = include_bytes!("assets/DejaVuSans.ttf") as &[u8];
    let font = FontRef::try_from_slice(font_data).unwrap();
    let scale = PxScale { x: text_size, y: text_size };

    // Queue points and lines
    let mut last_coords = None;
    let mut last_color = None;

    for (i, &current_value) in data.iter().enumerate() {
      let x = (i as f64 * node_width + origin.0 as f64) as i32;
      let y = ((1.0 - (current_value / top)) * size.1 as f64 + y_offset) as i32;

      let color = {
        const HIGH_THRESHOLD: f64 = 11.0 / 16.0; // paint red if 11+ players
        const LOW_THRESHOLD: f64 = 6.0 / 16.0; // otherwise paint yellow if 6+

        let relative_position = current_value / top;
        match relative_position {
          p if p >= HIGH_THRESHOLD => self.palette.red_line,
          p if p >= LOW_THRESHOLD => self.palette.yellow_line,
          _ => self.palette.green_line
        }
      };

      if let Some((last_x, last_y)) = last_coords {
        let steps = 100;
        for step in 0..steps {
          let factor = step as f32 / steps as f32;
          let inter_x = last_x + (x - last_x) * step / steps;
          let inter_y = last_y + (y - last_y) * step / steps;
          let inter_color = Self::interpolate_color(last_color.unwrap(), color, factor);
          batch.lines.push((inter_x as f32, inter_y as f32, x as f32, y as f32, inter_color));
        }
      }

      last_coords = Some((x, y));
      last_color = Some(color);

      batch.circles.push((x, y, self.dot_thickness, color));
      batch.rects.push((Rect::at(x - 2, y - 2).of_size(4, 4), color));
    }

    batch.exec_batch(self);

    // Draw text
    // Highest value
    let highest_text = top.to_string();
    draw_text_mut(
      &mut self.canvas,
      self.palette.text_color,
      origin.0 + size.0 + text_size as i32 / 2,
      origin.1 - (text_size / 2.0) as i32,
      scale,
      &font,
      &highest_text
    );

    // Middle value
    let middle_value = top / 2.0;
    draw_text_mut(
      &mut self.canvas,
      self.palette.text_color,
      origin.0 + size.0 + text_size as i32 / 2,
      origin.1 + (size.1 / 2) - (text_size / 2.0) as i32,
      scale,
      &font,
      &middle_value.to_string()
    );

    // Lowest value
    draw_text_mut(
      &mut self.canvas,
      self.palette.text_color,
      origin.0 + size.0 + text_size as i32 / 2,
      origin.1 + size.1 + (text_size / 3.0) as i32 - 30,
      scale,
      &font,
      "0"
    );

    // Time axis label
    draw_text_mut(
      &mut self.canvas,
      self.palette.text_color,
      origin.0,
      origin.1 + size.1 + 10,
      scale,
      &font,
      "time ->"
    );
  }

  pub fn export(&self) -> Vec<u8> {
    let rgba2rgb: ImageBuffer<Rgb<u8>, Vec<u8>> = self.canvas.convert();
    let mut bytes: Vec<u8> = Vec::new();
    let buffer = rgba2rgb.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg);

    if let Err(e) = buffer {
      eprintln!("Canvas[export:Error] {e}");
    }

    bytes
  }
}
