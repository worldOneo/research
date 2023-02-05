import {
  ArraySpec,
  FieldsSpec,
  FlaggedSpec,
  IdentifierSpec,
  MultiplechoiceSpec,
  NumberSpec,
} from "./Spec";

const cacheSpec = () =>
  new FlaggedSpec(
    "unified",
    new FieldsSpec("Caches", {
      data: new NumberSpec("Cache Size in Kb"),
      instructions: new NumberSpec("Cache Size in Kb"),
    }),
    new NumberSpec("Cache Size in Kb")
  );

const clusterSpec = () =>
  new ArraySpec(
    "Clusters",
    () =>
      new FieldsSpec("Cluster", {
        architecture: new IdentifierSpec(""),
        "word size": new NumberSpec("in bits"),
        cores: new NumberSpec(""),
        threads: new NumberSpec(""),
        "base frequency": new NumberSpec("in MHz"),
        "boost frequency": new NumberSpec("in MHz"),
        "burst frequency": new NumberSpec("in MHz"),
        caches: new ArraySpec("", cacheSpec),
      })
  );

const cpu = new FieldsSpec("CPU", {
  cluster: clusterSpec(),
  drives: new ArraySpec(
    "drives",
    () =>
      new MultiplechoiceSpec("drive", [
        ["ssd", new NumberSpec("size in MB")],
        ["hhd", new NumberSpec("size in GB")],
      ])
  ),
  memory: new FlaggedSpec(
    "",
    new IdentifierSpec("Form of storage"),
    new NumberSpec("Memory in MB")
  ),
}).display;

export default cpu;
