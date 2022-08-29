import { OmniOctTree, Cube } from "./tree.mjs";

/** @type {HTMLCanvasElement} */
//const canvas = document.getElementById("canvas");
//const ctx = canvas.getContext("2d");

let tree = new OmniOctTree();
const r = (m, i) => Math.random() * (m - i) + i;
const rw = () => r(-2000, 2000);

console.time("tree");
for (let i = 0; i < 1000000; i++) {
  tree.insert(new Cube(rw(), rw(), rw(), 10, 10, 10));
}
console.timeEnd("tree");
