# Units

arity writes each unit in UCUM, the Unified Code for Units of Measure. For
example, µM is `umol/L`, °C is `Cel`, and µg/mL is `ug/mL`. `arity vocab units`
lists the units that arity knows.

## Dimensions

Each unit has a dimension, with seven base quantities:

| Symbol | Quantity |
| --- | --- |
| L | length |
| M | mass |
| T | time |
| I | electric current |
| Θ | temperature |
| N | amount of substance |
| J | luminous intensity |

For example, `umol/L` has the dimension `L-3 N`, and `ug/mL` has `L-3 M`. A
molar concentration and a mass concentration have different dimensions, so
arity keeps them as different properties: a conversion between them needs a
molar mass, which the source often does not state.

## Conversions

arity converts each value into the unit of its property for the tables and
the figures. The stored value is always the value as the source states it,
with each digit that the source writes.

- A temperature in °C or °F has an offset. arity uses the offset for a
  temperature, but not for a difference of temperatures.
- An arbitrary unit, such as `[iU]` or `[CFU]`, converts only to itself.

## The unit text

Each quantity has two units: `unit`, in UCUM, and `unit_text`, as the source
writes it. arity reads the unit text with a lenient reader. For example, it
reads "µM" as `umol/L`, "kJ mol−1" as `kJ/mol`, and "°C" as `Cel`. The checks
find the unit text in the quotes and make sure that it is the same unit as the
UCUM unit. When arity cannot read the unit text, the checks give a warning, and
a person must check the unit.
