import { Vector } from "./ray.mjs";

/**
 * @param {Cube} a
 * @param {Cube} b
 */
const cubesIntersect = (a, b) =>
  !(
    a.x + a.w < b.x ||
    a.x > b.x + b.w ||
    a.y + a.h < b.y ||
    a.y > b.y + b.h ||
    a.z + a.l < b.z ||
    a.z > b.z + b.l
  );

export class Cube {
  constructor(x, y, z, w, h, l) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.w = w;
    this.h = h;
    this.l = l;
  }

  /**
   * @param {Cube} cube
   */
  containsCube(cube) {
    return (
      this.containsPoint(cube.x, cube.y, cube.z) &&
      this.containsPoint(cube.x + cube.w, cube.y + cube.h, cube.z + cube.l)
    );
  }

  containsPoint(x, y, z) {
    return !(
      x < this.x ||
      x > this.x + this.w ||
      y < this.y ||
      y > this.y + this.h ||
      z < this.z ||
      z > this.z + this.l
    );
  }

  distanceTo(x, y, z) {
    // let ax = x - this.x;
    // let ay = y - this.y;
    // let az = z - this.z;
    // let displacement = Math.sin(10 * x) * 0.25;
    // return Math.sqrt(ax ** 2 + ay ** 2 + az ** 2) - this.w + displacement;

    let dx = Math.max(this.x - x, 0, x - (this.x + this.w)); //+ this.w);
    let dy = Math.max(this.y - y, 0, y - (this.y + this.h));
    let dz = Math.max(this.z - z, 0, z - (this.z + this.l));

    return Math.sqrt(dx ** 2 + dy ** 2 + dz ** 2);
  }
}

/**
 * Octs:
 * [ Top:
 *  6,7
 *  4,5
 * ].[ Bottom:
 *  2,3
 *  0,1
 * ]
 * X -
 * Z |
 */

export class OmniOctTree {
  static options = [];
  constructor() {
    this._base = new Cube(-10_000, -10_000, -10_000, 20_000, 20_000, 20_000);
    this._oct = new OctTree(this._base);
  }

  /**
   * @param {Cube} cube
   */
  insert(cube) {
    if (this._oct.contains(cube)) {
      return this._oct.insert(cube);
    }
    return this._Explode(cube);
  }

  /**
   * @param {Cube} cube
   */
  query(cube) {
    return this._oct.query(cube);
  }

  /**
   *
   */
  findClosest(x, y, z, r) {
    return this._oct.findClosest(x, y, z, r);
  }

  /**
   * @param {Cube} cube
   */
  _Explode(cube) {
    let old = this._oct;
    let base = this._base;

    while (!this._oct.contains(cube)) {
      let ox = 0,
        oy = 0,
        oz = 0,
        ow = 2,
        oh = 2,
        ol = 2,
        s = 0;
      if (cube.x <= base.x) {
        ox = -1;
        s = 1;
      } else if (cube.y <= base.y) {
        oy = -1;
        s = 4;
      } else if (cube.z <= base.z) {
        oz = -1;
        s = 2;
      }
      this._base = new Cube(
        base.x + base.w * ox,
        base.y + base.h * oy,
        base.z + base.l * oz,
        base.w * ow,
        base.h * oh,
        base.l * ol
      );
      this._oct = new OctTree(this._base);
      this._oct._Split();
      this._oct._children[s] = old;
      base = this._base;
      old = this._oct;
    }
    return this.insert(cube);
  }
}

export class FindResult {
  /**
   * @param {Cube} cube
   * @param {Number} distance
   */
  constructor(cube, distance) {
    this.cube = cube;
    this.distance = distance;
  }
}

export class OctTree {
  /**
   * @param {Cube} cube
   * @param {Number} capacity
   */
  constructor(cube, capacity = 10) {
    this._cube = cube;
    this._capacity = capacity;
    this._children = null;
    /** @type {Array<Cube>} */
    this._items = [];
    this._split = false;
    this._contained = false;
  }

  /**
   * @param {Cube} cube
   */
  insert(cube) {
    if (!cubesIntersect(this._cube, cube)) {
      return false;
    }

    if (this._split) {
      let insert = false;
      for (let child of this._children) {
        insert |= child.insert(cube);
      }
      return insert;
    }
    this._items.push(cube);
    if (this._items.length > this._capacity) {
      this._Split();
    }
    return true;
  }

  /**
   * @param {number} x
   * @param {number} y
   * @param {number} z
   * @param {number} dist
   * @param {FindResult} r
   */
  findClosest(x, y, z, dist, r) {
    if (this._cube.distanceTo(x, y, z) > dist) {
      return false;
    }
    let d = dist;
    let min = null;
    if (this._split) {
      for (let child of this._children) {
        if (!child.findClosest(x, y, z, d, r)) continue;
        if (d > r.distance) {
          min = r.cube;
          d = r.distance;
        }
      }
    } else {
      for (let item of this._items) {
        let _d = item.distanceTo(x, y, z);
        if (d > _d) {
          d = _d;
          min = item;
        }
      }
    }

    if (min !== null) {
      r.cube = min;
      r.distance = d;
      return true;
    }
    return false;
  }

  /**
   * @param {Cube} query
   */
  query(
    query,
    res = { items: [], finished: false, maxJmpSize: Infinity },
    fast = false
  ) {
    if (!cubesIntersect(this._cube, query)) {
      return res.items;
    }

    res.maxJmpSize = Math.min(res.maxJmpSize, this._cube.w);
    res.items.push(...this._items);
    if (this._contained && fast) {
      res.finished = true;
      return res.items;
    }
    if (this._split) {
      for (let child of this._children) {
        child.query(query, res, fast);
        if (res.finished) {
          return res.items;
        }
      }
    }
  }

  /**
   * @param {Cube} cube
   */
  contains(cube) {
    return this._cube.containsCube(cube);
  }

  _Split() {
    this._split = true;
    let x = this._cube.x;
    let y = this._cube.y;
    let z = this._cube.z;
    let w2 = this._cube.w / 2;
    let h2 = this._cube.h / 2;
    let l2 = this._cube.l / 2;
    let c = this._capacity;
    this._children = [
      // Bottom
      new OctTree(new Cube(x, y, z, w2, h2, l2), c), //0
      new OctTree(new Cube(x + w2, y, z, w2, h2, l2), c),
      new OctTree(new Cube(x, y, z + l2, w2, h2, l2), c),
      new OctTree(new Cube(x + w2, y, z + l2, w2, h2, l2), c),
      // Top
      new OctTree(new Cube(x, y + h2, z, w2, h2, l2), c), //4
      new OctTree(new Cube(x + w2, y + h2, z + l2, w2, h2, l2), c),
      new OctTree(new Cube(x, y + h2, z + l2, w2, h2, l2), c),
      new OctTree(new Cube(x + w2, y + h2, z + l2, w2, h2, l2), c),
    ];
    
    for(let item of this._items) {
      let insert = 0;
      for (let children of this._children) {
        insert += children.insert(item) | 0;
      }
    }
    this._items = null;
  }
}
