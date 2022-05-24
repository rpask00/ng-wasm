import {Component, ElementRef, OnInit, ViewChild} from '@angular/core';
import init, {Direction, GameState, InitOutput, Snake, World} from "assets/snake/pkg"
import {FormControl} from "@angular/forms";


@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit {

  rewardCellColor = "#CCEA8D";
  gridColor = "#01415B";
  backGroundColor = "#005148";
  snakeColor = "#019587";

  width$ = new FormControl(12, [])
  height$ = new FormControl(12, [])
  SPAWN_INDEX = Date.now() % (this.width$.value * this.height$.value)
  fps$ = new FormControl(9, []);
  cell_size$ = new FormControl(40, []);

  @ViewChild('canvas')
  canvas: ElementRef<HTMLCanvasElement>;


  ctx: CanvasRenderingContext2D;
  snake: Snake;
  private world: World;
  private snakeCells: Uint32Array;
  private wasm: InitOutput;
  GameState = GameState;

  async ngOnInit() {

    document.body.style.backgroundColor = this.backGroundColor;

    this.wasm = await init();
    this.ctx = this.canvas?.nativeElement.getContext('2d')


    this.world = World.new(this.width$.value, this.height$.value)
    this.snake = Snake.new(this.SPAWN_INDEX, 4, this.world);

    this.resizeCanvas();

    this.snakeCells = this.getSnakeCells()


    document.addEventListener('keydown', (e) => {
      let direction: Direction;
      switch (e.code) {
        case "ArrowLeft":
          direction = Direction.Left;
          break
        case "ArrowUp":
          direction = Direction.Up;
          break
        case "ArrowRight":
          direction = Direction.Right;
          break
        case "ArrowDown":
          direction = Direction.Down;
          break

      }

      this.snake.change_direction(direction)
    });


    this.update()

    this.width$.valueChanges.subscribe(width => {
      if (!this.snake.set_world_width(width)) {
        this.width$.setValue(width + 1)
      }

      this.resizeCanvas();
    })
    this.height$.valueChanges.subscribe(height => {
      if (!this.snake.set_world_height(height)) {
        this.height$.setValue(height + 1)
      }
      this.resizeCanvas();
    })

    this.cell_size$.valueChanges.subscribe(() => this.resizeCanvas())

  }


  private resizeCanvas() {
    this.canvas.nativeElement.width = this.width$.value * this.cell_size$.value;
    this.canvas.nativeElement.height = this.height$.value * this.cell_size$.value;
  }

  drawWorld() {
    let {ctx, cell_size$, height$, width$} = this
    ctx.beginPath();
    ctx.fillStyle = this.gridColor;

    for (let x = 0; x < width$.value + 1; x++) {
      ctx.moveTo(cell_size$.value * x, 0)
      ctx.lineTo(x * cell_size$.value, height$.value * cell_size$.value)
    }

    for (let y = 0; y < height$.value + 1; y++) {
      ctx.moveTo(0, cell_size$.value * y)
      ctx.lineTo(width$.value * cell_size$.value, cell_size$.value * y)
    }
    ctx.stroke();
  }

  drawRewardCell() {
    let {ctx, cell_size$, width$, snake} = this


    const rewardCellIdx = snake.get_reward_cell_idx()

    if (rewardCellIdx == -1)
      return;

    const row = Math.floor(rewardCellIdx / width$.value);
    const col = rewardCellIdx % width$.value;


    ctx.beginPath();
    ctx.fillStyle = this.rewardCellColor;
    ctx.fillRect(col * cell_size$.value, row * cell_size$.value, cell_size$.value, cell_size$.value);
    ctx.stroke();
  }

  drawSnake() {
    let {ctx, cell_size$, width$, snakeCells} = this


    for (let snake_pos of Array.from(snakeCells)) {
      const col = snake_pos % width$.value;
      const row = Math.floor(snake_pos / width$.value);

      ctx.fillStyle = this.snakeColor;
      ctx.beginPath();
      ctx.fillRect(col * cell_size$.value, row * cell_size$.value, cell_size$.value, cell_size$.value);
      ctx.stroke();
    }
  }

  paint() {
    this.drawWorld();
    this.drawSnake();
    this.drawRewardCell();
  }


  update() {
    let {ctx, cell_size$, width$, snake, canvas, fps$} = this


    setTimeout(() => {
      ctx.clearRect(0, 0, canvas.nativeElement.width, canvas.nativeElement.height);
      snake.update_position();
      this.snakeCells = this.getSnakeCells();
      this.paint();

      requestAnimationFrame(() => this.update());
    }, 1000 / fps$.value)

  }

  getSnakeCells() {
    let snakePtr = this.snake.snake_cells();
    let snakeLength = this.snake.snake_length();
    return new Uint32Array(this.wasm.memory.buffer, snakePtr, snakeLength)
  }

  changeGameState() {
    if (this.snake.game_state == GameState.Lost || this.snake.game_state == GameState.Won) {
      this.snake.restart_game();
    } else {
      this.snake.set_game_state(this.snake.game_state == GameState.Running ? GameState.Stopped : GameState.Running);
    }

  }


}
