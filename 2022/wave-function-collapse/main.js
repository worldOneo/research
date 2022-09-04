const canvas = document.createElement("canvas");
canvas.height = 1000;
canvas.width = 1000;
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d");
const img = new Image();
const blockMargin = 1;
const blockSize = 2 * blockMargin + 1;
const LEFT = 0,
  TOP = 1,
  RIGHT = 2,
  BOTTOM = 3,
  UNDEF = 4;

const constraints = [
  [UNDEF, UNDEF, UNDEF, UNDEF],
  [LEFT, UNDEF, UNDEF, UNDEF],
  [UNDEF, TOP, UNDEF, UNDEF],
  [UNDEF, UNDEF, RIGHT, UNDEF],
  [UNDEF, UNDEF, UNDEF, BOTTOM],
  // opposite
  [UNDEF, TOP, UNDEF, BOTTOM],
  [LEFT, UNDEF, RIGHT, UNDEF],
  // corners
  [LEFT, TOP, UNDEF, UNDEF],
  [UNDEF, TOP, RIGHT, UNDEF],
  [UNDEF, UNDEF, RIGHT, BOTTOM],
  [LEFT, UNDEF, UNDEF, BOTTOM],
  // except one
  [LEFT, TOP, RIGHT, UNDEF],
  [LEFT, UNDEF, RIGHT, BOTTOM],
  [LEFT, TOP, UNDEF, BOTTOM],
  [UNDEF, TOP, RIGHT, BOTTOM],
  // all four
  [LEFT, TOP, RIGHT, BOTTOM],
];

const constraintIds = constraints.map(constraints =>
  constraints.reduce((acc, constraint) => (acc << 4) + (constraint + 1), 0)
);

const aCode = "A".charCodeAt(0);
/**
 *
 * @param {Uint8ClampedArray} data
 * @param {*} x
 * @param {*} y
 * @param {*} w
 */
const createCode = (data, x, y, w) => {
  let a = "";
  for (let i = 0; i < 4; i++) {
    // console.log(data[(x + y * w) * 4 + i], x, y);
    const c = data[(x + y * w) * 4 + i];
    a += String.fromCharCode(aCode + ((c & 0xf0) >> 4)); //.toString(16).padStart(2, "0");
    a += String.fromCharCode(aCode + (c & 0x0f));
  }
  return a;
};

const createRequirement = (grid, x, y, w, h) => {
  let top, bottom, left, right;
  if (x === 0) {
    left = "$";
  } else {
    left = grid[x - 1 + y * w]?.codes.at(RIGHT) ?? "$";
  }

  if (y === 0) {
    top = "$";
  } else {
    top = grid[x + (y - 1) * w]?.codes.at(BOTTOM) ?? "$";
  }

  if (x === w) {
    right = "$";
  } else {
    right = grid[x + 1 + y * w]?.codes.at(LEFT) ?? "$";
  }

  if (y === h) {
    bottom = "$";
  } else {
    bottom = grid[x + (y + 1) * w]?.codes.at(TOP) ?? "$";
  }
  return left + top + right + bottom;
};

const insVal = ([_, b]) => b;
const r = m => (Math.random() * m) | 0;

const createMap = (rules, x, y, w, h, grid) => {
  if (grid == null) {
    grid = new Array(w * h);
  }
  const gridIndex = x + y * w;
  if (x < 0 || x >= w || y < 0 || y >= h || grid[gridIndex] !== undefined) {
    return [grid, true];
  }
  const constraint = createRequirement(grid, x, y, w, h);
  //console.log(constraint);
  const options = rules[constraint];
  if (options === undefined || options.length == 0) {
    return [grid, false];
  }
  const offset = r(options.length);
  for (let i = 0; i < options.length; i++) {
    const index = (offset + i) % options.length;
    grid[gridIndex] = options[index];
    if (
      insVal(createMap(rules, x + 1, y, w, h, grid)) &&
      insVal(createMap(rules, x, y + 1, w, h, grid))
    ) {
      return [grid, true];
    }
  }
  grid[gridIndex] = undefined;
  return [grid, false];
};

img.addEventListener("load", e => {
  const width = img.width;
  const height = img.height;
  console.log(e);
  const rules = {};
  ctx.drawImage(img, 0, 0);
  const imageData = ctx.getImageData(0, 0, width, height);
  const data = imageData.data;

  for (let x = blockMargin; x < img.width - blockMargin; x++) {
    for (let y = blockMargin; y < img.height - blockMargin; y++) {
      let block = {
        x,
        y,
        codes: null,
      };
      let topCode = "";
      let bottomCode = "";
      let rightCode = "";
      let leftCode = "";
      for (let dy = -blockMargin; dy <= blockMargin; dy++) {
        rightCode += createCode(data, x - blockMargin, y + dy, width);
        leftCode += createCode(data, x + blockMargin, y + dy, width);
        for (let dx = -blockMargin; dx <= blockMargin; dx++) {
          if (dy == 0) {
            topCode += createCode(data, x + dx, y, width);
          } else if (dy == blockMargin) {
            bottomCode += createCode(data, x + dx, y + dy, width);
          }
        }
      }
      const codes = [leftCode, topCode, rightCode, bottomCode, "$"];
      block.codes = codes;
      for (let i = 0; i < constraints.length; i++) {
        const key = constraints[i]
          .map(constraint => codes[constraint])
          .join("");
        rules[key] ??= [];
        rules[key].push(block);
      }
    }
  }
  console.log(rules);
  const gridSize = 16;
  const [grid, ok] = createMap(rules, 0, 0, gridSize, gridSize);
  if (!ok) {
    throw "Grid is impossible";
  }
  console.log(grid);
  const size = blockSize - 2;
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, gridSize * size, gridSize * size);
  for (let y = 0; y < gridSize; y++) {
    for (let x = 0; x < gridSize; x++) {
      const idx = x + y * gridSize;
      const val = grid[idx];
      if (val === undefined) {
        console.log(idx, val, x, y);
        continue;
      }
      ctx.putImageData(
        imageData,
        x * size - val.x,
        y * size - val.y,
        val.x,
        val.y,
        blockSize,
        blockSize
      );
    }
  }
});
img.src = "/base2.png";
