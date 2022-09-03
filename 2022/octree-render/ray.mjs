export class Vector {
  constructor(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  add(x, y, z, f) {
    this.x += x * f;
    this.y += y * f;
    this.z += z * f;
  }

  get normalized() {
    let l = this.length;
    return new Vector(this.x / l, this.y / l, this.z / l);
  }

  get length() {
    let t = this.x * this.x + this.y * this.y + this.z * this.z;
    return Math.sqrt(t);
  }

  get cloned() {
    return new Vector(this.x, this.y, this.z);
  }
}
