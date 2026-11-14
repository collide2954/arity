use std::f64::consts::PI;

use crate::Dimension;

#[derive(Debug, Clone, Copy)]
pub struct Prefix {
    pub(crate) code: &'static str,
    pub(crate) exponent: i32,
}

pub(crate) const PREFIXES: &[Prefix] = &[
    Prefix {
        code: "da",
        exponent: 1,
    },
    Prefix {
        code: "Q",
        exponent: 30,
    },
    Prefix {
        code: "R",
        exponent: 27,
    },
    Prefix {
        code: "Y",
        exponent: 24,
    },
    Prefix {
        code: "Z",
        exponent: 21,
    },
    Prefix {
        code: "E",
        exponent: 18,
    },
    Prefix {
        code: "P",
        exponent: 15,
    },
    Prefix {
        code: "T",
        exponent: 12,
    },
    Prefix {
        code: "G",
        exponent: 9,
    },
    Prefix {
        code: "M",
        exponent: 6,
    },
    Prefix {
        code: "k",
        exponent: 3,
    },
    Prefix {
        code: "h",
        exponent: 2,
    },
    Prefix {
        code: "d",
        exponent: -1,
    },
    Prefix {
        code: "c",
        exponent: -2,
    },
    Prefix {
        code: "m",
        exponent: -3,
    },
    Prefix {
        code: "u",
        exponent: -6,
    },
    Prefix {
        code: "n",
        exponent: -9,
    },
    Prefix {
        code: "p",
        exponent: -12,
    },
    Prefix {
        code: "f",
        exponent: -15,
    },
    Prefix {
        code: "a",
        exponent: -18,
    },
    Prefix {
        code: "z",
        exponent: -21,
    },
    Prefix {
        code: "y",
        exponent: -24,
    },
    Prefix {
        code: "r",
        exponent: -27,
    },
    Prefix {
        code: "q",
        exponent: -30,
    },
];

/// One unit symbol: a scale to the coherent SI unit, an offset for
/// temperature scales, and a dimension.
#[derive(Debug, Clone, Copy)]
pub struct Atom {
    pub(crate) code: &'static str,
    pub(crate) name: &'static str,
    pub(crate) scale: f64,
    pub(crate) offset: f64,
    pub(crate) dimension: Dimension,
    pub(crate) metric: bool,
    pub(crate) arbitrary: bool,
    pub(crate) extension: bool,
}

const fn atom(
    code: &'static str,
    name: &'static str,
    scale: f64,
    dimension: [i8; 7],
    metric: bool,
) -> Atom {
    Atom {
        code,
        name,
        scale,
        offset: 0.0,
        dimension: Dimension::new(dimension),
        metric,
        arbitrary: false,
        extension: false,
    }
}

const fn extension(atom: Atom) -> Atom {
    Atom {
        extension: true,
        ..atom
    }
}

const fn arbitrary(code: &'static str, name: &'static str, metric: bool) -> Atom {
    Atom {
        arbitrary: true,
        ..atom(code, name, 1.0, NONE, metric)
    }
}

const NONE: [i8; 7] = [0; 7];
const LENGTH: [i8; 7] = [1, 0, 0, 0, 0, 0, 0];
const MASS: [i8; 7] = [0, 1, 0, 0, 0, 0, 0];
const TIME: [i8; 7] = [0, 0, 1, 0, 0, 0, 0];
const TEMPERATURE: [i8; 7] = [0, 0, 0, 0, 1, 0, 0];
const AMOUNT: [i8; 7] = [0, 0, 0, 0, 0, 1, 0];
const FREQUENCY: [i8; 7] = [0, 0, -1, 0, 0, 0, 0];
const FORCE: [i8; 7] = [1, 1, -2, 0, 0, 0, 0];
const PRESSURE: [i8; 7] = [-1, 1, -2, 0, 0, 0, 0];
const ENERGY: [i8; 7] = [2, 1, -2, 0, 0, 0, 0];
const DOSE: [i8; 7] = [2, 0, -2, 0, 0, 0, 0];
const VOLUME: [i8; 7] = [3, 0, 0, 0, 0, 0, 0];
const CATALYTIC: [i8; 7] = [0, 0, -1, 0, 0, 1, 0];
const MAGNETIC_FLUX_DENSITY: [i8; 7] = [0, 1, -2, -1, 0, 0, 0];

const ATOMIC_MASS: f64 = 1.660_539_066_6e-27;

pub(crate) const ATOMS: &[Atom] = &[
    atom("m", "metre", 1.0, LENGTH, true),
    atom("g", "gram", 1e-3, MASS, true),
    atom("s", "second", 1.0, TIME, true),
    atom("A", "ampere", 1.0, [0, 0, 0, 1, 0, 0, 0], true),
    atom("K", "kelvin", 1.0, TEMPERATURE, true),
    atom("mol", "mole", 1.0, AMOUNT, true),
    atom("cd", "candela", 1.0, [0, 0, 0, 0, 0, 0, 1], true),
    atom("rad", "radian", 1.0, NONE, true),
    atom("sr", "steradian", 1.0, NONE, true),
    atom("Hz", "hertz", 1.0, FREQUENCY, true),
    atom("N", "newton", 1.0, FORCE, true),
    atom("Pa", "pascal", 1.0, PRESSURE, true),
    atom("J", "joule", 1.0, ENERGY, true),
    atom("W", "watt", 1.0, [2, 1, -3, 0, 0, 0, 0], true),
    atom("C", "coulomb", 1.0, [0, 0, 1, 1, 0, 0, 0], true),
    atom("V", "volt", 1.0, [2, 1, -3, -1, 0, 0, 0], true),
    atom("F", "farad", 1.0, [-2, -1, 4, 2, 0, 0, 0], true),
    atom("Ohm", "ohm", 1.0, [2, 1, -3, -2, 0, 0, 0], true),
    atom("S", "siemens", 1.0, [-2, -1, 3, 2, 0, 0, 0], true),
    atom("Wb", "weber", 1.0, [2, 1, -2, -1, 0, 0, 0], true),
    atom("T", "tesla", 1.0, MAGNETIC_FLUX_DENSITY, true),
    atom("H", "henry", 1.0, [2, 1, -2, -2, 0, 0, 0], true),
    atom("lm", "lumen", 1.0, [0, 0, 0, 0, 0, 0, 1], true),
    atom("lx", "lux", 1.0, [-2, 0, 0, 0, 0, 0, 1], true),
    atom("Bq", "becquerel", 1.0, FREQUENCY, true),
    atom("Gy", "gray", 1.0, DOSE, true),
    atom("Sv", "sievert", 1.0, DOSE, true),
    atom("kat", "katal", 1.0, CATALYTIC, true),
    atom("U", "enzyme unit", 1e-6 / 60.0, CATALYTIC, true),
    Atom {
        offset: 273.15,
        ..atom("Cel", "degree Celsius", 1.0, TEMPERATURE, false)
    },
    Atom {
        offset: 459.67 * 5.0 / 9.0,
        ..atom("[degF]", "degree Fahrenheit", 5.0 / 9.0, TEMPERATURE, false)
    },
    atom("l", "litre", 1e-3, VOLUME, true),
    atom("L", "litre", 1e-3, VOLUME, true),
    atom("min", "minute", 60.0, TIME, false),
    atom("h", "hour", 3_600.0, TIME, false),
    atom("d", "day", 86_400.0, TIME, false),
    atom("wk", "week", 604_800.0, TIME, false),
    atom("mo", "month", 2_629_800.0, TIME, false),
    atom("a", "year", 31_557_600.0, TIME, false),
    atom("t", "tonne", 1_000.0, MASS, true),
    atom("u", "unified atomic mass unit", ATOMIC_MASS, MASS, true),
    extension(atom("Da", "dalton", ATOMIC_MASS, MASS, true)),
    atom("eV", "electronvolt", 1.602_176_634e-19, ENERGY, true),
    atom("bar", "bar", 1e5, PRESSURE, true),
    atom("atm", "standard atmosphere", 101_325.0, PRESSURE, false),
    atom("m[Hg]", "metre of mercury", 133_322.387_415, PRESSURE, true),
    atom("m[H2O]", "metre of water", 9_806.65, PRESSURE, true),
    extension(atom("Torr", "torr", 101_325.0 / 760.0, PRESSURE, true)),
    atom(
        "[psi]",
        "pound per square inch",
        6_894.757_293_168,
        PRESSURE,
        false,
    ),
    atom("Ao", "angstrom", 1e-10, LENGTH, false),
    atom("%", "percent", 1e-2, NONE, false),
    atom("[ppth]", "part per thousand", 1e-3, NONE, false),
    atom("[ppm]", "part per million", 1e-6, NONE, false),
    atom("[ppb]", "part per billion", 1e-9, NONE, false),
    atom("[pptr]", "part per trillion", 1e-12, NONE, false),
    atom("deg", "degree", PI / 180.0, NONE, false),
    atom("cal", "calorie", 4.184, ENERGY, true),
    atom(
        "[g]",
        "standard acceleration of gravity",
        9.806_65,
        [1, 0, -2, 0, 0, 0, 0],
        false,
    ),
    atom("G", "gauss", 1e-4, MAGNETIC_FLUX_DENSITY, true),
    atom("dyn", "dyne", 1e-5, FORCE, true),
    atom("erg", "erg", 1e-7, ENERGY, true),
    atom("P", "poise", 0.1, [-1, 1, -1, 0, 0, 0, 0], true),
    atom("St", "stokes", 1e-4, [2, 0, -1, 0, 0, 0, 0], true),
    atom("Ci", "curie", 3.7e10, FREQUENCY, true),
    atom("R", "roentgen", 2.58e-4, [0, -1, 1, 1, 0, 0, 0], true),
    atom("RAD", "radiation absorbed dose", 1e-2, DOSE, true),
    atom("REM", "roentgen equivalent man", 1e-2, DOSE, true),
    atom("pc", "parsec", 3.085_677_581_491_367e16, LENGTH, true),
    atom("AU", "astronomical unit", 149_597_870_700.0, LENGTH, false),
    atom("[ly]", "light year", 9.460_730_472_580_8e15, LENGTH, true),
    atom("b", "barn", 1e-28, [2, 0, 0, 0, 0, 0, 0], true),
    atom("[in_i]", "inch", 0.025_4, LENGTH, false),
    atom("[ft_i]", "foot", 0.304_8, LENGTH, false),
    atom("[mi_i]", "mile", 1_609.344, LENGTH, false),
    atom("[lb_av]", "pound", 0.453_592_37, MASS, false),
    atom("osm", "osmole", 1.0, AMOUNT, true),
    atom("eq", "equivalent", 1.0, AMOUNT, true),
    extension(atom("bp", "base pair", 1.0, NONE, true)),
    arbitrary("[iU]", "international unit", true),
    arbitrary("[IU]", "international unit", true),
    arbitrary("[arb'U]", "arbitrary unit", false),
    arbitrary("[CFU]", "colony forming unit", false),
    arbitrary("[PFU]", "plaque forming unit", false),
];

impl Prefix {
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// The power of ten that the prefix applies.
    pub fn exponent(&self) -> i32 {
        self.exponent
    }
}

impl Atom {
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// True when the symbol can take an SI prefix.
    pub fn is_metric(&self) -> bool {
        self.metric
    }

    /// True when the symbol is a unit that only counts against itself.
    pub fn is_arbitrary(&self) -> bool {
        self.arbitrary
    }

    /// True when arity adds the symbol and UCUM does not define it.
    pub fn is_extension(&self) -> bool {
        self.extension
    }
}

/// All unit symbols that the parser knows.
pub fn symbols() -> &'static [Atom] {
    ATOMS
}

/// All SI prefixes that the parser knows.
pub fn prefixes() -> &'static [Prefix] {
    PREFIXES
}

pub(crate) fn find_atom(code: &str) -> Option<&'static Atom> {
    ATOMS.iter().find(|atom| atom.code == code)
}

/// The tag that makes two arbitrary units the same unit.
pub(crate) fn arbitrary_tag(atom: &Atom) -> &'static str {
    if atom.code == "[IU]" {
        "[iU]"
    } else {
        atom.code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_codes_are_unique() {
        for (index, atom) in ATOMS.iter().enumerate() {
            let first = ATOMS.iter().position(|other| other.code == atom.code);
            assert_eq!(first, Some(index), "{}", atom.code);
        }
    }

    #[test]
    fn prefix_codes_are_unique() {
        for (index, prefix) in PREFIXES.iter().enumerate() {
            let first = PREFIXES.iter().position(|other| other.code == prefix.code);
            assert_eq!(first, Some(index), "{}", prefix.code);
        }
    }

    #[test]
    fn the_two_letter_prefix_comes_first() {
        assert_eq!(PREFIXES[0].code, "da");
        assert!(PREFIXES[1..].iter().all(|prefix| prefix.code.len() == 1));
    }

    #[test]
    fn extensions_are_marked() {
        let extensions: Vec<&str> = symbols()
            .iter()
            .filter(|atom| atom.is_extension())
            .map(Atom::code)
            .collect();
        assert_eq!(extensions, ["Da", "Torr", "bp"]);
    }

    #[test]
    fn every_scale_is_positive_and_finite() {
        for atom in ATOMS {
            assert!(atom.scale.is_finite() && atom.scale > 0.0, "{}", atom.code);
            assert!(!atom.name.is_empty(), "{}", atom.code);
        }
    }
}
