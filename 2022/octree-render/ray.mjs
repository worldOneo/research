const roundDown = (pos, d) => {
  return ((pos / d) | 0) * d;
};

const roundUp = (pos, d) => {
  return (((pos + d) / d) | 0) * d;
};

export class Vector {
  constructor(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;

    this.szx = Math.sqrt(1 + (x / z) * (x / z));
    this.szy = Math.sqrt(1 + (y / z) * (y / z));

    this.sxy = Math.sqrt(1 + (y / x) * (y / x));
    this.sxz = Math.sqrt(1 + (z / x) * (z / x));

    this.syx = Math.sqrt(1 + (x / y) * (x / y));
    this.syz = Math.sqrt(1 + (z / y) * (z / y));
  }

  step(d) {}

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
