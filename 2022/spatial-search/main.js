import { SpatialHash } from "./hash.js";
import { QuadTree, Rect } from "./qtree.js";

delete PIXI.Renderer.__plugins.interaction;
let app = new PIXI.Application({
  width: 1920,
  height: 900,
  antialis: true,
  autoDensity: true,
});
document.body.appendChild(app.view);

if (!("events" in app.renderer)) {
  app.renderer.addSystem(PIXI.EventSystem, "events");
}

// Create the sprite and add it to the stage

let gr = new PIXI.Graphics();

let qfactory = () => new QuadTree(new Rect(0, 0, 1920, 900), 10);
let sfactory = () => new SpatialHash(new Rect(-100, -100, 2000, 1000), 100);

let factory = sfactory;

let index = factory();

const r = (m, i) => Math.random() * (m - i) + i;
const rw = () => r(0, 1920);
const rh = () => r(0, 900);
let mouseX = 0,
  mouseY = 0,
  lastTime = performance.now();

let rects = [];

const crunch = () => {
  rects = [];
  gr.clear();
  gr.lineStyle(1, 0xffffff);
  index = factory();
  for (let i = 0; i < 10_000; i++) {
    let rect = new Rect(rw(), rh(), 3, 3);
    rects.push(rect);
    index.insert(rect);
  }
  index.draw(gr);
};

let rectgr = new PIXI.Graphics();
const drawRects = () => {
  rectgr.clear();
  let gr = rectgr;
  gr.clear();
  app.stage.addChild(gr);
  for (let rect of rects) {
    gr.lineStyle(1, 0xffffff);
    gr.drawRect(rect.x, rect.y, rect.w, rect.h);
  }
};

const query = () => {
  let query = new Rect(mouseX - 50, mouseY - 50, 100, 100);
  gr.lineStyle(3, 0x00ff00);
  gr.drawRect(query.x, query.y, query.w, query.h);
  let res = index.query(query);
  for (let rect of res) {
    gr.drawRect(rect.x, rect.y, rect.w, rect.h);
  }
};
const style = new PIXI.TextStyle({
  fontSize: 24,
  fill: ["#00aaaa"],
});

crunch();
drawRects();

const text = new PIXI.Text(`FPS: 0\nQ: 0`, style);
app.ticker.add(delta => {
  gr.clear();
  gr.lineStyle(1, 0xffffff);
  index.draw(gr);
  lastTime = performance.now();
  query();
  text.text = `FPS: ${(1 / (app.ticker.elapsedMS / 1000)).toFixed(2)}\nQ: ${
    performance.now() - lastTime
  }ms`;
});

app.stage.addEventListener("pointermove", e => {
  mouseX = e.global.x;
  mouseY = e.global.y;
});

console.log(app.stage);
app.stage.interactive = true;
app.stage.hitArea = app.renderer.screen;
app.stage.addChild(gr);
app.stage.addChild(text);

let now = performance.now();
for (let i = 0; i < 100; i++) {
  crunch();
}
console.log((performance.now() - now) / 100, "ms per crunch");

now = performance.now();
for (let i = 0; i < 100; i++) {
  index.query(new Rect(rw(), rh(), 100, 100));
}
console.log((performance.now() - now) / 100, "ms per query");
drawRects();

drawRects();
