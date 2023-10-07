import cv2
import numpy as np
import sys

# Read the original image
img = cv2.imread('scorecard.jpg',flags=0)  


# Blur the image for better edge detection
img_blur = cv2.GaussianBlur(img,(3,3), sigmaX=0.05, sigmaY=0.05) 
#img_blur=cv2.medianBlur(img,ksize=9)
img_contrast = np.array(
    [[0 if px > 125 else 255 for px in row] for row in img_blur]
)

cv2.imwrite("img_contrast.jpg",img_contrast)
img_contrast = cv2.imread("img_contrast.jpg",flags=0)

for y,row in enumerate(img_contrast[2:-2]):
    for x,px in enumerate(row[2:-2]) :
        sum = px
        for x2 in range(-5,5):
            for y2 in range(-5,5):
                if x2 == y2 and x2 == 0:
                    pass
                sum += img_contrast[y+y2][x+x2]
        if sum > 255:
            img_contrast[y][x] = 0




cv2.imwrite("img_contrast.jpg",img_contrast)

def points(img):
    points = []
    for y,row in enumerate(img):
        for x,px in enumerate(row):
            if px == 255:
                points.append(f"({x},{y}),")
    return points

points = points(img_contrast)
class_name = sys.argv[1]
file_name = sys.argv[2]

with open(file_name,"w",encoding="utf-8") as f:
    newline="\n\t\t\t"
    rs = f"""
use super::{{Map,Color}};

pub struct {class_name} {{
    color: Color
}}

impl {class_name} {{
    fn default() -> Self{{
        Self{{
            color:Color::White
        }}
    }}
}}

impl Map for {class_name} {{
    const WIDTH:usize = {img_contrast.shape[1]};
    const HEIGHT:usize = {img_contrast.shape[0]};
    
    fn default() -> Self{{
        Self{{
            color:Color::White
        }}
    }}

	fn render(& self, ctx: &mut ratatui::widgets::canvas::Context<'_>) {{
		ctx.draw(self);
	}}

    fn map(&self) -> Vec<(usize,usize)>{{
        [
            {newline.join(points)}
        ].to_vec()
    }}

    fn set_color(&mut self, color: ratatui::style::Color) {{
        self.color = color;
    }}

    fn get_color(&self) -> ratatui::style::Color {{
        self.color.clone()
    }}

}}
impl super::Shape for BoomerangAustralia {{
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {{
        for (x, y) in self.map() {{
            let y = Self::HEIGHT-y;
            if let Some((x, y)) = painter.get_point(x as f64, y as f64) {{
                painter.paint(x, y, self.get_color());
            }}
        }}
    }}
}}
    """
    f.write(rs)
