use macroquad::{math::Rect, texture::Texture2D};

pub enum Directions {
    Down,
    Up,
    Left,
    Right,
}

impl Directions {
    pub fn as_f32(&self) -> f32 {
        match self {
            Directions::Down => 0.0,
            Directions::Up => 1.0,
            Directions::Left => 2.0,
            Directions::Right => 3.0,
        }
    }
}

pub struct SpreadAnimation {
    draw_rect: Rect,
    time_per_frame: f32,
    elapsed_time: f32,
    num_frames: u32,
    current_frame: u32,
    must_repeat: bool,
}

impl SpreadAnimation {
    pub fn new(texture: &Texture2D, num_frames: u32, num_rows: u32, animation_time: f32) -> Self {
        let time_per_frame = animation_time / num_frames as f32;
        let width = texture.width() / num_frames as f32;
        let height = texture.height() / num_rows as f32;
        let draw_rect = Rect{
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        };
        Self {
           draw_rect,
           time_per_frame,
           elapsed_time: 0.0,
           num_frames,
           current_frame: 0,
           must_repeat: true,
        }
    }
    
    pub fn get_draw_rect(&self) -> &Rect {
        &self.draw_rect
    }

    pub fn set_row_frame(&mut self, direction: Directions) {
        self.draw_rect.y = self.draw_rect.h * direction.as_f32();
    }

    pub fn set_repeat(&mut self, should_repeat: bool) {
        self.must_repeat = should_repeat;
    }
    
    pub fn update(&mut self, dt: f32) {
       self.elapsed_time += dt; 
       while self.elapsed_time >= self.time_per_frame && 
           (self.current_frame < self.num_frames || self.must_repeat)
        {
            self.draw_rect.x += self.draw_rect.w;
            if self.draw_rect.x + self.draw_rect.w >= self.draw_rect.w*self.num_frames as f32 {
                self.draw_rect.x = 0.0;
            }

            self.elapsed_time -= self.time_per_frame;
            if self.must_repeat {
                self.current_frame = (self.current_frame + 1) % self.num_frames;
            }
            else {
                self.current_frame += 1;
            }
       }
    }
}
