import { OmniOctTree, Cube } from "./tree.mjs";
import { Vector } from "./ray.mjs";

/** @type {HTMLCanvasElement} */
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

let tree = new OmniOctTree();
const r = (m, i) => Math.random() * (m - i) + i;
const rw = () => r(-2000, 2000);
