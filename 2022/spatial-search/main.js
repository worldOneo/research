import { QuadTree, Rect } from "./qtree.js";

let app = new PIXI.Application({ width: 1920, height: 900, antialis: true });
document.body.appendChild(app.view);

// Create the sprite and add it to the stage

const gr = new PIXI.Graphics();

let qtree = new QuadTree(new Rect(0, 0, 1920, 900), 10);
const r = (m, i) => Math.random() * (m - i) + i;
const rw = () => r(0, 1920);
const rh = () => r(0, 900);

const crunch = () => {
  gr.clear();
  gr.lineStyle(1, 0xffffff);
  qtree = new QuadTree(new Rect(0, 0, 1920, 900), 10);
  console.time("insert");
  for (let i = 0; i < 10_000; i++) {
    let rect = new Rect(rw(), rh(), 3, 3);
    gr.drawRect(rect.x, rect.y, rect.w, rect.h);
    qtree.insert(rect);
  }
  console.timeEnd("insert");
  qtree.draw(gr);
};
crunch();
crunch();
crunch();
crunch();

let query = new Rect(rw(), rh(), 100, 100);
gr.lineStyle(3, 0x00ff00);
gr.drawRect(query.x, query.y, query.w, query.h);
console.time("query");
let res = qtree.query(query);
for (let rect of res) {
  gr.drawRect(rect.x, rect.y, rect.w, rect.h);
}
console.timeEnd("query");
console.log(qtree);
app.stage.addChild(gr);
