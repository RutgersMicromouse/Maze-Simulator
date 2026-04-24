use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::*;
use ratatui::{prelude::*, symbols::border, widgets::*};
use simplelog::*;
use std::fs::File;
use std::io;

fn main() -> Result<()> {
    color_eyre::install()?;

    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("logs/app.log")?,
    )?;

    info!("Starting mazesim");

    let mut tui = Tui::new()?;
    let mut app = App::new()?;
    let res = app.run(&mut tui.terminal);

    info!("Exiting mazesim");
    res
}

struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    fn new() -> Result<Self> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        if let Err(e) = enable_raw_mode() {
            let _ = Self::restore();
            return Err(e.into());
        }
        let mut terminal = match Terminal::new(CrosstermBackend::new(io::stdout())) {
            Ok(t) => t,
            Err(e) => {
                let _ = Self::restore();
                return Err(e.into());
            }
        };
        if let Err(e) = terminal.clear() {
            let _ = Self::restore();
            return Err(e.into());
        }
        Ok(Self { terminal })
    }

    fn restore() -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = Self::restore();
    }
}

struct App {
    maze: Maze,
    mouse: Mouse,
    exit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            maze: Maze::new(16, 16)?,
            mouse: Mouse::new(0.0, 0.0),
            exit: false,
        })
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        // Initial draw
        terminal.draw(|frame| self.render_frame(frame))?;

        while !self.exit {
            let mut should_redraw = false;
            self.handle_events(&mut should_redraw)?;

            if should_redraw {
                terminal.draw(|frame| self.render_frame(frame))?;
            }
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, should_redraw: &mut bool) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.exit = true,
                        KeyCode::Up => {
                            self.mouse.move_up(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Up to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Down => {
                            self.mouse.move_down(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Down to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Left => {
                            self.mouse.move_left(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Left to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Right => {
                            self.mouse.move_right(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Right to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Char('y') => {
                            self.mouse.move_up_left(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Up-Left to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Char('u') => {
                            self.mouse.move_up_right(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Up-Right to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Char('b') => {
                            self.mouse.move_down_left(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Down-Left to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        KeyCode::Char('n') => {
                            self.mouse.move_down_right(&self.maze);
                            *should_redraw = true;
                            info!("Mouse moved Down-Right to ({:.1}, {:.1})", self.mouse.x, self.mouse.y);
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
    }


impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let inner_area = block.inner(area);
        block.render(area, buf);

        self.maze.render(inner_area, buf);
        self.mouse.render(inner_area, buf);
    }
}

struct Mouse {
    x: f64,
    y: f64,
}

impl Mouse {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn is_blocked(maze: &Maze, x: f64, y: f64) -> bool {
        /* check bounds  */
        if x < 0.0 || x > (maze.width - 1) as f64 || y < 0.0 || y > (maze.height - 1) as f64 {
            return true;
        }

        /* check if in middle of maze */
        if (x % 0.5 == 0.0 || y % 0.5 == 0.0) && (x % 1.0 != 0.0 && y % 1.0 != 0.0) {
            return true;
        }

        let eps = 0.1;
        // Check vertical wall boundaries
        let x_fract = x.fract();
        if (x_fract - 0.5).abs() < eps {
            let i = x.floor() as usize;
            let y_floor = y.floor() as usize;
            let y_ceil = y.ceil() as usize;
            if maze.has_wall(i, y_floor, Direction::East)
                || (y_ceil < maze.height && maze.has_wall(i, y_ceil, Direction::East))
            {
                return true;
            }
        }
        // Check horizontal wall boundaries
        let y_fract = y.fract();
        if (y_fract - 0.5).abs() < eps {
            let j = y.floor() as usize;
            let x_floor = x.floor() as usize;
            let x_ceil = x.ceil() as usize;
            if maze.has_wall(x_floor, j, Direction::South)
                || (x_ceil < maze.width && maze.has_wall(x_ceil, j, Direction::South))
            {
                return true;
            }
        }
        false
    }

    fn move_up(&mut self, maze: &Maze) {
        let next_y = self.y - 0.5;
        if next_y >= 0.0 && !Self::is_blocked(maze, self.x, next_y) {
            self.y = next_y;
        }
    }

    fn move_down(&mut self, maze: &Maze) {
        let next_y = self.y + 0.5;
        if next_y <= (maze.height - 1) as f64 && !Self::is_blocked(maze, self.x, next_y) {
            self.y = next_y;
        }
    }

    fn move_left(&mut self, maze: &Maze) {
        let next_x = self.x - 0.5;
        if next_x >= 0.0 && !Self::is_blocked(maze, next_x, self.y) {
            self.x = next_x;
        }
    }

    fn move_right(&mut self, maze: &Maze) {
        let next_x = self.x + 0.5;
        if next_x <= (maze.width - 1) as f64 && !Self::is_blocked(maze, next_x, self.y) {
            self.x = next_x;
        }
    }

    fn move_up_left(&mut self, maze: &Maze) {
        let nx = self.x - 0.5;
        let ny = self.y - 0.5;
        if nx >= 0.0 && ny >= 0.0 && !Self::is_blocked(maze, nx, ny) {
            self.x = nx;
            self.y = ny;
        }
    }

    fn move_up_right(&mut self, maze: &Maze) {
        let nx = self.x + 0.5;
        let ny = self.y - 0.5;
        if nx <= (maze.width - 1) as f64 && ny >= 0.0 && !Self::is_blocked(maze, nx, ny) {
            self.x = nx;
            self.y = ny;
        }
    }

    fn move_down_left(&mut self, maze: &Maze) {
        let nx = self.x - 0.5;
        let ny = self.y + 0.5;
        if nx >= 0.0 && ny <= (maze.height - 1) as f64 && !Self::is_blocked(maze, nx, ny) {
            self.x = nx;
            self.y = ny;
        }
    }

    fn move_down_right(&mut self, maze: &Maze) {
        let nx = self.x + 0.5;
        let ny = self.y + 0.5;
        if nx <= (maze.width - 1) as f64
            && ny <= (maze.height - 1) as f64
            && !Self::is_blocked(maze, nx, ny)
        {
            self.x = nx;
            self.y = ny;
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let cell_w = 4;
        let cell_h = 2;
        let px = area.x + (self.x * cell_w as f64) as u16 + cell_w / 2;
        let py = area.y + (self.y * cell_h as f64) as u16 + cell_h / 2;

        if px < area.right() && py < area.bottom() {
            buf[(px, py)]
                .set_symbol("M")
                .set_fg(ratatui::prelude::Color::Red)
                .set_style(Style::default().bold());
        }
    }
}

struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Default, Clone, Copy)]
struct Cell {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

impl Maze {
    fn new(width: usize, height: usize) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(color_eyre::eyre::eyre!(
                "Maze dimensions must be greater than zero"
            ));
        }
        let mut cells = vec![Cell::default(); width * height];

        // Add some default outer walls
        for x in 0..width {
            cells[x].north = true;
            cells[(height - 1) * width + x].south = true;
        }
        for y in 0..height {
            cells[y * width].west = true;
            cells[y * width + (width - 1)].east = true;
        }

        Ok(Self {
            width,
            height,
            cells,
        })
    }

    fn has_wall(&self, x: usize, y: usize, dir: Direction) -> bool {
        if x >= self.width || y >= self.height {
            return true;
        }
        let cell = self.cells[y * self.width + x];
        match dir {
            Direction::North => cell.north,
            Direction::South => cell.south,
            Direction::East => cell.east,
            Direction::West => cell.west,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Calculate cell size based on area
        let cell_w = 4;
        let cell_h = 2;

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cells[y * self.width + x];
                let px = area.x + (x as u16 * cell_w);
                let py = area.y + (y as u16 * cell_h);

                if px + cell_w >= area.right() || py + cell_h >= area.bottom() {
                    continue;
                }

                // Draw cell contents (empty for now)
                // Draw walls
                if cell.north {
                    for i in 0..cell_w {
                        buf[(px + i, py)].set_symbol("─");
                    }
                }
                if cell.south {
                    for i in 0..cell_w {
                        buf[(px + i, py + cell_h)].set_symbol("─");
                    }
                }
                if cell.west {
                    for i in 0..cell_h {
                        buf[(px, py + i)].set_symbol("│");
                    }
                }
                if cell.east {
                    for i in 0..cell_h {
                        buf[(px + cell_w, py + i)].set_symbol("│");
                    }
                }

                // Corners (simplified)
                buf[(px, py)].set_symbol("+");
                buf[(px + cell_w, py)].set_symbol("+");
                buf[(px, py + cell_h)].set_symbol("+");
                buf[(px + cell_w, py + cell_h)].set_symbol("+");
            }
        }
    }
}
