import { OmniOctTree, Cube, OctTree } from "./tree.mjs";
import { Vector } from "./ray.mjs";

/** @type {HTMLCanvasElement} */
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

let tree = new OctTree(
  new Cube(-10_000, -10_000, -10_000, 20_000, 20_000, 20_000),
  1
);
const r = (m, i) => Math.random() * (m - i) + i;
const rw = () => r(-400, 400);

//tree.insert(new Cube(0, 0, 200, 100, 100, 100));
tree.insert(new Cube(24, 24, 50, 20, 20, 20));
//tree.insert(new Cube(0, 10, 200, 5, 4, 4));
console.log(tree, tree.findClosest(0, 0, 0));

console.time("find");
const end = 0.01;
for (let i = 0; i < 600; i++) {
  for (let j = 0; j < 600; j++) {
    let pos = new Vector(0, 0, 0);
    let dir = new Vector(i, j, 600).normalized;
    let ok = false;
    for (let s = 0; s < 1000; s++) {
      let res = tree.findClosest(pos.x, pos.y, pos.z);
      if (res == null) break;
      if (res[1] < end) {
        break;
      }
      pos.add(dir.x, dir.y, dir.z, res[1]); // Needs DDA to avoid overstepping
    }
    let level = (1 / (pos.length / 50) ** 2) * 255;
    ctx.fillStyle = `rgba(${level},${level}, ${level}, 255)`;
    ctx.fillRect(i, j, 1, 1);
    ok = true;
  }
}
console.timeEnd("find");
