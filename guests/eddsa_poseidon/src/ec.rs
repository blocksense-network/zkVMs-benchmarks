// Twisted Edwards curves

use num::BigInt;
pub type Field = BigInt;

// TEPoint in extended twisted Edwards coordinates
pub struct GroupPoint {
    pub x: Field,
    pub y: Field,
    pub t: Field,
    pub z: Field,
}

// TECurve specification
pub struct GroupCurve { // Twisted Edwards curve
    // Coefficients in defining equation a(x^2 + y^2)z^2 = z^4 + dx^2y^2
    pub a: Field,
    pub d: Field,
    // Generator as point in projective coordinates
    pub gen: GroupPoint,
}

impl GroupPoint {
    // GroupPoint constructor
    pub fn new(x: Field, y: Field, t: Field, z: Field) -> Self {
        Self { x, y, t, z }
    }

    // Check if zero
    pub fn is_zero(self) -> bool {
        let Self { x, y, t, z } = self;
        (&x == &BigInt::ZERO) & (&y == &z) & (&y != &BigInt::ZERO) & (&t == &BigInt::ZERO)
    }

    // Additive identity
    pub fn zero() -> Self {
        GroupPoint::new(0.into(), 1.into(), 0.into(), 1.into())
    }

    // Conversion to affine coordinates
    pub fn into_affine(self) -> TEPoint {
        let Self { x, y, t: _t, z } = self;

        TEPoint::new(&x / &z, &y / &z)
    }
}

impl PartialEq for GroupPoint {
    fn eq(&self, p: &Self) -> bool {
        let Self { x: x1, y: y1, t: _t1, z: z1 } = self;
        let Self { x: x2, y: y2, t: _t2, z: z2 } = p;

        (x1 * z2 == x2 * z1) & (y1 * z2 == y2 * z1)
    }
}

impl GroupCurve {
    // GroupCurve constructor
    pub fn new(a: Field, d: Field, gen: GroupPoint) -> GroupCurve {
        // Check curve coefficients
        assert!(&a * &d * (&a - &d) != 0.into());

        let curve = GroupCurve { a, d, gen };

        // gen should be on the curve
        assert!(curve.contains(curve.gen.clone()));

        curve
    }

    // Conversion to affine coordinates
    pub fn into_affine(self) -> TECurve {
        let GroupCurve { a, d, gen } = self;

        TECurve { a, d, gen: gen.into_affine() }
    }

    // Point addition
    pub fn add(self, p1: GroupPoint, p2: GroupPoint) -> GroupPoint {
        let GroupPoint { x: x1, y: y1, t: t1, z: z1 } = p1;
        let GroupPoint { x: x2, y: y2, t: t2, z: z2 } = p2;

        let a = &x1 * &x2;
        let b = &y1 * &y2;
        let c = self.d * &t1 * &t2;
        let d = &z1 * &z2;
        let e = (&x1 + &y1) * (&x2 + &y2) - &a - &b;
        let f = &d - &c;
        let g = &d + &c;
        let h = b - self.a * a;

        let x = &e * &f;
        let y = &g * &h;
        let t = &e * &h;
        let z = &f * &g;

        GroupPoint::new(x, y, t, z)
    }

    // Scalar multiplication with scalar represented by a bit array (little-endian convention).
    // If k is the natural number represented by `bits`, then this computes p + ... + p k times.
    pub fn bit_mul<const N: usize>(self, bits: [bool; N], p: GroupPoint) -> GroupPoint {
        let mut out = GroupPoint::zero();

        for i in 0..N {
            out = self.add(
                self.add(out, out),
                if bits[N - i - 1] == false {
                    GroupPoint::zero()
                } else {
                    p
                },
            );
        }

        out
    }

    // Scalar multiplication (p + ... + p n times)
    pub fn mul(self, n: Field, p: GroupPoint) -> GroupPoint {
        // TODO: temporary workaround until issue 1354 is solved
        let mut n_as_bits: [bool; 254] = [false; 254];
        let tmp: [bool; 254] = n.to_le_bits();
        for i in 0..254 {
            n_as_bits[i] = tmp[i];
        }

        self.bit_mul(n_as_bits, p)
    }

    // Membership check
    pub fn contains(self, p: GroupPoint) -> bool {
        let GroupPoint { x, y, t, z } = p;

        (z != BigInt::ZERO)
            & (z * t == x * y)
            & (z * z * (self.a * x * x + y * y) == z * z * z * z + self.d * x * x * y * y)
    }
}

// TEPoint in Cartesian coordinates
#[derive(Clone)]
pub struct TEPoint {
    pub x: Field,
    pub y: Field,
}

// TECurve specification
#[derive(Clone)]
pub struct TECurve { // Twisted Edwards curve
    // Coefficients in defining equation ax^2 + y^2 = 1 + dx^2y^2
    pub a: Field,
    pub d: Field,
    // Generator as point in Cartesian coordinates
    pub gen: TEPoint,
}

impl TEPoint {
    // TEPoint constructor
    pub fn new(x: Field, y: Field) -> Self {
        Self { x, y }
    }

    // Check if zero
    pub fn is_zero(self) -> bool {
        self.eq(&TEPoint::zero())
    }

    // Conversion to TECurveGroup coordinates
    pub fn into_group(self) -> GroupPoint {
        let Self { x, y } = self;

        GroupPoint::new(x.clone(), y.clone(), &x * y, 1.into())
    }

    // Additive identity
    pub fn zero() -> Self {
        TEPoint::new(0.into(), 1.into())
    }
}

impl PartialEq for TEPoint {
    fn eq(&self, p: &Self) -> bool {
        let Self { x: x1, y: y1 } = self;
        let Self { x: x2, y: y2 } = p;

        (x1 == x2) & (y1 == y2)
    }
}

impl TECurve {
    // TECurve constructor
    pub fn new(a: Field, d: Field, gen: TEPoint) -> TECurve {
        // Check curve coefficients
        assert!(&a * &d * (&a - &d) != 0.into());

        let curve = TECurve { a, d, gen };

        // gen should be on the curve
        assert!(curve.contains(curve.gen.clone()));

        curve
    }

    // Conversion to TECurveGroup coordinates
    pub fn into_group(self) -> GroupCurve {
        let TECurve { a, d, gen } = self;

        GroupCurve { a, d, gen: gen.into_group() }
    }

    // Membership check
    pub fn contains(self, p: TEPoint) -> bool {
        let TEPoint { x, y } = p;
        self.a * &x * &x + &y * &y == 1 + self.d * &x * &x * &y * &y
    }

    // TEPoint addition, implemented in terms of mixed addition for reasons of efficiency
    pub fn add(self, p1: TEPoint, p2: TEPoint) -> TEPoint {
        self.mixed_add(p1, p2.into_group()).into_affine()
    }

    // Mixed point addition, i.e. first argument in affine, second in TECurveGroup coordinates.
    pub fn mixed_add(self, p1: TEPoint, p2: GroupPoint) -> GroupPoint {
        let TEPoint { x: x1, y: y1 } = p1;
        let GroupPoint { x: x2, y: y2, t: t2, z: z2 } = p2;

        let a = &x1 * &x2;
        let b = &y1 * &y2;
        let c = self.d * &x1 * &y1 * &t2;
        let e = (x1 + y1) * (x2 + y2) - &a - &b;
        let f = &z2 - &c;
        let g = &z2 + &c;
        let h = &b - self.a * &a;

        let x = &e * &f;
        let y = &g * &h;
        let t = &e * &h;
        let z = &f * &g;

        GroupPoint::new(x, y, t, z)
    }

    // Scalar multiplication (p + ... + p n times)
    pub fn mul(self, n: Field, p: TEPoint) -> TEPoint {
        self.into_group().mul(n, p.into_group()).into_affine()
    }
}

pub struct BabyJubjub {
    pub curve: TECurve,
    pub base8: TEPoint,
    pub suborder: Field,
}

pub fn baby_jubjub() -> BabyJubjub {
    BabyJubjub {
        // Baby Jubjub (ERC-2494) parameters in affine representation
        curve: TECurve::new(
            168700.into(),
            168696.into(),
            // G
            TEPoint::new(
                995203441582195749578291179787384436505546430278305826713579947235728471134,
                5472060717959818805561601436314318772137091100104008585924551046643952123905,
            ),
        ),
        // [8]G precalculated
        base8: TEPoint::new(
            5299619240641551281634865583518297030282874472190772894086521144482721001553,
            16950150798460657717958625567821834550301663161624707787222815936182638968203,
        ),
        // The size of the group formed from multiplying the base field by 8.
        suborder: 2736030358979909402780800718157159386076813972158567259200215660948447373041,
    }
}
