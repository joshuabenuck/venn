use coffee::{
    graphics::{Color, Frame, Mesh, Point, Rectangle, Shape, Window, WindowSettings},
    input::{keyboard, keyboard::KeyCode, mouse, ButtonState, Event, Input, KeyboardAndMouse},
    load::Task,
    Game, Result, Timer,
};
use nalgebra;
use rand::{self, Rng};
use std::collections::HashSet;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 0.1,
};

const ORANGE: Color = Color {
    r: 1.0,
    g: 0.7,
    b: 0.0,
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
    a: 0.1,
};

const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 0.1,
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
    left_clicks: Vec<Point>,
    pressed_keys: HashSet<keyboard::KeyCode>,
    released_keys: HashSet<keyboard::KeyCode>,
}

impl VennInput {
    /// Returns the current cursor position.
    pub fn cursor_position(&self) -> Point {
        self.cursor_position
    }

    /// Returns true if the cursor is currently not available.
    ///
    /// This mostly happens when the cursor is currently over a
    /// [`UserInterface`].
    ///
    /// [`UserInterface`]: ../ui/trait.UserInterface.html
    pub fn is_cursor_taken(&self) -> bool {
        self.is_cursor_taken
    }

    /// Returns the positions of the mouse clicks during the last interaction.
    ///
    /// Clicks performed while the mouse cursor is not available are
    /// automatically ignored.
    pub fn left_clicks(&self) -> &[Point] {
        &self.left_clicks
    }

    /// Returns true if the given key is currently pressed.
    pub fn is_key_pressed(&self, key_code: keyboard::KeyCode) -> bool {
        self.pressed_keys.contains(&key_code)
    }

    /// Returns true if the given key was released during the last interaction.
    pub fn was_key_released(&self, key_code: keyboard::KeyCode) -> bool {
        self.released_keys.contains(&key_code)
    }
}

impl Input for VennInput {
    fn new() -> VennInput {
        VennInput {
            cursor_position: Point::new(0.0, 0.0),
            is_cursor_taken: false,
            is_mouse_pressed: false,
            left_clicks: Vec::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
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
                        if !self.is_cursor_taken && self.is_mouse_pressed {
                            self.left_clicks.push(self.cursor_position);
                        }

                        self.is_mouse_pressed = false;
                    }
                },
                mouse::Event::Input { .. } => {
                    // TODO: Track other buttons!
                }
                mouse::Event::CursorEntered => {
                    // TODO: Track it!
                }
                mouse::Event::CursorLeft => {
                    // TODO: Track it!
                }
                mouse::Event::WheelScrolled { .. } => {
                    // TODO: Track it!
                }
            },
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::Input { key_code, state } => {
                    match state {
                        ButtonState::Pressed => {
                            let _ = self.pressed_keys.insert(key_code);
                        }
                        ButtonState::Released => {
                            let _ = self.pressed_keys.remove(&key_code);
                            let _ = self.released_keys.insert(key_code);
                        }
                    };
                }
                keyboard::Event::TextEntered { .. } => {}
            },
            Event::Gamepad { .. } => {
                // Ignore gamepad events...
            }
            Event::Window(_) => {
                // Ignore window events...
            }
        }
    }

    fn clear(&mut self) {
        self.left_clicks.clear();
        self.released_keys.clear();
    }
}

struct VennTarget {
    color: VennColor,
    shape: VennShape,
    size: VennSize,
}

#[derive(PartialEq)]
enum VennColor {
    Green,
    Blue,
    Purple,
}

impl VennColor {
    fn to_color(&self) -> Color {
        match self {
            VennColor::Green => GREEN,
            VennColor::Blue => BLUE,
            VennColor::Purple => PURPLE,
        }
    }
}

#[derive(PartialEq)]
enum VennSize {
    Small,
    Medium,
    Large,
}

#[derive(PartialEq)]
enum VennShape {
    Circle,
    Triangle,
    Square,
}

impl VennShape {
    fn all_shapes() -> Vec<VennShape> {
        vec![VennShape::Circle, VennShape::Square, VennShape::Triangle]
    }
}

struct VennGuess {
    center: Point,
    radius: f32,
    color: Color,
    dragged: bool,
    target: VennTarget,
    matches: Option<bool>,
}

impl VennGuess {
    fn new(shape: VennShape) -> VennGuess {
        VennGuess {
            center: Point::new(0.0, 0.0),
            radius: 30.0,
            color: ORANGE,
            dragged: false,
            target: VennTarget {
                shape,
                size: VennSize::Small,
                color: VennColor::Purple,
            },
            matches: None,
        }
    }

    fn drag_to(&mut self, point: &Point) {
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
            None => ORANGE,
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
    target: VennTarget,
}

impl Default for VennCircle {
    fn default() -> VennCircle {
        VennCircle {
            center: Point::new(0.0, 0.0),
            radius: 1.0,
            color: Color::BLACK,
            selected: false,
            target: VennTarget {
                shape: VennShape::Circle,
                size: VennSize::Large,
                color: VennColor::Blue,
            },
        }
    }
}

impl VennCircle {
    fn draw(&self, mesh: &mut Mesh) {
        let mut color = self.color.clone();
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

    fn matches(&self, target: &VennTarget) -> bool {
        if self.target.shape == target.shape
            || self.target.size == target.size
            || self.target.color == target.color
        {
            return true;
        }
        false
    }
}

struct Venn {
    x_margin: f32,
    y_margin: f32,
    left: VennCircle,
    right: VennCircle,
    shapes: Vec<VennGuess>,
    drag_index: Option<usize>,
}

fn random_shape(rng: &mut rand::rngs::ThreadRng) -> VennShape {
    match rng.gen_range(0, 2) {
        0 => VennShape::Circle,
        1 => VennShape::Square,
        2 => VennShape::Triangle,
        _ => panic!("Unexpected value"),
    }
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
            for shape in VennShape::all_shapes() {
                shapes.push(VennGuess::new(shape));
            }
            Venn {
                x_margin,
                y_margin,
                left: VennCircle {
                    center: Point::new(x_margin + remaining_x / 3.0, y_margin + remaining_y / 2.0),
                    radius: 200.0,
                    color: BLUE,
                    target: VennTarget {
                        shape: random_shape(&mut rng),
                        size: VennSize::Large,
                        color: VennColor::Blue,
                    },
                    ..VennCircle::default()
                },
                right: VennCircle {
                    center: Point::new(
                        WIDTH - x_margin - remaining_x / 3.0,
                        HEIGHT - y_margin - remaining_y / 2.0,
                    ),
                    radius: 200.0,
                    color: YELLOW,
                    target: VennTarget {
                        shape: random_shape(&mut rng),
                        size: VennSize::Large,
                        color: VennColor::Blue,
                    },
                    ..VennCircle::default()
                },
                shapes,
                drag_index: None,
            }
        })
    }

    fn draw(&mut self, frame: &mut Frame<'_>, timer: &Timer) {
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
        self.left.selected = false;
        if self.left.contains(&input.cursor_position) {
            self.left.selected = true;
        }
        self.right.selected = false;
        if self.right.contains(&input.cursor_position) {
            self.right.selected = true;
        }
        if input.is_mouse_pressed {
            match self.drag_index {
                None => {
                    for (i, shape) in self.shapes.iter_mut().enumerate().rev() {
                        if shape.contains(&input.cursor_position) {
                            shape.matches = None;
                            shape.dragged = true;
                            shape.center = input.cursor_position;
                            self.drag_index = Some(i);
                            break;
                        }
                    }
                }
                Some(index) => {
                    self.shapes[index].center = input.cursor_position;
                }
            }
        } else {
            match self.drag_index {
                Some(index) => {
                    let mut shape = &mut self.shapes[index];
                    match (
                        self.left.contains(&shape.center),
                        self.right.contains(&shape.center),
                    ) {
                        (true, true) => {
                            // Does left and right need to match the same property of shape?
                            // Or is it okay if it contains at least one property of each, independently?
                            if self.left.matches(&shape.target) && self.right.matches(&shape.target)
                            {
                                shape.matches = Some(true);
                            } else {
                                shape.matches = Some(false);
                            }
                        }
                        (true, false) => {
                            if self.left.matches(&shape.target) {
                                shape.matches = Some(true);
                            } else {
                                shape.matches = Some(false);
                            }
                        }
                        (false, true) => {
                            if self.right.matches(&shape.target) {
                                shape.matches = Some(true);
                            } else {
                                shape.matches = Some(false);
                            }
                        }
                        (false, false) => {
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
