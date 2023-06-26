import { OmniOctTree, Cube, OctTree, FindResult } from "./tree.mjs";
import { Vector } from "./ray.mjs";

/** @type {HTMLCanvasElement} */
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
ctx.fillStyle = "black";
ctx.fillRect(0, 0, canvas.width, canvas.height);

let tree = new OctTree(
  new Cube(-10_000, -10_000, -10_000, 20_000, 20_000, 20_000),
  4
);
const r = (m, i) => (Math.random() * (m - i) + i) | 0;
const rw = () => r(0, 100);

tree.insert(new Cube(0, 0, 200, 5, 5, 5));
tree.insert(new Cube(20, 20, 30, 5, 5, 5));
tree.insert(new Cube(0, 10, 200, 5, 5, 5));

console.time("generate");
for (let i = 0; i < 50; i++) {
  tree.insert(new Cube(rw(), rw(), rw(), 2, 2, 2));
}
console.timeEnd("generate");

console.log(tree);

console.time("find");
const min_dist = 0.01;
const max_dist = 1000;
let fr = new FindResult(null, 0, false);
let data = ctx.getImageData(0, 0, 600, 600);
for (let i = 0; i < 600; i++) {
  pixel: for (let j = 0; j < 600; j++) {
    let pos = new Vector(0, 0, 0);
    let dir = new Vector(i, j, 600).normalized;
    let ok = false;
    for (let s = 0; s < 60; s++) {
      if (!tree.findClosest(pos.x, pos.y, pos.z, Infinity, fr)) continue pixel;
      ok = true;
      if (pos.length > max_dist) {
        break;
      }
      if (fr.distance < min_dist) {
        break;
      }
      pos.add(dir.x, dir.y, dir.z, fr.distance); // Needs DDA to avoid overstepping
    }
    if (!ok) continue;
    let level = ((1 / (pos.length / 40))**3 * 255) | 0;
    let idx = 600 * 4 * i + 4 * j;
    data.data[idx] = level;
    data.data[idx + 1] = level;
    data.data[idx + 2] = level;
    data.data[idx + 3] = 255;
  }
}
ctx.putImageData(data, 0, 0);
console.timeEnd("find");
