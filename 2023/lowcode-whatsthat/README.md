# Low Code - What does it take?

Low code is often suggested to be a great solution to empower everyone to build apps.
But what does it take for low code?

## The goals of low code

Wikipedia says:
> A low-code development platform (LCDP) provides a development environment used to create application software through a graphical user interface. A low-coded platform may produce entirely operational applications, or require additional coding for specific situations. Low-code development platforms can reduce the amount of traditional time spent, enabling accelerated delivery of business applications. A common benefit is that a wider range of people can contribute to the application's development—not only those with coding skills but require good governance to be able to adhere to common rules and regulations. LCDPs can also lower the initial cost of setup, training, deployment, and maintenance.

## Low Code and Development speed

It is suggested that low code development empowers everyone to develop applications without any special skills.
I hypothesize that low code is not achieved through ommiting code but by writing less and simpler code with support of additional tooling.

I will demonstrate this idea down below, with a part of an example application I specified some time ago:


## Component Codes - Specify any PC Component
This includes:
 - CPU w/ GPU
 - GPU
 - TPU
 - HDD
 - SSD
 - NIC
 - RAM

## Specifications

Each component can be encoded into a sequence of bytes.
These bytes will be encoded into a sequence of Alphanumeric characters as representation.
The first byte will be the type of the component. Followed by one byte of flags for the component.  
A field is always one to 9 bytes long. 
A field may be an identifier (i), a flagset (f) or a number (n).
Numbers are encoded as varint. The most significant bit of a byte determines if another byte of the number follows.
A component may be suffixed with its name. The name will be seperated from its spec with a colon.

The component specification for different components are:

### CPU
  - Cluster spec
  - [Memory Specs]
  - n Tmax in °C
  - n TDP in W
  - i socket
  - f onboard graphis
  - Onboard graphics [GPU Specs]

#### Clusters spec

Describes the how many kinds of cores the CPU has.
Fields:  
  - n Count
  - [Cluster specs]

#### Core specs per cluster

Describes a single cluster
Fields:
  - i chip architecture
  - n bits of operations
  - n Number of cores
  - n Number of threads
  - n Base frequency in MHz
  - n Boost frequency in MHz
  - n Burst frequency in MHz
  - n Layers of cache
  - [Cache Specs]
  - n PCIe version
  - n PCIe lanes

#### Cache specs per cluster
  - f is unified

if not is unified:
  - n data Cache in kb
  - n inst. Cache in kb

if is unified:
  - n Cache in kb

#### Memory specs
  - i memory version
  - n Max amount of memory in MB
  - n Memory channels
  - n Frequency in MHz


## result in our "low code" solution for a CPU

```tsx
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

const CPU = new ArraySpec("", clusterSpec);
```

With a handful of utility functions we can even further simplify the code into:

```tsx
const cacheSpec = () =>
  flagged(
    "unified",
    flagged("Caches", {
      data: integer("Cache Size in Kb"),
      instructions: integer("Cache Size in Kb"),
    }),
    integer("Cache Size in Kb")
  );

const clusterSpec = () =>
  array(
    "Clusters",
    () =>
      fields("Cluster", {
        architecture: text(),
        "word size": integer("in bits"),
        cores: integer(),
        threads: integer(),
        "base frequency": integer("in MHz"),
        "boost frequency": integer("in MHz"),
        "burst frequency": integer("in MHz"),
        caches: array(cacheSpec),
      })
  );

const CPU = array(clusterSpec);
```

Fo many developers this is a very simple and probably a "low code" approach.
We have very little code, to do a lot.

## GUIs and Builders

Why GUIs and other kind of website builders are not a great choice for applications more complicated than a blog is probably because of the lack of customizability.
There are only two ways to create a low code platform. The attempt I did, with a simple interface and little customizability. Or a lot of customizability and a complicated interface.
Often complex looks and features are unnecessarily required to be distinct which gives low code platform a hard time. If we focus on simple maintainable applications, applications become much easier to build.  

From todays standpoint I'd say: bells and whistles are the death of low code.