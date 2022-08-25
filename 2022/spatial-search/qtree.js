export class Rect {
  constructor(x, y, w, h, attrib = null) {
    this.x = x;
    this.y = y;
    this.w = w;
    this.h = h;
    this.attrib = attrib;
  }
}

/**
 * @param {Rect} out
 * @param {Rect} cont
 */
export const rectInRect = (out, cont) => {
  return !(
    out.x > cont.x + cont.w ||
    out.x + out.w < cont.x ||
    out.y > cont.y + cont.h ||
    out.y + out.h < cont.y
  );
};

export class QuadTree {
  /**
   * @param {Rect} rect
   * @param {Number} capacity
   */
  constructor(rect, capacity) {
    this._rect = rect;
    this._capacity = capacity;
    this._split = false;
    this._items = new Array();
    /** @type {Array<QuadTree>} */
    this._children;
  }

  /**
   * @param {Rect} rect
   */
  insert(rect) {
    if (!rectInRect(this._rect, rect)) {
      return;
    }

    if (this._split) {
      this._children[0].insert(rect);
      this._children[1].insert(rect);
      this._children[2].insert(rect);
      this._children[3].insert(rect);
      return;
    }

    this._items.push(rect);

    if (this._items.length <= this._capacity) {
      return;
    }

    let x = this._rect.x;
    let y = this._rect.y;
    let w = this._rect.w;
    let h = this._rect.h;

    let w2 = w / 2;
    let h2 = h / 2;

    this._split = true;
    this._children = [
      new QuadTree(new Rect(x, y, w2, h2), this._capacity),
      new QuadTree(new Rect(x + w2, y, w2, h2), this._capacity),
      new QuadTree(new Rect(x, y + h2, w2, h2), this._capacity),
      new QuadTree(new Rect(x + w2, y + h2, w2, h2), this._capacity),
    ];
  }

  /**
   * @param {Rect} rect
   * @param {Array<Rect>} result
   * @return {Array<Rect>}
   */
  query = (rect, result = []) => {
    if (!rect) {
      return result;
    }

    if (!rectInRect(this._rect, rect)) {
      return result;
    }
    result.push(...this._items);

    if (this._split) {
      this._children[0].query(rect, result);
      this._children[1].query(rect, result);
      this._children[2].query(rect, result);
      this._children[3].query(rect, result);
    }
    return result;
  };

  draw(gr) {
    gr.drawRect(this._rect.x, this._rect.y, this._rect.w, this._rect.h);
    if (this._split) {
      this._children[0].draw(gr);
      this._children[1].draw(gr);
      this._children[2].draw(gr);
      this._children[3].draw(gr);
    }
  }
}
