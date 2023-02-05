import { FC, Fragment, useState } from "react";

type SpecC<T> = FC<{ spec: T }>;

export class Spec<T> {
  constructor(
    private _name: string,
    initial: T,
    public render: SpecC<any & Spec<T>>,
    public data: T = initial
  ) {}

  name = () => {
    return this._name;
  };

  display = () => {
    return (
      <>
        {this._name}
        <this.render spec={this} />
      </>
    );
  };

  export = () => {
    return this.data;
  };
}

const NumberComponent: SpecC<NumberSpec> = ({ spec }) => {
  let [val, setVal] = useState(spec.data);

  return (
    <input
      type="number"
      value={val}
      onChange={e => {
        setVal(e.target.valueAsNumber | 0);
        spec.data = e.target.valueAsNumber | 0;
      }}
    />
  );
};

export class NumberSpec extends Spec<number> {
  constructor(_name: string) {
    super(_name, 0, NumberComponent);
  }
}

function ArrayComponent<T>({ spec }: { spec: ArraySpec<T> }) {
  type PointType = { id: number; data: Spec<T> };
  const [data, setData] = useState<PointType[]>([]);
  spec.points = data.map(d => d.data);
  spec.data = data.map(d => d.data.export());
  const [ctr, setCtr] = useState(0);

  return (
    <>
      <button
        onClick={() => {
          setData([...data, { id: ctr + 1, data: spec.spec() }]);
          setCtr(ctr + 1);
        }}
      >
        Add
      </button>
      {data.map(point => (
        <Fragment key={point.id}>
          <div>
            <point.data.display />
            <button
              onClick={() => {
                setData(data.filter(other => other.id !== point.id));
              }}
            >
              Remove
            </button>
          </div>
        </Fragment>
      ))}
    </>
  );
}

export class ArraySpec<T> extends Spec<T[]> {
  public points: Spec<T>[] = [];
  constructor(_name: string, public spec: () => Spec<T>) {
    super(_name, [], ArrayComponent);
  }

  export = () => {
    return this.points.map(p => p.export());
  };
}

const FieldsComponent: SpecC<FieldsSpec> = ({ spec }) => {
  return (
    <>
      {Object.entries(spec.specs).map(s => {
        let C = s[1].display;
        return (
          <div key={s[0]}>
            <div>{s[0]}</div>
            <C />
          </div>
        );
      })}
    </>
  );
};

export class FieldsSpec extends Spec<Record<string, any>> {
  constructor(_name: string, public specs: Record<string, Spec<any>>) {
    let base: Record<string, any> = {};
    for (let k of Object.entries(specs)) {
      base[k[0]] = k[1].data;
    }
    super(_name, base, FieldsComponent);
  }

  export = () => {
    let base: Record<string, any> = {};
    for (let k of Object.entries(this.specs)) {
      base[k[0]] = k[1].export();
    }
    return base;
  };
}

function FlaggedComponent<V, T>({ spec }: { spec: FlaggedSpec<V, T> }) {
  let [active, setActive] = useState(false);

  return (
    <>
      <input
        type="checkbox"
        checked={active}
        onChange={e => {
          setActive(e.target.checked);
          spec.data = e.target.checked;
        }}
      />
      {active ? (
        spec.trueFlag ? (
          <spec.trueFlag.display />
        ) : null
      ) : spec.falseFlag ? (
        <spec.falseFlag.display />
      ) : null}
    </>
  );
}

export class FlaggedSpec<V, T> extends Spec<boolean | V | T> {
  constructor(
    _name: string,
    public falseFlag: Spec<V> | null = null,
    public trueFlag: Spec<T> | null = null
  ) {
    super(_name, false, FlaggedComponent);
  }

  export = () => {
    if (this.data && this.trueFlag !== null) {
      return this.trueFlag.export();
    }

    if (!this.data && this.falseFlag !== null) {
      return this.falseFlag.export();
    }
    return this.data;
  };
}

const IdentifierComponent: SpecC<IdentifierSpec> = ({ spec }) => {
  let [val, setVal] = useState(spec.data);

  return (
    <input
      type="number"
      value={val}
      onChange={e => {
        setVal(e.target.value);
        spec.data = e.target.value;
      }}
    />
  );
};

export class IdentifierSpec extends Spec<string> {
  constructor(_name: string) {
    super(_name, "", IdentifierComponent);
  }
}

const MultiplechoiceComponent: SpecC<MultiplechoiceSpec> = ({ spec }) => {
  let [val, setVal] = useState<number | null>(null);
  let Selection = () => {
    if (val === null) return <></>;
    let A = spec.choices[val][1].display;
    return <A />;
  };
  return (
    <>
      <select onChange={e => setVal(e.target.selectedIndex)}>
        {spec.choices.map(c => (
          <option key={c[0]}>{c[0]}</option>
        ))}
      </select>
      <Selection />
    </>
  );
};

export class MultiplechoiceSpec extends Spec<any> {
  constructor(_name: string, public choices: [string, Spec<any>][]) {
    super(_name, "", MultiplechoiceComponent);
  }
}
