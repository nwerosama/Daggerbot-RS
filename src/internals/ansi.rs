pub enum Color {
  Red,
  Blue,
  Yellow,
  Green
}

impl Color {
  pub fn bold(self) -> StyledColor {
    StyledColor {
      color:  self,
      styles: vec!["\x1b[1m"]
    }
  }

  pub fn normal(self) -> StyledColor {
    StyledColor {
      color:  self,
      styles: vec![]
    }
  }
}

pub struct StyledColor {
  color:  Color,
  styles: Vec<&'static str>
}

impl StyledColor {
  pub fn paint(
    self,
    text: &str
  ) -> String {
    let color_code = match self.color {
      Color::Red => "\x1b[31m",
      Color::Blue => "\x1b[34m",
      Color::Yellow => "\x1b[33m",
      Color::Green => "\x1b[32m"
    };

    let mut string = String::new();

    for style in self.styles {
      string.push_str(style);
    }

    string.push_str(color_code);
    string.push_str(text);
    string.push_str("\x1b[0m"); // Reset styling for next line
    string
  }
}
