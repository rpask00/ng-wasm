use std::rc::Rc;
use rand::Rng;
use wasm_bindgen::prelude::*;
use web_sys::console;

macro_rules! modulo {
    ($a:expr,$b:expr)=>{
        {
            (($a % $b) + $b) % $b

        }
    }
}


#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}


#[derive(PartialEq)]
#[wasm_bindgen]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[wasm_bindgen]
pub enum GameState {
    Stopped,
    Running,
    Won,
    Lost,
}


#[derive(Clone)]
#[wasm_bindgen]
pub struct Cell {
    x: usize,
    y: usize,
}

#[wasm_bindgen]
impl Cell {
    fn to_index(&self, world: &World) -> usize {
        self.y * world.width + self.x
    }

    fn from_idx(idx: usize, world: &World) -> Cell {
        Cell {
            y: idx / world.width,
            x: idx % world.width,
        }
    }

    fn equal(&self, other: &Cell) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[wasm_bindgen]
struct Snake {
    body: Vec<Cell>,
    idx_vec: Vec<usize>,
    direction: Direction,
    next_cell: Option<Cell>,
    world: World,
    pub game_state: GameState,
}

#[wasm_bindgen]
impl Snake {
    pub fn new(spawn_index: usize, initial_length: usize, world: World) -> Snake {
        assert!(initial_length < world.width);

        let mut body: Vec<Cell> = vec![];

        body.push(Cell::from_idx(spawn_index, &world));

        for i in 1..initial_length {
            body.push(Cell::from_idx((spawn_index + i) % world.size, &world));
        }

        let idx_vec: Vec<usize> = body.clone().iter().map(|cell| cell.to_index(&world)).collect();

        Snake {
            body,
            direction: Direction::Up,
            next_cell: None,
            world,
            idx_vec,
            game_state: GameState::Stopped,
        }
    }

    fn length(&self) -> usize { self.body.len() }
    pub fn snake_cells(&self) -> *const usize { self.idx_vec.as_ptr() }
    pub fn snake_length(&self) -> usize { self.body.len() }
    pub fn head(&self) -> Cell { self.body[0].clone() }
    pub fn get_reward_cell_idx(&self) -> i32 { self.world.get_reward_cell_idx() }
    // pub fn get_game_state(&self) -> GameState { self.game_state }
    pub fn update_position(&mut self) {
        match self.game_state {
            GameState::Stopped => return,
            GameState::Running => {
                let next_cell = self.get_next_cell(&self.direction);
                let grow = match &self.world.reward_cell {
                    None => {
                        self.world.reward_cell = Some(self.world.generate_reward_cell());
                        false
                    }
                    Some(reward_cell) => {
                        let grow = next_cell.equal(&reward_cell);
                        if grow {
                            self.world.reward_cell = Some(self.world.generate_reward_cell());
                        }
                        grow
                    }
                };

                let last_cell = self.body.last().unwrap().clone();
                let last_index = last_cell.to_index(&self.world);
                self.world.board[last_index] = false;


                for i in (1..self.length()).rev() {
                    self.body[i] = self.body[i - 1].clone();
                    self.idx_vec[i] = self.body[i].to_index(&self.world);
                }

                self.set_snake_head(next_cell);

                if grow {
                    self.idx_vec.push(last_cell.to_index(&self.world));
                    self.body.push(last_cell);
                }
            }
            _ => return,
        }
    }

    pub fn restart_game(&mut self) {
        self.world.board = vec![false; self.world.size];

        while self.snake_length() > 4 {
            self.body.pop();
            self.idx_vec.pop();
        }

        self.set_game_state(GameState::Running);
    }


    fn get_next_cell(&self, direction: &Direction) -> Cell {
        let head = self.head();


        let width = self.world.width as i32;
        let height = self.world.height as i32;
        let x = head.x as i32;
        let y = head.y as i32;

        let (x, y) = match direction {
            Direction::Up => (x, modulo!((y - 1) , height)),
            Direction::Down => (x, modulo!((y + 1) , height)),

            Direction::Right => (modulo!((x + 1) , width), y),
            Direction::Left => (modulo!((x - 1) , width), y),
        };

        Cell {
            x: x as usize,
            y: y as usize,
        }
    }

    fn set_snake_head(&mut self, new_head: Cell) {
        self.body[0] = new_head;
        self.idx_vec[0] = self.body[0].to_index(&self.world);
        let cell_index = self.body[0].to_index(&self.world);

        match self.world.board[cell_index] {
            true => self.game_state = GameState::Lost,
            false => self.world.board[cell_index] = true
        }
    }

    pub fn set_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
    }

    pub fn change_direction(&mut self, direction: Direction) {
        let next_cell = self.get_next_cell(&direction);

        if self.body.len() > 1 {
            let prev_head = &self.body[1];
            if prev_head.equal(&next_cell) {
                return;
            }
        }

        self.direction = direction;
    }


    pub fn set_world_width(&mut self, width: usize) -> bool {
        let snake_in_removed_zone = self.check_if_snake_is_in_removed_zone(width, self.world.height);

        if !snake_in_removed_zone {
            self.world.set_world_width(width);
        }

        return !snake_in_removed_zone;
    }

    pub fn set_world_height(&mut self, height: usize) -> bool {
        let snake_in_removed_zone = self.check_if_snake_is_in_removed_zone(self.world.width, height);

        if !snake_in_removed_zone {
            self.world.set_world_height(height);
        }

        return !snake_in_removed_zone;
    }

    fn check_if_snake_is_in_removed_zone(&self, width: usize, height: usize) -> bool {
        for cell in self.body.iter() {
            if cell.y == height || cell.x == width {
                // console::log_1(&format!("{}", self.check_if_snake_is_in_removed_zone(self.world.width, height)).into());
                console::log_1(&"true".into());

                return true;
            }
        }
        console::log_1(&"False".into());

        false
    }
}


#[wasm_bindgen]
pub struct World {
    width: usize,
    height: usize,
    size: usize,
    board: Vec<bool>,
    reward_cell: Option<Cell>,

}


#[wasm_bindgen]
impl World {
    pub fn new(width: usize, height: usize) -> World {
        let size = width * height;
        World {
            width,
            height,
            size,
            board: vec![false; size],
            reward_cell: None,
        }
    }

    fn generate_reward_cell(&self) -> Cell {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if !self.board[y * self.width + x] {
                return Cell {
                    x,
                    y,
                };
            }
        }
    }

    pub fn get_reward_cell_idx(&self) -> i32 {
        match &self.reward_cell {
            None => -1,
            Some(cell) => cell.to_index(self) as i32
        }
    }


    pub fn set_world_width(&mut self, width: usize) {
        if width < self.width {
            for i in 1..(self.height + 1) {
                if self.board[i * self.width - 1] {
                    return;
                }
            }
        }

        self.width = width;
        self.size = self.width * self.height;
        self.regenerate_reward_cell_if_exist();
        self.redeclare_board();
    }

    pub fn set_world_height(&mut self, height: usize) {
        self.height = height;
        self.size = self.width * self.height;
        self.regenerate_reward_cell_if_exist();
        self.redeclare_board();
    }

    fn regenerate_reward_cell_if_exist(&mut self) {
        match self.reward_cell {
            None => {}
            Some(_) => self.reward_cell = Some(self.generate_reward_cell()),
        }
    }

    fn redeclare_board(&mut self) {
        let mut new_board = vec![false; self.size];

        for (i, cell) in self.board.iter().enumerate() {
            if i == self.size {
                break;
            }

            new_board[i] = *cell;
        }

        self.board = new_board;
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}
