use coffee::{
    graphics::{Color, Frame, Mesh, Point, Rectangle, Shape, Window, WindowSettings},
    input::{mouse, ButtonState, Event, Input},
    load::Task,
    Game, Result, Timer,
};
use nalgebra;
use rand::{self, Rng};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

const GRAY: Color = Color {
    r: 0.7,
    g: 0.7,
    b: 0.7,
    a: 0.1,
};

const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 0.1,
};

const BLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

const PURPLE: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

// Copy of KeyboardAndMouse in order to get access to mouse_pressed
struct VennInput {
    cursor_position: Point,
    is_cursor_taken: bool,
    is_mouse_pressed: bool,
}

impl Input for VennInput {
    fn new() -> VennInput {
        VennInput {
            cursor_position: Point::new(0.0, 0.0),
            is_cursor_taken: false,
            is_mouse_pressed: false,
        }
    }

    fn update(&mut self, event: Event) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { x, y } => {
                    self.cursor_position = Point::new(x, y);
                }
                mouse::Event::CursorTaken => {
                    self.is_cursor_taken = true;
                }
                mouse::Event::CursorReturned => {
                    self.is_cursor_taken = false;
                }
                mouse::Event::Input {
                    button: mouse::Button::Left,
                    state,
                } => match state {
                    ButtonState::Pressed => {
                        self.is_mouse_pressed = !self.is_cursor_taken;
                    }
                    ButtonState::Released => {
                        self.is_mouse_pressed = false;
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn clear(&mut self) {}
}

struct VennTarget {
    color: VennColor,
    shape: VennShape,
    size: VennSize,
}

struct VennAnswer {
    width: f32,
    height: f32,
    center: Point,
    hover: bool,
    target: VennTarget,
}

impl VennAnswer {
    fn draw(&self, mesh: &mut Mesh) {
        if self.hover {
            let mut color = YELLOW;
            color.a = 0.1;
            mesh.fill(
                Shape::Rectangle(Rectangle {
                    x: self.center.x - self.width / 2.0,
                    y: self.center.y - self.height / 2.0,
                    width: self.width,
                    height: self.height,
                }),
                color,
            );
        }
        mesh.stroke(
            Shape::Rectangle(Rectangle {
                x: self.center.x - self.width / 2.0,
                y: self.center.y - self.height / 2.0,
                width: self.width,
                height: self.height,
            }),
            Color::BLACK,
            2,
        );
    }

    fn contains(&self, point: &Point) -> bool {
        if point.x > self.center.x - self.width / 2.0
            && point.x < self.center.x + self.width / 2.0
            && point.y > self.center.y - self.height / 2.0
            && point.y < self.center.y + self.height / 2.0
        {
            return true;
        }
        false
    }

    fn matches(&self, target: &VennTarget) -> bool {
        if self.target.shape == target.shape
            // || self.target.size == target.size
            && self.target.color == target.color
        {
            return true;
        }
        false
    }
}

#[derive(PartialEq, Copy, Clone)]
enum VennColor {
    Yellow,
    Blue,
    Purple,
}

impl VennColor {
    fn to_color(&self) -> Color {
        match self {
            VennColor::Yellow => YELLOW,
            VennColor::Blue => BLUE,
            VennColor::Purple => PURPLE,
        }
    }
}

impl VennColor {
    fn all() -> Vec<VennColor> {
        vec![VennColor::Yellow, VennColor::Blue, VennColor::Purple]
    }

    fn random(rng: &mut rand::rngs::ThreadRng) -> VennColor {
        match rng.gen_range(0, 2) {
            0 => VennColor::Yellow,
            1 => VennColor::Blue,
            2 => VennColor::Purple,
            _ => panic!("Unexpected value"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum VennSize {
    Small,
    Medium,
    Large,
}

impl VennSize {
    fn all() -> Vec<VennSize> {
        vec![VennSize::Small, VennSize::Medium, VennSize::Large]
    }

    fn random(rng: &mut rand::rngs::ThreadRng) -> VennSize {
        match rng.gen_range(0, 2) {
            0 => VennSize::Small,
            1 => VennSize::Medium,
            2 => VennSize::Large,
            _ => panic!("Unexpected value"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum VennShape {
    Circle,
    Triangle,
    Square,
}

impl VennShape {
    fn all() -> Vec<VennShape> {
        vec![VennShape::Circle, VennShape::Square, VennShape::Triangle]
    }

    fn random(rng: &mut rand::rngs::ThreadRng) -> VennShape {
        match rng.gen_range(0, 2) {
            0 => VennShape::Circle,
            1 => VennShape::Square,
            2 => VennShape::Triangle,
            _ => panic!("Unexpected value"),
        }
    }
}

struct VennGuess {
    center: Point,
    radius: f32,
    dragged: bool,
    target: VennTarget,
    matches: Option<bool>,
}

impl VennGuess {
    fn new(i: usize, shape: VennShape, color: VennColor, size: VennSize) -> VennGuess {
        VennGuess {
            center: Point::new(20.0, (i + 1) as f32 * 40.0),
            radius: 30.0,
            dragged: false,
            target: VennTarget { shape, size, color },
            matches: None,
        }
    }

    fn drag_to(&mut self, point: &Point) {
        self.dragged = true;
        self.center = point.clone();
    }

    fn contains(&self, point: &Point) -> bool {
        if nalgebra::distance(point, &self.center) < self.radius {
            return true;
        }
        false
    }

    fn draw(&self, mesh: &mut Mesh) {
        let mut color = match self.matches {
            None => GRAY,
            Some(true) => GREEN,
            Some(false) => RED,
        };
        color.a = 1.0;
        if self.dragged {
            color.a -= 0.3;
        }
        mesh.fill(
            Shape::Circle {
                center: self.center,
                radius: self.radius,
            },
            color,
        );
        mesh.stroke(
            Shape::Circle {
                center: self.center,
                radius: self.radius,
            },
            Color::BLACK,
            1,
        );
        let shape = match self.target.shape {
            VennShape::Circle => Shape::Circle {
                center: self.center,
                radius: 10.0,
            },
            VennShape::Square => Shape::Rectangle(Rectangle {
                x: self.center.x - 10.0,
                y: self.center.y - 10.0,
                width: 10.0 * 2.0,
                height: 10.0 * 2.0,
            }),
            VennShape::Triangle => Shape::Polyline {
                points: vec![
                    Point::new(self.center.x, self.center.y - 10.0),
                    Point::new(self.center.x - 10.0, self.center.y + 10.0),
                    Point::new(self.center.x + 10.0, self.center.y + 10.0),
                    Point::new(self.center.x, self.center.y - 10.0),
                ],
            },
        };
        mesh.fill(shape.clone(), self.target.color.to_color());
        mesh.stroke(shape, Color::BLACK, 1);
    }
}

struct VennCircle {
    center: Point,
    radius: f32,
    color: Color,
    selected: bool,
    answer: VennAnswer,
}

impl Default for VennCircle {
    fn default() -> VennCircle {
        VennCircle {
            center: Point::new(0.0, 0.0),
            radius: 1.0,
            color: Color::BLACK,
            selected: false,
            answer: VennAnswer {
                center: Point::new(0.0, 0.0),
                width: 40.0,
                height: 30.0,
                hover: false,
                target: VennTarget {
                    shape: VennShape::Circle,
                    size: VennSize::Large,
                    color: VennColor::Blue,
                },
            },
        }
    }
}

impl VennCircle {
    fn draw(&self, mesh: &mut Mesh) {
        self.answer.draw(mesh);
        let mut color = self.color.clone();
        color.a = 0.1;
        if self.selected {
            color.a = 0.3;
        }
        mesh.fill(
            Shape::Circle {
                center: self.center,
                radius: self.radius,
            },
            color,
        );
        mesh.stroke(
            Shape::Circle {
                center: self.center,
                radius: self.radius,
            },
            Color::BLACK,
            1,
        );
    }

    fn contains(&self, point: &Point) -> bool {
        if nalgebra::distance(point, &self.center) < self.radius {
            return true;
        }
        false
    }

    fn interact(&mut self, input: &VennInput) {
        self.selected = false;
        if self.contains(&input.cursor_position) {
            self.selected = true;
        }
    }

    fn matches(&self, target: &VennTarget) -> bool {
        if self.answer.target.shape == target.shape
            // || self.target.size == target.size
            || self.answer.target.color == target.color
        {
            return true;
        }
        false
    }
}

struct Venn {
    left: VennCircle,
    right: VennCircle,
    shapes: Vec<VennGuess>,
    drag_index: Option<usize>,
}

impl Game for Venn {
    type Input = VennInput;
    type LoadingScreen = ();
    const TICKS_PER_SECOND: u16 = 60;

    fn load(_window: &Window) -> Task<Venn> {
        let x_margin = 10.0;
        let y_margin = 10.0;
        let remaining_x = WIDTH - x_margin * 2.0;
        let remaining_y = HEIGHT - y_margin * 2.0;
        Task::new(move || {
            let mut rng = rand::thread_rng();
            let mut shapes = Vec::new();
            let mut i = 0;
            for shape in VennShape::all() {
                for color in VennColor::all() {
                    // for size in VennSize::all() {
                    let size = VennSize::Small;
                    shapes.push(VennGuess::new(i, shape.clone(), color.clone(), size));
                    i += 1;
                    // }
                }
            }
            let left_center =
                Point::new(x_margin + remaining_x / 3.0, y_margin + remaining_y / 2.0);
            let mut left_answer_center = left_center.clone();
            left_answer_center.y = left_answer_center.y - 200.0 - 40.0 - 15.0;
            let right_center = Point::new(
                WIDTH - x_margin - remaining_x / 3.0,
                HEIGHT - y_margin - remaining_y / 2.0,
            );
            let mut right_answer_center = right_center.clone();
            right_answer_center.y = right_answer_center.y - 200.0 - 40.0 - 15.0;
            Venn {
                left: VennCircle {
                    center: left_center,
                    radius: 200.0,
                    color: BLUE,
                    answer: VennAnswer {
                        center: left_answer_center,
                        width: 100.0,
                        height: 80.0,
                        hover: false,
                        target: VennTarget {
                            shape: VennShape::random(&mut rng),
                            size: VennSize::random(&mut rng),
                            color: VennColor::random(&mut rng),
                        },
                    },
                    ..VennCircle::default()
                },
                right: VennCircle {
                    center: right_center,
                    radius: 200.0,
                    color: YELLOW,
                    answer: VennAnswer {
                        center: right_answer_center,
                        width: 100.0,
                        height: 80.0,
                        hover: false,
                        target: VennTarget {
                            shape: VennShape::random(&mut rng),
                            size: VennSize::random(&mut rng),
                            color: VennColor::random(&mut rng),
                        },
                    },
                    ..VennCircle::default()
                },
                shapes,
                drag_index: None,
            }
        })
    }

    fn draw(&mut self, frame: &mut Frame<'_>, _timer: &Timer) {
        frame.clear(Color::WHITE);
        let mut mesh = Mesh::new();
        self.left.draw(&mut mesh);
        self.right.draw(&mut mesh);
        for shape in &self.shapes {
            shape.draw(&mut mesh);
        }
        mesh.draw(&mut frame.as_target());
    }

    fn interact(&mut self, input: &mut Self::Input, _window: &mut Window) {
        self.left.interact(input);
        self.right.interact(input);
        if input.is_mouse_pressed {
            match self.drag_index {
                None => {
                    for (i, shape) in self.shapes.iter_mut().enumerate().rev() {
                        if shape.contains(&input.cursor_position) {
                            shape.matches = None;
                            shape.drag_to(&input.cursor_position);
                            self.drag_index = Some(i);
                            break;
                        }
                    }
                }
                Some(index) => {
                    self.shapes[index].drag_to(&input.cursor_position);
                }
            }
            if self.drag_index.is_some() {
                self.left.answer.hover = self.left.answer.contains(&input.cursor_position);
                self.right.answer.hover = self.right.answer.contains(&input.cursor_position);
            }
        } else {
            self.left.answer.hover = false;
            self.right.answer.hover = false;
            match self.drag_index {
                Some(index) => {
                    let mut shape = &mut self.shapes[index];
                    match (
                        self.left.contains(&shape.center),
                        self.right.contains(&shape.center),
                        self.left.answer.contains(&shape.center),
                        self.right.answer.contains(&shape.center),
                    ) {
                        (true, true, _, _) => {
                            // Does left and right need to match the same property of shape?
                            // Or is it okay if it contains at least one property of each, independently?
                            shape.matches = Some(
                                self.left.matches(&shape.target)
                                    && self.right.matches(&shape.target),
                            );
                        }
                        (true, false, _, _) => {
                            shape.matches = Some(self.left.matches(&shape.target));
                        }
                        (false, true, _, _) => {
                            shape.matches = Some(self.right.matches(&shape.target));
                        }
                        (false, false, true, false) => {
                            shape.matches = Some(self.left.answer.matches(&shape.target));
                            shape.center = self.left.answer.center;
                        }
                        (false, false, false, true) => {
                            shape.matches = Some(self.right.answer.matches(&shape.target));
                            shape.center = self.right.answer.center;
                        }
                        (false, false, _, _) => {
                            shape.matches = None;
                        }
                    }
                    shape.dragged = false;
                    self.drag_index = None;
                }
                None => {}
            }
        }
    }

    fn update(&mut self, _window: &Window) {}
}

fn main() -> Result<()> {
    Venn::run(WindowSettings {
        title: String::from("Venn Deduction"),
        size: (WIDTH as u32, HEIGHT as u32),
        resizable: false,
        fullscreen: false,
    })
}
