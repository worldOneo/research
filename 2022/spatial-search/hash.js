import { Rect } from "./qtree.js";

export class SpatialHash {
  /**
   * @param {Rect} rect
   */
  constructor(rect, cellsize) {
    let width = rect.w - rect.x;
    let height = rect.h - rect.y;
    this._off_x = rect.x;
    this._off_y = rect.y;
    this._w = Math.ceil(width / cellsize);
    this._h = Math.ceil(height / cellsize);
    this._cellsize = cellsize;
    this._cells = Array(this._w * this._h)
      .fill()
      .map(() => new Array());
  }

  /**
   *
   * @param {Rect} rect
   * @returns {[number, number, number, number]}
   */
  _getRanges(rect) {
    let minX = rect.x - this._off_x;
    let minY = rect.y - this._off_y;
    let maxX = rect.x + rect.w - this._off_x;
    let maxY = rect.y + rect.h - this._off_y;

    let i1 = Math.floor(minY / this._cellsize);
    let i2 = Math.ceil(maxY / this._cellsize);

    let j1 = Math.floor(minX / this._cellsize);
    let j2 = Math.ceil(maxX / this._cellsize);

    return [i1, i2, j1, j2];
  }

  /**
   * @param {Rect} rect
   */
  insert(rect) {
    let [i1, i2, j1, j2] = this._getRanges(rect);
    for (let i = i1; i < i2; i++) {
      for (let j = j1; j < j2; j++) {
        this._cells[j + i * this._w].push(rect);
      }
    }
  }

  /**
   * @param {Rect} rect
   * @return {Array<Rect>} result
   */
  query(rect) {
    let [i1, i2, j1, j2] = this._getRanges(rect);
    let result = [];
    for (let i = i1; i < i2; i++) {
      for (let j = j1; j < j2; j++) {
        result.push(...this._cells[j + i * this._w]);
      }
    }
    return result;
  }

  draw(gr) {
    for (let i = 0; i < this._w; i++) {
      for (let j = 0; j < this._h; j++) {
        gr.drawRect(
          i * this._cellsize,
          j * this._cellsize,
          this._cellsize,
          this._cellsize
        );
      }
    }
  }
}
